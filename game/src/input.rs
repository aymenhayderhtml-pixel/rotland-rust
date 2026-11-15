use engine::Input;
use glam::Vec2;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

/// Rebindable key and button bindings, matching Godot's `project.godot` default physical keys.
/// Kept in a single table so user rebinding and settings persistence can hook in cleanly.
#[derive(Clone, Debug)]
pub struct InputBindings {
    pub move_forward: KeyCode,
    pub move_back: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub jump: KeyCode,
    pub sprint: KeyCode,
    pub crouch: KeyCode,
    pub toggle_camera: KeyCode,
    pub aim_button: MouseButton,
}

impl Default for InputBindings {
    fn default() -> Self {
        Self {
            // Godot: physical_keycode 87 (W), 83 (S), 65 (A), 68 (D)
            move_forward: KeyCode::KeyW,
            move_back: KeyCode::KeyS,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,
            // Godot: physical_keycode 32 (Space)
            jump: KeyCode::Space,
            // Godot: physical_keycode 4194325 (Shift)
            sprint: KeyCode::ShiftLeft,
            // Godot: physical_keycode 67 (C)
            crouch: KeyCode::KeyC,
            // Godot: physical_keycode 86 (V)
            toggle_camera: KeyCode::KeyV,
            // Godot: button_index 2 (Right mouse button)
            aim_button: MouseButton::Right,
        }
    }
}

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

impl Actions {
    /// Builds actions for the current frame from engine input and the specified key bindings.
    pub fn from_input(input: &Input, bindings: &InputBindings) -> Self {
        let mut x = 0.0f32;
        let mut y = 0.0f32;
        if input.held(bindings.move_left) {
            x -= 1.0;
        }
        if input.held(bindings.move_right) {
            x += 1.0;
        }
        if input.held(bindings.move_forward) {
            y -= 1.0;
        }
        if input.held(bindings.move_back) {
            y += 1.0;
        }

        Self {
            move_vec: Vec2::new(x, y),
            sprint: input.held(bindings.sprint),
            crouch: input.held(bindings.crouch),
            jump: input.pressed(bindings.jump),
            toggle_camera: input.pressed(bindings.toggle_camera),
            zoom: input.wheel,
            aim: input.mouse_held(bindings.aim_button),
            look_delta: input.mouse_delta,
        }
    }

    /// Convenience helper using default key bindings.
    pub fn from_default_input(input: &Input) -> Self {
        Self::from_input(input, &InputBindings::default())
    }
}
