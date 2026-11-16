use std::collections::HashMap;
use serde::{Deserialize, Serialize};

fn default_highest_day() -> i32 {
    1
}

/// Lifetime stats for the player profile.
/// Port of `scripts/Player/PlayerStats.cs` (SaveId: "stats").
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatsSaveData {
    #[serde(default)]
    pub kills: HashMap<String, u32>,
    #[serde(default)]
    pub headshots: u32,
    #[serde(default)]
    pub deaths: u32,
    #[serde(default = "default_highest_day", rename = "highestDay")]
    pub highest_day: i32,
    #[serde(default, rename = "playTime")]
    pub play_time: f64,
    #[serde(default)]
    pub distance: f32,
}

impl Default for StatsSaveData {
    fn default() -> Self {
        Self {
            kills: HashMap::new(),
            headshots: 0,
            deaths: 0,
            highest_day: 1,
            play_time: 0.0,
            distance: 0.0,
        }
    }
}
