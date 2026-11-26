//! HUD minimap streets inside New Halden (pure). The caller clips to the disc and multiplies
//! the widths by the minimap scale, exactly like the Ashford streets in `minimap_draw.rs`.

use glam::Vec2;
use world::metro::MetroLayout;

/// Width of a street in minimap pixels, before the minimap scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapStreet {
    /// Offset from the minimap centre, in pixels (player at the origin).
    pub a: Vec2,
    pub b: Vec2,
    /// Stroke width in pixels before the minimap scale.
    pub width_px: f32,
    pub avenue: bool,
}

/// Width divisor and floors for the minimap stroke (same rule as `minimap_draw.rs`).
const WIDTH_DIVISOR: f32 = 10.0; // moved: rotland/src/app/minimap_draw.rs (width ratio)
const AVENUE_MIN_RATIO: f32 = 1.35; // moved: rotland/src/app/minimap_draw.rs (avenue floor)
const STREET_MIN_RATIO: f32 = 1.0; // moved: rotland/src/app/minimap_draw.rs (street floor)

/// Streets that reach the minimap disc around `player_xz` (world XZ), with their pixel offsets.
/// `street_min_px` is `street_min_px` from `data/minimap.json`.
pub fn minimap_streets(
    layout: &MetroLayout,
    player_xz: Vec2,
    radius_m: f32,
    radius_px: f32,
    street_min_px: f32,
) -> Vec<MinimapStreet> {
    let local_player = layout.site.to_local(player_xz);
    let mut out = Vec::new();
    for street in &layout.streets {
        let max_reach = radius_m + street.width * 0.5;
        if street.closest_point(local_player).distance(local_player) > max_reach {
            continue;
        }
        let ratio = if street.avenue {
            (street.width / WIDTH_DIVISOR).max(AVENUE_MIN_RATIO)
        } else {
            (street.width / WIDTH_DIVISOR).max(STREET_MIN_RATIO)
        };
        let a = layout.site.to_world(street.a);
        let b = layout.site.to_world(street.b);
        out.push(MinimapStreet {
            a: game::world_to_minimap_offset(a, player_xz, radius_m, radius_px),
            b: game::world_to_minimap_offset(b, player_xz, radius_m, radius_px),
            width_px: street_min_px * ratio,
            avenue: street.avenue,
        });
    }
    out
}
