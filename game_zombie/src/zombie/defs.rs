//! Zombie types and behaviour numbers, read from `data/zombies.json` (same key names as the Godot version).

use std::collections::BTreeMap;
use std::error::Error;
use std::path::Path;

use serde::Deserialize;

/// Stats for one zombie type.
///
/// Every field is required in the JSON: a missing key is an error, not a silent default, so a typo in
/// the data file cannot quietly change a zombie.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ZombieDef {
    pub max_health: f32,
    pub wander_speed: f32,
    pub chase_speed: f32,
    pub sight_range: f32,
    pub sight_fov_deg: f32,
    pub hearing_multiplier: f32,
    pub attack_range: f32,
    pub attack_damage: f32,
    pub attack_windup: f32,
    pub attack_cooldown: f32,
    pub lose_target_time: f32,
    /// Combat XP given to the player for killing this zombie.
    pub xp: f32,
    /// Placeholder colour as a hex string such as "#5c7a3a". Not used for the body any more.
    pub color: String,
    /// Body variants; one is picked at random per spawn.
    pub models: Vec<String>,
    /// Run clip for the fast chase blend (e.g. the runner's sprint). None keeps the default run clip.
    #[serde(default)]
    pub run_clip: Option<String>,
    /// Optional animation clip mapping overrides (e.g. for crawler).
    #[serde(default)]
    pub clips: std::collections::HashMap<String, String>,
}

/// Shared tuning numbers for zombie behaviour (the `behaviour` block of the JSON).
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ZombieBehaviourDef {
    /// Walk clips a walker picks from at spawn, so groups do not move in step.
    pub gaits: Vec<String>,
    /// A hit this strong staggers the zombie.
    pub strong_hit_damage: f32,
    pub stagger_fallback_seconds: f32,
    /// A hit below this height above the feet counts as a leg hit.
    pub leg_hit_height: f32,
    pub leg_hits_to_crawl: u32,
    pub crawler_speed: f32,
    pub crawler_radius: f32,
    pub crawler_height: f32,
    pub scream_chance: f32,
    pub scream_seconds: f32,
    pub scream_alert_radius: f32,
    pub eat_min_seconds: f32,
    pub eat_max_seconds: f32,
    pub eat_random_chance: f32,
    pub eat_near_corpse_chance: f32,
    pub corpse_spot_radius: f32,
    pub corpse_spot_seconds: f32,
    pub look_around_seconds: f32,
    pub herd_radius: f32,
    pub herd_check_seconds: f32,
    pub horde_chance: f32,
    pub horde_min: u32,
    pub horde_max: u32,
    pub horde_spacing: f32,
    pub bang_seconds: f32,
    pub stuck_seconds: f32,
    pub stuck_distance: f32,
    pub escape_seconds: f32,
    /// Escapes in a row that get nowhere before the zombie is moved aside (last resort).
    pub escape_tries_before_teleport: u32,
    pub teleport_metres: f32,
    pub blocked_memory: u32,
    pub blocked_radius: f32,
}

/// The whole of `data/zombies.json`: zombie types by id, plus the shared behaviour numbers.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZombieDefs {
    pub types: BTreeMap<String, ZombieDef>,
    pub behaviour: ZombieBehaviourDef,
}

impl ZombieDefs {
    /// Parse the JSON text. Used by `load` and by tests that want data without a file.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Read a zombies JSON file from `path`.
    pub fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::from_json(&text)?)
    }

    /// The real game data (`data/zombies.json` in the project). Found at run time, never baked in.
    pub fn load_default() -> Result<Self, Box<dyn Error>> {
        Self::load(&paths::repo_path("data/zombies.json"))
    }

    /// The zombie type with this id ("walker", "runner"...), if it exists.
    pub fn get(&self, id: &str) -> Option<&ZombieDef> {
        self.types.get(id)
    }

    /// Every zombie type id, in alphabetical order.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.types.keys().map(String::as_str)
    }
}
