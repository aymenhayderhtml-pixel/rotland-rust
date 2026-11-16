//! Pure map mathematics: world to screen projection, screen to world projection,
//! and zoom stepping.
//!
//! Port of Godot C# scripts/UI/MapMath.cs.

use glam::Vec2;

// Godot: MapMath.cs Scales
pub const SCALES: [f32; 3] = [0.25, 0.5, 1.0];

// Godot: MapMath.cs DefaultZoom
pub const DEFAULT_ZOOM: usize = 1;

/// Converts world coordinates (world +X is screen right, world +Z is screen down)
/// to screen pixel coordinates.
// Godot: MapMath.cs WorldToScreen
#[inline]
pub fn world_to_screen(world: Vec2, centre: Vec2, scale: f32, view_size: Vec2) -> Vec2 {
    (world - centre) * scale + view_size * 0.5
}

/// Converts screen pixel coordinates back into world XZ coordinates.
// Godot: MapMath.cs ScreenToWorld
#[inline]
pub fn screen_to_world(screen: Vec2, centre: Vec2, scale: f32, view_size: Vec2) -> Vec2 {
    (screen - view_size * 0.5) / scale + centre
}

/// The next zoom level in a direction (+1 in, -1 out), clamped inside valid zoom indices.
// Godot: MapMath.cs StepZoom
pub fn step_zoom(index: usize, direction: i32) -> usize {
    if direction >= 0 {
        (index + direction as usize).min(SCALES.len() - 1)
    } else {
        index.saturating_sub((-direction) as usize)
    }
}
