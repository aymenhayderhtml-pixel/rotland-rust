use glam::{IVec2, Vec3};

use crate::biome::BiomeId;
use crate::biome::layout::BiomeSample;
use crate::biome::registry::BiomeRegistry;
use crate::city::{CityGround, CityLayout};
use crate::color::Color;
use crate::generator::{GroundTriangle, WorldGenerator};
use crate::ground_paint::GroundPaint;
use crate::lake::LakeLayout;
use crate::math::{lerp, smooth_step};
use crate::noise::GodotNoise;
use crate::settlement::{SettlementConfigData, SettlementGround, SettlementSite};

/// Paints chunk triangles for both the exact core world and outer biomes.
pub struct TerrainPaint {
    paint: GroundPaint,
    hue: GodotNoise,
}

/// Per-chunk context passed to triangle painting so `emit_triangle` stays concise.
pub struct ChunkPaintContext<'a> {
    pub coord: IVec2,
    pub heights: &'a [f32],
    pub forests: &'a [f32],
    pub settlements: &'a [SettlementSite],
    pub config: &'a SettlementConfigData,
    pub city: Option<&'a CityLayout>,
    pub city_ground: Option<&'a CityGround>,
    pub lake: Option<&'a LakeLayout>,
    pub registry: &'a BiomeRegistry,
}

impl TerrainPaint {
    // godot: World/WorldGenerator.cs:23
    const BRIGHTNESS_JITTER: f32 = 0.06;
    // godot: World/WorldGenerator.cs:25
    const HUE_SHIFT: f32 = 0.08;
    // new: DECISIONS 2026-11-25
    const PEAK_FADE_M: f32 = 6.0;
    // new: DECISIONS 2026-11-25
    const SNOW_NORTH_START: f32 = 0.3;
    // new: DECISIONS 2026-11-25
    const SNOW_NORTH_FULL: f32 = 0.75;
    // new: DECISIONS 2026-11-25
    const SNOW_SHADOW_STRENGTH: f32 = 0.75;
    // new: DECISIONS 2026-11-25
    const SNOW_SHADOW: Color = Color::new(176.0 / 255.0, 196.0 / 255.0, 222.0 / 255.0, 1.0);
    // new: DECISIONS 2026-11-25
    const SWAMP_LOW_DEPTH_M: f32 = 1.5;
    // new: DECISIONS 2026-11-25
    const SWAMP_LOW_DARKEN: f32 = 0.42;
    // new: DECISIONS 2026-11-25
    const CORE_AVG_HEIGHT_M: f32 = 3.5;

    pub fn new(seed: i32) -> Self {
        Self {
            paint: GroundPaint::new(seed),
            // godot: World/WorldGenerator.cs:67
            hue: GodotNoise::simplex(seed + 3, 0.012),
        }
    }

    pub fn build_triangles(
        &self,
        ctx: &ChunkPaintContext<'_>,
        biome_at: &dyn Fn(f32, f32) -> BiomeSample,
    ) -> Vec<GroundTriangle> {
        let cells = WorldGenerator::CELLS;
        let n = WorldGenerator::SAMPLES;
        let mut triangles = Vec::with_capacity(cells * cells * 2);
        for j in 0..cells {
            for i in 0..cells {
                let a = j * n + i;
                let (b, c, d) = (a + 1, a + n, a + n + 1);
                triangles.push(self.emit_triangle(ctx, i as i32, j as i32, 0, [a, b, c], biome_at));
                triangles.push(self.emit_triangle(ctx, i as i32, j as i32, 1, [b, d, c], biome_at));
            }
        }
        triangles
    }

    fn emit_triangle(
        &self,
        ctx: &ChunkPaintContext<'_>,
        cell_x: i32,
        cell_z: i32,
        half: i32,
        corners: [usize; 3],
        biome_at: &dyn Fn(f32, f32) -> BiomeSample,
    ) -> GroundTriangle {
        let mut p = [Vec3::ZERO; 3];
        for (k, &idx) in corners.iter().enumerate() {
            p[k] = Vec3::new(
                (idx % WorldGenerator::SAMPLES) as f32 * WorldGenerator::CELL,
                ctx.heights[idx],
                (idx / WorldGenerator::SAMPLES) as f32 * WorldGenerator::CELL,
            );
        }
        let normal = (p[2] - p[0]).cross(p[1] - p[0]).normalize();
        let forest = (ctx.forests[corners[0]] + ctx.forests[corners[1]] + ctx.forests[corners[2]]) / 3.0;
        let wx = ctx.coord.x as f32 * WorldGenerator::CHUNK_SIZE + (p[0].x + p[1].x + p[2].x) / 3.0;
        let wz = ctx.coord.y as f32 * WorldGenerator::CHUNK_SIZE + (p[0].z + p[1].z + p[2].z) / 3.0;
        let tri_y = (p[0].y + p[1].y + p[2].y) / 3.0;

        let camp_distance = SettlementGround::nearest_box_distance(ctx.settlements, wx, wz);
        let mut core_color = self.paint.paint(
            WorldGenerator::PLAINS_GRASS.lerp(WorldGenerator::FOREST_GRASS, forest),
            wx,
            wz,
            forest,
            normal.y,
            camp_distance,
        );
        if let (Some(city), Some(cg)) = (ctx.city, ctx.city_ground) {
            core_color = cg.paint(core_color, wx, wz, city, ctx.settlements, ctx.config);
        }
        if let Some(lake) = ctx.lake {
            core_color = lake.paint(core_color, wx, wz);
        }

        let hue_n = self.hue.get_noise_2d(wx, wz);
        let sample = biome_at(wx, wz);
        let mut color = if sample.outer <= 0.0 {
            core_color
        } else {
            let biome_color = self.biome_colour(&sample, tri_y, normal, hue_n, ctx.registry);
            core_color.lerp(biome_color, sample.outer)
        };

        color *= 1.0 + Self::BRIGHTNESS_JITTER * Self::jitter(ctx.coord, cell_x, cell_z, half);
        color = Self::apply_hue(color, hue_n);
        GroundTriangle { positions: p, normal, color }
    }

    fn biome_colour(
        &self,
        sample: &BiomeSample,
        tri_y: f32,
        normal: Vec3,
        hue_n: f32,
        registry: &BiomeRegistry,
    ) -> Color {
        let c_primary = self.single_biome_colour(sample.primary, sample.outer, tri_y, normal, hue_n, registry);
        if sample.blend > 0.0 && sample.secondary != sample.primary {
            let c_secondary =
                self.single_biome_colour(sample.secondary, sample.outer, tri_y, normal, hue_n, registry);
            c_primary.lerp(c_secondary, sample.blend)
        } else {
            c_primary
        }
    }

    fn single_biome_colour(
        &self,
        id: BiomeId,
        outer: f32,
        tri_y: f32,
        normal: Vec3,
        hue_n: f32,
        registry: &BiomeRegistry,
    ) -> Color {
        let def = registry.get(id);
        let alt_mix = (hue_n * 0.5 + 0.5).clamp(0.0, 1.0);
        let mut c = Self::rgb_color(def.ground.base).lerp(Self::rgb_color(def.ground.alt), alt_mix);

        if id == BiomeId::Swamp {
            let local_avg = lerp(Self::CORE_AVG_HEIGHT_M, def.height.base, outer);
            if tri_y < local_avg {
                let low = smooth_step(0.0, Self::SWAMP_LOW_DEPTH_M, local_avg - tri_y);
                c *= 1.0 - Self::SWAMP_LOW_DARKEN * low;
            }
        }

        if id == BiomeId::Snow && normal.z > Self::SNOW_NORTH_START {
            let shadow = smooth_step(Self::SNOW_NORTH_START, Self::SNOW_NORTH_FULL, normal.z);
            c = c.lerp(Self::SNOW_SHADOW, shadow * Self::SNOW_SHADOW_STRENGTH);
        }

        let steep = GroundPaint::steepness(normal.y);
        c = c.lerp(Self::rgb_color(def.ground.slope), steep);

        if let Some(peak_rgb) = def.ground.peak {
            let peak_weight = smooth_step(
                def.ground.peak_height,
                def.ground.peak_height + Self::PEAK_FADE_M,
                tri_y,
            );
            c = c.lerp(Self::rgb_color(peak_rgb), peak_weight);
        }

        c
    }

    fn rgb_color(rgb: [u8; 3]) -> Color {
        Color::new(
            f32::from(rgb[0]) / f32::from(u8::MAX),
            f32::from(rgb[1]) / f32::from(u8::MAX),
            f32::from(rgb[2]) / f32::from(u8::MAX),
            1.0,
        )
    }

    fn apply_hue(color: Color, n: f32) -> Color {
        let r = (color.r * (1.0 + Self::HUE_SHIFT * n)).clamp(0.0, 1.0);
        let b = (color.b * (1.0 - Self::HUE_SHIFT * n)).clamp(0.0, 1.0);
        Color { r, g: color.g, b, a: color.a }
    }

    fn jitter(coord: IVec2, cell_x: i32, cell_z: i32, half: i32) -> f32 {
        // godot: World/WorldGenerator.cs:114
        let gx = coord.x * (WorldGenerator::CELLS as i32) + cell_x;
        let gz = coord.y * (WorldGenerator::CELLS as i32) + cell_z;
        let mut h = (gx as u32).wrapping_mul(73856093) // new: DECISIONS 2026-11-25
            ^ (gz as u32).wrapping_mul(19349663)
            ^ (half as u32).wrapping_mul(83492791); // new: DECISIONS 2026-11-25
        h ^= h >> 13; // new: DECISIONS 2026-11-25
        h = h.wrapping_mul(0x5bd1e995); // new: DECISIONS 2026-11-25
        h ^= h >> 15; // new: DECISIONS 2026-11-25
        ((h & 0xFFFF) as f32) / 32767.5 - 1.0 // new: DECISIONS 2026-11-25
    }
}
