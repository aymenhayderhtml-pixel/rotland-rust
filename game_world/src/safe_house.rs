//! Pure logic for the rooftop safe house (`data/safe_house.json`, R44).
//!
//! Covers the config (prompts, travel and rest numbers), the claimed / ladder state that is
//! saved in `systems.extra["metro_safehouse"]`, and the travel block rule.
//! The geometry blocks of the JSON (`shack`, `ladder`, `rooftopCamp`) belong to `world`.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Prompt and toast texts for the safe house, from the `prompts` block.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeHousePrompts {
    pub claim: String,
    pub bed: String,
    pub chest: String,
    pub ladder_up: String,
    pub ladder_down: String,
    pub travel_to_safe_house: String,
    pub travel_to_camp: String,
    pub travel_blocked: String,
    pub not_claimed: String,
    pub claimed_toast: String,
    pub rested: String,
}

/// Configuration for the rooftop safe house, loaded from `data/safe_house.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeHouseConfig {
    /// Game hours that pass during a fast travel.
    pub travel_hours: f32,
    /// Clock hour the bed moves to.
    pub rest_hour: f32,
    /// A zombie chasing within this flat distance blocks travel.
    pub chase_radius_m: f32,
    /// How close the player must be to use a prop.
    pub use_radius_m: f32,
    pub map_label: String,
    #[serde(default)]
    pub prompts: SafeHousePrompts,
}

impl SafeHouseConfig {
    /// Loads `data/safe_house.json` from the repository path.
    pub fn from_repo() -> Result<Self, String> {
        let path = paths::repo_path("data/safe_house.json");
        Self::from_path(&path)
    }

    /// Loads `SafeHouseConfig` from a JSON file at `path`.
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&text)
    }

    /// Parses `SafeHouseConfig` from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("invalid safe_house.json: {e}"))
    }
}

/// Persistent state of the safe house (`systems.extra["metro_safehouse"]`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SafeHouseState {
    pub claimed: bool,
    pub ladder_down: bool,
}

impl SafeHouseState {
    /// Claims the safe house. The rope ladder comes down on the first claim.
    /// A second claim changes nothing.
    pub fn claim(&mut self) {
        if self.claimed {
            return;
        }
        self.claimed = true;
        self.ladder_down = true;
    }

    /// Flips the rope ladder. Does nothing before the house is claimed.
    /// Returns the new `ladder_down` value (false when not claimed).
    pub fn toggle_ladder(&mut self) -> bool {
        if !self.claimed {
            return false;
        }
        self.ladder_down = !self.ladder_down;
        self.ladder_down
    }

    /// Serializes the state as `{"claimed": bool, "ladderDown": bool}`.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({ "claimed": self.claimed, "ladderDown": self.ladder_down })
    }

    /// Restores the state; a missing or broken value gives the default (unclaimed, ladder up).
    pub fn from_json(value: &serde_json::Value) -> Self {
        let claimed = value.get("claimed").and_then(serde_json::Value::as_bool);
        let ladder_down = value.get("ladderDown").and_then(serde_json::Value::as_bool);
        Self {
            claimed: claimed.unwrap_or(false),
            ladder_down: ladder_down.unwrap_or(false),
        }
    }
}

/// Fast travel is blocked while any zombie is chasing the player.
// new: DECISIONS 2026-11-28 (any zombie chasing blocks travel)
pub fn travel_blocked(chasing: usize) -> bool {
    chasing > 0
}
