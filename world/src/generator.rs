use glam::{IVec2, Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::biome::BiomeId;
use crate::biome::layout::{BiomeSample, WorldLayout, WorldLayoutData};
use crate::biome::registry::BiomeRegistry;
use crate::city::{CityConfigData, CityGround, CityGroundData, CityLayout};
use crate::color::Color;
use crate::lake::{LakeConfigData, LakeLayout};
use crate::math::{lerp, smooth_step};
use crate::noise::GodotNoise;
use crate::poi::{PoiConfigData, PoiGeometry, PoiInstance, PoiPlacer};
use crate::scatter::{PropInstance, ScatterConfigData, ScatterPlacer};
use crate::settlement::{SettlementConfigData, SettlementGround, SettlementPlacer, SettlementSite};
use crate::terrain_paint::{ChunkPaintContext, TerrainPaint};
use crate::terrain_shape::TerrainShape;

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
    seed: i32,
    config: SettlementConfigData,
    scatter_config: ScatterConfigData,
    poi_config: PoiConfigData,
    biome_registry: BiomeRegistry,
    layout_data: WorldLayoutData,
    layout: WorldLayout,
    origin: Vec2,
    core_radius_sq: f32,
    biome: GodotNoise,
    shape: TerrainShape,
    paint: TerrainPaint,
    settlements: Vec<SettlementSite>,
    city: Option<CityLayout>,
    city_ground: Option<CityGround>,
    lake: Option<LakeLayout>,
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

    /// The normal way to make a world: reads settlements, scatter, poi, biomes, and world_layout
    /// from `data_dir` (the repo's data/ folder).
    pub fn from_data_dir(seed: i32, data_dir: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let dir = data_dir.as_ref();
        let read = |name: &str| dir.join(name);
        let settlements = crate::settlement::SettlementConfig::load_from_file(read("settlements.json"))
            .map_err(|e| format!("settlements.json: {e}"))?;
        let scatter = crate::scatter::ScatterConfig::load_from_file(read("scatter.json"))
            .map_err(|e| format!("scatter.json: {e}"))?;
        let poi = crate::poi::PoiConfig::load_from_file(read("poi.json"))
            .map_err(|e| format!("poi.json: {e}"))?;
        let biomes_text = std::fs::read_to_string(read("biomes.json"))
            .map_err(|e| format!("biomes.json: {e}"))?;
        let biome_registry = BiomeRegistry::from_json(&biomes_text)
            .map_err(|e| format!("biomes.json: {e}"))?;
        let layout_text = std::fs::read_to_string(read("world_layout.json"))
            .map_err(|e| format!("world_layout.json: {e}"))?;
        let layout_data: WorldLayoutData = serde_json::from_str(&layout_text)
            .map_err(|e| format!("world_layout.json: {e}"))?;

        let mut generator = Self::with_all_configs(
            seed,
            settlements,
            scatter,
            poi,
            biome_registry,
            layout_data,
        );
        let city_path = read("city.json");
        if city_path.exists() {
            let city = CityConfigData::load_from_file(city_path)
                .map_err(|e| format!("city.json: {e}"))?;
            let city_ground_path = read("city_ground.json");
            let ground_cfg = if city_ground_path.exists() {
                Some(
                    CityGroundData::load_from_file(city_ground_path)
                        .map_err(|e| format!("city_ground.json: {e}"))?,
                )
            } else {
                None
            };
            generator.attach_city_with_ground(&city, ground_cfg.as_ref());
        }
        let lake_path = read("lake.json");
        if lake_path.exists() {
            let lake = LakeConfigData::load_from_file(lake_path)
                .map_err(|e| format!("lake.json: {e}"))?;
            generator.attach_lake(&lake);
        }
        Ok(generator)
    }

    /// Lays out the ruined city around the camp and orients the outer biome layout toward Ashford.
    pub fn attach_city(&mut self, cfg: &CityConfigData) {
        self.attach_city_with_ground(cfg, None);
    }

    pub fn attach_city_with_ground(&mut self, cfg: &CityConfigData, ground_cfg: Option<&CityGroundData>) {
        let camp = self.settlements.first().map_or(Vec2::ZERO, |s| Vec2::new(s.center.x, s.center.z));
        let mut layout = CityLayout::generate(self.seed, camp, cfg, |p| self.forest_weight(p.x, p.y));
        layout.road = vec![camp, layout.entry];
        if let Some(g_cfg) = ground_cfg {
            let cg = CityGround::new(
                self.seed,
                &layout,
                &|x, z| self.raw_height(x, z),
                cfg.max_slope,
                g_cfg.clone(),
            );
            self.city_ground = Some(cg);
        }
        let city_center = layout.origin + Vec2::splat(layout.size * 0.5);
        let to_city = city_center - camp;
        let reference_dir = if to_city.length_squared() > 1e-6 {
            to_city.normalize()
        } else {
            Vec2::X
        };
        self.origin = camp;
        self.layout = WorldLayout::new(&self.layout_data, self.seed, camp, reference_dir);
        self.city = Some(layout);
    }

    pub fn city(&self) -> Option<&CityLayout> { self.city.as_ref() }
    pub fn city_ground(&self) -> Option<&CityGround> { self.city_ground.as_ref() }

    /// Attaches the lake & coast opposite the city (or camp if no city).
    pub fn attach_lake(&mut self, cfg: &LakeConfigData) {
        let camp = self.settlements.first().map_or(Vec2::ZERO, |s| Vec2::new(s.center.x, s.center.z));
        let city_center = self.city.as_ref().map(|c| c.origin + Vec2::splat(c.size * 0.5));
        let layout = LakeLayout::generate(self.seed, camp, city_center, cfg, |p| self.raw_height(p.x, p.y));
        self.lake = Some(layout);
    }

    pub fn lake(&self) -> Option<&LakeLayout> { self.lake.as_ref() }

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
        let biome_registry = BiomeRegistry::load().expect("failed to load data/biomes.json");
        let layout_text = std::fs::read_to_string(paths::repo_path("data/world_layout.json"))
            .expect("failed to read data/world_layout.json");
        let layout_data: WorldLayoutData =
            serde_json::from_str(&layout_text).expect("failed to parse data/world_layout.json");
        Self::with_all_configs(
            seed,
            config,
            scatter_config,
            poi_config,
            biome_registry,
            layout_data,
        )
    }

    pub fn with_all_configs(
        seed: i32,
        config: SettlementConfigData,
        scatter_config: ScatterConfigData,
        poi_config: PoiConfigData,
        biome_registry: BiomeRegistry,
        layout_data: WorldLayoutData,
    ) -> Self {
        // godot: World/WorldGenerator.cs:45
        let biome = GodotNoise::simplex(seed, 0.0025);
        let shape = TerrainShape::new(seed, &biome_registry);
        let paint = TerrainPaint::new(seed);
        let core_radius_sq = layout_data.core_radius * layout_data.core_radius;
        let initial_layout = WorldLayout::new(&layout_data, seed, Vec2::ZERO, Vec2::X);

        let mut world_gen = Self {
            seed,
            config: config.clone(),
            scatter_config,
            poi_config,
            biome_registry,
            layout_data,
            layout: initial_layout,
            origin: Vec2::ZERO,
            core_radius_sq,
            biome,
            shape,
            paint,
            settlements: Vec::new(),
            city: None,
            city_ground: None,
            lake: None,
        };
        let site = SettlementPlacer::find(&world_gen, &config);
        let camp = Vec2::new(site.center.x, site.center.z);
        world_gen.origin = camp;
        world_gen.layout = WorldLayout::new(&world_gen.layout_data, seed, camp, Vec2::X);
        world_gen.settlements = vec![site];
        world_gen
    }

    pub fn seed(&self) -> i32 { self.seed }
    pub fn settlements(&self) -> &[SettlementSite] { &self.settlements }
    pub fn config(&self) -> &SettlementConfigData { &self.config }
    pub fn scatter_config(&self) -> &ScatterConfigData { &self.scatter_config }
    pub fn poi_config(&self) -> &PoiConfigData { &self.poi_config }
    pub fn biome_registry(&self) -> &BiomeRegistry { &self.biome_registry }
    pub fn world_layout(&self) -> &WorldLayout { &self.layout }
    pub fn set_scatter_config(&mut self, cfg: ScatterConfigData) { self.scatter_config = cfg; }
    pub fn set_poi_config(&mut self, cfg: PoiConfigData) { self.poi_config = cfg; }

    #[inline]
    fn in_core_radius(&self, x: f32, z: f32) -> bool {
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
        let height = self.shape.sample_height(
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
        let forest = if sample.outer <= 0.0 {
            core_fw
        } else {
            let outer_fw = if sample.primary == BiomeId::Forest { 1.0 } else { 0.0 };
            lerp(core_fw, outer_fw, sample.outer)
        };
        (height, forest)
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
        };
        let triangles = self.paint.build_triangles(&ctx, &|wx, wz| self.biome_sample(wx, wz));
        ChunkGround { coord, heights, triangles }
    }

    /// POIs and props for one chunk, following BuildChunk rules.
    pub fn chunk_contents(&self, coord: IVec2) -> (Vec<PoiInstance>, Vec<PropInstance>) {
        let placed_pois = PoiPlacer::place(self, &self.poi_config, coord);
        let mut pois = SettlementGround::remove_pois(
            &self.settlements,
            placed_pois,
            &self.poi_config,
            self.config.clear_radius,
        );
        let placed_props =
            ScatterPlacer::place_with_registry(self, &self.scatter_config, &self.biome_registry, coord);

        if let Some(city) = self.city() {
            pois.retain(|poi| !city.contains(Vec2::new(poi.position.x, poi.position.z)));
            for (idx, lot) in city.lots.iter().enumerate() {
                let cx = (lot.center.x / Self::CHUNK_SIZE).floor() as i32;
                let cy = (lot.center.y / Self::CHUNK_SIZE).floor() as i32;
                if cx == coord.x && cy == coord.y {
                    let y = self.height(lot.center.x, lot.center.y);
                    pois.push(PoiInstance::new(
                        format!("city_{idx}"),
                        &lot.building_id,
                        Vec3::new(lot.center.x, y, lot.center.y),
                        lot.yaw,
                    ));
                }
            }
        }

        if let Some(lake) = self.lake() {
            pois.retain(|poi| {
                let p = Vec2::new(poi.position.x, poi.position.z);
                !lake.contains_region(p) || lake.shore_distance(p) > lake.config.beach_width + 10.0
            });
            for (idx, lot) in lake.village_lots.iter().enumerate() {
                let cx = (lot.center.x / Self::CHUNK_SIZE).floor() as i32;
                let cy = (lot.center.y / Self::CHUNK_SIZE).floor() as i32;
                if cx == coord.x && cy == coord.y {
                    let y = self.height(lot.center.x, lot.center.y);
                    pois.push(PoiInstance::new(
                        format!("lake_village_{idx}"),
                        &lot.building_id,
                        Vec3::new(lot.center.x, y, lot.center.y),
                        lot.yaw,
                    ));
                }
            }
        }

        let filtered_props: Vec<PropInstance> = placed_props
            .into_iter()
            .filter(|prop| {
                let p = Vec2::new(prop.position.x, prop.position.z);
                if let Some(city) = self.city()
                    && city.contains(p)
                {
                    let is_park = city.district_at(p) == Some(crate::city::District::Park);
                    let on_street = city.street_at(p).is_some();
                    return is_park
                        && !on_street
                        && (prop.is_tree() || crate::city::place::is_ground_cover(&prop.model));
                }
                if let Some(lake) = self.lake()
                    && lake.contains_region(p)
                    && lake.shore_distance(p) < 0.0
                {
                    return false;
                }
                true
            })
            .collect();

        let cleared_props = PoiGeometry::remove_cleared(&pois, filtered_props, &self.poi_config);
        let mut props = SettlementGround::remove_props(&self.settlements, cleared_props, self.config.clear_radius);

        if let Some(city) = self.city() {
            let city_props = crate::city::place::place_street_props(self, city, coord);
            for cp in city_props {
                let clear = !props.iter().any(|p| {
                    let dx = p.position.x - cp.position.x;
                    let dz = p.position.z - cp.position.z;
                    dx * dx + dz * dz < 3.0 * 3.0 - 1e-4
                });
                if clear {
                    props.push(cp);
                }
            }
        }

        if let Some(lake) = self.lake() {
            let lake_props = crate::lake::place::place_lake_props(self, lake, coord);
            for lp in lake_props {
                let clear = !props.iter().any(|p| {
                    let dx = p.position.x - lp.position.x;
                    let dz = p.position.z - lp.position.z;
                    dx * dx + dz * dz < 2.0 * 2.0 - 1e-4
                });
                if clear {
                    props.push(lp);
                }
            }
        }

        (pois, props)
    }

    /// Loot crate spots for one chunk, following BuildChunk rules.
    pub fn chunk_crates(&self, coord: IVec2) -> Vec<crate::loot::LootSpot> {
        let (pois, props) = self.chunk_contents(coord);
        self.chunk_crates_from(coord, &pois, &props)
    }

    /// Crate spots for one chunk, reusing already-built POIs and props (R31-03).
    pub fn chunk_crates_from(
        &self,
        coord: IVec2,
        pois: &[PoiInstance],
        props: &[PropInstance],
    ) -> Vec<crate::loot::LootSpot> {
        let placed = crate::loot::LootPlacer::place(
            self,
            &self.scatter_config,
            &self.poi_config,
            coord,
            props,
            pois,
        );
        SettlementGround::remove_crates(&self.settlements, placed, self.config.clear_radius)
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
