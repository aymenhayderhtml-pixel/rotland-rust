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
    pub look_left: KeyCode,
    pub look_right: KeyCode,
    pub look_up: KeyCode,
    pub look_down: KeyCode,
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
            look_left: KeyCode::ArrowLeft,
            look_right: KeyCode::ArrowRight,
            look_up: KeyCode::ArrowUp,
            look_down: KeyCode::ArrowDown,
        }
    }
}

pub use crate::keybinds::*;

impl InputBindings {
    /// Builds `InputBindings` from user-configured `KeyBindings`.
    pub fn from_keybindings(kb: &crate::keybinds::KeyBindings) -> Self {
        Self {
            move_forward: kb.keycode_of("move_forward").unwrap_or(KeyCode::KeyW),
            move_back: kb.keycode_of("move_back").unwrap_or(KeyCode::KeyS),
            move_left: kb.keycode_of("move_left").unwrap_or(KeyCode::KeyA),
            move_right: kb.keycode_of("move_right").unwrap_or(KeyCode::KeyD),
            jump: kb.keycode_of("jump").unwrap_or(KeyCode::Space),
            sprint: kb.keycode_of("sprint").unwrap_or(KeyCode::ShiftLeft),
            crouch: kb.keycode_of("crouch").unwrap_or(KeyCode::KeyC),
            toggle_camera: kb.keycode_of("toggle_camera").unwrap_or(KeyCode::KeyV),
            aim_button: MouseButton::Right,
            look_left: kb.keycode_of("look_left").unwrap_or(KeyCode::ArrowLeft),
            look_right: kb.keycode_of("look_right").unwrap_or(KeyCode::ArrowRight),
            look_up: kb.keycode_of("look_up").unwrap_or(KeyCode::ArrowUp),
            look_down: kb.keycode_of("look_down").unwrap_or(KeyCode::ArrowDown),
        }
    }
}

impl From<&crate::keybinds::KeyBindings> for InputBindings {
    fn from(kb: &crate::keybinds::KeyBindings) -> Self {
        Self::from_keybindings(kb)
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

/// Keyboard camera turn speed in degrees per second at mouse sensitivity 1.0.
pub const KEYBOARD_LOOK_SPEED_DEG: f32 = 120.0; // new: DECISIONS 2026-11-21

/// Calculates look delta in mouse-pixel units from held arrow/look keys and frame delta `dt`.
///
/// Converts a 120 deg/s rotation rate into raw pixel units using `crate::camera_rig::MOUSE_SENSITIVITY`
/// (0.003 radians per pixel). Left arrow produces negative x delta (turning view left), and Up arrow
/// produces negative y delta (looking up, matching upward mouse motion).
pub fn keyboard_look_delta(
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    dt: f32,
) -> (f32, f32) {
    if dt <= 0.0 {
        return (0.0, 0.0);
    }
    let mut x = 0.0f32;
    let mut y = 0.0f32;
    if left {
        x -= 1.0;
    }
    if right {
        x += 1.0;
    }
    if up {
        y -= 1.0;
    }
    if down {
        y += 1.0;
    }
    if x == 0.0 && y == 0.0 {
        return (0.0, 0.0);
    }
    let speed_rad = KEYBOARD_LOOK_SPEED_DEG.to_radians();
    let speed_pixels = speed_rad / crate::camera_rig::MOUSE_SENSITIVITY;
    (x * speed_pixels * dt, y * speed_pixels * dt)
}
