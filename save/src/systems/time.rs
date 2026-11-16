use serde::{Deserialize, Serialize};

fn default_hour() -> f32 {
    8.0
}

fn default_day() -> i32 {
    1
}

/// In-game clock state.
/// Port of `scripts/World/TimeOfDay.cs` (SaveId: "time").
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeSaveData {
    #[serde(default = "default_hour")]
    pub hour: f32,
    #[serde(default = "default_day")]
    pub day: i32,
}

impl Default for TimeSaveData {
    fn default() -> Self {
        Self {
            hour: 8.0,
            day: 1,
        }
    }
}

impl TimeSaveData {
    pub fn new(hour: f32, day: i32) -> Self {
        Self { hour, day }
    }
}
