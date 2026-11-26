//! Collider hulls of towers: one solid upper box per tower (above the walk-in lobby) plus
//! the base model id/pose so its own wall colliders keep the lobby walkable.

use glam::{IVec2, Vec3};
use world::metro::{DamageSites, StyleDef, TowerSpot, style_for};

use super::near::{base_point, towers_in_chunk};
use super::parts::base_model_id;

/// Fallback lobby base height when a style is not found in `styles`.
const DEFAULT_BASE_HEIGHT: f32 = 7.6; // data: metro.json styles[].baseHeight

/// One upper box hull per tower in chunk `coord`: (base centre, yaw, 8 corner points in tower-local space).
/// Skips towers replaced by major damage set pieces.
pub fn tower_hulls(
    towers: &[TowerSpot],
    coord: IVec2,
    ground_y: f32,
    styles: &[StyleDef],
    damage: &DamageSites,
) -> Vec<(Vec3, f32, Vec<[f32; 3]>)> {
    towers_in_chunk(towers, coord)
        .filter_map(|t| {
            let idx = towers.iter().position(|x| std::ptr::eq(x, t))?;
            if damage.tower_is_replaced(idx) {
                return None;
            }
            let style = style_for(styles, t.style);
            let base_y = style.map_or(DEFAULT_BASE_HEIGHT, |s| s.base_height);
            Some((base_point(t, ground_y), t.yaw, box_points(t, base_y, roof_top(t, style))))
        })
        .collect()
}

/// Base model id (`tw_<style>_<size>_base`), world position and yaw for each tower in chunk `coord`.
/// Skips towers replaced by major damage set pieces.
pub fn tower_bases(
    towers: &[TowerSpot],
    coord: IVec2,
    ground_y: f32,
    damage: &DamageSites,
) -> Vec<(String, Vec3, f32)> {
    towers_in_chunk(towers, coord)
        .filter_map(|t| {
            let idx = towers.iter().position(|x| std::ptr::eq(x, t))?;
            if damage.tower_is_replaced(idx) {
                return None;
            }
            Some((base_model_id(t), base_point(t, ground_y), t.yaw))
        })
        .collect()
}

/// Collider top: the walkable roof deck for plain crowns (so the player stands on the deck
/// among the roof props and camps), the full height for landmark crowns.
fn roof_top(t: &TowerSpot, style: Option<&StyleDef>) -> f32 {
    match style {
        Some(s) if !t.landmark && s.roof_deck_height > 0.0 => t.height - s.crown_height + s.roof_deck_height,
        _ => t.height,
    }
}

/// The eight corners of a tower's upper footprint box, from `base_height` to `top`.
/// Leaving `0..base_height` open keeps the lobby doorway and interior walkable.
pub fn box_points(t: &TowerSpot, base_height: f32, top: f32) -> Vec<[f32; 3]> {
    let top = top.max(base_height);
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
