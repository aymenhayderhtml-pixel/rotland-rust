use serde::{Deserialize, Serialize};

fn default_weather_kind() -> String {
    "clear".to_string()
}

/// Saved weather snapshot stored inside [`WorldSaveData`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeatherSaveData {
    /// Weather kind key (`"clear"`, `"rain"`, `"snow"`, `"sandstorm"`, `"fog"`).
    #[serde(default = "default_weather_kind")]
    pub kind: String,
    /// Active weather intensity (`0.0..=1.0`).
    #[serde(default)]
    pub intensity: f32,
    /// Wind vector in m/s (`[x, z]`).
    #[serde(default)]
    pub wind: [f32; 2],
}

impl Default for WeatherSaveData {
    fn default() -> Self {
        Self {
            kind: default_weather_kind(),
            intensity: 0.0,
            wind: [0.0, 0.0],
        }
    }
}

/// World generator seed and optional weather state.
/// Port of `scripts/World/WorldRoot.cs` (SaveId: "world").
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WorldSaveData {
    #[serde(default)]
    pub seed: i32,
    /// Saved weather snapshot; absent in old saves (loads as clear weather).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weather: Option<WeatherSaveData>,
}

impl WorldSaveData {
    pub fn new(seed: i32) -> Self {
        Self {
            seed,
            weather: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_save_without_weather_loads_as_none() {
        let loaded: WorldSaveData = serde_json::from_str(r#"{"seed":42}"#).expect("old save json");
        assert_eq!(loaded.seed, 42);
        assert_eq!(loaded.weather, None);
    }

    #[test]
    fn world_save_round_trips_weather_state() {
        let data = WorldSaveData {
            seed: 99,
            weather: Some(WeatherSaveData {
                kind: "snow".to_string(),
                intensity: 0.85,
                wind: [2.5, -1.0],
            }),
        };
        let json = serde_json::to_string(&data).expect("serialize");
        let back: WorldSaveData = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, data);
    }
}
