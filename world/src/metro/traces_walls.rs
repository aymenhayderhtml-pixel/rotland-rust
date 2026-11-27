//! Street-end car barricade planning for `traces.rs`. Pure logic: no GPU.

use std::collections::HashSet;

use glam::Vec2;

use crate::metro::config::MetroConfig;
use crate::metro::layout::{MetroBlock, MetroDistrict, MetroLayout, MetroStreet};
use crate::metro::state::MetroWorld;
use crate::random::DotnetRandom;

use super::{
    TraceConfig, default_traces, dir_yaw, entrance_avenue_indices, is_damage_street, mix_seed,
    right_of, rng_range_f,
};

pub(super) const CAR_WALL_MODEL: &str = "car_wall_stack";
/// Minimum clear length beyond `carWalls.inM` so the car wall never touches a cross street.
const WALL_END_CLEARANCE_M: f32 = 10.0; // new: DECISIONS 2026-11-26 (clears cross-street width)

/// Lists `(world_xz, yaw)` of every planned street-end car barricade in New Halden.
pub fn car_wall_sites(metro: &MetroWorld) -> Vec<(Vec2, f32)> {
    let cfg = default_traces();
    let entrance_aves = entrance_avenue_indices(metro);
    plan_car_walls(metro, cfg, &entrance_aves)
        .into_iter()
        .map(|(_, wx, wz, yaw)| (Vec2::new(wx, wz), yaw))
        .collect()
}

/// Plans `carWalls.count` barricades at plain street ends near the city edge.
pub(super) fn plan_car_walls(
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
