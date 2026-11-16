//! Configuration data for dogs from data/dog.json.
// Godot: DogConfig.cs, DogConfigData.cs

use std::path::Path;
use serde::{Deserialize, Serialize};

/// Shape of data/dog.json. Numbers only; rules live in Dog, DogMotion, DogCombat and Dogs.
// Godot: DogConfigData.cs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DogConfigData {
    pub price: i32,
    pub default_name: String,
    pub max_name_length: usize,
    pub max_health: f32,
    pub regen_per_second: f32,
    pub regen_delay_seconds: f32,
    pub feed_heal: f32,
    pub feed_items: Vec<String>,
    pub follow_distance: f32,
    pub stop_distance: f32,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub catch_up_distance: f32,
    pub sit_after_seconds: f32,
    pub lie_after_seconds: f32,
    pub teleport_distance: f32,
    pub home_arrive_distance: f32,
    pub bark_range: f32,
    pub bark_cooldown: f32,
    pub bark_noise_radius: f32,
    pub attack_range: f32,
    pub bite_range: f32,
    pub bite_damage: f32,
    pub bite_cooldown: f32,
    pub body_seconds: f32,
    pub pack_slots: usize,
    pub pack_weight: f32,
    pub scan_interval: f32,
    pub puppy_scale: f32,
    pub grow_days: f32,
    pub stray_chance: f32,
    pub stray_buildings: Vec<String>,
    pub stray_flee_range: f32,
    pub stray_flee_seconds: f32,
    pub stray_flee_distance: f32,
    pub stray_spawn_range: f32,
    pub stray_despawn_range: f32,
}

impl Default for DogConfigData {
    fn default() -> Self {
        Self {
            price: 150,
            default_name: "Rex".to_string(),
            max_name_length: 12,
            max_health: 80.0,
            regen_per_second: 1.0,
            regen_delay_seconds: 10.0,
            feed_heal: 30.0,
            feed_items: vec!["raw_meat".to_string(), "cooked_meat".to_string()],
            follow_distance: 3.0,
            stop_distance: 0.5,
            walk_speed: 2.5,
            run_speed: 6.0,
            catch_up_distance: 8.0,
            sit_after_seconds: 5.0,
            lie_after_seconds: 20.0,
            teleport_distance: 60.0,
            home_arrive_distance: 4.0,
            bark_range: 20.0,
            bark_cooldown: 4.0,
            bark_noise_radius: 15.0,
            attack_range: 10.0,
            bite_range: 1.2,
            bite_damage: 15.0,
            bite_cooldown: 1.2,
            body_seconds: 60.0,
            pack_slots: 4,
            pack_weight: 10.0,
            scan_interval: 0.25,
            puppy_scale: 0.8,
            grow_days: 1.0,
            stray_chance: 0.05,
            stray_buildings: vec!["house_small".to_string(), "cabin".to_string()],
            stray_flee_range: 8.0,
            stray_flee_seconds: 4.0,
            stray_flee_distance: 8.0,
            stray_spawn_range: 70.0,
            stray_despawn_range: 90.0,
        }
    }
}

impl DogConfigData {
    /// Loads and parses DogConfigData from JSON text.
    pub fn from_json(text: &str) -> Result<Self, String> {
        let data: Self = serde_json::from_str(text).map_err(|e| e.to_string())?;
        data.validate()?;
        Ok(data)
    }

    /// Loads DogConfigData from a file on disk.
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("Could not open {}: {e}", path.display()))?;
        Self::from_json(&text)
    }

    /// Loads DogConfigData from repo path "data/dog.json".
    pub fn from_repo() -> Result<Self, String> {
        let path = paths::repo_path("data/dog.json");
        Self::from_file(&path)
    }

    /// Validates configuration values against valid gameplay ranges.
    // Godot: DogConfig.cs Get
    pub fn validate(&self) -> Result<(), String> {
        if self.price <= 0
            || self.max_health <= 0.0
            || self.run_speed < self.walk_speed
            || self.bite_range <= 0.0
            || self.pack_slots == 0
            || self.default_name.is_empty()
            || self.max_name_length < self.default_name.len()
        {
            return Err("data/dog.json has a value out of range".to_string());
        }
        Ok(())
    }
}
