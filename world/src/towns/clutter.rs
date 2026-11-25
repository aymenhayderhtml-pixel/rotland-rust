use std::collections::BTreeMap;
use std::f32::consts::{FRAC_PI_2, PI};
use std::sync::OnceLock;

use glam::{IVec2, Vec2, Vec3};
use serde::Deserialize;

use crate::generator::WorldGenerator;
use crate::random::DotnetRandom;
use crate::roads::RoadKind;
use crate::scatter::PropInstance;
use crate::towns::{BOARDWALK_MODEL, TownLot, TownSite};

// data: roads.json highway_width
const HIGHWAY_HALF_WIDTH_M: f32 = 4.0;
// data: roads.json trail_width
const TRAIL_HALF_WIDTH_M: f32 = 2.0;
// new: DECISIONS 2026-11-25
const MAX_ATTEMPTS_PER_PROP: usize = 8;
// new: DECISIONS 2026-11-25
const YAW_JITTER_RAD: f32 = 0.25;
// new: DECISIONS 2026-11-25
const DOOR_SIDE_FRAC: f32 = 0.85;
// new: DECISIONS 2026-11-25
const SIDE_WALL_MIN_FRAC: f32 = -0.7;
// new: DECISIONS 2026-11-25
const SIDE_WALL_MAX_FRAC: f32 = 0.8;
// new: DECISIONS 2026-11-25
const DIR_EPS_SQ: f32 = 0.5;

/// Per-biome town clutter config loaded from the `clutter` block of `data/towns.json`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ClutterData {
    pub per_lot_min: usize,
    pub per_lot_max: usize,
    pub street_clear_m: f32,
    pub wall_offset_min_m: f32,
    pub wall_offset_max_m: f32,
    pub min_spacing_m: f32,
    pub road_clear_m: f32,
    pub scale_range: [f32; 2],
    pub biomes: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct TownsClutterWrapper {
    clutter: ClutterData,
}

impl ClutterData {
    /// Returns the cached `clutter` config from `data/towns.json`.
    pub fn load_embedded() -> &'static Self {
        static CACHE: OnceLock<ClutterData> = OnceLock::new();
        CACHE.get_or_init(|| {
            let raw = include_str!("../../../data/towns.json");
            let parsed: TownsClutterWrapper =
                serde_json::from_str(raw).expect("data/towns.json must contain valid clutter");
            parsed.clutter
        })
    }
}

/// Places 2–5 biome-appropriate clutter props around each building lot in `town`.
/// Returned `PropInstance`s have `y = 0.0`; callers snap `y` to terrain height per chunk.
pub fn place(town: &TownSite, data: &ClutterData, seed: i32) -> Vec<PropInstance> {
    let Some(models) = data.biomes.get(town.biome.key()) else {
        return Vec::new();
    };
    if models.is_empty() {
        return Vec::new();
    }

    let street_dir = town.street_dir.normalize_or_zero();
    let street_right = Vec2::new(-street_dir.y, street_dir.x);
    let min_spacing_sq = data.min_spacing_m * data.min_spacing_m;
    let mut rng = DotnetRandom::new(seed ^ hash_str(&town.key));
    let mut out = Vec::new();

    for lot in &town.lots {
        if lot.building == BOARDWALK_MODEL {
            continue;
        }
        let min_c = data.per_lot_min;
        let max_c = data.per_lot_max.max(min_c);
        let span = max_c - min_c + 1;
        let target = min_c + ((rng.next_double() * span as f64) as usize).min(span - 1);

        let mut placed_for_lot = 0usize;
        let max_attempts = target * MAX_ATTEMPTS_PER_PROP;
        for _ in 0..max_attempts {
            if placed_for_lot >= target {
                break;
            }
            let (pos, base_yaw) = candidate_spot(lot, data, &mut rng);
            if town.lots.iter().any(|other| other.contains_point(pos)) {
                continue;
            }
            if street_right.length_squared() > DIR_EPS_SQ
                && (pos - town.center).dot(street_right).abs() < data.street_clear_m
            {
                continue;
            }
            if out
                .iter()
                .any(|p: &PropInstance| Vec2::new(p.position.x, p.position.z).distance_squared(pos) < min_spacing_sq)
            {
                continue;
            }

            let idx = ((rng.next_double() * models.len() as f64) as usize).min(models.len() - 1);
            let scale = rng_range(&mut rng, data.scale_range[0], data.scale_range[1]);
            let yaw = base_yaw + rng_range(&mut rng, -YAW_JITTER_RAD, YAW_JITTER_RAD);
            out.push(PropInstance::new(
                &models[idx],
                Vec3::new(pos.x, 0.0, pos.y),
                yaw,
                scale,
            ));
            placed_for_lot += 1;
        }
    }

    out
}

/// Filters `place` output to `coord`, drops props on roads, and snaps `y` via `height`.
pub fn place_in_chunk(
    town: &TownSite,
    coord: IVec2,
    seed: i32,
    height: impl Fn(f32, f32) -> f32,
    road_dist: impl Fn(Vec2) -> Option<(f32, RoadKind)>,
) -> Vec<PropInstance> {
    let data = ClutterData::load_embedded();
    let mut props = place(town, data, seed);
    props.retain(|p| {
        let cx = (p.position.x / WorldGenerator::CHUNK_SIZE).floor() as i32;
        let cz = (p.position.z / WorldGenerator::CHUNK_SIZE).floor() as i32;
        if cx != coord.x || cz != coord.y {
            return false;
        }
        let xz = Vec2::new(p.position.x, p.position.z);
        if let Some((dist, kind)) = road_dist(xz) {
            let half_w = match kind {
                RoadKind::Highway => HIGHWAY_HALF_WIDTH_M,
                RoadKind::Trail => TRAIL_HALF_WIDTH_M,
            };
            if dist < half_w + data.road_clear_m {
                return false;
            }
        }
        true
    });
    for p in &mut props {
        p.position.y = height(p.position.x, p.position.z);
    }
    props
}

fn candidate_spot(lot: &TownLot, data: &ClutterData, rng: &mut DotnetRandom) -> (Vec2, f32) {
    let forward = lot.forward();
    let right = Vec2::new(-forward.y, forward.x);
    let offset = rng_range(rng, data.wall_offset_min_m, data.wall_offset_max_m);
    let roll = rng.next_double();
    // new: DECISIONS 2026-11-25 (50% door side, 20% left wall, 20% right wall, 10% back wall)
    if roll < 0.50 {
        let u = rng_range(rng, -DOOR_SIDE_FRAC, DOOR_SIDE_FRAC) * lot.footprint.x;
        let pos = lot.pos + right * u + forward * (lot.footprint.y + offset);
        (pos, lot.yaw)
    } else if roll < 0.70 { // new: DECISIONS 2026-11-25
        let v = rng_range(rng, SIDE_WALL_MIN_FRAC, SIDE_WALL_MAX_FRAC) * lot.footprint.y;
        let pos = lot.pos + right * (lot.footprint.x + offset) + forward * v;
        (pos, lot.yaw - FRAC_PI_2)
    } else if roll < 0.90 { // new: DECISIONS 2026-11-25
        let v = rng_range(rng, SIDE_WALL_MIN_FRAC, SIDE_WALL_MAX_FRAC) * lot.footprint.y;
        let pos = lot.pos - right * (lot.footprint.x + offset) + forward * v;
        (pos, lot.yaw + FRAC_PI_2)
    } else {
        let u = rng_range(rng, -DOOR_SIDE_FRAC, DOOR_SIDE_FRAC) * lot.footprint.x;
        let pos = lot.pos + right * u - forward * (lot.footprint.y + offset);
        (pos, lot.yaw + PI)
    }
}

fn rng_range(rng: &mut DotnetRandom, lo: f32, hi: f32) -> f32 {
    lo + (rng.next_double() as f32) * (hi - lo)
}

fn hash_str(s: &str) -> i32 {
    // new: DECISIONS 2026-11-25 (FNV-1a 32-bit for deterministic per-town seed offset)
    let mut h: u32 = 2166136261;
    for b in s.as_bytes() {
        h ^= *b as u32;
        h = h.wrapping_mul(16777619); // new: DECISIONS 2026-11-25
    }
    h as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::towns::TownsData;
    use crate::towns::lots::generate_lots;

    fn sample_towns() -> Vec<TownSite> {
        let towns_data = TownsData::load_embedded();
        let center = Vec2::new(200.0, -150.0);
        let street_dir = Vec2::new(0.8, -0.6).normalize();
        towns_data
            .towns
            .iter()
            .map(|def| {
                let biome = def.biome_id().expect("valid biome");
                let lots = generate_lots(def, &towns_data, center, street_dir, 42);
                TownSite {
                    key: def.key.clone(),
                    name: def.name.clone(),
                    biome,
                    center,
                    radius: def.radius,
                    street_dir,
                    lots,
                }
            })
            .collect()
    }

    #[test]
    fn clutter_placement_is_deterministic() {
        let data = ClutterData::load_embedded();
        let towns = sample_towns();
        for town in &towns {
            let a = place(town, data, 1234);
            let b = place(town, data, 1234);
            assert_eq!(a, b, "clutter for {} must be deterministic", town.key);
            assert!(!a.is_empty(), "expected clutter for {}", town.key);
        }
    }

    #[test]
    fn no_clutter_inside_lot_or_within_street_clear() {
        let data = ClutterData::load_embedded();
        let towns = sample_towns();
        for town in &towns {
            let street_dir = town.street_dir.normalize_or_zero();
            let street_right = Vec2::new(-street_dir.y, street_dir.x);
            let allowed = data
                .biomes
                .get(town.biome.key())
                .expect("biome clutter list");

            let props = place(town, data, 99);
            assert!(!props.is_empty(), "town {} should have clutter", town.key);
            for p in &props {
                let xz = Vec2::new(p.position.x, p.position.z);
                assert!(
                    !town.lots.iter().any(|lot| lot.contains_point(xz)),
                    "clutter {} in {} landed inside a lot footprint at {:?}",
                    p.model,
                    town.key,
                    xz
                );
                let street_dist = (xz - town.center).dot(street_right).abs();
                assert!(
                    street_dist >= data.street_clear_m,
                    "clutter {} in {} is {street_dist:.2} m from street centre (< {})",
                    p.model,
                    town.key,
                    data.street_clear_m
                );
                assert!(
                    allowed.contains(&p.model),
                    "clutter {} not in {} biome list",
                    p.model,
                    town.biome.key()
                );
            }
        }
    }
}
