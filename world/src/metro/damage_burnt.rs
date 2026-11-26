//! Burnt-block facade soot: stacks `burnt_floor_overlay` sheets on all four faces of a burnt
//! tower, from the ground up to the top floor, so the whole tower reads as charred.

use std::f32::consts::FRAC_PI_2;

use glam::Vec2;

use crate::metro::styles::{default_styles, style_for};
use crate::metro::tower_parts::{PartKind, tower_parts};
use crate::metro::towers::TowerSpot;

/// Overlay sheet size in the model (R40-04 `burnt_overlay.py`).
const OVERLAY_WIDTH_M: f32 = 24.0; // new: DECISIONS 2026-11-26 (R40-04 overlay width)
const OVERLAY_HEIGHT_M: f32 = 3.8; // new: DECISIONS 2026-11-26 (R40-04 overlay height)
/// Extra push off the facade so the sheet clears piers and bands (no z-fighting).
const FACE_STANDOFF_M: f32 = 0.06; // new: DECISIONS 2026-11-26

/// One soot sheet: ground position, height above the tower ground, yaw and uniform scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SootSheet {
    pub pos: Vec2,
    pub y: f32,
    pub yaw: f32,
    pub scale: f32,
}

/// Soot sheets for one burnt tower: one row per floor (base included), four faces per row.
/// Rows follow the floor width so setbacks keep their sheets on the wall.
pub fn soot_sheets(spot: &TowerSpot) -> Vec<SootSheet> {
    let Some(style) = style_for(default_styles(), spot.style) else { return Vec::new() };
    let parts = tower_parts(spot, style, 0.0, 0);
    let mut out = Vec::new();
    for part in parts.iter().filter(|p| matches!(p.kind, PartKind::Base | PartKind::Floor | PartKind::Setback)) {
        let width = module_size(&part.model).unwrap_or(spot.size) as f32;
        let scale = width / OVERLAY_WIDTH_M;
        let rows = if part.kind == PartKind::Base { (style.base_height / (OVERLAY_HEIGHT_M * scale)).round().max(1.0) as usize } else { 1 };
        for row in 0..rows {
            let y = part.pos.y + row as f32 * OVERLAY_HEIGHT_M * scale;
            for face in 0..4 {
                let yaw = spot.yaw + face as f32 * FRAC_PI_2;
                let dir = Vec2::new(-yaw.sin(), -yaw.cos());
                out.push(SootSheet { pos: spot.pos + dir * (width * 0.5 + FACE_STANDOFF_M), y, yaw, scale });
            }
        }
    }
    out
}

/// Module size from a part id like `tw_brick_24_floor_c`.
fn module_size(model: &str) -> Option<u32> {
    model.split('_').nth(2)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::module_size;

    #[test]
    fn module_size_reads_part_ids() {
        assert_eq!(module_size("tw_brick_24_floor_c"), Some(24));
        assert_eq!(module_size("tw_glass_32_base"), Some(32));
        assert_eq!(module_size("tw_x"), None);
    }
}
