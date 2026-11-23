use std::path::Path;
use serde::{Deserialize, Serialize};

/// Configuration data for fishing mechanics.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct FishingConfig {
    pub max_water_distance: f32,
    pub cast_wait_min: f32,
    pub cast_wait_max: f32,
    pub bite_duration: f32,
    pub sweet_spot_min: f32,
    pub sweet_spot_max: f32,
    pub result_seconds: f32,
}

impl Default for FishingConfig {
    fn default() -> Self {
        Self {
            // data: fishing.json maxWaterDistance
            max_water_distance: 3.0,
            // data: fishing.json castWaitMin
            cast_wait_min: 2.0,
            // data: fishing.json castWaitMax
            cast_wait_max: 5.0,
            // data: fishing.json biteDuration
            bite_duration: 2.0,
            // data: fishing.json sweetSpotMin
            sweet_spot_min: 0.35,
            // data: fishing.json sweetSpotMax
            sweet_spot_max: 0.65,
            // data: fishing.json resultSeconds
            result_seconds: 2.0,
        }
    }
}

impl FishingConfig {
    pub fn from_repo() -> Result<Self, Box<dyn std::error::Error>> {
        let path = paths::repo_path("data/fishing.json");
        Self::load(&path)
    }

    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let text = std::fs::read_to_string(path)?;
        Self::from_json(&text).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}
