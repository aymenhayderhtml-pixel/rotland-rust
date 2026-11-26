//! Evacuation jam placement (pure, no GPU). Outbound lanes of cars leave the city along the
//! highway and the first two avenues, with a few wrecks inbound. Buses, army trucks, luggage,
//! bodies and the gantry follow the rules in `jam_vehicles.rs`. Tests in `jam_tests.rs`.

use std::f32::consts::FRAC_PI_2;

use glam::{Vec2, Vec3};

use crate::metro::config::MetroConfig;
use crate::metro::highway::Highway;
use crate::metro::layout::MetroLayout;
use crate::metro::place::MetroSite;

#[path = "jam_vehicles.rs"]
mod vehicles;
#[cfg(test)]
#[path = "jam_tests.rs"]
mod tests;

use vehicles::GANTRY_MODEL;

/// Number of avenues (from the city entrance) that get a jam.
const AVENUE_COUNT: usize = 2; // new: DECISIONS 2026-11-26 (the first two avenues by the entrance)
/// Lane centres sit this many lane widths out from the median edge (or the road centre).
const LANE_CENTRE_STEPS: [f32; 2] = [0.5, 1.5]; // new: DECISIONS 2026-11-26 (half, then one and a half lanes out)
/// Each car's centre sits this fraction of its pitch past the start of its slot.
const CAR_CENTRE_STEP: f32 = 0.5; // new: DECISIONS 2026-11-26 (car centre is half a pitch in)
/// Half of something (medians, shoulders, lane centres).
const HALF: f32 = 0.5; // new: DECISIONS 2026-11-26 (plain half)
/// FNV-1a 32-bit offset basis and prime (same scheme as the layout and town seeds).
const FNV_OFFSET: u32 = 2_166_136_261; // new: DECISIONS 2026-11-26
const FNV_PRIME: u32 = 16_777_619; // new: DECISIONS 2026-11-26

/// One jam prop in site-world XZ. `pos.y` is 0 for ground props; bodies store their seat height
/// there (added to the ground height by `jam_chunks.rs`). Pitch and roll stay 0 in R38.
#[derive(Debug, Clone, PartialEq)]
pub struct JamProp {
    pub model: String,
    pub pos: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

/// What stands in a car slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Car,
    /// The bus lying on its side across the outer lane, nearest the city.
    BusSide,
    Bus,
    Truck,
}

/// One place in a lane: a point on the road centre line plus a lateral offset.
#[derive(Debug, Clone, Copy)]
pub(super) struct Slot {
    road: Vec2,
    dir: Vec2,
    lateral: f32,
    heading: f32,
}

impl Slot {
    /// The point at `lateral` metres from the road centre line, same station along the road.
    pub(super) fn pos_at(&self, lateral: f32) -> Vec2 {
        self.road + right_of(self.dir) * lateral
    }

    pub(super) fn pos(&self) -> Vec2 {
        self.pos_at(self.lateral)
    }

    /// Unit direction away from the road centre, toward this slot's side.
    pub(super) fn outward(&self) -> Vec2 {
        right_of(self.dir) * self.lateral.signum()
    }

    /// Unit direction the vehicle faces.
    pub(super) fn heading_dir(&self) -> Vec2 {
        self.dir * self.heading
    }
}

/// Plans the whole evacuation jam for one highway and city. Same inputs, same output.
pub fn plan_jam(highway: &Highway, layout: &MetroLayout, cfg: &MetroConfig) -> Vec<JamProp> {
    let spacing = cfg.jam.car_spacing_m;
    if highway.points.len() < 2 || spacing[0] <= 0.0 {
        return Vec::new();
    }
    let entrance = highway.points[highway.points.len() - 1];
    let mut planner = vehicles::Planner::new(jam_seed(&layout.site), cfg);
    let length = cfg.jam.jam_length;
    let lane_w = cfg.jam.lane_width_m;
    let median_half = cfg.highway.median_width * HALF;
    let edge = highway.width * HALF - cfg.highway.shoulder_width * HALF;
    let outbound = Track::new(highway.points.iter().rev().copied().collect());
    let lanes = lane_offsets(median_half, lane_w);

    let out_slots: [Vec<Slot>; 2] =
        lanes.map(|lat| walk(&mut planner, &outbound, lat, 1.0, length));
    let kinds = assign_specials(&mut planner, [out_slots[0].len(), out_slots[1].len()]);
    for (lane, slots) in out_slots.iter().enumerate() {
        for (slot, kind) in slots.iter().zip(&kinds[lane]) {
            match kind {
                Kind::Car => planner.push_car(slot, false, Some(edge)),
                big => push_big(&mut planner, slot, *big, lane_w),
            }
        }
    }

    for lat in lanes {
        for slot in walk(&mut planner, &outbound, -lat, -1.0, length) {
            if planner.chance(cfg.jam.inbound_chance) {
                planner.push_car(&slot, true, Some(edge));
            }
        }
    }

    for (near, far) in pick_avenues(layout, entrance) {
        let track = Track::new(vec![near, far]);
        for lat in avenue_offsets(lane_w) {
            for slot in walk(&mut planner, &track, lat, lat.signum(), length) {
                if planner.chance(cfg.jam.avenue_jam_chance) {
                    planner.push_car(&slot, false, None);
                }
            }
        }
    }

    let (at, dir) = outbound.sample(length);
    planner.push_ground(GANTRY_MODEL, at, dir_yaw(dir));
    planner.out
}

/// The two outbound lane centres (positive = right of the city-to-town direction).
fn lane_offsets(median_half: f32, lane_w: f32) -> [f32; 2] {
    LANE_CENTRE_STEPS.map(|step| median_half + lane_w * step)
}

/// The four avenue lane centres, two each way.
fn avenue_offsets(lane_w: f32) -> [f32; 4] {
    let [a, b] = LANE_CENTRE_STEPS.map(|step| lane_w * step);
    [a, b, -a, -b]
}

/// Yaw of a unit direction on the XZ plane (Godot convention, forward = -Z at yaw 0).
pub(super) fn dir_yaw(d: Vec2) -> f32 {
    (-d.x).atan2(-d.y)
}

/// Right-hand side of a direction on the XZ plane (`Vec2.y` is world Z).
pub(super) fn right_of(d: Vec2) -> Vec2 {
    Vec2::new(-d.y, d.x)
}

/// A polyline with its cumulative length, sampled by distance along it.
#[derive(Debug, Clone)]
pub(super) struct Track {
    pts: Vec<Vec2>,
    cum: Vec<f32>,
}

impl Track {
    pub(super) fn new(pts: Vec<Vec2>) -> Self {
        let mut cum = Vec::with_capacity(pts.len());
        let mut acc = 0.0;
        for (i, p) in pts.iter().enumerate() {
            if i > 0 {
                acc += p.distance(pts[i - 1]);
            }
            cum.push(acc);
        }
        Self { pts, cum }
    }

    pub(super) fn total(&self) -> f32 {
        self.cum.last().copied().unwrap_or(0.0)
    }

    /// Point and unit direction at distance `s` (clamped to the track). Needs two points.
    pub(super) fn sample(&self, s: f32) -> (Vec2, Vec2) {
        let s = s.clamp(0.0, self.total());
        let i = self.cum.partition_point(|&c| c <= s).saturating_sub(1).min(self.pts.len() - 2);
        let seg = self.pts[i + 1] - self.pts[i];
        let dir = seg.normalize_or(Vec2::X);
        let along = (s - self.cum[i]).clamp(0.0, seg.length());
        (self.pts[i] + dir * along, dir)
    }
}

/// Walks one lane from the track start, one car per pitch, until `length` is used up.
fn walk(planner: &mut vehicles::Planner, track: &Track, lateral: f32, heading: f32, length: f32) -> Vec<Slot> {
    let end = length.min(track.total());
    let spacing = planner.cfg.jam.car_spacing_m;
    let mut slots = Vec::new();
    let mut s = 0.0;
    loop {
        let pitch = planner.range_f(spacing);
        let mid = s + pitch * CAR_CENTRE_STEP;
        if mid > end {
            break;
        }
        let (road, dir) = track.sample(mid);
        slots.push(Slot { road, dir, lateral, heading });
        s += pitch;
    }
    slots
}

/// Picks the bus (first slot of the outer outbound lane), the other buses and the army trucks,
/// each on a free outbound slot.
fn assign_specials(planner: &mut vehicles::Planner, lens: [usize; 2]) -> [Vec<Kind>; 2] {
    let mut kinds = lens.map(|n| vec![Kind::Car; n]);
    if lens[1] > 0 {
        kinds[1][0] = Kind::BusSide;
    }
    let buses = planner.range_usize(planner.cfg.jam.bus_count);
    let trucks = planner.range_usize(planner.cfg.jam.army_truck_count);
    for _ in 1..buses {
        place_special(planner, &mut kinds, Kind::Bus);
    }
    for _ in 0..trucks {
        place_special(planner, &mut kinds, Kind::Truck);
    }
    kinds
}

fn place_special(planner: &mut vehicles::Planner, kinds: &mut [Vec<Kind>; 2], kind: Kind) {
    let free: Vec<(usize, usize)> = kinds
        .iter()
        .enumerate()
        .flat_map(|(lane, slots)| {
            slots
                .iter()
                .enumerate()
                .filter(|(_, k)| **k == Kind::Car)
                .map(move |(i, _)| (lane, i))
        })
        .collect();
    if free.is_empty() {
        return;
    }
    let (lane, i) = free[planner.index(free.len())];
    kinds[lane][i] = kind;
}

/// Buses and trucks: the bus on its side lies across the outer lane and onto the shoulder.
fn push_big(planner: &mut vehicles::Planner, slot: &Slot, kind: Kind, lane_w: f32) {
    let yaw = dir_yaw(slot.heading_dir());
    match kind {
        Kind::BusSide => {
            let across = slot.lateral + lane_w * LANE_CENTRE_STEPS[0];
            planner.push_ground(vehicles::BUS_SIDE_MODEL, slot.pos_at(across), yaw + FRAC_PI_2);
        }
        Kind::Bus => planner.push_ground(vehicles::BUS_MODEL, slot.pos(), yaw),
        Kind::Truck => planner.push_ground(vehicles::TRUCK_MODEL, slot.pos(), yaw),
        Kind::Car => {}
    }
}

/// The first `AVENUE_COUNT` avenues (inside the square) whose end is nearest the entrance,
/// each as (near end, far end).
fn pick_avenues(layout: &MetroLayout, entrance: Vec2) -> Vec<(Vec2, Vec2)> {
    let mut found: Vec<(f32, Vec2, Vec2)> = layout
        .streets
        .iter()
        .filter(|s| s.avenue && layout.contains((s.a + s.b) * HALF))
        .map(|s| {
            let (near, far) = if s.a.distance(entrance) <= s.b.distance(entrance) {
                (s.a, s.b)
            } else {
                (s.b, s.a)
            };
            (near.distance(entrance), near, far)
        })
        .collect();
    found.sort_by(|x, y| x.0.total_cmp(&y.0));
    found.into_iter().take(AVENUE_COUNT).map(|(_, n, f)| (n, f)).collect()
}

/// Seed from the site key and centre bits: the same site always gives the same jam.
fn jam_seed(site: &MetroSite) -> i32 {
    let mut hash = FNV_OFFSET;
    let bytes = site
        .key
        .bytes()
        .chain(site.center.x.to_bits().to_le_bytes())
        .chain(site.center.y.to_bits().to_le_bytes());
    for b in bytes {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash as i32
}
