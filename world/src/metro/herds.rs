//! Slow zombie herd routes along New Halden's avenues. Pure geometry, no GPU.

use std::collections::HashMap;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::metro::herds_grid::{GridPt, StreetGrid};
use crate::metro::herds_route::{build_closed_loop, mark_segment_used};
use crate::metro::layout::{MetroDistrict, MetroLayout};
use crate::random::DotnetRandom;

const FNV_OFFSET: u32 = 2_166_136_261; // new: DECISIONS 2026-11-27
const FNV_PRIME: u32 = 16_777_619; // new: DECISIONS 2026-11-27
pub(super) const LEN_EPS: f32 = 1e-4; // new: DECISIONS 2026-11-27
pub(super) const MIN_LOOP_LEGS: usize = 4; // data: metro.json herds.routeLegs[0]

/// Herd counts per New Halden district (`herds.count` in `data/metro.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HerdsDistrictCount {
    pub edge: usize,
    pub midtown: usize,
    pub core: usize,
}

impl Default for HerdsDistrictCount {
    fn default() -> Self {
        Self {
            edge: 2,    // data: metro.json herds.count.edge
            midtown: 4, // data: metro.json herds.count.midtown
            core: 4,    // data: metro.json herds.count.core
        }
    }
}

/// Settings for avenue zombie herds (`herds` in `data/metro.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HerdsConfig {
    pub count: HerdsDistrictCount,
    pub size: [u32; 2],
    pub speed: f32,
    pub route_legs: [usize; 2],
    pub active_radius: f32,
    pub despawn_radius: f32,
    pub spread_m: f32,
    pub aggro_share: f32,
}

impl Default for HerdsConfig {
    fn default() -> Self {
        Self {
            count: HerdsDistrictCount::default(),
            size: [12, 30],        // data: metro.json size
            speed: 0.55,           // data: metro.json speed
            route_legs: [4, 9],    // data: metro.json routeLegs
            active_radius: 170.0,  // data: metro.json activeRadius
            despawn_radius: 210.0, // data: metro.json despawnRadius
            spread_m: 7.0,         // data: metro.json spreadM
            aggro_share: 0.6,      // data: metro.json aggroShare
        }
    }
}

impl HerdsConfig {
    /// Checks that herd counts, ranges, speed and radii are valid.
    pub fn validate(&self) -> Result<(), String> {
        if self.size[0] == 0 || self.size[0] > self.size[1] {
            return Err("metro.json herds.size must have 1 <= min <= max".into());
        }
        if self.speed <= 0.0 || self.active_radius <= 0.0 || self.despawn_radius < self.active_radius {
            return Err("metro.json herds: speed/activeRadius must be > 0 and despawnRadius >= activeRadius".into());
        }
        if self.route_legs[0] < MIN_LOOP_LEGS || self.route_legs[0] > self.route_legs[1] {
            return Err("metro.json herds.routeLegs must have 4 <= min <= max".into());
        }
        if self.spread_m <= 0.0 || !(0.0..=1.0).contains(&self.aggro_share) {
            return Err("metro.json herds: spreadM > 0 and aggroShare in 0..1 required".into());
        }
        Ok(())
    }
}

/// A closed-loop walking route for one street herd (world XZ points).
#[derive(Debug, Clone, PartialEq)]
pub struct HerdRoute {
    pub id: u32,
    pub points: Vec<Vec2>,
    pub size: u32,
    pub speed: f32,
    pub zone: MetroDistrict,
}

/// Generates deterministic closed-loop herd routes per district.
/// Never enters reserved blocks or outer perimeter street ends.
pub fn herd_routes(layout: &MetroLayout, cfg: &HerdsConfig, seed: u64) -> Vec<HerdRoute> {
    herd_routes_avoiding(layout, cfg, &[], seed)
}

/// Generates closed-loop herd routes while avoiding world XZ points on blocked street segments
/// (such as the fallen-tower street, car-wall ends, and the highway entrance avenue's first block).
pub fn herd_routes_avoiding(
    layout: &MetroLayout,
    cfg: &HerdsConfig,
    blocked_world_points: &[Vec2],
    seed: u64,
) -> Vec<HerdRoute> {
    let Some(grid) = StreetGrid::from_layout(layout, blocked_world_points) else {
        return Vec::new();
    };
    let mut rng = DotnetRandom::new(mix_seed(seed));
    let mut out = Vec::new();
    let mut edge_uses: HashMap<(GridPt, GridPt), usize> = HashMap::new();
    let mut next_id = 1u32;

    for (zone, want) in [
        (MetroDistrict::Edge, cfg.count.edge),
        (MetroDistrict::Midtown, cfg.count.midtown),
        (MetroDistrict::Core, cfg.count.core),
    ] {
        let edges = grid.edges_for_district(zone);
        if edges.is_empty() {
            continue;
        }
        for _ in 0..want {
            let want_legs = rng.next_range(cfg.route_legs[0] as i32, cfg.route_legs[1] as i32 + 1) as usize;
            let size = rng.next_range(cfg.size[0] as i32, cfg.size[1] as i32 + 1) as u32;
            if let Some(loop_pts) = build_closed_loop(&grid, &edges, want_legs, &edge_uses, &mut rng) {
                for pair in loop_pts.windows(2) {
                    mark_segment_used(&mut edge_uses, pair[0], pair[1]);
                }
                if let (Some(&last), Some(&first)) = (loop_pts.last(), loop_pts.first()) {
                    mark_segment_used(&mut edge_uses, last, first);
                }
                let points = loop_pts.into_iter().map(|p| grid.to_world(p)).collect();
                out.push(HerdRoute { id: next_id, points, size, speed: cfg.speed, zone });
                next_id += 1;
            }
        }
    }
    out
}

/// Returns `(world_xz, yaw)` on `route` at `time_s` (looped, Godot yaw: forward = `(-sin yaw, -cos yaw)`).
pub fn route_pos(route: &HerdRoute, time_s: f64) -> (Vec2, f32) {
    let n = route.points.len();
    if n == 0 {
        return (Vec2::ZERO, 0.0);
    }
    if n == 1 {
        return (route.points[0], 0.0);
    }
    let mut total_len = 0.0_f32;
    for i in 0..n {
        total_len += route.points[i].distance(route.points[(i + 1) % n]);
    }
    if total_len <= LEN_EPS {
        return (route.points[0], 0.0);
    }
    let dist = (time_s * f64::from(route.speed)).rem_euclid(f64::from(total_len)) as f32;
    let mut rem = dist;
    for i in 0..n {
        let a = route.points[i];
        let b = route.points[(i + 1) % n];
        let seg = a.distance(b);
        if seg <= LEN_EPS {
            continue;
        }
        if rem <= seg || i + 1 == n {
            let t = (rem / seg).clamp(0.0, 1.0);
            let dir = (b - a) / seg;
            return (a.lerp(b, t), (-dir.x).atan2(-dir.y));
        }
        rem -= seg;
    }
    (route.points[0], 0.0)
}

fn mix_seed(seed: u64) -> i32 {
    let mut hash = FNV_OFFSET;
    for b in seed.to_le_bytes() {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash as i32
}
