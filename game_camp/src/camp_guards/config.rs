//! Configuration data for camp guards (loaded from data/settlements.json).

use std::path::Path;
use serde::{Deserialize, Serialize};

fn default_guard_range() -> f32 {
    40.0
}
fn default_guard_sight_interval() -> f32 {
    0.5
}
fn default_guard_fire_interval() -> f32 {
    1.2
}
fn default_guard_damage() -> f32 {
    60.0
}
fn default_guard_miss_chance() -> f32 {
    0.15
}
fn default_guard_eye_height() -> f32 {
    1.6
}

/// Guard configuration values matching Godot data/settlements.json keys.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampGuardConfig {
    #[serde(default = "default_guard_range", alias = "GuardRange")]
    pub guard_range: f32,
    #[serde(default = "default_guard_sight_interval", alias = "GuardSightInterval")]
    pub guard_sight_interval: f32,
    #[serde(default = "default_guard_fire_interval", alias = "GuardFireInterval")]
    pub guard_fire_interval: f32,
    #[serde(default = "default_guard_damage", alias = "GuardDamage")]
    pub guard_damage: f32,
    #[serde(default = "default_guard_miss_chance", alias = "GuardMissChance")]
    pub guard_miss_chance: f32,
    #[serde(default = "default_guard_eye_height", alias = "GuardEyeHeight")]
    pub guard_eye_height: f32,
}

impl Default for CampGuardConfig {
    fn default() -> Self {
        Self {
            guard_range: default_guard_range(),
            guard_sight_interval: default_guard_sight_interval(),
            guard_fire_interval: default_guard_fire_interval(),
            guard_damage: default_guard_damage(),
            guard_miss_chance: default_guard_miss_chance(),
            guard_eye_height: default_guard_eye_height(),
        }
    }
}

impl CampGuardConfig {
    /// Constructs guard configuration from world's SettlementConfigData.
    #[allow(dead_code)]
    fn from_settlement_data(cfg: &world::SettlementConfigData) -> Self {
        Self {
            guard_range: if cfg.guard_range > 0.0 { cfg.guard_range } else { 40.0 },
            guard_sight_interval: if cfg.guard_sight_interval > 0.0 { cfg.guard_sight_interval } else { 0.5 },
            guard_fire_interval: if cfg.guard_fire_interval > 0.0 { cfg.guard_fire_interval } else { 1.2 },
            guard_damage: if cfg.guard_damage > 0.0 { cfg.guard_damage } else { 60.0 },
            guard_miss_chance: cfg.guard_miss_chance,
            guard_eye_height: if cfg.guard_eye_height > 0.0 { cfg.guard_eye_height } else { 1.6 },
        }
    }

    /// Loads configuration from a settlements.json file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let p = path.as_ref();
        let content = std::fs::read_to_string(p)
            .map_err(|e| format!("{}: {e}", p.display()))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("{}: {e}", p.display()))
    }

    /// Loads configuration from repo path "data/settlements.json".
    pub fn from_repo() -> Result<Self, String> {
        let path = paths::repo_path("data/settlements.json");
        Self::from_file(&path)
    }
}
