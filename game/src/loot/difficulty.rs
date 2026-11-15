use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DifficultyPresetData {
    pub id: String,
    pub name: String,
    pub zombies: f32,
    pub loot: f32,
    pub damage: f32,
    pub death: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeathRuleData {
    pub id: String,
    pub name: String,
}

/// Difficulty settings shape from data/difficulty.json (port of scripts/Difficulty/DifficultyConfigData.cs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DifficultyConfigData {
    pub min_multiplier: f32,
    pub max_multiplier: f32,
    pub step: f32,
    pub default_preset: String,
    pub presets: Vec<DifficultyPresetData>,
    pub death_rules: Vec<DeathRuleData>,
}

impl DifficultyConfigData {
    pub fn from_repo() -> Result<Self, Box<dyn std::error::Error>> {
        let path = paths::repo_path("data/difficulty.json");
        Self::load(&path)
    }

    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::from_json(&text)?)
    }

    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Finds a preset by id.
    pub fn preset(&self, id: &str) -> Option<&DifficultyPresetData> {
        self.presets.iter().find(|p| p.id == id)
    }

    /// Returns the loot multiplier for a preset, or 1.0 if not found.
    pub fn loot_multiplier(&self, preset_id: &str) -> f32 {
        self.preset(preset_id).map_or(1.0, |p| p.loot)
    }

    /// Returns the loot multiplier for the default preset ("normal").
    pub fn default_loot_multiplier(&self) -> f32 {
        self.loot_multiplier(&self.default_preset)
    }
}
