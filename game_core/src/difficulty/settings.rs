use serde::{Deserialize, Serialize};

use crate::difficulty::config::{DifficultyConfigData, DifficultyPresetData};
use crate::difficulty::death_rule::DeathRule;

/// One world's difficulty settings (port of scripts/Difficulty/DifficultySettings.cs).
/// Saved in the save slot under the id "difficulty".
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DifficultySettings {
    #[serde(default = "default_one")]
    pub zombie_amount: f32,
    #[serde(default = "default_one")]
    pub loot_amount: f32,
    #[serde(default = "default_one")]
    pub zombie_damage: f32,
    #[serde(default)]
    pub death: DeathRule,
}

fn default_one() -> f32 {
    1.0
}

impl Default for DifficultySettings {
    fn default() -> Self {
        Self {
            zombie_amount: 1.0,
            loot_amount: 1.0,
            zombie_damage: 1.0,
            death: DeathRule::LoseAll,
        }
    }
}

impl DifficultySettings {
    pub const SAVE_ID: &'static str = "difficulty";

    pub fn new(zombie_amount: f32, loot_amount: f32, zombie_damage: f32, death: DeathRule) -> Self {
        Self {
            zombie_amount,
            loot_amount,
            zombie_damage,
            death,
        }
    }

    /// Settings configured from a preset definition.
    // Godot: DifficultySettings.cs FromPreset
    pub fn from_preset(preset: &DifficultyPresetData, config: &DifficultyConfigData) -> Self {
        let death = DeathRule::from_id(&preset.death).unwrap_or(DeathRule::LoseAll);
        let settings = Self {
            zombie_amount: preset.zombies,
            loot_amount: preset.loot,
            zombie_damage: preset.damage,
            death,
        };
        settings.clamped(config)
    }

    /// The default preset (Normal), as a new game starts with.
    // Godot: DifficultySettings.cs Default
    pub fn default_preset(config: &DifficultyConfigData) -> Self {
        if let Some(preset) = config.preset(&config.default_preset) {
            Self::from_preset(preset, config)
        } else {
            Self::default()
        }
    }

    /// Returns the preset name these values match, or "Custom".
    // Godot: DifficultySettings.cs Label
    pub fn label(&self, config: &DifficultyConfigData) -> String {
        for preset in &config.presets {
            let preset_settings = Self::from_preset(preset, config);
            if self.same_as(&preset_settings) {
                return preset.name.clone();
            }
        }
        "Custom".to_string()
    }

    /// True if all multipliers and death rules match approximately (epsilon 0.001).
    // Godot: DifficultySettings.cs SameAs
    pub fn same_as(&self, other: &Self) -> bool {
        const EPS: f32 = 0.001;
        (self.zombie_amount - other.zombie_amount).abs() < EPS
            && (self.loot_amount - other.loot_amount).abs() < EPS
            && (self.zombie_damage - other.zombie_damage).abs() < EPS
            && self.death == other.death
    }

    /// Keeps each multiplier inside the range from data/difficulty.json.
    // Godot: DifficultySettings.cs Clamped
    pub fn clamped(&self, config: &DifficultyConfigData) -> Self {
        Self {
            zombie_amount: self.zombie_amount.clamp(config.min_multiplier, config.max_multiplier),
            loot_amount: self.loot_amount.clamp(config.min_multiplier, config.max_multiplier),
            zombie_damage: self.zombie_damage.clamp(config.min_multiplier, config.max_multiplier),
            death: self.death,
        }
    }

    /// Converts to Godot save dictionary format.
    // Godot: DifficultySettings.cs ToDictionary
    pub fn to_save(&self) -> serde_json::Value {
        serde_json::json!({
            "zombies": self.zombie_amount as f64,
            "loot": self.loot_amount as f64,
            "damage": self.zombie_damage as f64,
            "death": self.death.id(),
        })
    }

    /// Reads saved values, defaulting to Normal if fields are absent, and clamping.
    // Godot: DifficultySettings.cs FromDictionary
    pub fn from_save(data: &serde_json::Value, config: &DifficultyConfigData) -> Self {
        let death_str = data
            .get("death")
            .and_then(|v| v.as_str())
            .unwrap_or("lose_all");
        let death = DeathRule::from_id(death_str).unwrap_or(DeathRule::LoseAll);

        let zombie_amount = data
            .get("zombies")
            .and_then(|v| v.as_f64())
            .map_or(1.0, |v| v as f32);
        let loot_amount = data
            .get("loot")
            .and_then(|v| v.as_f64())
            .map_or(1.0, |v| v as f32);
        let zombie_damage = data
            .get("damage")
            .and_then(|v| v.as_f64())
            .map_or(1.0, |v| v as f32);

        Self {
            zombie_amount,
            loot_amount,
            zombie_damage,
            death,
        }
        .clamped(config)
    }
}
