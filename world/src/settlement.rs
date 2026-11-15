use std::collections::HashMap;
use std::path::Path;
use glam::{IVec2, Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::generator::WorldGenerator;
use crate::math::{lerp, smooth_step};

/// Shape of data/settlements.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementConfigData {
    #[serde(alias = "SearchRadiusChunks")]
    pub search_radius_chunks: i32,
    #[serde(alias = "LotSize")]
    pub lot_size: f32,
    #[serde(alias = "FlatRadius")]
    pub flat_radius: f32,
    #[serde(alias = "BlendRadius")]
    pub blend_radius: f32,
    #[serde(alias = "ReliefLimit")]
    pub relief_limit: f32,
    #[serde(alias = "CliffCheckRadius")]
    pub cliff_check_radius: f32,
    #[serde(alias = "MaxSlopeDeg")]
    pub max_slope_deg: f32,
    #[serde(alias = "ClearRadius")]
    pub clear_radius: f32,
    #[serde(alias = "SafeZoneHalf")]
    pub safe_zone_half: f32,
    #[serde(alias = "SafeZonePadding")]
    pub safe_zone_padding: f32,
    #[serde(alias = "RespawnOffset")]
    pub respawn_offset: [f32; 2],
    #[serde(default, alias = "GuardRange")]
    pub guard_range: f32,
    #[serde(default, alias = "GuardSightInterval")]
    pub guard_sight_interval: f32,
    #[serde(default, alias = "GuardFireInterval")]
    pub guard_fire_interval: f32,
    #[serde(default, alias = "GuardDamage")]
    pub guard_damage: f32,
    #[serde(default, alias = "GuardMissChance")]
    pub guard_miss_chance: f32,
    #[serde(default, alias = "GuardEyeHeight")]
    pub guard_eye_height: f32,
    #[serde(default, alias = "NpcModels")]
    pub npc_models: HashMap<String, String>,
}

impl SettlementConfigData {
    pub fn validate(&self) -> Result<(), String> {
        let ok = self.lot_size > 0.0
            && self.flat_radius > 0.0
            && self.blend_radius >= self.flat_radius
            && self.clear_radius > 0.0
            && self.safe_zone_half > 0.0
            && self.safe_zone_padding >= 0.0
            && self.respawn_offset.len() == 2;
        if !ok {
            return Err("SettlementConfigData has values out of range".to_string());
        }
        Ok(())
    }
}

pub struct SettlementConfig;

impl SettlementConfig {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<SettlementConfigData, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let data: SettlementConfigData = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        data.validate()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(data)
    }

    pub fn from_json(json: &str) -> Result<SettlementConfigData, serde_json::Error> {
        let data: SettlementConfigData = serde_json::from_str(json)?;
        Ok(data)
    }
}

/// One walled camp placed from the seed. Center is the ground point (Y = flattened ground height).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettlementSite {
    pub id: String,
    pub center: Vec3,
    pub yaw: f32,
}

impl SettlementSite {
    pub fn new(id: impl Into<String>, center: Vec3, yaw: f32) -> Self {
        Self {
            id: id.into(),
            center,
            yaw,
        }
    }

    /// The chunk that holds the centre.
    pub fn chunk(&self) -> IVec2 {
        IVec2::new(
            (self.center.x / WorldGenerator::CHUNK_SIZE).floor() as i32,
            (self.center.z / WorldGenerator::CHUNK_SIZE).floor() as i32,
        )
    }

    /// Where the player stands after a new game or respawn.
    pub fn respawn_point(&self, cfg: &SettlementConfigData) -> Vec3 {
        self.center + self.local(cfg.respawn_offset[0], 0.0, cfg.respawn_offset[1])
    }

    /// Converts a point in the settlement's own axes to the world (yaw only, no tilt).
    pub fn local(&self, x: f32, y: f32, z: f32) -> Vec3 {
        let c = self.yaw.cos();
        let s = self.yaw.sin();
        Vec3::new(c * x + s * z, y, -s * x + c * z)
    }

    /// True when a world point is inside the safe zone box (plus the padding when given).
    pub fn contains(&self, p: Vec3, padding: f32, cfg: &SettlementConfigData) -> bool {
        let half = cfg.safe_zone_half + padding;
        let d = p - self.center;
        d.x.abs() <= half && d.z.abs() <= half
    }
}

/// Pure ground rules for settlements: the flattened pad under the lot.
pub struct SettlementGround;

impl SettlementGround {
    /// Blends the raw height toward the site's pad height. Flat inside FlatRadius, normal past BlendRadius.
    pub fn flatten(
        raw: f32,
        x: f32,
        z: f32,
        sites: &[SettlementSite],
        cfg: &SettlementConfigData,
    ) -> f32 {
        if sites.is_empty() {
            return raw;
        }
        let mut height = raw;
        for site in sites {
            let d = Self::box_distance(site, x, z);
            if d >= cfg.blend_radius {
                continue;
            }
            let t = if d <= cfg.flat_radius {
                0.0
            } else {
                smooth_step(cfg.flat_radius, cfg.blend_radius, d)
            };
            height = lerp(site.center.y, height, t);
        }
        height
    }

    /// Box distance (metres) from a point to the nearest camp centre, or f32::MAX with no camps.
    pub fn nearest_box_distance(sites: &[SettlementSite], x: f32, z: f32) -> f32 {
        let mut best = f32::MAX;
        for site in sites {
            best = best.min(Self::box_distance(site, x, z));
        }
        best
    }

    /// Distance from the centre in the box sense: the larger of the two axis offsets.
    pub fn box_distance(site: &SettlementSite, x: f32, z: f32) -> f32 {
        (x - site.center.x).abs().max((z - site.center.z).abs())
    }
}

/// Finds the settlement site from the seed.
pub struct SettlementPlacer;

impl SettlementPlacer {
    const ID: &'static str = "camp_1";
    const GRID_STEP: f32 = 4.0;
    const SLOPE_PROBE: f32 = 1.0;
    const FOREST_THRESHOLD: f32 = 0.5;

    pub fn find(generator: &WorldGenerator, cfg: &SettlementConfigData) -> SettlementSite {
        let chunk = WorldGenerator::CHUNK_SIZE;
        let mut fallback_centre = Vec2::ZERO;
        let mut fallback_relief = f32::MAX;

        for ring in 0..=cfg.search_radius_chunks {
            for dz in -ring..=ring {
                for dx in -ring..=ring {
                    if dx.abs().max(dz.abs()) != ring {
                        continue;
                    }
                    let centre = Vec2::new((dx as f32 + 0.5) * chunk, (dz as f32 + 0.5) * chunk);
                    let (relief, slope) = Self::measure(generator, centre, cfg);
                    let plains = generator.forest_weight(centre.x, centre.y) < Self::FOREST_THRESHOLD;
                    if plains && relief <= cfg.relief_limit && slope <= cfg.max_slope_deg {
                        return Self::make_site(generator, centre);
                    }
                    if relief < fallback_relief {
                        fallback_relief = relief;
                        fallback_centre = centre;
                    }
                }
            }
        }
        Self::make_site(generator, fallback_centre)
    }

    fn make_site(generator: &WorldGenerator, centre: Vec2) -> SettlementSite {
        let ground_y = generator.raw_height(centre.x, centre.y);
        SettlementSite::new(Self::ID, Vec3::new(centre.x, ground_y, centre.y), 0.0)
    }

    fn measure(generator: &WorldGenerator, centre: Vec2, cfg: &SettlementConfigData) -> (f32, f32) {
        let mut low = f32::MAX;
        let mut high = f32::MIN;
        let mut steepest = 0.0f32;
        let reach = cfg.cliff_check_radius;

        let mut z = -reach;
        while z <= reach + 0.01 {
            let mut x = -reach;
            while x <= reach + 0.01 {
                let px = centre.x + x;
                let pz = centre.y + z;
                let h = generator.raw_height(px, pz);
                low = low.min(h);
                high = high.max(h);
                let dx = (generator.raw_height(px + Self::SLOPE_PROBE, pz)
                    - generator.raw_height(px - Self::SLOPE_PROBE, pz))
                    / (2.0 * Self::SLOPE_PROBE);
                let dz = (generator.raw_height(px, pz + Self::SLOPE_PROBE)
                    - generator.raw_height(px, pz - Self::SLOPE_PROBE))
                    / (2.0 * Self::SLOPE_PROBE);
                steepest = steepest.max((dx * dx + dz * dz).sqrt().atan());
                x += Self::GRID_STEP;
            }
            z += Self::GRID_STEP;
        }
        let relief = high - low;
        let max_slope_deg = steepest.to_degrees();
        (relief, max_slope_deg)
    }
}
