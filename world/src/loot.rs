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
    /// Danger tier at this crate (0 in the core, 1..=5 outside).
    #[serde(default)]
    pub tier: u8,
    /// Optional loot table override (falls back to the biome table when `None`).
    #[serde(default)]
    pub table: Option<String>,
    /// Optional model override (`None` draws `crate_loot`, `Some("")` draws no model).
    #[serde(default)]
    pub model: Option<String>,
}

impl LootSpot {
    pub fn new(crate_id: impl Into<String>, position: Vec3, yaw: f32, biome: BiomeId) -> Self {
        Self::with_tier(crate_id, position, yaw, biome, 0)
    }

    pub fn with_tier(
        crate_id: impl Into<String>,
        position: Vec3,
        yaw: f32,
        biome: BiomeId,
        tier: u8,
    ) -> Self {
        Self {
            crate_id: crate_id.into(),
            position,
            yaw,
            biome,
            tier,
            table: None,
            model: None,
        }
    }
}

/// Maps a danger tier (0 = core, 1..=N = outer tiers) to its `loot_rolls_bonus` entry.
pub fn loot_rolls_bonus_for_tier(tier: u8, tier_bonuses: &[i32]) -> i32 {
    if tier == 0 || tier_bonuses.is_empty() {
        return 0; // new: DECISIONS 2026-11-25
    }
    let idx = (tier as usize - 1).min(tier_bonuses.len() - 1);
    tier_bonuses[idx].max(0)
}

/// Deterministic total loot roll count for a crate or POI given its table's `[min, max]` rolls
/// and the danger tier's `loot_rolls_bonus`.
pub fn roll_crate_item_count(base_rolls: [i32; 2], bonus_rolls: i32, seed: i32) -> i32 {
    let mut rng = DotnetRandom::new(seed);
    let min_rolls = base_rolls[0].min(base_rolls[1]).max(0);
    let max_rolls = base_rolls[0].max(base_rolls[1]).max(min_rolls);
    let span = max_rolls - min_rolls + 1;
    let rolled = min_rolls + if span > 1 { rng.next(span) } else { 0 };
    rolled + bonus_rolls.max(0)
}

/// Picks loot crate spots for one chunk from seed and rules in scatter.json.
pub struct LootPlacer;

impl LootPlacer {
    const ATTEMPTS_PER_CRATE: i32 = 20; // godot: World/LootPlacer.cs:13
    /// Keeps the crate random numbers apart from prop numbers, which use the same chunk seed.
    const PLACEMENT_SALT: i32 = 0x3C6EF372; // godot: LootPlacer.cs

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
            Self::try_place(
                generator,
                scatter_config,
                poi_config,
                coord,
                &mut rng,
                cfg,
                props,
                pois,
                &mut placed,
            );
            if placed.len() == 1 && rng.next_double() < cfg.second_crate_chance as f64 {
                Self::try_place(
                    generator,
                    scatter_config,
                    poi_config,
                    coord,
                    &mut rng,
                    cfg,
                    props,
                    pois,
                    &mut placed,
                );
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
            let tier = generator.tier(x, z);
            placed.push(LootSpot::with_tier(crate_id, position, yaw, biome, tier));
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
