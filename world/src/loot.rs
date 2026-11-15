use glam::{IVec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::biome::BiomeId;
use crate::generator::WorldGenerator;
use crate::poi::{PoiConfigData, PoiGeometry, PoiInstance};
use crate::random::DotnetRandom;
use crate::scatter::{PropInstance, ScatterConfigData, ScatterLootData, ScatterPlacer};

/// Where one loot crate goes in a chunk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LootSpot {
    pub crate_id: String,
    pub position: Vec3,
    pub yaw: f32,
    pub biome: BiomeId,
}

impl LootSpot {
    pub fn new(crate_id: impl Into<String>, position: Vec3, yaw: f32, biome: BiomeId) -> Self {
        Self {
            crate_id: crate_id.into(),
            position,
            yaw,
            biome,
        }
    }
}

/// Picks loot crate spots for one chunk from seed and rules in scatter.json.
pub struct LootPlacer;

impl LootPlacer {
    const ATTEMPTS_PER_CRATE: i32 = 20;
    /// Keeps the crate random numbers apart from prop numbers, which use the same chunk seed.
    const PLACEMENT_SALT: i32 = 0x3C6EF372;

    pub fn place(
        generator: &WorldGenerator,
        scatter_config: &ScatterConfigData,
        poi_config: &PoiConfigData,
        coord: IVec2,
        props: &[PropInstance],
        pois: &[PoiInstance],
    ) -> Vec<LootSpot> {
        let cfg = &scatter_config.loot;
        let seed = ScatterPlacer::stable_seed(generator.seed(), coord) ^ Self::PLACEMENT_SALT;
        let mut rng = DotnetRandom::new(seed);
        let mut placed = Vec::new();

        if rng.next_double() < cfg.crate_chance_per_chunk as f64 {
            Self::try_place(generator, scatter_config, poi_config, coord, &mut rng, cfg, props, pois, &mut placed);
            if placed.len() == 1 && rng.next_double() < cfg.second_crate_chance as f64 {
                Self::try_place(generator, scatter_config, poi_config, coord, &mut rng, cfg, props, pois, &mut placed);
            }
        }

        placed
    }

    #[allow(clippy::too_many_arguments)]
    fn try_place(
        generator: &WorldGenerator,
        scatter_config: &ScatterConfigData,
        poi_config: &PoiConfigData,
        coord: IVec2,
        rng: &mut DotnetRandom,
        cfg: &ScatterLootData,
        props: &[PropInstance],
        pois: &[PoiInstance],
        placed: &mut Vec<LootSpot>,
    ) {
        let clear = scatter_config.spawn_clear_radius;
        for _ in 0..Self::ATTEMPTS_PER_CRATE {
            let x = coord.x as f32 * WorldGenerator::CHUNK_SIZE
                + (rng.next_double() as f32) * WorldGenerator::CHUNK_SIZE;
            let z = coord.y as f32 * WorldGenerator::CHUNK_SIZE
                + (rng.next_double() as f32) * WorldGenerator::CHUNK_SIZE;
            let yaw = (rng.next_double() * std::f64::consts::PI * 2.0) as f32;

            if x * x + z * z < clear * clear {
                continue;
            }
            if ScatterPlacer::ground_normal(generator, x, z).y < cfg.min_ground_flatness {
                continue;
            }
            if Self::is_too_close_to_props(x, z, cfg.min_distance_from_props, props, placed) {
                continue;
            }
            if PoiGeometry::is_in_any_lot(pois, poi_config, x, z, 0.0) {
                continue;
            }

            let crate_id = format!("{}_{}_{}", coord.x, coord.y, placed.len());
            let position = Vec3::new(x, generator.height(x, z), z);
            let biome = generator.biome(x, z);
            placed.push(LootSpot::new(crate_id, position, yaw, biome));
            return;
        }
    }

    fn is_too_close_to_props(
        x: f32,
        z: f32,
        min_distance: f32,
        props: &[PropInstance],
        placed: &[LootSpot],
    ) -> bool {
        let min2 = min_distance * min_distance;
        for prop in props {
            let dx = prop.position.x - x;
            let dz = prop.position.z - z;
            if dx * dx + dz * dz < min2 {
                return true;
            }
        }
        for crate_spot in placed {
            let dx = crate_spot.position.x - x;
            let dz = crate_spot.position.z - z;
            if dx * dx + dz * dz < min2 {
                return true;
            }
        }
        false
    }
}
