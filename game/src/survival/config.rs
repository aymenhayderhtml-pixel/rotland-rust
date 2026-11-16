use std::path::Path;

use serde::{Deserialize, Serialize};

/// Rates and thresholds for one survival need (port of scripts/Survival/NeedDef.cs).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NeedDef {
    pub max: f32,
    pub start: f32,
    pub drain_per_sec: f32,
    pub sprint_drain_multiplier: f32,
    pub empty_damage_per_sec: f32,
    /// At or below this value the need counts as low (slower stamina regen).
    pub low_threshold: f32,
}

impl Default for NeedDef {
    fn default() -> Self {
        Self {
            max: 100.0,
            start: 100.0,
            drain_per_sec: 0.0,
            sprint_drain_multiplier: 1.0,
            empty_damage_per_sec: 0.0,
            low_threshold: 25.0,
        }
    }
}

/// Stamina costs and low-needs scaling from data/survival.json (port of scripts/Survival/StaminaDef.cs).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StaminaDef {
    pub sprint_cost_per_sec: f32,
    pub min_to_start_sprint: f32,
    pub jump_cost: f32,
    pub low_needs_regen_multiplier: f32,
}

impl Default for StaminaDef {
    fn default() -> Self {
        Self {
            sprint_cost_per_sec: 15.0,
            min_to_start_sprint: 10.0,
            jump_cost: 8.0,
            low_needs_regen_multiplier: 0.5,
        }
    }
}

/// Healing when well-fed from data/survival.json (port of scripts/Survival/WellFedDef.cs).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WellFedDef {
    pub threshold: f32,
    pub heal_per_sec: f32,
}

impl Default for WellFedDef {
    fn default() -> Self {
        Self {
            threshold: 80.0,
            heal_per_sec: 0.5,
        }
    }
}

/// Entire structure of data/survival.json (port of scripts/Survival/SurvivalData.cs and SurvivalConfig.cs).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SurvivalData {
    pub hunger: NeedDef,
    pub thirst: NeedDef,
    pub stamina: StaminaDef,
    pub well_fed: WellFedDef,
    pub respawn_needs_value: f32,
}

impl SurvivalData {
    pub fn from_repo() -> Result<Self, Box<dyn std::error::Error>> {
        let path = paths::repo_path("data/survival.json");
        Self::load(&path)
    }

    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let text = std::fs::read_to_string(path)?;
        Self::from_json(&text).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Looks up need definitions for "hunger" or "thirst".
    // Godot: SurvivalConfig.cs Need
    pub fn need(&self, need_id: &str) -> Option<&NeedDef> {
        match need_id {
            "hunger" => Some(&self.hunger),
            "thirst" => Some(&self.thirst),
            _ => None,
        }
    }
}
