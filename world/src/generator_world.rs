use std::collections::HashMap;
use std::f32::consts::TAU;

use glam::Vec2;

use crate::biome::layout::{WorldLayout, WorldLayoutData};
use crate::biome::registry::BiomeRegistry;
use crate::city::{CityConfigData, CityGround, CityGroundData, CityLayout};
use crate::generator::WorldGenerator;
use crate::lake::{LakeConfigData, LakeLayout};
use crate::math::{lerp, smooth_step};
use crate::noise::GodotNoise;
use crate::poi::PoiConfigData;
use crate::rivers::{RiverNetwork, RiversData};
use crate::roads::{RoadKind, RoadNetwork, RoadsData};
use crate::scatter::ScatterConfigData;
use crate::settlement::{SettlementConfigData, SettlementPlacer};
use crate::terrain_paint::TerrainPaint;
use crate::terrain_shape::TerrainShape;
use crate::towns::{TownsData, place_towns};

// new: DECISIONS 2026-11-25
const TOWN_MAX_RELIEF_M: f32 = 6.0;
// new: DECISIONS 2026-11-25
const TOWN_SAMPLE_RINGS: usize = 8;
// new: DECISIONS 2026-11-25
const ROAD_SMOOTH_WINDOW_M: f32 = 40.0;
// new: DECISIONS 2026-11-25
const ROAD_FLATTEN_EDGE_M: f32 = 8.0;
// data: roads.json highway_width
const HIGHWAY_HALF_WIDTH_M: f32 = 4.0;
// data: roads.json trail_width
const TRAIL_HALF_WIDTH_M: f32 = 2.0;
// new: DECISIONS 2026-11-25
const ROAD_GRID_CELL_M: f32 = 64.0;

#[derive(Debug, Clone)]
pub(crate) struct RoadHeightField {
    roads: Vec<(RoadKind, Vec<Vec2>, Vec<f32>)>,
    cells: HashMap<(i32, i32), Vec<(usize, usize)>>,
}

impl RoadHeightField {
    pub(crate) fn empty() -> Self {
        Self {
            roads: Vec::new(),
            cells: HashMap::new(),
        }
    }

    pub(crate) fn build(network: &RoadNetwork, base_height: &dyn Fn(f32, f32) -> f32) -> Self {
        let mut roads = Vec::with_capacity(network.roads().len());
        let mut cells: HashMap<(i32, i32), Vec<(usize, usize)>> = HashMap::new();
        let pad = HIGHWAY_HALF_WIDTH_M + ROAD_FLATTEN_EDGE_M;

        for (road_idx, road) in network.roads().iter().enumerate() {
            let pts = &road.points;
            let mut arc = vec![0.0_f32; pts.len()];
            for i in 1..pts.len() {
                arc[i] = arc[i - 1] + pts[i - 1].distance(pts[i]);
            }
            let raw_h: Vec<f32> = pts.iter().map(|p| base_height(p.x, p.y)).collect();
            let mut smoothed = Vec::with_capacity(pts.len());
            for i in 0..pts.len() {
                let (mut sum, mut count) = (0.0_f32, 0usize);
                for j in 0..pts.len() {
                    if (arc[j] - arc[i]).abs() <= ROAD_SMOOTH_WINDOW_M {
                        sum += raw_h[j];
                        count += 1;
                    }
                }
                smoothed.push(sum / (count.max(1) as f32));
            }

            for seg_idx in 0..pts.len().saturating_sub(1) {
                let (a, b) = (pts[seg_idx], pts[seg_idx + 1]);
                let cx0 = ((a.x.min(b.x) - pad) / ROAD_GRID_CELL_M).floor() as i32;
                let cx1 = ((a.x.max(b.x) + pad) / ROAD_GRID_CELL_M).floor() as i32;
                let cz0 = ((a.y.min(b.y) - pad) / ROAD_GRID_CELL_M).floor() as i32;
                let cz1 = ((a.y.max(b.y) + pad) / ROAD_GRID_CELL_M).floor() as i32;
                for cz in cz0..=cz1 {
                    for cx in cx0..=cx1 {
                        cells.entry((cx, cz)).or_default().push((road_idx, seg_idx));
                    }
                }
            }
            roads.push((road.kind, pts.clone(), smoothed));
        }

        Self { roads, cells }
    }

    pub(crate) fn sample(&self, x: f32, z: f32) -> Option<(f32, f32)> {
        let key = (
            (x / ROAD_GRID_CELL_M).floor() as i32,
            (z / ROAD_GRID_CELL_M).floor() as i32,
        );
        let candidates = self.cells.get(&key)?;
        let p = Vec2::new(x, z);
        let mut best: Option<(f32, f32, f32)> = None;

        for &(road_idx, seg_idx) in candidates {
            let (kind, ref pts, ref heights) = self.roads[road_idx];
            let (a, b) = (pts[seg_idx], pts[seg_idx + 1]);
            let ab = b - a;
            let len_sq = ab.length_squared();
            let t = if len_sq > 1e-6 {
                ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let d = p.distance(a + ab * t);
            let half_w = match kind {
                RoadKind::Highway => HIGHWAY_HALF_WIDTH_M,
                RoadKind::Trail => TRAIL_HALF_WIDTH_M,
            };
            let max_d = half_w + ROAD_FLATTEN_EDGE_M;
            if d < max_d && best.is_none_or(|(_, _, bd)| d < bd) {
                let road_h = lerp(heights[seg_idx], heights[seg_idx + 1], t);
                let weight = if d <= half_w {
                    1.0
                } else {
                    1.0 - smooth_step(half_w, max_d, d)
                };
                best = Some((road_h, weight, d));
            }
        }
        best.map(|(h, w, _)| (h, w))
    }

    pub(crate) fn road_height_at(&self, x: f32, z: f32) -> Option<f32> {
        self.sample(x, z).map(|(h, _)| h)
    }
}

impl WorldGenerator {
    /// The normal way to make a world: reads settlements, scatter, poi, biomes, world_layout,
    /// city, lake, and outer-world towns/rivers/roads from `data_dir`.
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
        let towns_path = read("towns.json");
        let rivers_path = read("rivers.json");
        let roads_path = read("roads.json");
        if towns_path.exists() && rivers_path.exists() && roads_path.exists() {
            let towns_text = std::fs::read_to_string(&towns_path)
                .map_err(|e| format!("towns.json: {e}"))?;
            let towns_data = TownsData::from_json(&towns_text)
                .map_err(|e| format!("towns.json: {e}"))?;
            let rivers_data = RiversData::load_from_file(&rivers_path)
                .map_err(|e| format!("rivers.json: {e}"))?;
            let roads_data = RoadsData::load_from_file(&roads_path)
                .map_err(|e| format!("roads.json: {e}"))?;
            generator.attach_outer_world(&towns_data, &rivers_data, &roads_data);
            let metro_path = read("metro.json");
            if metro_path.exists() {
                let metro_cfg = crate::metro::MetroConfig::load_from_file(&metro_path)
                    .map_err(|e| format!("metro.json: {e}"))?;
                let look = crate::metro::MetroLook::load_from_file(&metro_path)
                    .map_err(|e| format!("metro.json: {e}"))?;
                generator.attach_metro(metro_cfg, look, &roads_data);
            }
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

    /// Attaches the lake & coast opposite the city (or camp if no city).
    pub fn attach_lake(&mut self, cfg: &LakeConfigData) {
        let camp = self.settlements.first().map_or(Vec2::ZERO, |s| Vec2::new(s.center.x, s.center.z));
        let city_center = self.city.as_ref().map(|c| c.origin + Vec2::splat(c.size * 0.5));
        let layout = LakeLayout::generate(self.seed, camp, city_center, cfg, |p| self.raw_height(p.x, p.y));
        self.lake = Some(layout);
    }

    /// Builds outer towns, rivers, roads, and bridge crossings after camp/city/lake/layout exist.
    pub fn attach_outer_world(
        &mut self,
        towns_data: &TownsData,
        rivers_data: &RiversData,
        roads_data: &RoadsData,
    ) {
        let towns = place_towns(
            towns_data,
            &self.layout,
            self.seed,
            self.origin,
            &|center, radius| self.town_relief_ok(center, radius),
        );
        let town_avg_heights: Vec<f32> = towns
            .iter()
            .map(|t| self.town_average_height(t.center, t.radius))
            .collect();
        let rivers = RiverNetwork::build(
            rivers_data,
            &self.layout,
            self.seed,
            self.origin,
            &|x, z| self.raw_height(x, z),
        );
        let roads = RoadNetwork::build(roads_data, &towns, &self.layout, self.seed, self.origin);
        let crossings = rivers.crossings(&roads);
        let road_heights = RoadHeightField::build(&roads, &|x, z| {
            let mut h = self.raw_height(x, z);
            let p = Vec2::new(x, z);
            for (town, &avg_h) in towns.iter().zip(&town_avg_heights) {
                if p.distance_squared(town.center) <= town.radius * town.radius {
                    h = avg_h;
                }
            }
            h
        });

        self.towns = towns;
        self.town_avg_heights = town_avg_heights;
        self.rivers = rivers;
        self.roads = roads;
        self.crossings = crossings;
        self.road_heights = road_heights;
        self.towns_data = towns_data.clone();
        self.has_outer = true;
    }

    fn town_relief_ok(&self, center: Vec2, radius: f32) -> bool {
        let (mut min_h, mut max_h) = (f32::MAX, f32::MIN);
        for p in town_sample_points(center, radius) {
            let h = self.raw_height(p.x, p.y);
            min_h = min_h.min(h);
            max_h = max_h.max(h);
        }
        (max_h - min_h) < TOWN_MAX_RELIEF_M
    }

    fn town_average_height(&self, center: Vec2, radius: f32) -> f32 {
        let pts = town_sample_points(center, radius);
        let sum: f32 = pts.iter().map(|p| self.raw_height(p.x, p.y)).sum();
        sum / (pts.len() as f32)
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
            towns: Vec::new(),
            town_avg_heights: Vec::new(),
            roads: RoadNetwork::build(&default_roads_data(), &[], &WorldLayout::new(&WorldLayoutData {
                core_radius: 2200.0, // data: world_layout.json core_radius
                blend_band: 400.0,   // data: world_layout.json blend_band
                sectors: Vec::new(),
                sector_blend_deg: 14.0, // data: world_layout.json sector_blend_deg
                warp_deg: 22.0,         // data: world_layout.json warp_deg
                warp_frequency: 0.0004, // data: world_layout.json warp_frequency
                tier_band: 1500.0,      // data: world_layout.json tier_band
                tiers: Vec::new(),
            }, seed, Vec2::ZERO, Vec2::X), seed, Vec2::ZERO),
            road_heights: RoadHeightField::empty(),
            rivers: RiverNetwork::from_rivers(&default_rivers_data(), Vec::new()),
            crossings: Vec::new(),
            towns_data: TownsData::load_embedded(),
            has_outer: false,
            metro: None,
        };
        let site = SettlementPlacer::find(&world_gen, &config);
        let camp = Vec2::new(site.center.x, site.center.z);
        world_gen.origin = camp;
        world_gen.layout = WorldLayout::new(&world_gen.layout_data, seed, camp, Vec2::X);
        world_gen.settlements = vec![site];
        world_gen
    }
}

fn town_sample_points(center: Vec2, radius: f32) -> Vec<Vec2> {
    let mut pts = Vec::with_capacity(1 + TOWN_SAMPLE_RINGS * 2);
    pts.push(center);
    for ring in [0.5_f32, 1.0_f32] {
        let r = radius * ring;
        for i in 0..TOWN_SAMPLE_RINGS {
            let a = (i as f32) * TAU / (TOWN_SAMPLE_RINGS as f32);
            pts.push(center + Vec2::new(a.cos() * r, a.sin() * r));
        }
    }
    pts
}

fn default_roads_data() -> RoadsData {
    RoadsData {
        trailhead_margin: 30.0,     // data: roads.json trailhead_margin
        trail_width: 4.0,           // data: roads.json trail_width
        highway_width: 8.0,         // data: roads.json highway_width
        point_spacing: 20.0,        // data: roads.json point_spacing
        wiggle_m: 40.0,             // data: roads.json wiggle_m
        wiggle_frequency: 0.0012,   // data: roads.json wiggle_frequency
        ring_radius_from_towns: 1.0, // data: roads.json ring_radius_from_towns
    }
}

fn default_rivers_data() -> RiversData {
    RiversData {
        step_m: 25.0,               // data: rivers.json step_m
        max_length_m: 9000.0,       // data: rivers.json max_length_m
        core_margin_m: 150.0,       // data: rivers.json core_margin_m
        sample_count: 5,            // data: rivers.json sample_count
        sample_spread_deg: 50.0,    // data: rivers.json sample_spread_deg
        wiggle_deg: 6.0,            // data: rivers.json wiggle_deg
        wiggle_frequency: 0.0025,   // data: rivers.json wiggle_frequency
        target_pull: 0.6,           // data: rivers.json target_pull
        max_turn_deg: 12.0,         // data: rivers.json max_turn_deg
        width_start_m: 6.0,         // data: rivers.json width_start_m
        width_end_m: 16.0,          // data: rivers.json width_end_m
        depth_m: 1.6,               // data: rivers.json depth_m
        bed_depth_factor: 0.5,      // data: rivers.json bed_depth_factor
        bank_m: 6.0,                // data: rivers.json bank_m
        mesh_bank_overlap_m: 1.0,   // data: rivers.json mesh_bank_overlap_m
        grid_cell_m: 64.0,          // data: rivers.json grid_cell_m
        bridge_spans_m: vec![12.0, 16.0], // data: rivers.json bridge_spans_m
        crossing_merge_dist_m: 8.0, // data: rivers.json crossing_merge_dist_m
        rivers: Vec::new(),
    }
}
