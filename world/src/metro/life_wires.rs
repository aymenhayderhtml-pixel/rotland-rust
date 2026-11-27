//! Power line poles and wire spans along selected New Halden avenues. Pure logic: no GPU.

use std::collections::HashSet;
use std::f32::consts::FRAC_PI_2;

use glam::{Vec2, Vec3};

use crate::metro::layout::{MetroBlock, MetroStreet};
use crate::metro::life::{LifeConfig, dir_yaw, in_chunk, mix_seed};
use crate::metro::state::MetroWorld;
use crate::metro::traces::car_wall_sites;
use crate::random::DotnetRandom;
use crate::scatter::PropInstance;

const PROP_SCALE: f32 = 1.0; // new: DECISIONS 2026-11-27
const HALF: f32 = 0.5; // new: DECISIONS 2026-11-27
const ENDPOINT_EPS: f32 = 0.01; // new: DECISIONS 2026-11-27
/// Curb setback into the sidewalk, in metres (R41-03 spec).
const POLE_CURB_OFFSET_M: f32 = 0.6; // new: DECISIONS 2026-11-27
/// Minimum margin from the outer sidewalk edge so poles never touch tower lots.
const POLE_OUTER_MARGIN_M: f32 = 0.3; // new: DECISIONS 2026-11-27
/// Margin from cross-street curb trims along an avenue segment.
const POLE_END_MARGIN_M: f32 = 2.0; // new: DECISIONS 2026-11-27
/// Model length of `power_wire_span` and `power_wire_down` along +X, in metres (R41-04).
const WIRE_MODEL_LEN_M: f32 = 20.0; // new: DECISIONS 2026-11-27
/// Height of cross-arm insulators on `power_pole` (R41-04).
pub(super) const WIRE_ATTACH_HEIGHT_M: f32 = 8.5; // new: DECISIONS 2026-11-27
/// Mid-span sag of `power_wire_span`, in metres (R41-04).
pub(super) const WIRE_SAG_M: f32 = 1.2; // new: DECISIONS 2026-11-27
/// Slight Y drop when a wire attaches to a leaning pole.
const LEANING_WIRE_DROP_M: f32 = 0.25; // new: DECISIONS 2026-11-27
/// Clearance around car walls and lobby doors where poles are skipped.
const OBSTACLE_CLEAR_M: f32 = 5.0; // new: DECISIONS 2026-11-27
/// Maximum distance between adjacent standing poles to connect with a wire span.
const MAX_SPAN_FACTOR: f32 = 1.5; // new: DECISIONS 2026-11-27

/// One planned wire span between two standing poles (world XZ + base Y).
#[derive(Debug, Clone, Copy)]
pub(super) struct WireSpan {
    pub a: Vec2,
    pub b: Vec2,
    pub y_base: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PoleState {
    Standing,
    Leaning,
    Fallen,
}

#[derive(Debug, Clone, Copy)]
struct PlannedPole {
    pos: Vec2,
    yaw: f32,
    state: PoleState,
}

/// Yaw in radians so local `+X` points along `dir_w` in world XZ.
fn yaw_for_x(dir_w: Vec2) -> f32 {
    (-dir_w.y).atan2(dir_w.x)
}

/// Emits power poles, `power_wire_span`, and `power_wire_down` props inside `chunk`,
/// and returns all wire spans overlapping `chunk` (for crow wire-perches).
pub(super) fn collect_wires_in_chunk(
    metro: &MetroWorld,
    cfg: &LifeConfig,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
    out_props: &mut Vec<PropInstance>,
) -> Vec<WireSpan> {
    let active_lines = picked_avenue_lines(metro, cfg);
    if active_lines.is_empty() || cfg.pole_spacing_m <= 0.0 {
        return Vec::new();
    }
    let car_walls = car_wall_sites(metro);
    let mut spans = Vec::new();

    for &(horiz, line_idx) in &active_lines {
        let poles = plan_line_poles(metro, cfg, horiz, line_idx, &car_walls);
        emit_line_props(metro, cfg, &poles, chunk, height, out_props, &mut spans);
    }
    spans
}

/// Picks `poleAvenueShare` of the city's avenue grid lines deterministically.
fn picked_avenue_lines(metro: &MetroWorld, cfg: &LifeConfig) -> HashSet<(bool, i32)> {
    let n = metro.config.size_blocks;
    let every = metro.config.avenue_every;
    if every <= 0 || cfg.pole_avenue_share <= 0.0 {
        return HashSet::new();
    }
    let mut all = Vec::new();
    for idx in (0..=n).step_by(every as usize) {
        all.push((true, idx));
        all.push((false, idx));
    }
    let mut rng = DotnetRandom::new(mix_seed(&metro.layout.site, None, b"life_avenues"));
    // Shuffle deterministically and take `round(len * share)`.
    for i in (1..all.len()).rev() {
        let j = rng.next((i + 1) as i32) as usize;
        all.swap(i, j);
    }
    let want = ((all.len() as f32 * cfg.pole_avenue_share).round() as usize).clamp(1, all.len());
    all.into_iter().take(want).collect()
}

fn plan_line_poles(
    metro: &MetroWorld,
    cfg: &LifeConfig,
    horiz: bool,
    line_idx: i32,
    car_walls: &[(Vec2, f32)],
) -> Vec<Vec<Option<PlannedPole>>> {
    let layout = &metro.layout;
    let bs = metro.config.block_size;
    let half = layout.site.half_size;
    let coord = -half + line_idx as f32 * bs;
    let side_sign = if line_idx % 2 == 0 { 1.0_f32 } else { -1.0_f32 };
    let mut segments = Vec::new();

    for seg in 0..metro.config.size_blocks {
        let b_lo = if horiz { layout.block_at(seg, line_idx - 1) } else { layout.block_at(line_idx - 1, seg) };
        let b_hi = if horiz { layout.block_at(seg, line_idx) } else { layout.block_at(line_idx, seg) };
        if b_lo.is_none() && b_hi.is_none() {
            continue;
        }
        // Pick the side with a non-reserved block if possible.
        let chosen_block: Option<&MetroBlock> = match (side_sign > 0.0, b_hi, b_lo) {
            (true, Some(b), _) | (false, _, Some(b)) => Some(b),
            (true, None, Some(b)) | (false, Some(b), None) => Some(b),
            _ => None,
        };
        let Some(block) = chosen_block else { continue };
        if block.reserved.is_some() || metro.damage.tower_is_burnt((block.ix, block.iz)) {
            continue;
        }
        let a = if horiz {
            Vec2::new(-half + seg as f32 * bs, coord)
        } else {
            Vec2::new(coord, -half + seg as f32 * bs)
        };
        let b = a + if horiz { Vec2::X * bs } else { Vec2::Y * bs };
        let Some(street) = find_avenue_segment(&layout.streets, a, b) else { continue };
        let dir = (street.b - street.a).normalize_or_zero();
        let len = street.a.distance(street.b);
        let t0 = endpoint_trim(street.a, dir, &layout.streets) + POLE_END_MARGIN_M;
        let t1 = len - endpoint_trim(street.b, dir, &layout.streets) - POLE_END_MARGIN_M;
        if t1 <= t0 {
            continue;
        }

        let perp = Vec2::new(-dir.y, dir.x);
        let block_mid = (block.rect.0 + block.rect.1) * HALF;
        let into_block = if (block_mid - (street.a + street.b) * HALF).dot(perp) >= 0.0 {
            perp
        } else {
            -perp
        };
        let d_curb = (street.width * HALF - metro.look.sidewalk_width).max(0.0);
        let d_pole = (d_curb + POLE_CURB_OFFSET_M).min(street.width * HALF - POLE_OUTER_MARGIN_M);

        let mut rng = DotnetRandom::new(mix_seed(&layout.site, Some((block.ix, block.iz)), b"life_poles"));
        let mut seg_poles = Vec::new();
        let mut t = t0;
        while t <= t1 + ENDPOINT_EPS {
            let local = street.a + dir * t + into_block * d_pole;
            let wp = layout.site.to_world(local);
            let skip = should_skip_pole(metro, block, wp, car_walls);
            let state = roll_pole_state(&mut rng, cfg);
            if skip {
                seg_poles.push(None);
            } else {
                let into_road_w = (layout.site.to_world(street.a) - layout.site.to_world(street.a + into_block)).normalize_or_zero();
                let street_dir_w = (layout.site.to_world(street.b) - layout.site.to_world(street.a)).normalize_or_zero();
                let yaw = match state {
                    PoleState::Standing => dir_yaw(street_dir_w),
                    PoleState::Leaning => dir_yaw(into_road_w) + if rng.next(2) == 0 { 0.0 } else { FRAC_PI_2 },
                    PoleState::Fallen => dir_yaw(into_road_w),
                };
                seg_poles.push(Some(PlannedPole { pos: wp, yaw, state }));
            }
            t += cfg.pole_spacing_m;
        }
        segments.push(seg_poles);
    }
    segments
}

fn roll_pole_state(rng: &mut DotnetRandom, cfg: &LifeConfig) -> PoleState {
    let r = rng.next_double() as f32;
    if r < cfg.pole_fallen_chance {
        PoleState::Fallen
    } else if r < cfg.pole_fallen_chance + cfg.pole_leaning_chance {
        PoleState::Leaning
    } else {
        PoleState::Standing
    }
}

fn should_skip_pole(
    metro: &MetroWorld,
    block: &MetroBlock,
    wp: Vec2,
    car_walls: &[(Vec2, f32)],
) -> bool {
    if metro.in_tower_footprint(wp) {
        return true;
    }
    let fallen = &metro.damage.fallen;
    if (fallen.pos != Vec2::ZERO || fallen.block != (0, 0)) && wp.distance(fallen.pos) < metro.config.block_size * HALF {
        return true;
    }
    if car_walls.iter().any(|(cw, _)| cw.distance(wp) < OBSTACLE_CLEAR_M) {
        return true;
    }
    if let Some(indices) = metro.towers_by_block.get(&(block.ix, block.iz)) {
        for &idx in indices {
            if metro.damage.tower_is_replaced(idx) {
                continue;
            }
            let t = &metro.towers[idx];
            let f = Vec2::new(-t.yaw.sin(), -t.yaw.cos());
            let door = t.pos + f * t.half.x;
            if door.distance(wp) < OBSTACLE_CLEAR_M {
                return true;
            }
        }
    }
    false
}

fn emit_line_props(
    _metro: &MetroWorld,
    cfg: &LifeConfig,
    segments: &[Vec<Option<PlannedPole>>],
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
    out_props: &mut Vec<PropInstance>,
    out_spans: &mut Vec<WireSpan>,
) {
    let max_span = cfg.pole_spacing_m * MAX_SPAN_FACTOR;
    for seg in segments {
        for (i, slot) in seg.iter().enumerate() {
            let Some(pole) = slot else { continue };
            if in_chunk(pole.pos.x, pole.pos.y, chunk) {
                let model = match pole.state {
                    PoleState::Standing => "power_pole",
                    PoleState::Leaning => "power_pole_leaning",
                    PoleState::Fallen => "power_pole_fallen",
                };
                let y = height(pole.pos.x, pole.pos.y);
                out_props.push(PropInstance::new(model, Vec3::new(pole.pos.x, y, pole.pos.y), pole.yaw, PROP_SCALE));
            }
            if i + 1 >= seg.len() {
                continue;
            }
            let Some(next) = seg[i + 1] else { continue };
            let span_vec = next.pos - pole.pos;
            let span_len = span_vec.length();
            if span_len <= ENDPOINT_EPS || span_len > max_span {
                continue;
            }
            let dir = span_vec / span_len;
            let yaw = yaw_for_x(dir);
            let scale = span_len / WIRE_MODEL_LEN_M;
            let mid = (pole.pos + next.pos) * HALF;

            match (pole.state, next.state) {
                (PoleState::Fallen, _) | (_, PoleState::Fallen) => {
                    if in_chunk(pole.pos.x, pole.pos.y, chunk) {
                        let y = height(pole.pos.x, pole.pos.y);
                        out_props.push(PropInstance::new(
                            "power_wire_down",
                            Vec3::new(pole.pos.x, y, pole.pos.y),
                            yaw,
                            scale,
                        ));
                    }
                }
                _ => {
                    let leaning = pole.state == PoleState::Leaning || next.state == PoleState::Leaning;
                    let drop = if leaning { LEANING_WIRE_DROP_M } else { 0.0 };
                    let y_base = height(pole.pos.x, pole.pos.y) - drop;
                    if in_chunk(pole.pos.x, pole.pos.y, chunk) {
                        out_props.push(PropInstance::new(
                            "power_wire_span",
                            Vec3::new(pole.pos.x, y_base, pole.pos.y),
                            yaw,
                            scale,
                        ));
                    }
                    if in_chunk(mid.x, mid.y, chunk) {
                        out_spans.push(WireSpan { a: pole.pos, b: next.pos, y_base });
                    }
                }
            }
        }
    }
}

fn find_avenue_segment(streets: &[MetroStreet], a: Vec2, b: Vec2) -> Option<&MetroStreet> {
    streets.iter().find(|s| {
        s.avenue && ((s.a.distance(a) < ENDPOINT_EPS && s.b.distance(b) < ENDPOINT_EPS)
            || (s.a.distance(b) < ENDPOINT_EPS && s.b.distance(a) < ENDPOINT_EPS))
    })
}

fn endpoint_trim(p: Vec2, dir: Vec2, streets: &[MetroStreet]) -> f32 {
    streets
        .iter()
        .filter(|o| {
            (o.b - o.a).normalize_or_zero().dot(dir).abs() < HALF
                && (o.a.distance(p) < ENDPOINT_EPS || o.b.distance(p) < ENDPOINT_EPS || o.contains_point(p))
        })
        .map(|o| o.width * HALF)
        .fold(0.0_f32, f32::max)
}
