//! Collider hulls of the placeholder towers: one box per tower in a chunk. Towers are solid.

use glam::{IVec2, Vec3};
use world::metro::TowerSpot;

use super::near::{base_point, towers_in_chunk};

/// One box hull per tower in chunk `coord`: (base centre, yaw, 8 corner points in tower-local space).
pub fn tower_hulls(towers: &[TowerSpot], coord: IVec2, ground_y: f32) -> Vec<(Vec3, f32, Vec<[f32; 3]>)> {
    towers_in_chunk(towers, coord)
        .map(|t| (base_point(t, ground_y), t.yaw, box_points(t)))
        .collect()
}

/// The eight corners of a tower's footprint box, from the base (y = 0) to `height`.
pub fn box_points(t: &TowerSpot) -> Vec<[f32; 3]> {
    let mut pts = Vec::with_capacity(8);
    for y in [0.0, t.height] {
        for x in [-t.half.x, t.half.x] {
            for z in [-t.half.y, t.half.y] {
                pts.push([x, y, z]);
            }
        }
    }
    pts
}
