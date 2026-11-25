use std::path::Path;

use glam::{IVec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::biome::registry::BiomeRegistry;
use crate::biome::BiomeId;
use crate::generator::WorldGenerator;
use crate::random::DotnetRandom;

/// One scattered tree, rock or bush. Position is in world space.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropInstance {
    pub model: String,
    pub position: Vec3,
    pub yaw: f32,
    pub scale: f32,
}

impl PropInstance {
    pub fn new(model: impl Into<String>, position: Vec3, yaw: f32, scale: f32) -> Self {
        Self {
            model: model.into(),
            position,
            yaw,
            scale,
        }
    }

    /// True for trees (model names start with "tree" or "pine_"). Only trees can be chopped down.
    pub fn is_tree(&self) -> bool {
        self.model.starts_with("tree") || self.model.starts_with("pine_")
    }
}

/// One kind of prop to scatter in a biome, read from data/scatter.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScatterRule {
    pub model: String,
    pub per_chunk: i32,
    #[serde(default = "default_scale")]
    pub scale: [f32; 2],
    pub min_spacing: f32,
    pub max_slope: f32,
    #[serde(default = "default_chance")]
    pub chance: f32,
}

fn default_scale() -> [f32; 2] {
    [1.0, 1.0]
}
fn default_chance() -> f32 {
    1.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ScatterVisibility {
    pub tree: f32,
    pub tree_lod1: f32,
    pub tree_lod2: f32,
    pub rock: f32,
    pub bush: f32,
    pub bush_lod1: f32,
    pub ground: f32,
    pub prop: f32,
}

impl Default for ScatterVisibility {
    fn default() -> Self {
        Self {
            tree: 0.0,
            tree_lod1: 45.0, // new: DECISIONS 2026-11-25
            tree_lod2: 95.0, // new: DECISIONS 2026-11-25
            rock: 0.0,
            bush: 0.0,
            bush_lod1: 28.0, // new: DECISIONS 2026-11-25
            ground: 0.0,
            prop: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ScatterLootData {
    pub crate_chance_per_chunk: f32,
    pub second_crate_chance: f32,
    pub min_distance_from_props: f32,
    pub min_ground_flatness: f32,
}

/// Shape of data/scatter.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScatterConfigData {
    #[serde(default)]
    pub forest: Vec<ScatterRule>,
    #[serde(default)]
    pub plains: Vec<ScatterRule>,
    #[serde(default)]
    pub spawn_clear_radius: f32,
    #[serde(default)]
    pub visibility: ScatterVisibility,
    #[serde(default)]
    pub loot: ScatterLootData,
}

impl ScatterConfigData {
    pub fn rules(&self, biome: BiomeId) -> &[ScatterRule] {
        if biome == BiomeId::Forest {
            &self.forest
        } else {
            &self.plains
        }
    }
}

pub struct ScatterConfig;

impl ScatterConfig {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<ScatterConfigData, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let data: ScatterConfigData = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(data)
    }

    pub fn from_json(json: &str) -> Result<ScatterConfigData, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Clone, Copy)]
enum RegionFilter {
    /// Core-only chunks: matches legacy behaviour without extra `outer` checks.
    Any,
    /// Edge-crossing chunks: accepts only candidates inside `core_radius` (`outer <= 0.0`).
    CoreOnly,
    /// Outer or edge-crossing chunks: accepts only candidates beyond `core_radius` (`outer > 0.0`).
    OuterOnly,
}

pub struct ScatterPlacer;

impl ScatterPlacer {
    const ATTEMPTS_PER_PROP: i32 = 3; // godot: World/ScatterPlacer.cs:13
    const COVERAGE_SAMPLES: usize = 8; // godot: World/ScatterPlacer.cs:14
    const GROUND_SINK: f32 = 0.05; // godot: World/ScatterPlacer.cs:16
    const SLOPE_STEP: f32 = 1.0; // godot: World/ScatterPlacer.cs:16

    /// Legacy core-only scatter path (forest/plains from `data/scatter.json`).
    pub fn place(
        generator: &WorldGenerator,
        config: &ScatterConfigData,
        coord: IVec2,
    ) -> Vec<PropInstance> {
        let mut rng = DotnetRandom::new(Self::stable_seed(generator.seed(), coord));
        let forest = Self::forest_coverage(generator, coord);
        let mut placed = Vec::new();
        let mut spacing = Vec::new();

        Self::place_rules(
            generator,
            config,
            coord,
            &mut rng,
            BiomeId::Forest,
            config.rules(BiomeId::Forest),
            forest,
            RegionFilter::Any,
            &mut placed,
            &mut spacing,
        );
        Self::place_rules(
            generator,
            config,
            coord,
            &mut rng,
            BiomeId::Plains,
            config.rules(BiomeId::Plains),
            1.0 - forest,
            RegionFilter::Any,
            &mut placed,
            &mut spacing,
        );
        placed
    }

    /// Biome-aware scatter path:
    /// - Chunks fully inside `core_radius` run `Self::place` verbatim (identical RNG order for golden).
    /// - Chunks crossing `core_radius` use today's core rules for candidates inside `core_radius` and
    ///   `BiomeRegistry::get(primary).scatter` for candidates beyond `core_radius`.
    /// - Chunks fully outside `core_radius` use `BiomeRegistry::get(primary).scatter` weighted by each
    ///   biome's share of the 8x8 coverage grid.
    pub fn place_with_registry(
        generator: &WorldGenerator,
        config: &ScatterConfigData,
        registry: &BiomeRegistry,
        coord: IVec2,
    ) -> Vec<PropInstance> {
        if Self::is_core_chunk(generator, coord) {
            return Self::place(generator, config, coord);
        }

        let mut rng = DotnetRandom::new(Self::stable_seed(generator.seed(), coord));
        let (core_forest, core_plains, outer_coverages) = Self::biome_coverages(generator, coord);
        let mut placed = Vec::new();
        let mut spacing = Vec::new();

        if core_forest > 0.0 || core_plains > 0.0 {
            Self::place_rules(
                generator,
                config,
                coord,
                &mut rng,
                BiomeId::Forest,
                config.rules(BiomeId::Forest),
                core_forest,
                RegionFilter::CoreOnly,
                &mut placed,
                &mut spacing,
            );
            Self::place_rules(
                generator,
                config,
                coord,
                &mut rng,
                BiomeId::Plains,
                config.rules(BiomeId::Plains),
                core_plains,
                RegionFilter::CoreOnly,
                &mut placed,
                &mut spacing,
            );
        }

        for biome in BiomeId::ALL {
            let coverage = outer_coverages[biome as usize];
            if coverage <= 0.0 {
                continue;
            }
            let rules = &registry.get(biome).scatter;
            Self::place_rules(
                generator,
                config,
                coord,
                &mut rng,
                biome,
                rules,
                coverage,
                RegionFilter::OuterOnly,
                &mut placed,
                &mut spacing,
            );
        }

        placed
    }

    /// True when all four corners of the chunk square sit inside `core_radius` (`outer == 0`).
    pub fn is_core_chunk(generator: &WorldGenerator, coord: IVec2) -> bool {
        let origin_x = coord.x as f32 * WorldGenerator::CHUNK_SIZE;
        let origin_z = coord.y as f32 * WorldGenerator::CHUNK_SIZE;
        let max_x = origin_x + WorldGenerator::CHUNK_SIZE;
        let max_z = origin_z + WorldGenerator::CHUNK_SIZE;
        let corners = [
            (origin_x, origin_z),
            (max_x, origin_z),
            (origin_x, max_z),
            (max_x, max_z),
        ];
        corners
            .iter()
            .all(|&(x, z)| generator.biome_sample(x, z).outer <= 0.0)
    }

    pub fn forest_coverage(generator: &WorldGenerator, coord: IVec2) -> f32 {
        let step = WorldGenerator::CHUNK_SIZE / (Self::COVERAGE_SAMPLES as f32);
        let origin_x = coord.x as f32 * WorldGenerator::CHUNK_SIZE;
        let origin_z = coord.y as f32 * WorldGenerator::CHUNK_SIZE;
        let mut forest = 0;
        for j in 0..Self::COVERAGE_SAMPLES {
            for i in 0..Self::COVERAGE_SAMPLES {
                if generator.biome(
                    origin_x + (i as f32 + 0.5) * step,
                    origin_z + (j as f32 + 0.5) * step,
                ) == BiomeId::Forest
                {
                    forest += 1;
                }
            }
        }
        (forest as f32) / ((Self::COVERAGE_SAMPLES * Self::COVERAGE_SAMPLES) as f32)
    }

    /// Samples the 8x8 chunk grid, splitting core samples (`outer <= 0`) from outer samples (`outer > 0`).
    fn biome_coverages(generator: &WorldGenerator, coord: IVec2) -> (f32, f32, [f32; BiomeId::ALL.len()]) {
        let step = WorldGenerator::CHUNK_SIZE / (Self::COVERAGE_SAMPLES as f32);
        let origin_x = coord.x as f32 * WorldGenerator::CHUNK_SIZE;
        let origin_z = coord.y as f32 * WorldGenerator::CHUNK_SIZE;
        let mut core_forest = 0u32;
        let mut core_plains = 0u32;
        let mut outer_counts = [0u32; BiomeId::ALL.len()];

        for j in 0..Self::COVERAGE_SAMPLES {
            for i in 0..Self::COVERAGE_SAMPLES {
                let x = origin_x + (i as f32 + 0.5) * step;
                let z = origin_z + (j as f32 + 0.5) * step;
                let sample = generator.biome_sample(x, z);
                if sample.outer <= 0.0 {
                    if sample.primary == BiomeId::Forest {
                        core_forest += 1;
                    } else {
                        core_plains += 1;
                    }
                } else {
                    outer_counts[sample.primary as usize] += 1;
                }
            }
        }

        let total = (Self::COVERAGE_SAMPLES * Self::COVERAGE_SAMPLES) as f32;
        let mut outer_cov = [0.0f32; BiomeId::ALL.len()];
        for (slot, &count) in outer_cov.iter_mut().zip(outer_counts.iter()) {
            *slot = (count as f32) / total;
        }
        ((core_forest as f32) / total, (core_plains as f32) / total, outer_cov)
    }

    pub fn ground_normal(generator: &WorldGenerator, x: f32, z: f32) -> Vec3 {
        let dhdx = (generator.height(x + Self::SLOPE_STEP, z)
            - generator.height(x - Self::SLOPE_STEP, z))
            / (2.0 * Self::SLOPE_STEP);
        let dhdz = (generator.height(x, z + Self::SLOPE_STEP)
            - generator.height(x, z - Self::SLOPE_STEP))
            / (2.0 * Self::SLOPE_STEP);
        Vec3::new(-dhdx, 1.0, -dhdz).normalize()
    }

    #[allow(clippy::too_many_arguments)]
    fn place_rules(
        generator: &WorldGenerator,
        config: &ScatterConfigData,
        coord: IVec2,
        rng: &mut DotnetRandom,
        biome: BiomeId,
        rules: &[ScatterRule],
        coverage: f32,
        region: RegionFilter,
        placed: &mut Vec<PropInstance>,
        spacing: &mut Vec<f32>,
    ) {
        if coverage <= 0.0 {
            return;
        }

        for rule in rules {
            if rule.chance < 1.0 && rng.next_double() >= rule.chance as f64 {
                continue;
            }
            let attempts = ((rule.per_chunk as f32
                * Self::ATTEMPTS_PER_PROP as f32
                * coverage)
                .ceil()) as i32;
            let mut accepted = 0;
            for _ in 0..attempts {
                if accepted >= rule.per_chunk {
                    break;
                }
                let x = coord.x as f32 * WorldGenerator::CHUNK_SIZE
                    + (rng.next_double() as f32) * WorldGenerator::CHUNK_SIZE;
                let z = coord.y as f32 * WorldGenerator::CHUNK_SIZE
                    + (rng.next_double() as f32) * WorldGenerator::CHUNK_SIZE;
                if !Self::accepts(generator, config, rule, biome, region, x, z, placed, spacing) {
                    continue;
                }

                let yaw = (rng.next_double() * std::f64::consts::PI * 2.0) as f32;
                let scale =
                    rule.scale[0] + (rng.next_double() as f32) * (rule.scale[1] - rule.scale[0]);
                let ground_y = generator.height(x, z) - Self::GROUND_SINK;
                placed.push(PropInstance::new(
                    &rule.model,
                    Vec3::new(x, ground_y, z),
                    yaw,
                    scale,
                ));
                spacing.push(rule.min_spacing);
                accepted += 1;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn accepts(
        generator: &WorldGenerator,
        config: &ScatterConfigData,
        rule: &ScatterRule,
        biome: BiomeId,
        region: RegionFilter,
        x: f32,
        z: f32,
        placed: &[PropInstance],
        spacing: &[f32],
    ) -> bool {
        match region {
            RegionFilter::Any => {
                if generator.biome(x, z) != biome {
                    return false;
                }
            }
            RegionFilter::CoreOnly => {
                let sample = generator.biome_sample(x, z);
                if sample.outer > 0.0 || sample.primary != biome {
                    return false;
                }
            }
            RegionFilter::OuterOnly => {
                let sample = generator.biome_sample(x, z);
                if sample.outer <= 0.0 || sample.primary != biome {
                    return false;
                }
            }
        }

        let clear = config.spawn_clear_radius;
        if x * x + z * z < clear * clear {
            return false;
        }
        if Self::ground_normal(generator, x, z).y < rule.max_slope {
            return false;
        }

        for (prop, &s) in placed.iter().zip(spacing.iter()) {
            let need = rule.min_spacing.max(s);
            let dx = prop.position.x - x;
            let dz = prop.position.z - z;
            if dx * dx + dz * dz < need * need {
                return false;
            }
        }
        true
    }

    pub fn stable_seed(world_seed: i32, coord: IVec2) -> i32 {
        let mut h = 2166136261u32;
        h = Self::mix(h ^ (world_seed as u32));
        h = Self::mix(h ^ (coord.x as u32));
        h = Self::mix(h ^ (coord.y as u32));
        h as i32
    }

    fn mix(mut h: u32) -> u32 {
        h ^= h >> 16;
        h = h.wrapping_mul(0x7feb352d);
        h ^= h >> 15;
        h = h.wrapping_mul(0x846ca68b);
        h ^= h >> 16;
        h
    }
}
