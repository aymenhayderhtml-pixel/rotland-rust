use std::path::Path;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::biome::BiomeId;
use crate::math::smooth_step;
use crate::noise::GodotNoise;

// new: DECISIONS 2026-11-25
const WARP_SEED_OFFSET: i32 = 201;
// data: world_layout.json sector_blend_deg
const MAX_SECTOR_BLEND: f32 = 0.5;
// new: DECISIONS 2026-11-25
const FULL_CIRCLE_DEG: f32 = 360.0;
// new: DECISIONS 2026-11-25
const HALF_SPAN: f32 = 0.5;

/// One directional outer biome sector in `data/world_layout.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectorConfig {
    pub biome: BiomeId,
    pub center_deg: f32,
}

/// Danger tier scaling entry in `data/world_layout.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TierConfig {
    pub zombie_mult: f32,
    pub loot_rolls_bonus: u32,
}

/// Radial ring, outer sector, border warp, and danger tier configuration from `data/world_layout.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldLayoutData {
    pub core_radius: f32,
    pub blend_band: f32,
    pub sectors: Vec<SectorConfig>,
    pub sector_blend_deg: f32,
    pub warp_deg: f32,
    pub warp_frequency: f32,
    pub tier_band: f32,
    pub tiers: Vec<TierConfig>,
}

impl WorldLayoutData {
    /// Loads `data/world_layout.json` via `paths::try_repo_path`.
    pub fn load() -> Result<Self, String> {
        let path = paths::try_repo_path("data/world_layout.json")
            .map_err(|e| format!("data/world_layout.json: {e}"))?;
        Self::load_from_file(path)
    }

    /// Loads world layout data from an explicit file path.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&json)
    }

    /// Parses world layout data from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let data: Self = serde_json::from_str(json)
            .map_err(|e| format!("failed to parse world_layout JSON: {e}"))?;
        if data.sectors.is_empty() {
            return Err("world_layout.json must define at least one sector".to_string());
        }
        Ok(data)
    }

}

/// Outer biome sector and danger tier sample at a world `(x, z)` coordinate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiomeSample {
    pub primary: BiomeId,
    pub secondary: BiomeId,
    /// `0.0` = all `primary`, rising smoothly up to `0.5` right on a sector border.
    pub blend: f32,
    /// `0.0` inside `core_radius`, smoothly rising to `1.0` at `core_radius + blend_band`.
    pub outer: f32,
    /// `0` inside `core_radius`, `1..=tiers.len()` outside.
    pub tier: u8,
}

#[derive(Debug, Clone, Copy)]
struct SortedSector {
    biome: BiomeId,
    lower_border_deg: f32,
    upper_border_deg: f32,
}

/// Evaluates outer biome sectors, border blending, core blend weight, and danger tiers.
#[derive(Debug, Clone)]
pub struct WorldLayout {
    data: WorldLayoutData,
    origin: Vec2,
    reference_dir: Vec2,
    warp_noise: GodotNoise,
    sorted_sectors: Vec<SortedSector>,
}

impl WorldLayout {
    /// Creates a new layout anchored at `origin` (camp pad center) with `reference_dir` (camp -> Ashford, or `+X`).
    pub fn new(data: &WorldLayoutData, seed: i32, origin: Vec2, reference_dir: Vec2) -> Self {
        let ref_dir = reference_dir.try_normalize().unwrap_or(Vec2::X);
        let warp_noise = GodotNoise::simplex(seed.wrapping_add(WARP_SEED_OFFSET), data.warp_frequency);
        let sorted_sectors = Self::build_sorted_sectors(&data.sectors);
        Self {
            data: data.clone(),
            origin,
            reference_dir: ref_dir,
            warp_noise,
            sorted_sectors,
        }
    }

    pub fn data(&self) -> &WorldLayoutData {
        &self.data
    }

    pub fn origin(&self) -> Vec2 {
        self.origin
    }

    pub fn reference_dir(&self) -> Vec2 {
        self.reference_dir
    }

    /// Unit XZ direction rotated by `deg` counter-clockwise from `reference_dir` when viewed from above (`+Y` up).
    pub fn direction_at_deg(&self, deg: f32) -> Vec2 {
        let (sin_a, cos_a) = deg.to_radians().sin_cos();
        // Right-handed Y-up rotation around +Y: (x, z) -> (x cos a + z sin a, -x sin a + z cos a)
        Vec2::new(
            self.reference_dir.x * cos_a + self.reference_dir.y * sin_a,
            -self.reference_dir.x * sin_a + self.reference_dir.y * cos_a,
        )
    }

    /// Samples the outer biome sector, border blend, core-to-outer weight, and danger tier at `(x, z)`.
    pub fn sample(&self, x: f32, z: f32) -> BiomeSample {
        let delta = Vec2::new(x, z) - self.origin;
        let dist = delta.length();
        let outer = smooth_step(
            self.data.core_radius,
            self.data.core_radius + self.data.blend_band,
            dist,
        );
        let tier = self.tier_at_distance(dist);

        let base_deg = self.angle_deg(delta);
        let warp = self.warp_noise.get_noise_2d(x, z) * self.data.warp_deg;
        let warped_deg = (base_deg + warp).rem_euclid(FULL_CIRCLE_DEG);

        let (primary, secondary, blend) = self.sample_sector(warped_deg);
        BiomeSample {
            primary,
            secondary,
            blend,
            outer,
            tier,
        }
    }

    fn tier_at_distance(&self, dist: f32) -> u8 {
        if dist <= self.data.core_radius || self.data.tiers.is_empty() {
            return 0;
        }
        // new: DECISIONS 2026-11-25
        let step = ((dist - self.data.core_radius) / self.data.tier_band).floor() as usize;
        // new: DECISIONS 2026-11-25
        let raw_tier = 1 + step;
        raw_tier.min(self.data.tiers.len()) as u8
    }

    fn angle_deg(&self, delta: Vec2) -> f32 {
        if delta.length_squared() <= f32::EPSILON {
            return 0.0;
        }
        let forward = delta.dot(self.reference_dir);
        let ccw_left = delta.x * self.reference_dir.y - delta.y * self.reference_dir.x;
        ccw_left.atan2(forward).to_degrees().rem_euclid(FULL_CIRCLE_DEG)
    }

    fn sample_sector(&self, deg: f32) -> (BiomeId, BiomeId, f32) {
        let count = self.sorted_sectors.len();
        if count == 1 {
            let only = self.sorted_sectors[0].biome;
            return (only, only, 0.0);
        }

        let mut chosen = 0;
        for (idx, sector) in self.sorted_sectors.iter().enumerate() {
            let span = (sector.upper_border_deg - sector.lower_border_deg).rem_euclid(FULL_CIRCLE_DEG);
            let offset = (deg - sector.lower_border_deg).rem_euclid(FULL_CIRCLE_DEG);
            if offset < span || (span == 0.0 && offset == 0.0) {
                chosen = idx;
                break;
            }
        }

        let current = self.sorted_sectors[chosen];
        let prev = self.sorted_sectors[(chosen + count - 1) % count];
        let next = self.sorted_sectors[(chosen + 1) % count];

        let dist_lower = (deg - current.lower_border_deg).rem_euclid(FULL_CIRCLE_DEG);
        let dist_upper = (current.upper_border_deg - deg).rem_euclid(FULL_CIRCLE_DEG);

        let (dist_border, neighbor) = if dist_lower <= dist_upper {
            (dist_lower, prev.biome)
        } else {
            (dist_upper, next.biome)
        };

        if self.data.sector_blend_deg <= 0.0 || dist_border >= self.data.sector_blend_deg {
            return (current.biome, current.biome, 0.0);
        }

        let t = 1.0 - (dist_border / self.data.sector_blend_deg).clamp(0.0, 1.0);
        let blend = MAX_SECTOR_BLEND * smooth_step(0.0, 1.0, t);
        (current.biome, neighbor, blend)
    }

    fn build_sorted_sectors(sectors: &[SectorConfig]) -> Vec<SortedSector> {
        let mut ordered: Vec<(BiomeId, f32)> = sectors
            .iter()
            .map(|s| (s.biome, s.center_deg.rem_euclid(FULL_CIRCLE_DEG)))
            .collect();
        ordered.sort_by(|a, b| a.1.total_cmp(&b.1));

        let count = ordered.len();
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let (biome, center_deg) = ordered[i];
            let (_, prev_center) = ordered[(i + count - 1) % count];
            let (_, next_center) = ordered[(i + 1) % count];

            let prev_span = if count == 1 {
                FULL_CIRCLE_DEG
            } else {
                (center_deg - prev_center).rem_euclid(FULL_CIRCLE_DEG)
            };
            let next_span = if count == 1 {
                FULL_CIRCLE_DEG
            } else {
                (next_center - center_deg).rem_euclid(FULL_CIRCLE_DEG)
            };

            let lower_border_deg = (prev_center + prev_span * HALF_SPAN).rem_euclid(FULL_CIRCLE_DEG);
            let upper_border_deg = (center_deg + next_span * HALF_SPAN).rem_euclid(FULL_CIRCLE_DEG);

            result.push(SortedSector {
                biome,
                lower_border_deg,
                upper_border_deg,
            });
        }
        result
    }
}
