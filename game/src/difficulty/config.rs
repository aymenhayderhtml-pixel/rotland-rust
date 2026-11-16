use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::difficulty::death_rule::DeathRule;

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

/// Difficulty settings shape from data/difficulty.json (port of scripts/Difficulty/DifficultyConfigData.cs and DifficultyConfig.cs).
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
        let data = Self::from_json(&text)?;
        data.validate().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(data)
    }

    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Validates preset death rules and default preset, matching C# DifficultyConfig.Load.
    // Godot: DifficultyConfig.cs Load
    pub fn validate(&self) -> Result<(), String> {
        for preset in &self.presets {
            if DeathRule::from_id(&preset.death).is_none() {
                return Err(format!(
                    "preset '{}' has unknown death rule '{}'",
                    preset.id, preset.death
                ));
            }
        }
        if !self.presets.iter().any(|p| p.id == self.default_preset) {
            return Err(format!(
                "defaultPreset '{}' is not a preset",
                self.default_preset
            ));
        }
        Ok(())
    }

    /// Finds a preset by id.
    // Godot: DifficultyConfig.cs Preset
    pub fn preset(&self, id: &str) -> Option<&DifficultyPresetData> {
        self.presets.iter().find(|p| p.id == id)
    }

    /// Name of the death rule shown to the player, or its id if not found.
    // Godot: DifficultyConfig.cs DeathRuleName
    pub fn death_rule_name(&self, rule: DeathRule) -> &str {
        let rule_id = rule.id();
        self.death_rules
            .iter()
            .find(|r| r.id == rule_id)
            .map_or(rule_id, |r| r.name.as_str())
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
