use std::collections::HashMap;
use std::path::Path;

use glam::{IVec2, Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::biome::BiomeId;
use crate::generator::WorldGenerator;
use crate::random::DotnetRandom;
use crate::scatter::{PropInstance, ScatterPlacer};

/// One building placed in a chunk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoiInstance {
    pub id: String,
    pub building_id: String,
    pub position: Vec3,
    pub yaw: f32,
}

impl PoiInstance {
    pub fn new(id: impl Into<String>, building_id: impl Into<String>, position: Vec3, yaw: f32) -> Self {
        Self {
            id: id.into(),
            building_id: building_id.into(),
            position,
            yaw,
        }
    }
}

/// One building type's rules, from data/poi.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoiBuildingDef {
    pub id: String,
    pub lot: [f32; 2],
    #[serde(default)]
    pub biome: HashMap<String, f32>,
    pub chance_per_chunk: f32,
    pub min_flatness: f32,
    pub min_distance_from_spawn: f32,
    #[serde(default)]
    pub loot_table: String,
    #[serde(default)]
    pub zombies: [i32; 2],
}

impl PoiBuildingDef {
    pub fn half_width(&self) -> f32 {
        self.lot[0] / 2.0
    }

    pub fn half_depth(&self) -> f32 {
        self.lot[1] / 2.0
    }

    pub fn reach(&self) -> f32 {
        (self.half_width() * self.half_width() + self.half_depth() * self.half_depth()).sqrt()
    }
}

/// Shape of data/poi.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoiConfigData {
    #[serde(default = "default_lot_margin")]
    pub lot_margin: f32,
    #[serde(default = "default_max_sink")]
    pub max_sink: f32,
    #[serde(default = "default_approach_distance")]
    pub approach_distance: f32,
    #[serde(default = "default_max_active_zombies")]
    pub max_active_zombies: i32,
    #[serde(default)]
    pub buildings: Vec<PoiBuildingDef>,
}

fn default_lot_margin() -> f32 {
    1.5 // data: poi.json lotMargin
}
fn default_max_sink() -> f32 {
    1.5 // data: poi.json maxSink
}
fn default_approach_distance() -> f32 {
    35.0 // data: poi.json approachDistance
}
fn default_max_active_zombies() -> i32 {
    16 // data: poi.json maxActiveZombies
}

impl Default for PoiConfigData {
    fn default() -> Self {
        Self {
            lot_margin: default_lot_margin(),
            max_sink: default_max_sink(),
            approach_distance: default_approach_distance(),
            max_active_zombies: default_max_active_zombies(),
            buildings: Vec::new(),
        }
    }
}

impl PoiConfigData {
    pub fn get(&self, building_id: &str) -> Option<&PoiBuildingDef> {
        self.buildings.iter().find(|b| b.id == building_id)
    }
}

pub struct PoiConfig;

impl PoiConfig {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<PoiConfigData, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let data: PoiConfigData = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(data)
    }

    pub fn from_json(json: &str) -> Result<PoiConfigData, serde_json::Error> {
        serde_json::from_str(json)
    }
}

pub struct PoiGeometry;

impl PoiGeometry {
    pub const CENTRE_INDEX: usize = 4;

    pub fn footprint(cx: f32, cz: f32, yaw: f32, hw: f32, hd: f32) -> [Vec2; 5] {
        let c = yaw.cos();
        let s = yaw.sin();
        let xs = [-hw, hw, hw, -hw];
        let zs = [-hd, -hd, hd, hd];
        let mut points = [Vec2::ZERO; 5];
        for k in 0..4 {
            points[k] = Vec2::new(cx + c * xs[k] + s * zs[k], cz - s * xs[k] + c * zs[k]);
        }
        points[Self::CENTRE_INDEX] = Vec2::new(cx, cz);
        points
    }

    pub fn reach(def: &PoiBuildingDef) -> f32 {
        def.reach()
    }

    pub fn contains(poi: &PoiInstance, def: &PoiBuildingDef, x: f32, z: f32, margin: f32) -> bool {
        let (lx, lz) = Self::to_local(poi, x, z);
        lx.abs() <= def.half_width() + margin && lz.abs() <= def.half_depth() + margin
    }

    pub fn is_in_any_lot(pois: &[PoiInstance], config: &PoiConfigData, x: f32, z: f32, margin: f32) -> bool {
        for poi in pois {
            if let Some(def) = config.get(&poi.building_id)
                && Self::contains(poi, def, x, z, margin)
            {
                return true;
            }
        }
        false
    }

    pub fn to_local(poi: &PoiInstance, x: f32, z: f32) -> (f32, f32) {
        let dx = x - poi.position.x;
        let dz = z - poi.position.z;
        let c = poi.yaw.cos();
        let s = poi.yaw.sin();
        (c * dx - s * dz, s * dx + c * dz)
    }

    pub fn remove_cleared(
        pois: &[PoiInstance],
        props: Vec<PropInstance>,
        config: &PoiConfigData,
    ) -> Vec<PropInstance> {
        if pois.is_empty() {
            return props;
        }
        let margin = config.lot_margin;
        props
            .into_iter()
            .filter(|prop| !Self::is_in_any_lot(pois, config, prop.position.x, prop.position.z, margin))
            .collect()
    }
}

pub struct PoiPlacer;

impl PoiPlacer {
    const PLACEMENT_SALT: i32 = 0x51A7E3B1; // godot: PoiPlacer.cs
    const TURNS: i32 = 4; // godot: PoiPlacer.cs

    pub fn place(
        generator: &WorldGenerator,
        config: &PoiConfigData,
        coord: IVec2,
    ) -> Vec<PoiInstance> {
        let seed = ScatterPlacer::stable_seed(generator.seed(), coord) ^ Self::PLACEMENT_SALT;
        let mut rng = DotnetRandom::new(seed);
        let mut found = Vec::new();

        for (t, def) in config.buildings.iter().enumerate() {
            let roll = rng.next_double();
            let fx = rng.next_double();
            let fz = rng.next_double();
            let turn = rng.next(Self::TURNS);

            if !found.is_empty() || !Self::has_slot(coord, t) {
                continue;
            }

            let margin = PoiGeometry::reach(def) + config.lot_margin;
            let x = coord.x as f32 * WorldGenerator::CHUNK_SIZE
                + margin
                + (fx as f32) * (WorldGenerator::CHUNK_SIZE - 2.0 * margin);
            let z = coord.y as f32 * WorldGenerator::CHUNK_SIZE
                + margin
                + (fz as f32) * (WorldGenerator::CHUNK_SIZE - 2.0 * margin);
            let weight = Self::biome_weight(def, generator.biome(x, z));
            if weight <= 0.0 || roll >= (def.chance_per_chunk * weight) as f64 {
                continue;
            }

            let yaw = turn as f32 * (std::f64::consts::PI / 2.0) as f32;
            if let Some(poi) = Self::try_place(generator, config, def, coord, x, z, yaw) {
                found.push(poi);
            }
        }
        found
    }

    pub fn has_slot(coord: IVec2, type_index: usize) -> bool {
        let offset_x = (type_index & 1) as i32;
        let offset_z = ((type_index >> 1) & 1) as i32;
        ((coord.x + offset_x) & 1) == 0 && ((coord.y + offset_z) & 1) == 0
    }

    pub fn biome_weight(def: &PoiBuildingDef, biome: BiomeId) -> f32 {
        def.biome.get(biome.key()).copied().unwrap_or(0.0)
    }

    fn try_place(
        generator: &WorldGenerator,
        config: &PoiConfigData,
        def: &PoiBuildingDef,
        coord: IVec2,
        x: f32,
        z: f32,
        yaw: f32,
    ) -> Option<PoiInstance> {
        if (x * x + z * z).sqrt() < def.min_distance_from_spawn {
            return None;
        }

        let points = PoiGeometry::footprint(x, z, yaw, def.half_width(), def.half_depth());
        let mut lowest = f32::MAX;
        let mut highest = f32::MIN;

        for p in points {
            if ScatterPlacer::ground_normal(generator, p.x, p.y).y < def.min_flatness {
                return None;
            }
            let h = generator.height(p.x, p.y);
            lowest = lowest.min(h);
            highest = highest.max(h);
        }

        if highest - lowest > config.max_sink {
            return None;
        }

        Some(PoiInstance::new(
            format!("poi_{}_{}", coord.x, coord.y),
            &def.id,
            Vec3::new(x, lowest, z),
            yaw,
        ))
    }
}
