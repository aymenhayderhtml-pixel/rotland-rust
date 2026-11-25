//! Configuration data and order types for the zombie spawner, loaded from data/spawning.json.

use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

use glam::Vec3;
use serde::{Deserialize, Serialize};
use world::BiomeId;

fn default_hard_cap() -> usize {
    60 // data: spawning.json hardCap
}

/// Shape of data/spawning.json.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpawnConfig {
    pub max_active: u32,
    #[serde(default = "default_hard_cap")]
    pub hard_cap: usize,
    pub min_spawn_dist: f32,
    pub max_spawn_dist: f32,
    pub despawn_dist: f32,
    pub spawn_interval: f32,
    pub start_delay: f32,
    pub out_of_view_angle_deg: f32,
    pub biome_density: HashMap<String, f32>,
    pub night_max_active_multiplier: f32,
    pub camp_density: CampDensityRule,
    pub types: SpawnTypeTable,
    #[serde(default)]
    pub city: Option<CitySpawnConfig>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CitySpawnConfig {
    pub density: f32,
    pub max_active: u32,
    pub core_night_types: Vec<SpawnTypeEntry>,
}

pub type SpawnConfigData = SpawnConfig;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CampDensityRule {
    pub radius: f32,
    pub factor: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpawnTypeTable {
    pub day: Vec<SpawnTypeEntry>,
    pub night: Vec<SpawnTypeEntry>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpawnTypeEntry {
    pub id: String,
    pub weight: f32,
}

impl SpawnConfig {
    /// Parse raw JSON text. Missing fields or unknown fields produce errors.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Load the configuration file at the specified filesystem path.
    pub fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::from_json(&text)?)
    }

    /// Load default configuration from data/spawning.json via repository root path.
    pub fn load_default() -> Result<Self, Box<dyn Error>> {
        Self::load(&paths::repo_path("data/spawning.json"))
    }

    /// Lighter daytime spawning near a camp, so a new player can get going.
    pub fn camp_factor_for(&self, distance_to_camp: f32, night: bool) -> f32 {
        if !night && distance_to_camp <= self.camp_density.radius {
            self.camp_density.factor
        } else {
            1.0
        }
    }

    /// Biome density multiplier; unknown biomes count as 1.
    pub fn density_for(&self, biome_name: &str) -> f32 {
        for (k, v) in &self.biome_density {
            if k.eq_ignore_ascii_case(biome_name) {
                return *v;
            }
        }
        1.0
    }

    /// Biome density multiplier for a typed `BiomeId`.
    pub fn density_for_biome(&self, biome: BiomeId) -> f32 {
        self.density_for(biome.key())
    }

    /// Every zombie type id that either table can spawn.
    pub fn all_type_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        for entry in self.types.day.iter().chain(self.types.night.iter()) {
            if !ids.contains(&entry.id) {
                ids.push(entry.id.clone());
            }
        }
        if let Some(city) = &self.city {
            for entry in &city.core_night_types {
                if !ids.contains(&entry.id) {
                    ids.push(entry.id.clone());
                }
            }
        }
        ids
    }

    /// Picks a zombie type id by weight from the day or night table.
    pub fn pick_type_id<'a>(&'a self, rng: &mut SpawnerRng, night: bool) -> &'a str {
        self.pick_type_id_core(rng, night, false)
    }

    /// Picks a zombie type id by weight taking day/night and city core into account.
    pub fn pick_type_id_core<'a>(
        &'a self,
        rng: &mut SpawnerRng,
        night: bool,
        in_city_core: bool,
    ) -> &'a str {
        if night && in_city_core && let Some(city) = &self.city && !city.core_night_types.is_empty() {
            Self::pick_from_slice(rng, &city.core_night_types)
        } else {
            let table = if night {
                &self.types.night
            } else {
                &self.types.day
            };
            Self::pick_from_slice(rng, table)
        }
    }

    /// Picks a zombie type id by weight from a slice of entries.
    pub fn pick_from_slice<'a>(rng: &mut SpawnerRng, table: &'a [SpawnTypeEntry]) -> &'a str {
        let mut total = 0.0f32;
        for entry in table {
            total += entry.weight.max(0.0);
        }
        if total <= 0.0 || table.is_empty() {
            return "walker";
        }
        let mut roll = rng.range(0.0, total);
        for entry in table {
            roll -= entry.weight.max(0.0);
            if roll <= 0.0 {
                return &entry.id;
            }
        }
        &table[table.len() - 1].id
    }
}

/// Deterministic xorshift64 random generator for reproduction across platforms.
#[derive(Clone, Debug)]
pub struct SpawnerRng(u64);

impl SpawnerRng {
    pub fn new(seed: u64) -> Self {
        Self(if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        })
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / 16_777_216.0
    }

    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.next_f32()
    }

    pub fn next_bool(&mut self) -> bool {
        (self.next_u64() & 1) != 0
    }
}

/// Position and state of one active zombie, used to check despawns.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActiveZombie {
    pub id: u64,
    pub position: Vec3,
    pub is_chasing: bool,
}

impl ActiveZombie {
    pub fn new(id: u64, position: Vec3) -> Self {
        Self {
            id,
            position,
            is_chasing: false,
        }
    }

    pub fn chasing(id: u64, position: Vec3) -> Self {
        Self {
            id,
            position,
            is_chasing: true,
        }
    }
}

impl From<(u64, Vec3)> for ActiveZombie {
    fn from((id, position): (u64, Vec3)) -> Self {
        Self::new(id, position)
    }
}

impl From<(u64, Vec3, bool)> for ActiveZombie {
    fn from((id, position, is_chasing): (u64, Vec3, bool)) -> Self {
        Self {
            id,
            position,
            is_chasing,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpawnOrder {
    pub type_id: String,
    pub position: Vec3,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpawnOrders {
    pub spawns: Vec<SpawnOrder>,
    pub despawns: Vec<u64>,
}
