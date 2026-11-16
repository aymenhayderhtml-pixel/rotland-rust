//! Pure compass mathematics: bearings, relative angles, marker strip positioning,
//! and cardinal/ordinal direction naming.
//!
//! Port of Godot C# scripts/UI/CompassMath.cs.

use glam::{Vec2, Vec3};

// Godot: CompassMath.cs HalfViewDeg
/// The strip shows this many degrees to each side of the view. Beyond it the marker sits at the edge.
pub const HALF_VIEW_DEG: f32 = 90.0;

// Godot: CompassMath.cs DirectionNames
const DIRECTION_NAMES: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];

/// Angle from the view direction to a target, on the flat ground. Positive is to the right.
// Godot: CompassMath.cs RelativeAngleDeg
pub fn relative_angle_deg(from: Vec3, target: Vec3, yaw_rad: f32) -> f32 {
    let delta = Vec2::new(target.x - from.x, target.z - from.z);
    let forward = Vec2::new(-yaw_rad.sin(), -yaw_rad.cos());
    let right = Vec2::new(yaw_rad.cos(), -yaw_rad.sin());
    delta.dot(right).atan2(delta.dot(forward)).to_degrees()
}

/// Where a bearing (0 = north) sits relative to the view, in degrees, from -180 to 180.
// Godot: CompassMath.cs BearingToViewDeg
pub fn bearing_to_view_deg(bearing_deg: f32, yaw_rad: f32) -> f32 {
    // The view looks along bearing -yaw, so a bearing is shifted by the yaw
    let raw = bearing_deg + yaw_rad.to_degrees();
    pos_mod_f32(raw + 180.0, 360.0) - 180.0
}

/// The x position in the strip (0 at the left edge, width at the right).
/// Angles past the view stick to the edge.
// Godot: CompassMath.cs MarkerX
pub fn marker_x(angle_deg: f32, width: f32) -> f32 {
    let t = (angle_deg / HALF_VIEW_DEG).clamp(-1.0, 1.0);
    width * 0.5 + t * width * 0.5
}

/// Short name for a bearing that is a multiple of 45 degrees: N, NE, E, and so on.
// Godot: CompassMath.cs Name
pub fn direction_name(bearing_deg: i32) -> &'static str {
    let idx = (((bearing_deg % 360 + 360) % 360) / 45) as usize % 8;
    DIRECTION_NAMES[idx]
}

#[inline]
fn pos_mod_f32(val: f32, m: f32) -> f32 {
    let r = val % m;
    if r < 0.0 {
        r + m
    } else {
        r
    }
}
