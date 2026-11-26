use glam::{IVec2, Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::biome::BiomeId;
use crate::biome::layout::{BiomeSample, WorldLayout, WorldLayoutData};
use crate::biome::registry::BiomeRegistry;
use crate::city::{CityGround, CityLayout};
use crate::color::Color;
use crate::generator_world::RoadHeightField;
use crate::lake::LakeLayout;
use crate::math::{lerp, smooth_step};
use crate::noise::GodotNoise;
use crate::poi::{PoiConfigData, PoiGeometry, PoiPlacer};
use crate::rivers::RiverNetwork;
use crate::roads::RoadNetwork;
use crate::scatter::{ScatterConfigData, ScatterPlacer};
use crate::settlement::{SettlementConfigData, SettlementSite};
use crate::terrain_paint::{ChunkPaintContext, TerrainPaint};
use crate::terrain_shape::TerrainShape;
use crate::towns::TownSite;

// new: DECISIONS 2026-11-25
const TOWN_EDGE_BLEND_M: f32 = 30.0;
// new: DECISIONS 2026-11-25
const BRIDGE_APPROACH_PAD_M: f32 = 3.0;

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

/// Makes the terrain from a seed, biome registry, world layout, and settlement config.
pub struct WorldGenerator {
    pub(crate) seed: i32,
    pub(crate) config: SettlementConfigData,
    pub(crate) scatter_config: ScatterConfigData,
    pub(crate) poi_config: PoiConfigData,
    pub(crate) biome_registry: BiomeRegistry,
    pub(crate) layout_data: WorldLayoutData,
    pub(crate) layout: WorldLayout,
    pub(crate) origin: Vec2,
    pub(crate) core_radius_sq: f32,
    pub(crate) biome: GodotNoise,
    pub(crate) shape: TerrainShape,
    pub(crate) paint: TerrainPaint,
    pub(crate) settlements: Vec<SettlementSite>,
    pub(crate) city: Option<CityLayout>,
    pub(crate) city_ground: Option<CityGround>,
    pub(crate) lake: Option<LakeLayout>,
    pub(crate) towns: Vec<TownSite>,
    pub(crate) town_avg_heights: Vec<f32>,
    pub(crate) roads: RoadNetwork,
    pub(crate) road_heights: RoadHeightField,
    pub(crate) rivers: RiverNetwork,
    pub(crate) crossings: Vec<(Vec2, f32, f32)>,
    pub(crate) towns_data: crate::towns::TownsData,
    pub(crate) has_outer: bool,
}

impl WorldGenerator {
    // godot: World/WorldGenerator.cs:16
    pub const CHUNK_SIZE: f32 = 64.0;
    // godot: World/WorldGenerator.cs:17
    pub const CELL: f32 = 2.0;
    // godot: World/WorldGenerator.cs:18
    pub const SAMPLES: usize = 33;
    // godot: World/WorldGenerator.cs:19
    pub const CELLS: usize = Self::SAMPLES - 1;

    // godot: World/WorldGenerator.cs:26
    pub const PLAINS_GRASS: Color = Color::new(93.0 / 255.0, 136.0 / 255.0, 54.0 / 255.0, 1.0);
    // godot: World/WorldGenerator.cs:27
    pub const FOREST_GRASS: Color = Color::new(63.0 / 255.0, 106.0 / 255.0, 44.0 / 255.0, 1.0);

    pub fn seed(&self) -> i32 { self.seed }
    pub fn settlements(&self) -> &[SettlementSite] { &self.settlements }
    pub fn config(&self) -> &SettlementConfigData { &self.config }
    pub fn scatter_config(&self) -> &ScatterConfigData { &self.scatter_config }
    pub fn poi_config(&self) -> &PoiConfigData { &self.poi_config }
    pub fn biome_registry(&self) -> &BiomeRegistry { &self.biome_registry }
    pub fn world_layout(&self) -> &WorldLayout { &self.layout }
    pub fn city(&self) -> Option<&CityLayout> { self.city.as_ref() }
    pub fn city_ground(&self) -> Option<&CityGround> { self.city_ground.as_ref() }
    pub fn lake(&self) -> Option<&LakeLayout> { self.lake.as_ref() }
    pub fn towns(&self) -> &[TownSite] { &self.towns }
    pub fn roads(&self) -> &RoadNetwork { &self.roads }
    pub fn rivers(&self) -> &RiverNetwork { &self.rivers }

    /// Returns the town whose circular radius contains `(x, z)`, if any.
    pub fn town_at(&self, x: f32, z: f32) -> Option<&TownSite> {
        let p = Vec2::new(x, z);
        self.towns.iter().find(|t| t.contains_point(p))
    }

    /// True when `(x, z)` lies inside any town building lot or POI building footprint.
    pub fn indoors(&self, x: f32, z: f32) -> bool {
        let p = Vec2::new(x, z);
        if self.towns.iter().any(|t| t.contains_point(p) && t.indoors(p)) {
            return true;
        }
        let coord = IVec2::new(
            (x / Self::CHUNK_SIZE).floor() as i32,
            (z / Self::CHUNK_SIZE).floor() as i32,
        );
        let placed = PoiPlacer::place(self, &self.poi_config, coord);
        if PoiGeometry::is_in_any_lot(&placed, &self.poi_config, x, z, 0.0) {
            return true;
        }
        if let Some(city) = self.city()
            && city.contains(p)
        {
            for (idx, lot) in city.lots.iter().enumerate() {
                let poi = crate::poi::PoiInstance::new(
                    format!("city_{idx}"),
                    &lot.building_id,
                    Vec3::new(lot.center.x, 0.0, lot.center.y),
                    lot.yaw,
                );
                if let Some(def) = self.poi_config.get(&poi.building_id)
                    && PoiGeometry::contains(&poi, def, x, z, 0.0)
                {
                    return true;
                }
            }
        }
        false
    }

    #[inline]
    pub(crate) fn in_core_radius(&self, x: f32, z: f32) -> bool {
        let dx = x - self.origin.x;
        let dz = z - self.origin.y;
        dx * dx + dz * dz <= self.core_radius_sq
    }

    #[inline]
    fn core_forest_weight(&self, x: f32, z: f32) -> f32 {
        // godot: World/WorldGenerator.cs:209
        let b = self.biome.get_noise_2d(x, z) * 0.5 + 0.5;
        smooth_step(0.45, 0.6, b) // godot: World/WorldGenerator.cs:209
    }

    #[inline]
    fn core_biome_sample(&self, x: f32, z: f32) -> BiomeSample {
        let primary = if self.core_forest_weight(x, z) > 0.5 {
            BiomeId::Forest
        } else {
            BiomeId::Plains
        };
        BiomeSample {
            primary,
            secondary: primary,
            blend: 0.0,
            outer: 0.0,
            tier: 0,
        }
    }

    /// Full biome sample: exact legacy Plains/Forest in the core, sector layout outside.
    pub fn biome_sample(&self, x: f32, z: f32) -> BiomeSample {
        if self.in_core_radius(x, z) {
            return self.core_biome_sample(x, z);
        }
        let sample = self.layout.sample(x, z);
        if sample.outer <= 0.0 {
            return self.core_biome_sample(x, z);
        }
        // Inner half of the blend band: the home biome still wins (so props, zombies and the
        // banner switch midway through the fade, not at its first metre).
        if sample.outer < 0.5 {
            let core = self.core_biome_sample(x, z);
            return BiomeSample { secondary: sample.primary, blend: sample.outer, ..sample_with(core, sample) };
        }
        sample
    }

    /// Danger tier: 0 inside the core, 1..=5 in outer rings.
    pub fn tier(&self, x: f32, z: f32) -> u8 {
        if self.in_core_radius(x, z) {
            0
        } else {
            self.layout.sample(x, z).tier
        }
    }

    pub fn sample(&self, x: f32, z: f32) -> (f32, f32) {
        let core_fw = self.core_forest_weight(x, z);
        let sample = if self.in_core_radius(x, z) {
            BiomeSample {
                primary: if core_fw > 0.5 { BiomeId::Forest } else { BiomeId::Plains },
                secondary: if core_fw > 0.5 { BiomeId::Forest } else { BiomeId::Plains },
                blend: 0.0,
                outer: 0.0,
                tier: 0,
            }
        } else {
            self.layout.sample(x, z)
        };
        let mut height = self.shape.sample_height(
            x,
            z,
            core_fw,
            &sample,
            &self.biome_registry,
            &self.settlements,
            &self.config,
            self.city.as_ref(),
            self.city_ground.as_ref(),
            self.lake.as_ref(),
            &|rx, rz| self.raw_height(rx, rz),
        );
        if self.has_outer && sample.outer > 0.0 {
            height = self.apply_outer_height(x, z, height);
        }
        let forest = if sample.outer <= 0.0 {
            core_fw
        } else {
            let outer_fw = if sample.primary == BiomeId::Forest { 1.0 } else { 0.0 };
            lerp(core_fw, outer_fw, sample.outer)
        };
        (height, forest)
    }

    fn apply_outer_height(&self, x: f32, z: f32, mut h: f32) -> f32 {
        let p = Vec2::new(x, z);
        for (town, &avg_h) in self.towns.iter().zip(&self.town_avg_heights) {
            let d = p.distance(town.center);
            if d < town.radius + TOWN_EDGE_BLEND_M {
                let t = if d <= town.radius {
                    0.0
                } else {
                    smooth_step(town.radius, town.radius + TOWN_EDGE_BLEND_M, d)
                };
                h = lerp(avg_h, h, t);
            }
        }

        let at_bridge = self.crossings.iter().any(|(cp, _, span)| {
            let r = span * 0.5 + BRIDGE_APPROACH_PAD_M;
            p.distance_squared(*cp) <= r * r
        });
        let in_water = self.rivers.water_at(x, z).is_some();

        if at_bridge {
            if in_water {
                if let Some((bed, blend)) = self.rivers.carve(x, z) {
                    h = lerp(h, bed, blend);
                }
            } else if let Some((road_h, w)) = self.road_heights.sample(x, z) {
                h = lerp(h, road_h, w);
            }
            return h;
        }

        if let Some((road_h, w)) = self.road_heights.sample(x, z) {
            h = lerp(h, road_h, w);
        }
        if let Some((bed, blend)) = self.rivers.carve(x, z) {
            h = lerp(h, bed, blend);
        }
        h
    }

    pub fn height(&self, x: f32, z: f32) -> f32 { self.sample(x, z).0 }

    pub fn raw_height(&self, x: f32, z: f32) -> f32 {
        let core_fw = self.core_forest_weight(x, z);
        if self.in_core_radius(x, z) {
            return self.shape.raw_core_height(x, z, core_fw);
        }
        let sample = self.layout.sample(x, z);
        self.shape.raw_height(x, z, core_fw, &sample, &self.biome_registry)
    }

    pub fn forest_weight(&self, x: f32, z: f32) -> f32 {
        let core_fw = self.core_forest_weight(x, z);
        if self.in_core_radius(x, z) {
            return core_fw;
        }
        let sample = self.layout.sample(x, z);
        if sample.outer <= 0.0 {
            core_fw
        } else {
            let outer_fw = if sample.primary == BiomeId::Forest { 1.0 } else { 0.0 };
            lerp(core_fw, outer_fw, sample.outer)
        }
    }

    pub fn biome(&self, x: f32, z: f32) -> BiomeId {
        self.biome_sample(x, z).primary
    }

    /// Builds the flat-shaded mesh triangles and collision heights for one 64 m chunk.
    pub fn build_chunk_ground(&self, coord: IVec2) -> ChunkGround {
        let n = Self::SAMPLES;
        let (ox, oz) = (coord.x as f32 * Self::CHUNK_SIZE, coord.y as f32 * Self::CHUNK_SIZE);
        let mut heights = vec![0.0f32; n * n];
        let mut core_forests = vec![0.0f32; n * n];

        for j in 0..n {
            for i in 0..n {
                let wx = ox + i as f32 * Self::CELL;
                let wz = oz + j as f32 * Self::CELL;
                let (h, _) = self.sample(wx, wz);
                heights[j * n + i] = h;
                core_forests[j * n + i] = self.core_forest_weight(wx, wz);
            }
        }

        let ctx = ChunkPaintContext {
            coord,
            heights: &heights,
            forests: &core_forests,
            settlements: &self.settlements,
            config: &self.config,
            city: self.city.as_ref(),
            city_ground: self.city_ground.as_ref(),
            lake: self.lake.as_ref(),
            registry: &self.biome_registry,
            roads: self.has_outer.then_some(&self.roads),
            rivers: self.has_outer.then_some(&self.rivers),
            towns: &self.towns,
        };
        let triangles = self.paint.build_triangles(&ctx, &|wx, wz| self.biome_sample(wx, wz));
        ChunkGround { coord, heights, triangles }
    }

    /// Densest forest chunk within 300m of spawn, matching BenchmarkPlacement.
    pub fn densest_forest_near(&self, spawn: Vec3) -> IVec2 {
        let sc = IVec2::new(
            (spawn.x / Self::CHUNK_SIZE).floor() as i32,
            (spawn.z / Self::CHUNK_SIZE).floor() as i32,
        );
        let (mut best, mut coord) = (-1, IVec2::ZERO);
        let half = Self::CHUNK_SIZE * 0.5;
        for dx in -5..=5 {
            for dz in -5..=5 {
                let cand = sc + IVec2::new(dx, dz);
                let c = Vec2::new(
                    cand.x as f32 * Self::CHUNK_SIZE + half,
                    cand.y as f32 * Self::CHUNK_SIZE + half,
                );
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
}

/// The core sample's biome, with the outer sample's `outer` and `tier`.
fn sample_with(core: BiomeSample, outer: BiomeSample) -> BiomeSample {
    BiomeSample { outer: outer.outer, tier: outer.tier, ..core }
}
