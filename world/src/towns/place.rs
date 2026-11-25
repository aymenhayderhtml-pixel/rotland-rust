use std::f32::consts::TAU;

use glam::Vec2;

use crate::biome::BiomeId;
use crate::biome::layout::WorldLayout;
use crate::towns::lots::generate_lots;
use crate::towns::{PlacementConfig, TownSite, TownsData};

/// Minimum squared length before normalizing a 2D direction vector.
// new: DECISIONS 2026-11-25
const DIR_EPS_SQ: f32 = 1e-6;

/// Base angular sample count on ring 1 of the flat-ground spiral search (ring `k` uses `8 * k`).
// new: DECISIONS 2026-11-25
const SPIRAL_BASE_SAMPLES: usize = 8;

/// Extracted world-layout parameters needed for pure town placement.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutFrame {
    pub reference_dir: Vec2,
    pub min_outer_radius: f32,
    pub sector_angles_deg: Vec<(BiomeId, f32)>,
}

/// Places all towns defined in `data` into their outer biome sectors outside `core_radius + blend_band`.
pub fn place_towns(
    data: &TownsData,
    layout: &WorldLayout,
    seed: i32,
    origin: Vec2,
    flat_check: &dyn Fn(Vec2, f32) -> bool,
) -> Vec<TownSite> {
    let frame = layout_frame(layout);
    place_towns_in_frame(data, &frame, seed, origin, flat_check)
}

/// Places all towns using an explicit `LayoutFrame` (shared core used by `place_towns` and unit tests).
pub fn place_towns_in_frame(
    data: &TownsData,
    frame: &LayoutFrame,
    seed: i32,
    origin: Vec2,
    flat_check: &dyn Fn(Vec2, f32) -> bool,
) -> Vec<TownSite> {
    let mut sites = Vec::with_capacity(data.towns.len());

    for def in &data.towns {
        let Some(biome) = def.biome_id() else {
            eprintln!("[towns] skipping town '{}' with unknown biome '{}'", def.key, def.biome);
            continue;
        };

        let sector_dir = sector_center_dir(frame, biome);
        let ideal_center = origin + sector_dir * def.distance_m;

        let (center, found_flat) = find_flat_center(
            ideal_center,
            origin,
            def.radius,
            frame.min_outer_radius,
            &data.placement,
            flat_check,
        );

        if !found_flat {
            eprintln!(
                "[towns] no flat site found for '{}' within {:.0} m of ({:.1}, {:.1}); using fallback center",
                def.key, data.placement.spiral_max_m, ideal_center.x, ideal_center.y
            );
        }

        let to_origin = origin - center;
        let street_dir = if to_origin.length_squared() > DIR_EPS_SQ {
            to_origin.normalize()
        } else {
            -sector_dir
        };

        let town_seed = derive_town_seed(seed, &def.key);
        let lots = generate_lots(def, data, center, street_dir, town_seed);

        sites.push(TownSite {
            key: def.key.clone(),
            name: def.name.clone(),
            biome,
            center,
            radius: def.radius,
            street_dir,
            lots,
        });
    }

    sites
}

/// Searches `start` then concentric rings (`spiral_step_m` up to `spiral_max_m`) for the first
/// flat site outside `min_outer_radius + town_radius`. Returns `(pos, found_flat)`.
fn find_flat_center(
    start: Vec2,
    origin: Vec2,
    town_radius: f32,
    min_outer_radius: f32,
    cfg: &PlacementConfig,
    flat_check: &dyn Fn(Vec2, f32) -> bool,
) -> (Vec2, bool) {
    let min_center_dist = min_outer_radius + town_radius;
    if start.distance(origin) > min_center_dist && flat_check(start, town_radius) {
        return (start, true);
    }

    let step = cfg.spiral_step_m; // data: towns.json placement.spiral_step_m
    let max_dist = cfg.spiral_max_m; // data: towns.json placement.spiral_max_m
    if step <= 0.0 || max_dist < step {
        return (start, false);
    }

    let rings = (max_dist / step).floor() as usize;
    for ring in 1..=rings {
        let r = (ring as f32) * step;
        let samples = SPIRAL_BASE_SAMPLES * ring;
        let d_theta = TAU / (samples as f32);
        for i in 0..samples {
            let theta = (i as f32) * d_theta;
            let candidate = start + Vec2::new(theta.cos(), theta.sin()) * r;
            if candidate.distance(origin) > min_center_dist && flat_check(candidate, town_radius) {
                return (candidate, true);
            }
        }
    }

    (start, false)
}

/// Computes the unit XZ direction of `biome`'s sector centre by rotating `reference_dir`
/// counter-clockwise when viewed from above (`+Y` up), per `docs/WORLD.md`.
pub fn sector_center_dir(frame: &LayoutFrame, biome: BiomeId) -> Vec2 {
    let angle_deg = frame
        .sector_angles_deg
        .iter()
        .find_map(|&(b, deg)| (b == biome).then_some(deg))
        .unwrap_or(0.0); // data: world_layout.json sectors[0].center_deg
    let base = if frame.reference_dir.length_squared() > DIR_EPS_SQ {
        frame.reference_dir.normalize()
    } else {
        Vec2::X
    };
    rotate_xz_ccw_from_above(base, angle_deg)
}

/// Rotates an XZ vector counter-clockwise by `angle_deg` when viewed from `+Y` looking down.
pub fn rotate_xz_ccw_from_above(dir: Vec2, angle_deg: f32) -> Vec2 {
    let rad = angle_deg.to_radians();
    let cos = rad.cos();
    let sin = rad.sin();
    // +X right, -Z up from +Y view: (x, -z) rotated CCW by +rad gives (x*cos + z*sin, -x*sin + z*cos).
    Vec2::new(dir.x * cos + dir.y * sin, -dir.x * sin + dir.y * cos).normalize_or_zero()
}

/// Derives a deterministic per-town RNG seed from the world seed and town key.
fn derive_town_seed(seed: i32, key: &str) -> i32 {
    // new: DECISIONS 2026-11-25 (FNV-1a 32-bit offset & prime for deterministic key mixing)
    let mut hash: u32 = 2_166_136_261;
    for &b in key.as_bytes() {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(16_777_619); // new: DECISIONS 2026-11-25
    }
    seed.wrapping_add(hash as i32)
}

/// Adapter extracting `LayoutFrame` from `WorldLayout`.
/// Verify field/method names against `world/src/biome/layout.rs` (see NOTES).
fn layout_frame(layout: &WorldLayout) -> LayoutFrame {
    let data = layout.data();
    let sector_angles_deg = data
        .sectors
        .iter()
        .map(|s| (s.biome, s.center_deg))
        .collect();
    LayoutFrame {
        reference_dir: layout.reference_dir(),
        min_outer_radius: data.core_radius + data.blend_band,
        sector_angles_deg,
    }
}
