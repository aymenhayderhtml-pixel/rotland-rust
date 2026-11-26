use glam::{IVec2, Vec2, Vec3};

use crate::biome::BiomeId;
use crate::biome::layout::BiomeSample;
use crate::biome::registry::BiomeRegistry;
use crate::city::{CityGround, CityLayout};
use crate::color::Color;
use crate::generator::{GroundTriangle, WorldGenerator};
use crate::ground_paint::GroundPaint;
use crate::lake::LakeLayout;
use crate::metro::MetroWorld;
use crate::math::{lerp, smooth_step};
use crate::noise::GodotNoise;
use crate::rivers::RiverNetwork;
use crate::roads::{RoadKind, RoadNetwork};
use crate::settlement::{SettlementConfigData, SettlementGround, SettlementSite};
use crate::towns::{BOARDWALK_MODEL, TownSite};

/// Paints chunk triangles for both the exact core world and outer biomes.
pub struct TerrainPaint {
    paint: GroundPaint,
    hue: GodotNoise,
    patch_broad: GodotNoise,
    patch_fine: GodotNoise,
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
    pub roads: Option<&'a RoadNetwork>,
    pub rivers: Option<&'a RiverNetwork>,
    pub towns: &'a [TownSite],
    pub metro: Option<&'a MetroWorld>,
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
    // data: city_ground.json asphalt
    const ASPHALT: Color = Color::new(50.0 / 255.0, 50.0 / 255.0, 52.0 / 255.0, 1.0);
    // data: city_ground.json dirt
    const PACKED_DIRT: Color = Color::new(110.0 / 255.0, 86.0 / 255.0, 56.0 / 255.0, 1.0);
    // data: roads.json highway_width
    const HIGHWAY_HALF_WIDTH_M: f32 = 4.0;
    // data: roads.json trail_width
    const TRAIL_HALF_WIDTH_M: f32 = 2.0;
    // new: DECISIONS 2026-11-25
    const ROAD_PAINT_EDGE_M: f32 = 2.0;
    // new: DECISIONS 2026-11-25
    const RIVERBANK_DARKEN: f32 = 0.62;
    // new: DECISIONS 2026-11-25
    const RIVERBANK_BLEND: f32 = 0.85;
    // new: DECISIONS 2026-11-25
    const RIVERBANK_DAMP_POWER: f32 = 0.75;
    // new: DECISIONS 2026-11-25
    const PATCH_BROAD_WEIGHT: f32 = 0.65;
    // new: DECISIONS 2026-11-25
    const PATCH_FINE_WEIGHT: f32 = 0.35;
    // new: DECISIONS 2026-11-25
    const PATCH_BRIGHTNESS: f32 = 0.08;
    // new: DECISIONS 2026-11-25
    const DESERT_DRY_START: f32 = 0.15;
    // new: DECISIONS 2026-11-25
    const DESERT_DRY_FULL: f32 = 0.65;
    // new: DECISIONS 2026-11-25
    const DESERT_DRY_STRENGTH: f32 = 0.35;
    // new: DECISIONS 2026-11-25
    const DESERT_DRY_COLOR: Color = Color::new(242.0 / 255.0, 222.0 / 255.0, 178.0 / 255.0, 1.0);
    // new: DECISIONS 2026-11-25
    const SNOW_HOLLOW_DEPTH_M: f32 = 2.0;
    // new: DECISIONS 2026-11-25
    const SNOW_HOLLOW_STRENGTH: f32 = 0.55;

    pub fn new(seed: i32) -> Self {
        Self {
            paint: GroundPaint::new(seed),
            // godot: World/WorldGenerator.cs:67
            hue: GodotNoise::simplex(seed + 3, 0.012),
            // new: DECISIONS 2026-11-25
            patch_broad: GodotNoise::simplex(seed + 17, 0.007),
            // new: DECISIONS 2026-11-25
            patch_fine: GodotNoise::simplex(seed + 31, 0.042),
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
            let patch_n = (Self::PATCH_BROAD_WEIGHT * self.patch_broad.get_noise_2d(wx, wz)
                + Self::PATCH_FINE_WEIGHT * self.patch_fine.get_noise_2d(wx, wz))
            .clamp(-1.0, 1.0);
            let mut biome_color =
                self.biome_colour(&sample, tri_y, normal, hue_n, patch_n, ctx.registry);
            biome_color = Self::paint_outer_features(ctx, wx, wz, biome_color);
            core_color.lerp(biome_color, sample.outer)
        };
        if let Some(metro) = ctx.metro {
            color = metro.paint_ground(color, wx, wz);
        }

        color *= 1.0 + Self::BRIGHTNESS_JITTER * Self::jitter(ctx.coord, cell_x, cell_z, half);
        color = Self::apply_hue(color, hue_n);
        GroundTriangle { positions: p, normal, color }
    }

    fn paint_outer_features(
        ctx: &ChunkPaintContext<'_>,
        wx: f32,
        wz: f32,
        mut color: Color,
    ) -> Color {
        let p = Vec2::new(wx, wz);
        if let Some(town) = ctx.towns.iter().find(|t| t.contains_point(p)) {
            if town
                .lots
                .iter()
                .any(|lot| lot.building != BOARDWALK_MODEL && lot.contains_point(p))
            {
                color = Self::PACKED_DIRT;
            }
            let right = Vec2::new(-town.street_dir.y, town.street_dir.x);
            let street_dist = (p - town.center).dot(right).abs();
            if street_dist < Self::HIGHWAY_HALF_WIDTH_M + Self::ROAD_PAINT_EDGE_M {
                let t = if street_dist <= Self::HIGHWAY_HALF_WIDTH_M {
                    0.0
                } else {
                    smooth_step(
                        Self::HIGHWAY_HALF_WIDTH_M,
                        Self::HIGHWAY_HALF_WIDTH_M + Self::ROAD_PAINT_EDGE_M,
                        street_dist,
                    )
                };
                color = Self::ASPHALT.lerp(color, t);
            }
        }

        if let Some(rivers) = ctx.rivers
            && let Some((_, blend)) = rivers.carve(wx, wz)
        {
            let damp = blend.powf(Self::RIVERBANK_DAMP_POWER);
            let wet_bank = Color::new(
                color.r * Self::RIVERBANK_DARKEN,
                color.g * Self::RIVERBANK_DARKEN,
                color.b * Self::RIVERBANK_DARKEN,
                1.0,
            );
            color = color.lerp(wet_bank, damp * Self::RIVERBANK_BLEND);
        }

        let in_water = ctx.rivers.is_some_and(|r| r.water_at(wx, wz).is_some());
        if !in_water
            && let Some(roads) = ctx.roads
            && let Some((dist, kind)) = roads.distance_to_road(p)
        {
            let half_w = match kind {
                RoadKind::Highway => Self::HIGHWAY_HALF_WIDTH_M,
                RoadKind::Trail => Self::TRAIL_HALF_WIDTH_M,
            };
            if dist < half_w + Self::ROAD_PAINT_EDGE_M {
                let road_col = match kind {
                    RoadKind::Highway => Self::ASPHALT,
                    RoadKind::Trail => Self::PACKED_DIRT,
                };
                let t = if dist <= half_w {
                    0.0
                } else {
                    smooth_step(half_w, half_w + Self::ROAD_PAINT_EDGE_M, dist)
                };
                color = road_col.lerp(color, t);
            }
        }
        color
    }

    fn biome_colour(
        &self,
        sample: &BiomeSample,
        tri_y: f32,
        normal: Vec3,
        hue_n: f32,
        patch_n: f32,
        registry: &BiomeRegistry,
    ) -> Color {
        let c_primary = self.single_biome_colour(
            sample.primary,
            sample.outer,
            tri_y,
            normal,
            hue_n,
            patch_n,
            registry,
        );
        if sample.blend > 0.0 && sample.secondary != sample.primary {
            let c_secondary = self.single_biome_colour(
                sample.secondary,
                sample.outer,
                tri_y,
                normal,
                hue_n,
                patch_n,
                registry,
            );
            c_primary.lerp(c_secondary, sample.blend)
        } else {
            c_primary
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn single_biome_colour(
        &self,
        id: BiomeId,
        outer: f32,
        tri_y: f32,
        normal: Vec3,
        hue_n: f32,
        patch_n: f32,
        registry: &BiomeRegistry,
    ) -> Color {
        let def = registry.get(id);
        let alt_mix = ((hue_n * 0.6 + patch_n * 0.4) * 0.5 + 0.5).clamp(0.0, 1.0); // new: DECISIONS 2026-11-25
        let mut c = Self::rgb_color(def.ground.base).lerp(Self::rgb_color(def.ground.alt), alt_mix);
        c *= 1.0 + Self::PATCH_BRIGHTNESS * patch_n;

        if id == BiomeId::Desert && patch_n > Self::DESERT_DRY_START {
            let dry = smooth_step(Self::DESERT_DRY_START, Self::DESERT_DRY_FULL, patch_n);
            c = c.lerp(Self::DESERT_DRY_COLOR, dry * Self::DESERT_DRY_STRENGTH);
        }

        if id == BiomeId::Swamp {
            let local_avg = lerp(Self::CORE_AVG_HEIGHT_M, def.height.base, outer);
            if tri_y < local_avg {
                let low = smooth_step(0.0, Self::SWAMP_LOW_DEPTH_M, local_avg - tri_y);
                c *= 1.0 - Self::SWAMP_LOW_DARKEN * low;
            }
        }

        if id == BiomeId::Snow {
            let local_avg = lerp(Self::CORE_AVG_HEIGHT_M, def.height.base, outer);
            if tri_y < local_avg {
                let hollow = smooth_step(0.0, Self::SNOW_HOLLOW_DEPTH_M, local_avg - tri_y);
                c = c.lerp(Self::SNOW_SHADOW, hollow * Self::SNOW_HOLLOW_STRENGTH);
            }
            if normal.z > Self::SNOW_NORTH_START {
                let shadow = smooth_step(Self::SNOW_NORTH_START, Self::SNOW_NORTH_FULL, normal.z);
                c = c.lerp(Self::SNOW_SHADOW, shadow * Self::SNOW_SHADOW_STRENGTH);
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desert_dry_patches_brighten_and_snow_hollows_tint_blue() {
        let registry = BiomeRegistry::load().expect("load data/biomes.json");
        let paint = TerrainPaint::new(42);

        let desert_base = paint.single_biome_colour(BiomeId::Desert, 1.0, 4.0, Vec3::Y, 0.0, 0.0, &registry);
        let desert_dry = paint.single_biome_colour(BiomeId::Desert, 1.0, 4.0, Vec3::Y, 0.0, 0.8, &registry);
        assert!(desert_dry.r > desert_base.r && desert_dry.g > desert_base.g);

        let snow_flat = paint.single_biome_colour(BiomeId::Snow, 1.0, 10.0, Vec3::Y, 0.0, 0.0, &registry);
        let snow_hollow = paint.single_biome_colour(BiomeId::Snow, 1.0, 0.0, Vec3::Y, 0.0, 0.0, &registry);
        assert!(snow_hollow.b > snow_hollow.r);
        assert!(snow_hollow.r < snow_flat.r);
    }

    #[test]
    fn outer_plains_grass_stays_green_with_patch_variation() {
        let registry = BiomeRegistry::load().expect("load data/biomes.json");
        let paint = TerrainPaint::new(42);
        for p in [-0.9, -0.3, 0.0, 0.4, 0.9] {
            let c = paint.single_biome_colour(BiomeId::Plains, 1.0, 2.0, Vec3::Y, 0.0, p, &registry);
            assert!(c.g > c.r && c.g > c.b, "plains grass must stay green: {c:?}");
        }
    }
}
