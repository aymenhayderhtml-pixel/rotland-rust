//! Pure player input actions for one frame, independent of windowing/engine types.

use glam::Vec2;

/// Player input actions for one frame, mapped from raw device input.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Actions {
    /// Move vector: X in [-1, 1] (-1 left, +1 right), Y in [-1, 1] (-1 forward, +1 back).
    /// Matches Godot's `Input.GetVector("move_left", "move_right", "move_forward", "move_back")`.
    pub move_vec: Vec2,
    /// Sprint held.
    pub sprint: bool,
    /// Crouch held.
    pub crouch: bool,
    /// Jump pressed this frame.
    pub jump: bool,
    /// Toggle camera pressed this frame (first/third person).
    pub toggle_camera: bool,
    /// Zoom delta from mouse wheel (positive = zoom in / closer, negative = zoom out / farther).
    pub zoom: f32,
    /// Aim held (right mouse button).
    pub aim: bool,
    /// Mouse delta (dx, dy) in raw device units.
    pub look_delta: (f32, f32),
}
