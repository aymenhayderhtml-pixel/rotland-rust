//! Configuration shapes for the pressure director, loaded from `data/director.json`.

use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// State-specific pacing settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectorStateConfig {
    pub duration_day: f32,
    pub duration_night: f32,
    pub duration_city: f32,
    pub spawn_rate_multiplier: f32,
    pub max_active_multiplier: f32,
    pub special_chance: f32,
}

/// Global rules for waves and night hordes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectorRulesConfig {
    pub low_health_threshold: f32,
    pub night_horde_delay: f32,
    pub night_horde_size: usize,
    pub wave_size_day: usize,
    pub wave_size_night: usize,
    pub wave_size_city: usize,
    pub wave_size_noise: usize,
}

/// Extra parameters for special zombie behaviors.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SpecialZombieConfig {
    #[serde(default)]
    pub structure_damage: Option<f32>,
    #[serde(default)]
    pub call_radius: Option<f32>,
    #[serde(default)]
    pub hitbox_scale: Option<f32>,
}

/// Formation and movement parameters for hordes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HordeConfigData {
    pub speed: f32,
    pub spacing: f32,
    pub max_distance: f32,
    pub merge_distance: f32,
    pub cohesion_weight: f32,
    pub separation_weight: f32,
    pub alignment_weight: f32,
}

/// Full schema of `data/director.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectorConfig {
    pub states: HashMap<String, DirectorStateConfig>,
    pub rules: DirectorRulesConfig,
    pub special_zombies: HashMap<String, SpecialZombieConfig>,
    pub horde: HordeConfigData,
}

impl Default for DirectorConfig {
    fn default() -> Self {
        let mut states = HashMap::new();
        states.insert(
            "calm".to_string(),
            DirectorStateConfig {
                duration_day: 40.0, // data: director.json durationDay
                duration_night: 20.0, // data: director.json durationNight
                duration_city: 15.0, // data: director.json durationCity
                spawn_rate_multiplier: 0.5,
                max_active_multiplier: 0.8, // data: director.json maxActiveMultiplier
                special_chance: 0.0,
            },
        );
        states.insert(
            "building".to_string(),
            DirectorStateConfig {
                duration_day: 25.0, // data: director.json durationDay
                duration_night: 15.0, // data: director.json durationNight
                duration_city: 12.0, // data: director.json durationCity
                spawn_rate_multiplier: 1.2, // data: director.json spawnRateMultiplier
                max_active_multiplier: 1.1, // data: director.json maxActiveMultiplier
                special_chance: 0.1, // data: director.json specialChance
            },
        );
        states.insert(
            "wave".to_string(),
            DirectorStateConfig {
                duration_day: 20.0, // data: director.json durationDay
                duration_night: 20.0, // data: director.json durationNight
                duration_city: 25.0, // data: director.json durationCity
                spawn_rate_multiplier: 2.0,
                max_active_multiplier: 1.5, // data: director.json maxActiveMultiplier
                special_chance: 0.25,
            },
        );
        states.insert(
            "relax".to_string(),
            DirectorStateConfig {
                duration_day: 25.0, // data: director.json durationDay
                duration_night: 15.0, // data: director.json durationNight
                duration_city: 15.0, // data: director.json durationCity
                spawn_rate_multiplier: 0.3, // data: director.json spawnRateMultiplier
                max_active_multiplier: 0.6, // data: director.json maxActiveMultiplier
                special_chance: 0.05, // data: director.json specialChance
            },
        );

        let rules = DirectorRulesConfig {
            low_health_threshold: 0.3, // data: director.json lowHealthThreshold
            night_horde_delay: 30.0, // data: director.json nightHordeDelay
            night_horde_size: 20, // data: director.json nightHordeSize
            wave_size_day: 6, // data: director.json waveSizeDay
            wave_size_night: 10,
            wave_size_city: 14, // data: director.json waveSizeCity
            wave_size_noise: 8, // data: director.json waveSizeNoise
        };

        let mut special_zombies = HashMap::new();
        special_zombies.insert(
            "brute".to_string(),
            SpecialZombieConfig {
                structure_damage: Some(50.0), // data: director.json structureDamage
                call_radius: None,
                hitbox_scale: None,
            },
        );
        special_zombies.insert(
            "screamer".to_string(),
            SpecialZombieConfig {
                structure_damage: None,
                call_radius: Some(60.0), // data: director.json callRadius
                hitbox_scale: None,
            },
        );
        special_zombies.insert(
            "crawler".to_string(),
            SpecialZombieConfig {
                structure_damage: None,
                call_radius: None,
                hitbox_scale: Some(0.5),
            },
        );

        let horde = HordeConfigData {
            speed: 2.2, // data: director.json speed
            spacing: 1.5, // data: director.json spacing
            max_distance: 12.0, // data: director.json maxDistance
            merge_distance: 6.0, // data: director.json mergeDistance
            cohesion_weight: 1.0,
            separation_weight: 1.5, // data: director.json separationWeight
            alignment_weight: 1.2, // data: director.json alignmentWeight
        };

        Self {
            states,
            rules,
            special_zombies,
            horde,
        }
    }
}

impl DirectorConfig {
    /// Parse JSON string.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Load director configuration from given path.
    pub fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::from_json(&text)?)
    }

    /// Load default director configuration from `data/director.json`.
    pub fn load_default() -> Result<Self, Box<dyn Error>> {
        Self::load(&paths::repo_path("data/director.json"))
    }
}
