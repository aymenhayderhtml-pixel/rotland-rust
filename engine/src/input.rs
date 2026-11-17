use std::collections::HashSet;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

/// Keyboard and mouse state for one frame. Keys are physical (the same place on any keyboard layout).
#[derive(Default)]
pub struct Input {
    held: HashSet<KeyCode>,
    pressed: HashSet<KeyCode>,
    mouse_held: HashSet<MouseButton>,
    mouse_pressed: HashSet<MouseButton>,
    /// Mouse movement since the last frame, in raw device units.
    pub mouse_delta: (f32, f32),
    pub wheel: f32,
}

impl Input {
    pub fn held(&self, key: KeyCode) -> bool {
        self.held.contains(&key)
    }

    /// True only on the frame the key went down.
    pub fn pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    /// The keys that went down this frame (key rebinding waits for one).
    pub fn pressed_keys(&self) -> Vec<KeyCode> {
        self.pressed.iter().copied().collect()
    }

    pub fn mouse_held(&self, button: MouseButton) -> bool {
        self.mouse_held.contains(&button)
    }

    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_pressed.contains(&button)
    }

    pub(crate) fn key(&mut self, key: KeyCode, down: bool) {
        if down {
            if self.held.insert(key) {
                self.pressed.insert(key);
            }
        } else {
            self.held.remove(&key);
        }
    }

    pub(crate) fn button(&mut self, button: MouseButton, down: bool) {
        if down {
            if self.mouse_held.insert(button) {
                self.mouse_pressed.insert(button);
            }
        } else {
            self.mouse_held.remove(&button);
        }
    }

    /// Called after each frame: one-frame events are cleared.
    pub(crate) fn end_frame(&mut self) {
        self.pressed.clear();
        self.mouse_pressed.clear();
        self.mouse_delta = (0.0, 0.0);
        self.wheel = 0.0;
    }
}
