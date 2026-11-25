use glam::Vec2;

use crate::biome::BiomeId;
use crate::biome::layout::BiomeSample;
use crate::biome::registry::{BiomeHeight, BiomeRegistry};
use crate::city::{CityGround, CityLayout};
use crate::lake::LakeLayout;
use crate::math::lerp;
use crate::noise::GodotNoise;
use crate::settlement::{SettlementConfigData, SettlementGround, SettlementSite};

/// Evaluates core and outer-biome ground elevations using a single shared core height formula.
pub struct TerrainShape {
    height: GodotNoise,
    detail: GodotNoise,
    biome_noises: [BiomeNoise; BiomeId::COUNT],
    dune: GodotNoise,
}

struct BiomeNoise {
    fbm: GodotNoise,
    effective_amplitude: f32,
}

impl TerrainShape {
    // new: DECISIONS 2026-11-25
    const BIOME_SEED_BASE: i32 = 101;
    // new: DECISIONS 2026-11-25
    const DUNE_SEED_OFFSET: i32 = 121;
    // new: DECISIONS 2026-11-25
    const DUNE_FREQUENCY: f32 = 0.025;
    // new: DECISIONS 2026-11-25
    const DUNE_ALONG_SCALE: f32 = 0.25;
    // new: DECISIONS 2026-11-25
    const DUNE_DIRECTION_RAD: f32 = 0.35;
    // new: DECISIONS 2026-11-25
    const RIDGE_CUSP_EPS: f32 = 0.01;
    // new: DECISIONS 2026-11-25
    const RIDGE_MAX_OCTAVES: i32 = 2;
    // new: DECISIONS 2026-11-25
    const MAX_CELL_STEP_M: f32 = 1.6;
    // new: DECISIONS 2026-11-25
    const SIMPLEX_DERIV_BOUND: f32 = 2.6;

    pub fn new(seed: i32, registry: &BiomeRegistry) -> Self {
        // godot: World/WorldGenerator.cs:53
        let height = GodotNoise::fbm(seed + 1, 0.006, 4);
        // godot: World/WorldGenerator.cs:60
        let detail = GodotNoise::simplex(seed + 2, 0.05);
        let dune = GodotNoise::fbm(
            seed.wrapping_add(Self::DUNE_SEED_OFFSET),
            Self::DUNE_FREQUENCY,
            2,
        );
        let biome_noises = BiomeId::ALL.map(|id| {
            let bh = &registry.get(id).height;
            let octaves = if bh.ridged {
                bh.octaves.clamp(1, Self::RIDGE_MAX_OCTAVES)
            } else {
                bh.octaves.max(1)
            };
            let fbm = GodotNoise::fbm(
                seed.wrapping_add(Self::BIOME_SEED_BASE + id as i32),
                bh.frequency,
                octaves,
            );
            let effective_amplitude = Self::clamped_amplitude(bh, octaves);
            BiomeNoise {
                fbm,
                effective_amplitude,
            }
        });
        Self {
            height,
            detail,
            biome_noises,
            dune,
        }
    }

    /// Limits amplitude when frequency/octaves would otherwise make a 2 m cell rise more than ~1.6 m.
    fn clamped_amplitude(bh: &BiomeHeight, octaves: i32) -> f32 {
        if bh.amplitude <= 0.0 || bh.frequency <= 0.0 {
            return bh.amplitude;
        }
        let weight_sum = 2.0 - 0.5_f32.powi(octaves - 1);
        let ridge_factor = if bh.ridged { 2.0 } else { 1.0 };
        // new: DECISIONS 2026-11-25
        let cell_m = 2.0;
        let max_step_per_amp =
            cell_m * Self::SIMPLEX_DERIV_BOUND * bh.frequency * (octaves as f32) * ridge_factor
                / weight_sum;
        if max_step_per_amp <= 1e-6 {
            return bh.amplitude;
        }
        let max_amp = Self::MAX_CELL_STEP_M / max_step_per_amp;
        bh.amplitude.min(max_amp)
    }

    /// Unflattened core height shared by `raw_height` and `sample` (bit-for-bit identical to legacy).
    pub fn raw_core_height(&self, x: f32, z: f32, core_forest: f32) -> f32 {
        let n = self.height.get_noise_2d(x, z);
        // godot: World/WorldGenerator.cs:89
        lerp(n * 4.0 + 2.0, n * 16.0 + 6.0, core_forest) + self.detail.get_noise_2d(x, z) * 0.4
    }

    /// Unflattened height including outer biome blending.
    pub fn raw_height(
        &self,
        x: f32,
        z: f32,
        core_forest: f32,
        biome_sample: &BiomeSample,
        registry: &BiomeRegistry,
    ) -> f32 {
        let core_h = self.raw_core_height(x, z, core_forest);
        if biome_sample.outer <= 0.0 {
            return core_h;
        }
        let biome_h = self.biome_height(biome_sample, x, z, registry);
        lerp(core_h, biome_h, biome_sample.outer)
    }

    /// Full ground height with city/road/settlement/lake flattening inside the core.
    #[allow(clippy::too_many_arguments)]
    pub fn sample_height(
        &self,
        x: f32,
        z: f32,
        core_forest: f32,
        biome_sample: &BiomeSample,
        registry: &BiomeRegistry,
        settlements: &[SettlementSite],
        config: &SettlementConfigData,
        city: Option<&CityLayout>,
        city_ground: Option<&CityGround>,
        lake: Option<&LakeLayout>,
        raw_height_fn: &dyn Fn(f32, f32) -> f32,
    ) -> f32 {
        let mut core_h = self.raw_core_height(x, z, core_forest);
        if let (Some(cg), Some(city_layout)) = (city_ground, city) {
            core_h = cg.flatten_road(
                core_h,
                x,
                z,
                &city_layout.road,
                settlements,
                config,
                raw_height_fn,
            );
        }
        core_h = SettlementGround::flatten(core_h, x, z, settlements, config);
        if let Some(cg) = city_ground {
            core_h = cg.flatten_city(core_h, x, z);
        }
        if let Some(lake_layout) = lake {
            let p = Vec2::new(x, z);
            if lake_layout.contains_region(p) {
                core_h = lake_layout.ground_height(p, core_h);
            }
        }
        if biome_sample.outer <= 0.0 {
            return core_h;
        }
        let biome_h = self.biome_height(biome_sample, x, z, registry);
        lerp(core_h, biome_h, biome_sample.outer)
    }

    fn biome_height(
        &self,
        biome_sample: &BiomeSample,
        x: f32,
        z: f32,
        registry: &BiomeRegistry,
    ) -> f32 {
        let primary_h = self.single_biome_shape(biome_sample.primary, x, z, registry);
        if biome_sample.blend > 0.0 && biome_sample.secondary != biome_sample.primary {
            let secondary_h = self.single_biome_shape(biome_sample.secondary, x, z, registry);
            lerp(primary_h, secondary_h, biome_sample.blend)
        } else {
            primary_h
        }
    }

    fn single_biome_shape(
        &self,
        id: BiomeId,
        x: f32,
        z: f32,
        registry: &BiomeRegistry,
    ) -> f32 {
        let bh = &registry.get(id).height;
        let bn = &self.biome_noises[id as usize];
        let n = bn.fbm.get_noise_2d(x, z);
        let shaped = if bh.ridged {
            // Smooth the |n| cusp at 0 so ridge crests stay walkable while preserving sharp spines
            let norm_denom = 1.0 - Self::RIDGE_CUSP_EPS.sqrt();
            let ridge = ((1.0 - (n * n + Self::RIDGE_CUSP_EPS).sqrt()) / norm_denom).clamp(0.0, 1.0);
            ridge * ridge
        } else {
            n
        };
        let mut h = bh.base + bn.effective_amplitude * shaped;
        if bh.dune > 0.0 {
            let (sin_a, cos_a) = Self::DUNE_DIRECTION_RAD.sin_cos();
            let along = x * cos_a + z * sin_a;
            let across = -x * sin_a + z * cos_a;
            h += self.dune.get_noise_2d(along * Self::DUNE_ALONG_SCALE, across) * bh.dune;
        }
        h
    }
}
