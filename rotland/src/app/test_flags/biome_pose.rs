//! Teleport search for `--biome=<key>`: core biomes by ring search, outer biomes by sector ray.

use std::f32::consts::TAU;

use glam::Vec2;
use world::biome::layout::BiomeSample;
use world::BiomeId;

/// Extra metres past `core_radius + blend_band` when teleporting into an outer biome.
pub(super) const BIOME_TP_PAST_BLEND_M: f32 = 300.0; // new: DECISIONS 2026-11-25
/// Search radii (m) to detect where the outer sectors (`outer >= 1.0`) begin.
const OUTER_PROBE_RADII: [f32; 7] = [500.0, 1000.0, 2000.0, 3500.0, 6000.0, 10_000.0, 16_000.0]; // new: DECISIONS 2026-11-25
const SECTOR_SWEEP_STEPS: usize = 72; // new: DECISIONS 2026-11-25
const RAY_STEP_M: f32 = 25.0; // new: DECISIONS 2026-11-25
const RAY_MAX_M: f32 = 20_000.0; // new: DECISIONS 2026-11-25
const CORE_SEARCH_STEP_M: f32 = 32.0; // new: DECISIONS 2026-11-25
const CORE_SEARCH_MAX_M: f32 = 4000.0; // new: DECISIONS 2026-11-25

/// Finds a teleport target `(xz, godot_yaw)` for `--biome=<key>`, facing outward from `camp`.
///
/// - For `Plains` / `Forest`: nearest point inside the core (`outer == 0.0`) of that biome.
/// - For outer biomes (`Desert`, `Snow`, `Swamp`, `Mountains`): searches outward from `camp`
///   along the biome's sector centre angle to the first point at `core_radius + blend_band + 300 m`.
pub fn find_biome_point(
    camp: Vec2,
    target: BiomeId,
    mut sample_fn: impl FnMut(f32, f32) -> BiomeSample,
) -> Option<(Vec2, f32)> {
    if matches!(target, BiomeId::Plains | BiomeId::Forest) {
        return find_core_biome_point(camp, target, &mut sample_fn);
    }

    let probe_r = OUTER_PROBE_RADII.into_iter().find(|&r| {
        (0..8).any(|k| {
            let a = k as f32 * (TAU / 8.0);
            let p = camp + Vec2::new(a.cos(), a.sin()) * r;
            sample_fn(p.x, p.y).outer >= 1.0 - 1e-4
        })
    })?;

    let sweep_r = probe_r + 500.0; // new: DECISIONS 2026-11-25
    let mut sum_cos = 0.0f32;
    let mut sum_sin = 0.0f32;
    let mut hits = 0usize;
    for i in 0..SECTOR_SWEEP_STEPS {
        let angle = i as f32 * (TAU / SECTOR_SWEEP_STEPS as f32);
        let p = camp + Vec2::new(angle.cos(), angle.sin()) * sweep_r;
        let s = sample_fn(p.x, p.y);
        if s.outer >= 1.0 - 1e-4 && s.primary == target {
            sum_cos += angle.cos();
            sum_sin += angle.sin();
            hits += 1;
        }
    }
    if hits == 0 {
        return None;
    }

    let center_angle = sum_sin.atan2(sum_cos);
    let dir = Vec2::new(center_angle.cos(), center_angle.sin());

    let mut r = 0.0f32;
    let mut outer_edge = None;
    while r <= RAY_MAX_M {
        let p = camp + dir * r;
        if sample_fn(p.x, p.y).outer >= 1.0 - 1e-4 {
            outer_edge = Some(r);
            break;
        }
        r += RAY_STEP_M;
    }
    let mut hi = outer_edge?;
    let mut lo = (hi - RAY_STEP_M).max(0.0);
    for _ in 0..10 {
        let mid = 0.5 * (lo + hi);
        let p = camp + dir * mid;
        if sample_fn(p.x, p.y).outer >= 1.0 - 1e-4 {
            hi = mid;
        } else {
            lo = mid;
        }
    }

    let pos = camp + dir * (hi + BIOME_TP_PAST_BLEND_M);
    let yaw = (-dir.x).atan2(-dir.y);
    Some((pos, yaw))
}

fn find_core_biome_point(
    camp: Vec2,
    target: BiomeId,
    sample_fn: &mut impl FnMut(f32, f32) -> BiomeSample,
) -> Option<(Vec2, f32)> {
    let at_camp = sample_fn(camp.x, camp.y);
    if at_camp.outer <= 0.0 && at_camp.primary == target {
        return Some((camp, 0.0));
    }

    let mut r = CORE_SEARCH_STEP_M;
    while r <= CORE_SEARCH_MAX_M {
        let steps = ((TAU * r / CORE_SEARCH_STEP_M).round() as usize).clamp(12, 180);
        for i in 0..steps {
            let angle = i as f32 * (TAU / steps as f32);
            let dir = Vec2::new(angle.cos(), angle.sin());
            let p = camp + dir * r;
            let s = sample_fn(p.x, p.y);
            if s.outer <= 0.0 && s.primary == target {
                let yaw = (-dir.x).atan2(-dir.y);
                return Some((p, yaw));
            }
        }
        r += CORE_SEARCH_STEP_M;
    }
    None
}
