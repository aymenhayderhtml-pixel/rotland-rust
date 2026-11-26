//! Survivor trace placement for New Halden: street-end car walls, facade graffiti and
//! missing-person posters, rooftop camps, sidewalk bodies and lobby dressing. Pure logic: no GPU.

use std::collections::HashSet;
use std::f32::consts::SQRT_2;
use std::sync::OnceLock;

use glam::{Vec2, Vec3};
use serde::Deserialize;

use crate::metro::config::MetroConfig;
use crate::metro::layout::{MetroBlock, MetroDistrict, MetroLayout, MetroStreet};
use crate::metro::place::MetroSite;
use crate::metro::state::MetroWorld;
use crate::metro::styles::default_styles;
use crate::random::DotnetRandom;
use crate::scatter::PropInstance;

#[path = "traces_camps.rs"]
mod camps;
#[cfg(test)]
#[path = "traces_tests.rs"]
mod tests;

pub use camps::{CampSite, camp_sites, lobby_dressing};

/// One chunk is 64 m square, matching `jam_chunks.rs` and city blocks.
pub(super) const CHUNK_SIZE_M: f32 = 64.0; // data: metro.json blockSize
/// Trace props are built at full world scale.
pub(super) const PROP_SCALE: f32 = 1.0;
const AVENUE_COUNT: usize = 2; // new: DECISIONS 2026-11-26 (the first two avenues by the entrance)
const FNV_OFFSET: u32 = 2_166_136_261; // new: DECISIONS 2026-11-26
const FNV_PRIME: u32 = 16_777_619; // new: DECISIONS 2026-11-26
const CAR_WALL_MODEL: &str = "car_wall_stack";
/// Minimum clear length beyond `carWalls.inM` so the car wall never touches a cross street.
const WALL_END_CLEARANCE_M: f32 = 10.0; // new: DECISIONS 2026-11-26 (clears cross-street width)

/// Settings for street-end car barricades (`traces.carWalls`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CarWallsConfig {
    pub count: [usize; 2],
    pub min_district: String,
    pub in_m: [f32; 2],
}

impl Default for CarWallsConfig {
    fn default() -> Self {
        Self {
            count: [2, 3], // data: metro.json traces
            min_district: "edge".into(), // data: metro.json traces
            in_m: [20.0, 40.0], // data: metro.json traces
        }
    }
}

/// One candidate interior dressing piece for a tower lobby (`traces.lobbyDressing`).
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LobbyDressingEntry {
    pub model: String,
    pub chance: f32,
    pub y: f32,
    pub x: f32,
    pub z: f32,
    pub yaw: f32,
}

/// Survivor trace settings (`traces` in `data/metro.json`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TraceConfig {
    pub car_walls: CarWallsConfig,
    pub wall_width_m: f32,
    pub gap_m: f32,
    pub wall_offset_m: f32,
    pub graffiti_lobby_chance: f32,
    pub graffiti_street_wall_chance: f32,
    pub graffiti_height_m: [f32; 2],
    pub poster_chance: f32,
    pub poster_height_m: [f32; 2],
    pub poster_x_share: [f32; 2],
    pub camp_chance: f32,
    pub camp_floors: [u32; 2],
    pub camp_size_m: f32,
    pub camp_margin_m: f32,
    pub body_sidewalk_chance: f32,
    pub lobby_ceiling_y: f32,
    pub lobby_dressing: Vec<LobbyDressingEntry>,
}

impl Default for TraceConfig {
    fn default() -> Self {
        let e = |model: &str, chance: f32, y: f32, x: f32, z: f32, yaw: f32| LobbyDressingEntry {
            model: model.into(),
            chance,
            y,
            x,
            z,
            yaw,
        };
        Self {
            car_walls: CarWallsConfig::default(),
            wall_width_m: 10.0,
            gap_m: 2.0,
            wall_offset_m: 0.02, // data: metro.json traces
            graffiti_lobby_chance: 0.35, // data: metro.json traces
            graffiti_street_wall_chance: 0.12, // data: metro.json traces
            graffiti_height_m: [1.0, 3.0],
            poster_chance: 0.3, // data: metro.json traces
            poster_height_m: [1.2, 2.0], // data: metro.json traces
            poster_x_share: [0.3, 0.4], // data: metro.json traces
            camp_chance: 0.15, // data: metro.json traces
            camp_floors: [4, 8], // data: metro.json traces
            camp_size_m: 6.0, // data: metro.json traces
            camp_margin_m: 1.0,
            body_sidewalk_chance: 0.35, // data: metro.json traces
            lobby_ceiling_y: 7.3, // data: metro.json traces
            lobby_dressing: vec![
                e("lobby_column", 0.65, 0.1, -0.22, 0.18, 0.0), // data: metro.json traces
                e("lobby_column", 0.65, 0.1, 0.22, -0.18, 0.0), // data: metro.json traces
                e("lobby_ceiling_hang", 0.45, 5.8, 0.0, -0.10, 0.2), // data: metro.json traces
                e("lobby_light_broken", 0.50, 5.6, 0.12, 0.08, -0.3), // data: metro.json traces
                e("lobby_rug", 0.55, 0.1, 0.0, -0.22, 0.0), // data: metro.json traces
                e("lobby_chair_tipped", 0.40, 0.1, 0.16, -0.15, 1.1), // data: metro.json traces
                e("lobby_debris", 0.50, 0.1, -0.14, -0.08, 0.7), // data: metro.json traces
                e("body_sheet_b", 0.18, 0.1, 0.20, -0.24, 0.5), // data: metro.json traces
            ],
        }
    }
}

impl TraceConfig {
    /// Checks all chances, ranges and lobby bounds.
    pub fn validate(&self) -> Result<(), String> {
        let chances = [
            ("graffitiLobbyChance", self.graffiti_lobby_chance),
            ("graffitiStreetWallChance", self.graffiti_street_wall_chance),
            ("posterChance", self.poster_chance),
            ("campChance", self.camp_chance),
            ("bodySidewalkChance", self.body_sidewalk_chance),
        ];
        for (name, c) in chances {
            if !(0.0..=1.0).contains(&c) {
                return Err(format!("metro.json traces.{name} ({c}) must be within 0..1"));
            }
        }
        if self.car_walls.count[0] > self.car_walls.count[1]
            || self.car_walls.in_m[0] <= 0.0
            || self.car_walls.in_m[0] > self.car_walls.in_m[1]
            || !matches!(self.car_walls.min_district.as_str(), "edge" | "midtown" | "core")
        {
            return Err("metro.json traces.carWalls is invalid".into());
        }
        if self.wall_width_m <= 0.0
            || self.gap_m <= 0.0
            || self.wall_offset_m <= 0.0
            || self.graffiti_height_m[0] < 0.0
            || self.graffiti_height_m[0] > self.graffiti_height_m[1]
            || self.poster_height_m[0] < 0.0
            || self.poster_height_m[0] > self.poster_height_m[1]
            || self.poster_x_share[0] <= 0.0
            || self.poster_x_share[0] > self.poster_x_share[1]
            || self.poster_x_share[1] >= 0.5
            || self.camp_floors[0] > self.camp_floors[1]
            || self.camp_size_m <= 0.0
            || self.camp_margin_m < 0.0
            || self.lobby_ceiling_y <= 0.0
        {
            return Err("metro.json traces: dimensions or ranges are invalid".into());
        }
        for e in &self.lobby_dressing {
            if e.model.trim().is_empty()
                || !(0.0..=1.0).contains(&e.chance)
                || e.y < 0.0
                || e.y > self.lobby_ceiling_y
                || e.x.abs() >= 0.5
                // new: DECISIONS 2026-11-26 (keep lobby props clear of front door at z < 0.3)
                || e.z >= 0.3
                || e.z <= -0.5
            {
                return Err(format!("metro.json traces.lobbyDressing entry '{}' is invalid", e.model));
            }
        }
        Ok(())
    }
}

#[derive(Deserialize)]
struct TracesWrapper {
    traces: TraceConfig,
}

/// Parses and validates the top-level `traces` object from a `metro.json` string.
pub fn load_traces(json: &str) -> Result<TraceConfig, String> {
    let wrapper: TracesWrapper =
        serde_json::from_str(json).map_err(|e| format!("metro.json traces: {e}"))?;
    wrapper.traces.validate()?;
    Ok(wrapper.traces)
}

/// Cached `TraceConfig` loaded from `data/metro.json` (uses `TraceConfig::default()` if the
/// `"traces"` key is not yet in the file).
pub fn default_traces() -> &'static TraceConfig {
    static CACHE: OnceLock<TraceConfig> = OnceLock::new();
    CACHE.get_or_init(|| {
        let path = paths::repo_path("data/metro.json");
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        let root: serde_json::Value = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()));
        if root.get("traces").is_none() {
            return TraceConfig::default();
        }
        load_traces(&json).expect("data/metro.json traces must be valid")
    })
}

/// Lists `(world_xz, yaw)` of every planned street-end car barricade in New Halden.
pub fn car_wall_sites(metro: &MetroWorld) -> Vec<(Vec2, f32)> {
    let cfg = default_traces();
    let entrance_aves = entrance_avenue_indices(metro);
    plan_car_walls(metro, cfg, &entrance_aves)
        .into_iter()
        .map(|(_, wx, wz, yaw)| (Vec2::new(wx, wz), yaw))
        .collect()
}

/// Emits every survivor-trace prop (`car_wall_stack`, graffiti, posters, rooftop camps, sidewalk
/// bodies, lobby dressing) whose XZ centre falls inside chunk `chunk`.
pub fn traces_props_in_chunk(
    metro: &MetroWorld,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
) -> Vec<PropInstance> {
    let cfg = default_traces();
    let styles = default_styles();
    let entrance_aves = entrance_avenue_indices(metro);
    let mut out = Vec::new();

    for (model, wx, wz, yaw) in plan_car_walls(metro, cfg, &entrance_aves) {
        if in_chunk(wx, wz, chunk) {
            out.push(PropInstance::new(model, Vec3::new(wx, height(wx, wz), wz), yaw, PROP_SCALE));
        }
    }

    let chunk_center = Vec2::new(
        (chunk.0 as f32 + 0.5) * CHUNK_SIZE_M,
        (chunk.1 as f32 + 0.5) * CHUNK_SIZE_M,
    );
    let tower_reach_pad = CHUNK_SIZE_M * 0.5 + 4.0; // new: DECISIONS 2026-11-26 (chunk half + wall offset)
    for (idx, tower) in metro.towers.iter().enumerate() {
        if metro.damage.tower_is_replaced(idx)
            || (tower.pos - chunk_center).abs().max_element() > tower_reach_pad + tower.half.x * SQRT_2
        {
            continue;
        }
        camps::append_tower_facade_traces(metro, idx, tower, cfg, chunk, height, &mut out);
        if !metro.damage.tower_is_burnt(tower.block) {
            let ground_y = height(tower.pos.x, tower.pos.y);
            if let Some(camp) = camps::camp_for_tower(metro, idx, tower, cfg, styles, ground_y)
                && in_chunk(camp.pos.x, camp.pos.z, chunk)
            {
                out.push(PropInstance::new("rooftop_camp", camp.pos, camp.yaw, PROP_SCALE));
            }
            camps::append_lobby_dressing(metro, idx, tower, cfg, chunk, height, &mut out);
        }
    }

    let block_reach = CHUNK_SIZE_M * 0.5 + metro.config.block_size * SQRT_2 * 0.5 + 2.0;
    for block in &metro.layout.blocks {
        if block.reserved.is_some() {
            continue;
        }
        let b_center = metro.layout.site.to_world((block.rect.0 + block.rect.1) * 0.5);
        if (b_center - chunk_center).abs().max_element() <= block_reach {
            camps::append_sidewalk_bodies(metro, block, cfg, &entrance_aves, chunk, height, &mut out);
        }
    }

    out
}

/// Plans `carWalls.count` barricades at plain street ends near the city edge.
fn plan_car_walls(
    metro: &MetroWorld,
    cfg: &TraceConfig,
    entrance_aves: &HashSet<usize>,
) -> Vec<(&'static str, f32, f32, f32)> {
    let layout = &metro.layout;
    let half = layout.site.half_size;
    let min_len = cfg.car_walls.in_m[1] + WALL_END_CLEARANCE_M;
    let mut candidates: Vec<(Vec2, Vec2, f32)> = Vec::new();
    for (idx, s) in layout.streets.iter().enumerate() {
        if s.avenue || entrance_aves.contains(&idx) || is_damage_street(metro, s) {
            continue;
        }
        if s.width + 1e-3 < cfg.wall_width_m + cfg.gap_m || s.a.distance(s.b) < min_len {
            continue;
        }
        let Some((edge_end, other_end)) = perimeter_street_ends(s, half) else { continue };
        let adj = adjacent_blocks(layout, &metro.config, s);
        if adj.is_empty()
            || adj.iter().any(|b| b.reserved.is_some())
            || !matches_min_district(&adj, &cfg.car_walls.min_district)
        {
            continue;
        }
        candidates.push((edge_end, other_end, s.width));
    }
    if candidates.is_empty() {
        return Vec::new();
    }
    let mut rng = DotnetRandom::new(mix_seed(&layout.site, None, b"traces_walls"));
    let [lo, hi] = cfg.car_walls.count;
    let want = (rng.next_range(lo as i32, hi as i32 + 1) as usize).min(candidates.len());
    let mut out = Vec::with_capacity(want);
    for _ in 0..want {
        let pick = rng.next(candidates.len() as i32) as usize;
        let (edge_end, other_end, width) = candidates.swap_remove(pick);
        let in_m = rng_range_f(&mut rng, cfg.car_walls.in_m);
        let along_local = edge_end + (other_end - edge_end).normalize() * in_m;
        let inward_w = (layout.site.to_world(other_end) - layout.site.to_world(edge_end)).normalize();
        let right_w = right_of(inward_w);
        let gap_sign = if rng.next(2) == 0 { -1.0 } else { 1.0 };
        let pos_w = layout.site.to_world(along_local) - right_w * gap_sign * ((width - cfg.wall_width_m) * 0.5);
        out.push((CAR_WALL_MODEL, pos_w.x, pos_w.y, dir_yaw(inward_w * gap_sign)));
    }
    out
}

fn perimeter_street_ends(s: &MetroStreet, half: f32) -> Option<(Vec2, Vec2)> {
    let on_edge = |p: Vec2| p.x.abs() >= half - 1.0 || p.y.abs() >= half - 1.0;
    match (on_edge(s.a), on_edge(s.b)) {
        (true, false) | (true, true) => Some((s.a, s.b)),
        (false, true) => Some((s.b, s.a)),
        (false, false) => None,
    }
}

fn adjacent_blocks<'a>(layout: &'a MetroLayout, cfg: &MetroConfig, s: &MetroStreet) -> Vec<&'a MetroBlock> {
    let half = layout.site.half_size;
    let bs = cfg.block_size;
    let mid = (s.a + s.b) * 0.5;
    let coords = if (s.b.x - s.a.x).abs() < (s.b.y - s.a.y).abs() {
        let i = ((mid.x + half) / bs).round() as i32;
        let by = ((mid.y + half) / bs).floor() as i32;
        [(i - 1, by), (i, by)]
    } else {
        let j = ((mid.y + half) / bs).round() as i32;
        let bx = ((mid.x + half) / bs).floor() as i32;
        [(bx, j - 1), (bx, j)]
    };
    coords.into_iter().filter_map(|(ix, iz)| layout.block_at(ix, iz)).collect()
}

fn matches_min_district(adj: &[&MetroBlock], min_district: &str) -> bool {
    match min_district {
        "edge" => adj.iter().all(|b| b.district == MetroDistrict::Edge),
        "midtown" => adj.iter().any(|b| matches!(b.district, MetroDistrict::Edge | MetroDistrict::Midtown)),
        _ => true,
    }
}

pub(super) fn entrance_avenue_indices(metro: &MetroWorld) -> HashSet<usize> {
    let Some(&entrance) = metro.highway.points.last() else {
        return HashSet::new();
    };
    let mut found: Vec<(f32, usize)> = metro
        .layout
        .streets
        .iter()
        .enumerate()
        .filter(|(_, s)| s.avenue && metro.layout.contains(metro.layout.site.to_world((s.a + s.b) * 0.5)))
        .map(|(i, s)| {
            let wa = metro.layout.site.to_world(s.a);
            let wb = metro.layout.site.to_world(s.b);
            (wa.distance(entrance).min(wb.distance(entrance)), i)
        })
        .collect();
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    found.into_iter().take(AVENUE_COUNT).map(|(_, i)| i).collect()
}

pub(super) fn is_damage_street(metro: &MetroWorld, s: &MetroStreet) -> bool {
    let has_fallen = metro.damage.fallen.pos != Vec2::ZERO || metro.damage.fallen.block != (0, 0);
    has_fallen && s.contains_point(metro.layout.site.to_local(metro.damage.fallen.pos))
}

pub(super) fn in_chunk(x: f32, z: f32, chunk: (i32, i32)) -> bool {
    (x / CHUNK_SIZE_M).floor() as i32 == chunk.0 && (z / CHUNK_SIZE_M).floor() as i32 == chunk.1
}

pub(super) fn forward_of(yaw: f32) -> Vec2 {
    Vec2::new(-yaw.sin(), -yaw.cos())
}

pub(super) fn right_of(d: Vec2) -> Vec2 {
    Vec2::new(-d.y, d.x)
}

pub(super) fn dir_yaw(d: Vec2) -> f32 {
    (-d.x).atan2(-d.y)
}

pub(super) fn rng_range_f(rng: &mut DotnetRandom, range: [f32; 2]) -> f32 {
    range[0] + (range[1] - range[0]) * rng.next_double() as f32
}

pub(super) fn mix_seed(site: &MetroSite, block: Option<(i32, i32)>, salt: &[u8]) -> i32 {
    let mut hash = FNV_OFFSET;
    let mut feed = |bytes: &[u8]| {
        for &b in bytes {
            hash ^= u32::from(b);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    };
    feed(site.key.as_bytes());
    feed(&site.center.x.to_bits().to_le_bytes());
    feed(&site.center.y.to_bits().to_le_bytes());
    if let Some((ix, iz)) = block {
        feed(&ix.to_le_bytes());
        feed(&iz.to_le_bytes());
    }
    feed(salt);
    hash as i32
}

pub(super) fn tower_seed(site: &MetroSite, idx: usize, block: (i32, i32), salt: &[u8]) -> i32 {
    let mut bytes = Vec::with_capacity(salt.len() + std::mem::size_of::<u64>());
    bytes.extend_from_slice(&(idx as u64).to_le_bytes());
    bytes.extend_from_slice(salt);
    mix_seed(site, Some(block), &bytes)
}
