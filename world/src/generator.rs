use glam::{IVec2, Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::biome::BiomeId;
use crate::color::Color;
use crate::ground_paint::GroundPaint;
use crate::math::{lerp, smooth_step};
use crate::noise::GodotNoise;
use crate::poi::{PoiConfigData, PoiGeometry, PoiInstance, PoiPlacer};
use crate::scatter::{PropInstance, ScatterConfigData, ScatterPlacer};
use crate::settlement::{SettlementConfigData, SettlementGround, SettlementPlacer, SettlementSite};

/// A flat-shaded triangle with local vertex positions, surface normal, and RGBA color.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundTriangle {
    pub positions: [Vec3; 3],
    pub normal: Vec3,
    pub color: Color,
}

/// The 33x33 height samples and flat-shaded triangle list for one 64m chunk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkGround {
    pub coord: IVec2,
    pub heights: Vec<f32>,
    pub triangles: Vec<GroundTriangle>,
}

/// Makes the terrain from a seed and settlement config.
pub struct WorldGenerator {
    seed: i32,
    config: SettlementConfigData,
    scatter_config: ScatterConfigData,
    poi_config: PoiConfigData,
    biome: GodotNoise,
    height: GodotNoise,
    detail: GodotNoise,
    hue: GodotNoise,
    paint: GroundPaint,
    settlements: Vec<SettlementSite>,
}

impl WorldGenerator {
    pub const CHUNK_SIZE: f32 = 64.0;
    pub const CELL: f32 = 2.0;
    pub const SAMPLES: usize = 33;
    pub const CELLS: usize = Self::SAMPLES - 1;

    pub const PLAINS_GRASS: Color = Color::new(93.0 / 255.0, 136.0 / 255.0, 54.0 / 255.0, 1.0);
    pub const FOREST_GRASS: Color = Color::new(63.0 / 255.0, 106.0 / 255.0, 44.0 / 255.0, 1.0);
    const BRIGHTNESS_JITTER: f32 = 0.06;
    const HUE_SHIFT: f32 = 0.08;

    /// The normal way to make a world: reads settlements.json, scatter.json and poi.json from `data_dir`
    /// (the repo's data/ folder). `new` alone has NO props or buildings (empty scatter and POI data).
    pub fn from_data_dir(seed: i32, data_dir: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let dir = data_dir.as_ref();
        let read = |name: &str| dir.join(name);
        let settlements = crate::settlement::SettlementConfig::load_from_file(read("settlements.json")).map_err(|e| format!("settlements.json: {e}"))?;
        let scatter = crate::scatter::ScatterConfig::load_from_file(read("scatter.json")).map_err(|e| format!("scatter.json: {e}"))?;
        let poi = crate::poi::PoiConfig::load_from_file(read("poi.json")).map_err(|e| format!("poi.json: {e}"))?;
        Ok(Self::with_configs(seed, settlements, scatter, poi))
    }

    /// Terrain only (empty scatter and POI data). Use `from_data_dir` for the real world.
    pub fn new(seed: i32, config: SettlementConfigData) -> Self {
        Self::with_configs(seed, config, ScatterConfigData::default(), PoiConfigData::default())
    }

    pub fn with_configs(
        seed: i32,
        config: SettlementConfigData,
        scatter_config: ScatterConfigData,
        poi_config: PoiConfigData,
    ) -> Self {
        let biome = GodotNoise::simplex(seed, 0.0025);
        let height = GodotNoise::fbm(seed + 1, 0.006, 4);
        let detail = GodotNoise::simplex(seed + 2, 0.05);
        let hue = GodotNoise::simplex(seed + 3, 0.012);
        let paint = GroundPaint::new(seed);
        let mut world_gen = Self {
            seed,
            config: config.clone(),
            scatter_config,
            poi_config,
            biome,
            height,
            detail,
            hue,
            paint,
            settlements: Vec::new(),
        };
        let site = SettlementPlacer::find(&world_gen, &config);
        world_gen.settlements = vec![site];
        world_gen
    }

    pub fn seed(&self) -> i32 { self.seed }
    pub fn settlements(&self) -> &[SettlementSite] { &self.settlements }
    pub fn config(&self) -> &SettlementConfigData { &self.config }
    pub fn scatter_config(&self) -> &ScatterConfigData { &self.scatter_config }
    pub fn poi_config(&self) -> &PoiConfigData { &self.poi_config }
    pub fn set_scatter_config(&mut self, cfg: ScatterConfigData) { self.scatter_config = cfg; }
    pub fn set_poi_config(&mut self, cfg: PoiConfigData) { self.poi_config = cfg; }

    /// POIs and props for one chunk, following BuildChunk rules.
    pub fn chunk_contents(&self, coord: IVec2) -> (Vec<PoiInstance>, Vec<PropInstance>) {
        let placed_pois = PoiPlacer::place(self, &self.poi_config, coord);
        let pois = SettlementGround::remove_pois(&self.settlements, placed_pois, &self.poi_config, self.config.clear_radius);
        let placed_props = ScatterPlacer::place(self, &self.scatter_config, coord);
        let cleared_props = PoiGeometry::remove_cleared(&pois, placed_props, &self.poi_config);
        let props = SettlementGround::remove_props(&self.settlements, cleared_props, self.config.clear_radius);
        (pois, props)
    }

    /// Densest forest chunk within 300m of spawn, matching BenchmarkPlacement.
    pub fn densest_forest_near(&self, spawn: Vec3) -> IVec2 {
        let sc = IVec2::new((spawn.x / Self::CHUNK_SIZE).floor() as i32, (spawn.z / Self::CHUNK_SIZE).floor() as i32);
        let (mut best, mut coord) = (-1, IVec2::ZERO);
        let half = Self::CHUNK_SIZE * 0.5;
        for dx in -5..=5 {
            for dz in -5..=5 {
                let cand = sc + IVec2::new(dx, dz);
                let c = Vec2::new(cand.x as f32 * Self::CHUNK_SIZE + half, cand.y as f32 * Self::CHUNK_SIZE + half);
                if (c - Vec2::new(spawn.x, spawn.z)).length() > 300.0 {
                    continue;
                }
                let trees = ScatterPlacer::place(self, &self.scatter_config, cand)
                    .iter()
                    .filter(|p| p.model.starts_with("tree"))
                    .count() as i32;
                if trees > best {
                    best = trees;
                    coord = cand;
                }
            }
        }
        coord
    }

    pub fn sample(&self, x: f32, z: f32) -> (f32, f32) {
        let b = self.biome.get_noise_2d(x, z) * 0.5 + 0.5;
        let forest = smooth_step(0.45, 0.6, b);
        let n = self.height.get_noise_2d(x, z);
        let mut height = lerp(n * 4.0 + 2.0, n * 16.0 + 6.0, forest) + self.detail.get_noise_2d(x, z) * 0.4;
        height = SettlementGround::flatten(height, x, z, &self.settlements, &self.config);
        (height, forest)
    }

    pub fn height(&self, x: f32, z: f32) -> f32 { self.sample(x, z).0 }

    pub fn raw_height(&self, x: f32, z: f32) -> f32 {
        let n = self.height.get_noise_2d(x, z);
        lerp(n * 4.0 + 2.0, n * 16.0 + 6.0, self.forest_weight(x, z)) + self.detail.get_noise_2d(x, z) * 0.4
    }

    pub fn forest_weight(&self, x: f32, z: f32) -> f32 {
        let b = self.biome.get_noise_2d(x, z) * 0.5 + 0.5;
        smooth_step(0.45, 0.6, b)
    }

    pub fn biome(&self, x: f32, z: f32) -> BiomeId {
        if self.forest_weight(x, z) > 0.5 { BiomeId::Forest } else { BiomeId::Plains }
    }

    /// Builds the flat-shaded mesh triangles and collision heights for one 64 m chunk.
    pub fn build_chunk_ground(&self, coord: IVec2) -> ChunkGround {
        let n = Self::SAMPLES;
        let (ox, oz) = (coord.x as f32 * Self::CHUNK_SIZE, coord.y as f32 * Self::CHUNK_SIZE);
        let mut heights = vec![0.0f32; n * n];
        let mut forests = vec![0.0f32; n * n];

        for j in 0..n {
            for i in 0..n {
                let (h, f) = self.sample(ox + i as f32 * Self::CELL, oz + j as f32 * Self::CELL);
                heights[j * n + i] = h;
                forests[j * n + i] = f;
            }
        }

        let cells = Self::CELLS;
        let mut triangles = Vec::with_capacity(cells * cells * 2);
        for j in 0..cells {
            for i in 0..cells {
                let a = j * n + i;
                let (b, c, d) = (a + 1, a + n, a + n + 1);
                triangles.push(self.emit_triangle(coord, i as i32, j as i32, 0, a, b, c, &heights, &forests));
                triangles.push(self.emit_triangle(coord, i as i32, j as i32, 1, b, d, c, &heights, &forests));
            }
        }
        ChunkGround { coord, heights, triangles }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_triangle(
        &self, coord: IVec2, cell_x: i32, cell_z: i32, half: i32,
        i0: usize, i1: usize, i2: usize, heights: &[f32], forests: &[f32],
    ) -> GroundTriangle {
        let corners = [i0, i1, i2];
        let mut p = [Vec3::ZERO; 3];
        for (k, &idx) in corners.iter().enumerate() {
            p[k] = Vec3::new((idx % Self::SAMPLES) as f32 * Self::CELL, heights[idx], (idx / Self::SAMPLES) as f32 * Self::CELL);
        }
        let normal = (p[2] - p[0]).cross(p[1] - p[0]).normalize();
        let forest = (forests[i0] + forests[i1] + forests[i2]) / 3.0;
        let wx = coord.x as f32 * Self::CHUNK_SIZE + (p[0].x + p[1].x + p[2].x) / 3.0;
        let wz = coord.y as f32 * Self::CHUNK_SIZE + (p[0].z + p[1].z + p[2].z) / 3.0;
        let camp_distance = SettlementGround::nearest_box_distance(&self.settlements, wx, wz);
        let mut color = self.paint.paint(Self::PLAINS_GRASS.lerp(Self::FOREST_GRASS, forest), wx, wz, forest, normal.y, camp_distance);
        color *= 1.0 + Self::BRIGHTNESS_JITTER * Self::jitter(coord, cell_x, cell_z, half);
        color = self.hue(color, coord, p[0], p[1], p[2]);
        GroundTriangle { positions: p, normal, color }
    }

    fn hue(&self, color: Color, coord: IVec2, p0: Vec3, p1: Vec3, p2: Vec3) -> Color {
        let x = coord.x as f32 * Self::CHUNK_SIZE + (p0.x + p1.x + p2.x) / 3.0;
        let z = coord.y as f32 * Self::CHUNK_SIZE + (p0.z + p1.z + p2.z) / 3.0;
        let n = self.hue.get_noise_2d(x, z);
        let r = (color.r * (1.0 + Self::HUE_SHIFT * n)).clamp(0.0, 1.0);
        let b = (color.b * (1.0 - Self::HUE_SHIFT * n)).clamp(0.0, 1.0);
        Color { r, g: color.g, b, a: color.a }
    }

    fn jitter(coord: IVec2, cell_x: i32, cell_z: i32, half: i32) -> f32 {
        let gx = coord.x * (Self::CELLS as i32) + cell_x;
        let gz = coord.y * (Self::CELLS as i32) + cell_z;
        let mut h = (gx as u32).wrapping_mul(73856093) ^ (gz as u32).wrapping_mul(19349663) ^ (half as u32).wrapping_mul(83492791);
        h ^= h >> 13;
        h = h.wrapping_mul(0x5bd1e995);
        h ^= h >> 15;
        ((h & 0xFFFF) as f32) / 32767.5 - 1.0
    }
}
