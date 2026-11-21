use std::path::Path;
use serde::{Deserialize, Serialize};

/// Shore noise configuration for wobbly coastline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShoreNoiseConfig {
    pub amplitude: f32,
    pub frequency: f32,
}

/// Fishing village configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VillageConfig {
    pub count: usize,
    pub sizes: Vec<[f32; 2]>,
}

/// Docks and piers configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocksConfig {
    pub count: usize,
    pub width: f32,
    pub length: f32,
}

/// Configuration data loaded from data/lake.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LakeConfigData {
    pub distance_from_camp: f32,
    pub radius: [f32; 2],
    pub shore_noise: ShoreNoiseConfig,
    pub beach_width: f32,
    pub shallow_width: f32,
    pub max_depth: f32,
    pub water_level_above_shore: f32,
    pub village: VillageConfig,
    pub docks: DocksConfig,
    pub angle_from_city: f32,
    pub beach_blend: f32,
    pub shore_samples: usize,
    pub region_margin: f32,
}

pub type LakeConfig = LakeConfigData;

impl Default for LakeConfigData {
    fn default() -> Self {
        Self {
            // data: lake.json distanceFromCamp
            distance_from_camp: 1500.0,
            // data: lake.json radius
            radius: [260.0, 200.0],
            shore_noise: ShoreNoiseConfig {
                // data: lake.json shoreNoise
                amplitude: 12.0,
                // data: lake.json shoreNoise
                frequency: 0.02,
            },
            // data: lake.json beachWidth
            beach_width: 14.0,
            // data: lake.json shallowWidth
            shallow_width: 12.0,
            // data: lake.json maxDepth
            max_depth: 8.0,
            // data: lake.json waterLevelAboveShore
            water_level_above_shore: 0.3,
            village: VillageConfig {
                // data: lake.json village
                count: 8,
                sizes: vec![
                    // data: lake.json village
                    [14.0, 10.0],
                    // data: lake.json village
                    [12.0, 8.0],
                    // data: lake.json village
                    [16.0, 12.0],
                    // data: lake.json village
                    [10.0, 8.0],
                    // data: lake.json village
                    [12.0, 10.0],
                    // data: lake.json village
                    [14.0, 12.0],
                    // data: lake.json village
                    [10.0, 10.0],
                    // data: lake.json village
                    [12.0, 8.0],
                ],
            },
            docks: DocksConfig {
                // data: lake.json docks
                count: 3,
                width: 4.0,
                // data: lake.json docks
                length: 20.0,
            },
            angle_from_city: 180.0,
            // data: lake.json beachBlend
            beach_blend: 16.0,
            // data: lake.json shoreSamples
            shore_samples: 128,
            // data: lake.json regionMargin
            region_margin: 32.0,
        }
    }
}

impl LakeConfigData {
    pub fn validate(&self) -> Result<(), String> {
        let ok = self.distance_from_camp > 0.0
            && self.radius[0] > 0.0
            && self.radius[1] > 0.0
            && self.shore_noise.amplitude >= 0.0
            && self.shore_noise.frequency > 0.0
            && self.beach_width >= 0.0
            && self.shallow_width >= 0.0
            && self.max_depth > 0.0
            && self.water_level_above_shore >= 0.0
            && self.village.count > 0
            && !self.village.sizes.is_empty()
            && self.docks.count > 0
            && self.docks.width > 0.0
            && self.docks.length > 0.0
            && self.beach_blend >= 0.0
            && self.shore_samples >= 4
            && self.region_margin >= 0.0;
        if !ok {
            return Err("LakeConfigData has invalid values".to_string());
        }
        Ok(())
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let data: Self = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        data.validate()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(data)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let data: Self = serde_json::from_str(json)?;
        Ok(data)
    }
}
