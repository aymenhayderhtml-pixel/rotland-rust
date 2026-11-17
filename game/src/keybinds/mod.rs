//! Key rebinding logic matching Godot's `scripts/Core/KeyBindings.cs`.
//! Tracks current key codes per action, conflict detection and swapping,
//! text prompt formatting, and save/load serialization for player settings.

mod actions;
mod keycode_map;

pub use actions::{find_action, BindableAction, KEYBINDS_ALL_ACTIONS};
pub use keycode_map::{
    describe_godot_code, godot_code_to_key, key_to_godot_code, KEYBINDS_CODE_MASK,
    KEYBINDS_SHIFT_MASK,
};

use std::collections::HashMap;
use winit::keyboard::KeyCode;

/// Key rebinding system.
/// Holds the currently active key binding code for each action.
/// Defaults match `project.godot`. Overrides can be serialized to/from settings.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyBindings {
    pub current: HashMap<String, u32>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        let mut current = HashMap::with_capacity(KEYBINDS_ALL_ACTIONS.len());
        for action in KEYBINDS_ALL_ACTIONS {
            current.insert(action.action.to_string(), action.default_code);
        }
        Self { current }
    }
}

impl KeyBindings {
    /// Creates a new `KeyBindings` instance populated with all defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if `action` is one of the 31 bindable gameplay actions.
    // Godot: KeyBindings.cs IsBindable
    pub fn is_bindable(action: &str) -> bool {
        find_action(action).is_some()
    }

    /// User-friendly display label for an action (e.g. "Interact / use").
    pub fn label_of(action: &str) -> &'static str {
        find_action(action).map_or("", |a| a.label)
    }

    /// Section group name for an action (e.g. "Move", "Actions", "Hotbar").
    pub fn section_of(action: &str) -> &'static str {
        find_action(action).map_or("", |a| a.section)
    }

    /// Default Godot integer key code for an action (0 if unknown).
    // Godot: KeyBindings.cs DefaultOf
    pub fn default_of(action: &str) -> u32 {
        find_action(action).map_or(0, |a| a.default_code)
    }

    /// Default winit `KeyCode` and shift flag for an action.
    pub fn default_key_of(action: &str) -> Option<(KeyCode, bool)> {
        find_action(action).map(|a| (a.default_key, a.default_shift))
    }

    /// Active Godot key code for an action (falls back to default if not present).
    // Godot: KeyBindings.cs CodeOf
    pub fn code_of(&self, action: &str) -> u32 {
        self.current
            .get(action)
            .copied()
            .unwrap_or_else(|| Self::default_of(action))
    }

    /// Active winit `KeyCode` and shift flag for an action.
    pub fn key_of(&self, action: &str) -> Option<(KeyCode, bool)> {
        godot_code_to_key(self.code_of(action))
    }

    /// Active winit `KeyCode` for an action (ignoring shift modifier).
    pub fn keycode_of(&self, action: &str) -> Option<KeyCode> {
        self.key_of(action).map(|(k, _)| k)
    }

    /// Finds another action that already uses the given key code, or None.
    /// The action itself is ignored.
    // Godot: KeyBindings.cs ConflictWith
    pub fn conflict_with(&self, action: &str, code: u32) -> Option<&'static str> {
        if code == 0 {
            return None;
        }
        for other in KEYBINDS_ALL_ACTIONS {
            if other.action != action && self.code_of(other.action) == code {
                return Some(other.action);
            }
        }
        None
    }

    /// Checks if another action already uses the given winit key.
    pub fn conflict_with_key(&self, action: &str, key: KeyCode, shift: bool) -> Option<&'static str> {
        key_to_godot_code(key, shift).and_then(|code| self.conflict_with(action, code))
    }

    /// Sets one action's binding code.
    // Godot: KeyBindings.cs Set
    pub fn set(&mut self, action: &str, code: u32) {
        if !Self::is_bindable(action) || code == 0 {
            return;
        }
        self.current.insert(action.to_string(), code);
    }

    /// Sets one action's binding from a winit `KeyCode`.
    pub fn set_key(&mut self, action: &str, key: KeyCode, shift: bool) {
        if let Some(code) = key_to_godot_code(key, shift) {
            self.set(action, code);
        }
    }

    /// Swaps the key bindings of two actions.
    // Godot: KeyBindings.cs Swap
    pub fn swap(&mut self, a: &str, b: &str) {
        let code_a = self.code_of(a);
        let code_b = self.code_of(b);
        self.set(a, code_b);
        self.set(b, code_a);
    }

    /// Resets all bindings back to project defaults.
    // Godot: KeyBindings.cs ResetAll
    pub fn reset_all(&mut self) {
        self.current.clear();
        for action in KEYBINDS_ALL_ACTIONS {
            self.current.insert(action.action.to_string(), action.default_code);
        }
    }

    /// Returns only the bindings that differ from the defaults (what the settings file keeps).
    // Godot: KeyBindings.cs Overrides
    pub fn overrides(&self) -> HashMap<String, u32> {
        let mut map = HashMap::new();
        for action in KEYBINDS_ALL_ACTIONS {
            let code = self.code_of(action.action);
            if code != action.default_code {
                map.insert(action.action.to_string(), code);
            }
        }
        map
    }

    /// Applies saved overrides on top of defaults.
    // Godot: KeyBindings.cs Apply
    pub fn apply(&mut self, overrides: &HashMap<String, u32>) {
        self.reset_all();
        for action in KEYBINDS_ALL_ACTIONS {
            if let Some(&code) = overrides.get(action.action)
                && code > 0 {
                    self.current.insert(action.action.to_string(), code);
                }
        }
    }

    /// Text representation for an action's current key (e.g. "E", "Shift+G").
    // Godot: KeyBindings.cs KeyName
    pub fn key_name(&self, action: &str) -> String {
        Self::describe(self.code_of(action))
    }

    /// Prompt line with the action's current key, e.g. "[E] Search crate".
    // Godot: KeyBindings.cs Prompt
    pub fn prompt(&self, action: &str, text: &str) -> String {
        let key = self.key_name(action);
        if key.is_empty() {
            text.to_string()
        } else {
            format!("[{key}] {text}")
        }
    }

    /// Text representation for an integer key code.
    // Godot: KeyBindings.cs Describe
    pub fn describe(code: u32) -> String {
        describe_godot_code(code)
    }

    /// Converts a winit KeyCode and shift flag into a Godot integer key code.
    pub fn key_to_godot_code(key: KeyCode, shift: bool) -> Option<u32> {
        key_to_godot_code(key, shift)
    }

    /// Converts a Godot integer key code into a winit KeyCode and shift flag.
    pub fn godot_code_to_key(code: u32) -> Option<(KeyCode, bool)> {
        godot_code_to_key(code)
    }

    /// Replaces each `{action}` token in a string with the action's current key name.
    // Godot: KeyBindings.cs Expand
    pub fn expand(&self, text: &str) -> String {
        let mut result = text.to_string();
        for action in KEYBINDS_ALL_ACTIONS {
            let token = format!("{{{}}}", action.action);
            if result.contains(&token) {
                result = result.replace(&token, &self.key_name(action.action));
            }
        }
        result
    }

    /// Serializes active key overrides to JSON for settings storage.
    pub fn to_settings(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        for (k, v) in self.overrides() {
            map.insert(k, serde_json::Value::from(v));
        }
        serde_json::Value::Object(map)
    }

    /// Loads key overrides from settings JSON.
    pub fn from_settings(&mut self, data: &serde_json::Value) {
        let mut map = HashMap::new();
        if let Some(obj) = data.as_object() {
            for (k, v) in obj {
                if Self::is_bindable(k) {
                    if let Some(code) = v.as_u64() {
                        if code > 0 {
                            map.insert(k.clone(), code as u32);
                        }
                    } else if let Some(code) = v.as_i64()
                        && code > 0 {
                            map.insert(k.clone(), code as u32);
                        }
                }
            }
        }
        self.apply(&map);
    }

    /// Alias for `to_settings`.
    pub fn to_save(&self) -> serde_json::Value {
        self.to_settings()
    }

    /// Alias for `from_settings`.
    pub fn from_save(&mut self, data: &serde_json::Value) {
        self.from_settings(data);
    }
}
