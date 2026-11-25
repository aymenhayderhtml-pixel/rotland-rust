use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::biome::BiomeId;
use crate::scatter::ScatterRule;

/// Ground RGB colors (0-255) for a biome outside the core circle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiomeGround {
    pub base: [u8; 3],
    pub alt: [u8; 3],
    pub slope: [u8; 3],
    #[serde(default)]
    pub peak: Option<[u8; 3]>,
    #[serde(default)]
    pub peak_height: f32,
}

/// Procedural heightfield parameters for a biome outside the core circle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiomeHeight {
    pub base: f32,
    pub amplitude: f32,
    pub frequency: f32,
    pub octaves: i32,
    #[serde(default)]
    pub ridged: bool,
    #[serde(default)]
    pub dune: f32,
}

/// Complete data-driven definition for one biome loaded from `data/biomes.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiomeDef {
    pub key: String,
    pub name: String,
    #[serde(rename = "_why", default)]
    pub why: String,
    pub map_color: [u8; 3],
    pub fog_tint: [u8; 3],
    pub ground: BiomeGround,
    pub height: BiomeHeight,
    #[serde(default)]
    pub scatter: Vec<ScatterRule>,
    pub zombie_density: f32,
    pub loot_table: String,
    #[serde(default)]
    pub animals: Vec<String>,
    pub temperature_day: f32,
    pub temperature_night: f32,
    pub thirst_mult: f32,
    pub footstep: String,
}

#[derive(Debug, Deserialize)]
struct BiomesFile {
    biomes: Vec<BiomeDef>,
}

/// Registry holding all 6 biome definitions indexed by `BiomeId`.
#[derive(Debug, Clone, PartialEq)]
pub struct BiomeRegistry {
    biomes: [BiomeDef; BiomeId::COUNT],
}

impl BiomeRegistry {
    /// Loads `data/biomes.json` via `paths::try_repo_path`.
    pub fn load() -> Result<Self, String> {
        let path = paths::try_repo_path("data/biomes.json")
            .map_err(|e| format!("data/biomes.json: {e}"))?;
        Self::load_from_file(path)
    }

    /// Loads a biome registry from an explicit file path.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&json)
    }

    /// Parses and validates biome definitions from a JSON string, ensuring all 6 biomes exist.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let parsed: BiomesFile = serde_json::from_str(json)
            .map_err(|e| format!("failed to parse biomes JSON: {e}"))?;

        let mut slots: [Option<BiomeDef>; BiomeId::COUNT] = std::array::from_fn(|_| None);
        for def in parsed.biomes {
            let Some(id) = BiomeId::from_key(&def.key) else {
                return Err(format!("unknown biome key in biomes.json: '{}'", def.key));
            };
            let slot = &mut slots[id.index()];
            if slot.is_some() {
                return Err(format!("duplicate biome key in biomes.json: '{}'", def.key));
            }
            *slot = Some(def);
        }

        let missing: Vec<&'static str> = BiomeId::ALL
            .iter()
            .filter(|id| slots[id.index()].is_none())
            .map(|id| id.key())
            .collect();
        if !missing.is_empty() {
            return Err(format!(
                "missing required biome(s) in biomes.json: {}",
                missing.join(", ")
            ));
        }

        let [Some(plains), Some(forest), Some(desert), Some(snow), Some(swamp), Some(mountains)] =
            slots
        else {
            return Err("internal error assembling biome registry".to_string());
        };

        Ok(Self {
            biomes: [plains, forest, desert, snow, swamp, mountains],
        })
    }

    /// Returns the biome definition for `id`. Never panics because `from_json` guarantees all 6 keys.
    pub fn get(&self, id: BiomeId) -> &BiomeDef {
        &self.biomes[id.index()]
    }
}
