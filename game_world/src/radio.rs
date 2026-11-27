//! Pure logic for the New Halden rooftop radio goal (`data/radio.json`).

use std::path::Path;

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Configuration for the rooftop radio console, loaded from `data/radio.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadioConfig {
    pub use_radius_m: f32,
    pub prompt: String,
    pub used_prompt: String,
    pub line_seconds: f32,
    pub lines: Vec<String>,
    pub crate_id: String,
    pub map_label: String,
}

impl RadioConfig {
    /// Loads `data/radio.json` from the repository path.
    pub fn from_repo() -> Result<Self, String> {
        let path = paths::repo_path("data/radio.json");
        Self::from_path(&path)
    }

    /// Loads `RadioConfig` from a JSON file at `path`.
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&text)
    }

    /// Parses `RadioConfig` from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("invalid radio.json: {e}"))
    }
}

/// Persistent state of the rooftop radio (`systems.extra["metro_radio"]`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RadioState {
    #[serde(default)]
    pub used: bool,
}

impl RadioState {
    /// Serializes the radio state for `systems.extra["metro_radio"]`.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({ "used": self.used })
    }

    /// Restores the radio state from JSON; missing or invalid values default to not used.
    pub fn from_json(value: &serde_json::Value) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}

/// Result of activating the radio for the first time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioOutcome {
    pub lines: Vec<String>,
    pub unlock_crate: String,
}

fn in_range_3d(player_pos: Vec3, console_pos: Vec3, radius_m: f32) -> bool {
    radius_m >= 0.0 && player_pos.distance_squared(console_pos) <= radius_m * radius_m
}

/// Returns the HUD prompt when `player_pos` is within `cfg.use_radius_m` (3D) of `console_pos`.
pub fn radio_prompt<'a>(
    state: &RadioState,
    player_pos: Vec3,
    console_pos: Vec3,
    cfg: &'a RadioConfig,
) -> Option<&'a str> {
    if !in_range_3d(player_pos, console_pos, cfg.use_radius_m) {
        return None;
    }
    Some(if state.used {
        &cfg.used_prompt
    } else {
        &cfg.prompt
    })
}

/// Uses the radio if in 3D range and not yet used; returns lines and crate ID on first use only.
pub fn use_radio(
    state: &mut RadioState,
    player_pos: Vec3,
    console_pos: Vec3,
    cfg: &RadioConfig,
) -> Option<RadioOutcome> {
    if state.used || !in_range_3d(player_pos, console_pos, cfg.use_radius_m) {
        return None;
    }
    state.used = true;
    Some(RadioOutcome {
        lines: cfg.lines.clone(),
        unlock_crate: cfg.crate_id.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cfg() -> RadioConfig {
        RadioConfig::from_json(include_str!("../../data/radio.json"))
            .expect("data/radio.json must parse")
    }

    #[test]
    fn prompt_range_is_3d() {
        let cfg = test_cfg();
        let console = Vec3::new(10.0, 68.0, -5.0);
        let state = RadioState::default();

        // Standing directly under the mast on the roof (18 m below): out of range.
        let under_mast = Vec3::new(10.0, 50.0, -5.0);
        assert_eq!(radio_prompt(&state, under_mast, console, &cfg), None);

        // Standing beside the console on the mast platform: in range.
        let beside = Vec3::new(11.5, 68.0, -5.0);
        assert_eq!(
            radio_prompt(&state, beside, console, &cfg),
            Some(cfg.prompt.as_str())
        );

        let used = RadioState { used: true };
        assert_eq!(
            radio_prompt(&used, beside, console, &cfg),
            Some(cfg.used_prompt.as_str())
        );
    }

    #[test]
    fn first_use_returns_lines_and_crate_second_returns_none() {
        let cfg = test_cfg();
        let console = Vec3::new(0.0, 68.0, 0.0);
        let near = Vec3::new(1.0, 68.0, 0.0);
        let far = Vec3::new(0.0, 50.0, 0.0);
        let mut state = RadioState::default();

        assert_eq!(use_radio(&mut state, far, console, &cfg), None);
        assert!(!state.used);

        let outcome = use_radio(&mut state, near, console, &cfg).expect("first use succeeds");
        assert!(state.used);
        assert_eq!(outcome.lines, cfg.lines);
        assert_eq!(outcome.unlock_crate, "metro_radio_crate");
        assert!(!outcome.lines.is_empty());

        assert_eq!(use_radio(&mut state, near, console, &cfg), None);
    }

    #[test]
    fn save_round_trip_and_missing_defaults_unused() {
        let used = RadioState { used: true };
        let json = used.to_json();
        assert_eq!(RadioState::from_json(&json), used);

        let unused = RadioState { used: false };
        assert_eq!(RadioState::from_json(&unused.to_json()), unused);

        assert_eq!(RadioState::from_json(&serde_json::json!({})), unused);
        assert_eq!(RadioState::from_json(&serde_json::Value::Null), unused);
        assert_eq!(RadioState::from_json(&serde_json::json!("bad")), unused);
    }
}
