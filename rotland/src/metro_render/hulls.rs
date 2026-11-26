//! Collider hulls of towers: one solid upper box per tower (above the walk-in lobby) plus
//! the base model id/pose so its own wall colliders keep the lobby walkable.

use glam::{IVec2, Vec3};
use world::metro::{StyleDef, TowerSpot, style_for};

use super::near::{base_point, towers_in_chunk};
use super::parts::base_model_id;

/// Fallback lobby base height when a style is not found in `styles`.
const DEFAULT_BASE_HEIGHT: f32 = 7.6; // data: metro.json styles[].baseHeight

/// One upper box hull per tower in chunk `coord`: (base centre, yaw, 8 corner points in tower-local space).
pub fn tower_hulls(
    towers: &[TowerSpot],
    coord: IVec2,
    ground_y: f32,
    styles: &[StyleDef],
) -> Vec<(Vec3, f32, Vec<[f32; 3]>)> {
    towers_in_chunk(towers, coord)
        .map(|t| {
            let base_y = style_for(styles, t.style).map_or(DEFAULT_BASE_HEIGHT, |s| s.base_height);
            (base_point(t, ground_y), t.yaw, box_points(t, base_y))
        })
        .collect()
}

/// Base model id (`tw_<style>_<size>_base`), world position and yaw for each tower in chunk `coord`.
pub fn tower_bases(towers: &[TowerSpot], coord: IVec2, ground_y: f32) -> Vec<(String, Vec3, f32)> {
    towers_in_chunk(towers, coord)
        .map(|t| (base_model_id(t), base_point(t, ground_y), t.yaw))
        .collect()
}

/// The eight corners of a tower's upper footprint box, from `base_height` to `height`.
/// Leaving `0..base_height` open keeps the lobby doorway and interior walkable.
pub fn box_points(t: &TowerSpot, base_height: f32) -> Vec<[f32; 3]> {
    let top = t.height.max(base_height);
    let mut pts = Vec::with_capacity(8);
    for y in [base_height, top] {
        for x in [-t.half.x, t.half.x] {
            for z in [-t.half.y, t.half.y] {
                pts.push([x, y, z]);
            }
        }
    }
    pts
}
