use std::collections::BTreeMap;

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::biome::BiomeId;

pub mod clutter;
pub mod interior;
pub mod lots;
pub mod place;

pub use place::place_towns;

/// Model name for boardwalk strip lots in swamp towns.
pub const BOARDWALK_MODEL: &str = "boardwalk_section";
/// Model name for the unique gun shop lot in every town.
pub const GUN_SHOP_MODEL: &str = "gun_shop";

/// Building-local counter spot for a town gun shopkeeper (`x` = right, `y` = forward/+Y in Blender).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShopkeeperSpot {
    pub building: String,
    pub x: f32,
    pub y: f32,
    pub floor_y: f32,
    #[serde(default)]
    pub yaw: f32,
}

impl Default for ShopkeeperSpot {
    fn default() -> Self {
        Self {
            building: GUN_SHOP_MODEL.to_string(),
            x: -0.9,      // data: towns.json shopkeeper
            y: -2.45,     // data: towns.json shopkeeper
            floor_y: 0.1, // data: towns.json shopkeeper
            yaw: 0.0,     // data: towns.json shopkeeper
        }
    }
}

/// World-space spawn info for a town gun shopkeeper (`shopkeeper_<town_key>`).
#[derive(Debug, Clone, PartialEq)]
pub struct TownShopkeeper {
    pub id: String,
    pub town_key: String,
    pub town_name: String,
    pub position: Vec3,
    pub yaw: f32,
}

/// Fallback half-extents (m) when a model is absent from `data/towns.json` footprints.
// new: DECISIONS 2026-11-25
const DEFAULT_FOOTPRINT_HALF_M: f32 = 4.0;

/// Root schema for `data/towns.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TownsData {
    pub placement: PlacementConfig,
    pub footprints: BTreeMap<String, [f32; 2]>,
    pub towns: Vec<TownDef>,
    #[serde(default)]
    pub shopkeeper: ShopkeeperSpot,
    #[serde(default)]
    pub interiors: interior::InteriorsConfig,
}

/// Shared placement and street-layout parameters from `data/towns.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacementConfig {
    pub spiral_step_m: f32,
    pub spiral_max_m: f32,
    pub street_setback_m: f32,
    pub boardwalk_offset_m: f32,
    pub boardwalk_step_m: f32,
}

/// One town entry in `data/towns.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TownDef {
    pub key: String,
    pub name: String,
    pub biome: String,
    pub distance_m: f32,
    pub radius: f32,
    pub street_length: f32,
    pub lot_spacing: f32,
    #[serde(default)]
    pub boardwalk: bool,
    pub buildings: Vec<BuildingEntry>,
    pub loot_tables: BTreeMap<String, String>,
}

/// Weighted building rule for a town.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildingEntry {
    pub model: String,
    pub weight: f32,
    #[serde(default)]
    pub unique: bool,
    #[serde(default)]
    pub max_count: Option<usize>,
}

/// A placed outer-biome town with its main street direction and building lots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TownSite {
    pub key: String,
    pub name: String,
    pub biome: BiomeId,
    pub center: Vec2,
    pub radius: f32,
    pub street_dir: Vec2,
    pub lots: Vec<TownLot>,
}

/// One building or boardwalk lot inside a `TownSite`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TownLot {
    pub building: String,
    pub pos: Vec2,
    pub yaw: f32,
    pub footprint: Vec2,
    pub loot_table: String,
}

impl TownsData {
    /// Parses `TownsData` from JSON text.
    pub fn from_json(raw: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(raw)
    }

    /// Loads the compiled-in `data/towns.json`.
    pub fn load_embedded() -> Self {
        let raw = include_str!("../../../data/towns.json");
        Self::from_json(raw).expect("data/towns.json must be valid TownsData JSON")
    }

    /// Returns the half-extent footprint (local X along street, local Z perpendicular) for `model`.
    pub fn footprint(&self, model: &str) -> Vec2 {
        match self.footprints.get(model) {
            Some(&[hx, hz]) => Vec2::new(hx, hz),
            None => Vec2::splat(DEFAULT_FOOTPRINT_HALF_M),
        }
    }
}

impl TownDef {
    /// Resolves the town's biome key to a `BiomeId`.
    pub fn biome_id(&self) -> Option<BiomeId> {
        BiomeId::from_key(&self.biome)
    }
}

impl TownSite {
    /// Returns true if XZ point `p` lies inside the town's circular radius.
    pub fn contains_point(&self, p: Vec2) -> bool {
        self.center.distance_squared(p) <= self.radius * self.radius
    }

    /// Returns true if XZ point `p` lies inside any enclosed building lot (excluding boardwalks).
    pub fn indoors(&self, p: Vec2) -> bool {
        self.lots
            .iter()
            .any(|lot| lot.building != BOARDWALK_MODEL && lot.contains_point(p))
    }

    /// Returns the `gun_shop` lot in this town, if present.
    pub fn gun_shop_lot(&self) -> Option<&TownLot> {
        self.lots.iter().find(|lot| lot.building == GUN_SHOP_MODEL)
    }

    /// Computes the world-space position and yaw for `shopkeeper_<town_key>` inside `gun_shop`.
    pub fn shopkeeper_spawn(&self, data: &TownsData, ground_y: f32) -> Option<TownShopkeeper> {
        let lot = self.gun_shop_lot()?;
        let spot = &data.shopkeeper;
        let forward = lot.forward();
        let right = Vec2::new(-forward.y, forward.x);
        let world_xz = lot.pos + right * spot.x + forward * spot.y;
        Some(TownShopkeeper {
            id: format!("shopkeeper_{}", self.key),
            town_key: self.key.clone(),
            town_name: self.name.clone(),
            position: Vec3::new(world_xz.x, ground_y + spot.floor_y, world_xz.y),
            yaw: lot.yaw + spot.yaw,
        })
    }
}

impl TownLot {
    /// Unit forward direction on the XZ plane using Godot yaw convention `(-sin yaw, -cos yaw)`.
    pub fn forward(&self) -> Vec2 {
        Vec2::new(-self.yaw.sin(), -self.yaw.cos())
    }

    /// Four world-space XZ corners of the oriented lot rectangle.
    pub fn corners(&self) -> [Vec2; 4] {
        let forward = self.forward();
        let right = Vec2::new(-forward.y, forward.x);
        let hx = right * self.footprint.x;
        let hy = forward * self.footprint.y;
        [
            self.pos - hx - hy,
            self.pos + hx - hy,
            self.pos + hx + hy,
            self.pos - hx + hy,
        ]
    }

    /// World-space axis-aligned bounding box `(min, max)` on XZ.
    pub fn aabb(&self) -> (Vec2, Vec2) {
        let corners = self.corners();
        let mut min = corners[0];
        let mut max = corners[0];
        for &c in &corners[1..] {
            min = min.min(c);
            max = max.max(c);
        }
        (min, max)
    }

    /// Returns true if XZ point `p` is inside this oriented lot rectangle.
    pub fn contains_point(&self, p: Vec2) -> bool {
        let forward = self.forward();
        let right = Vec2::new(-forward.y, forward.x);
        let rel = p - self.pos;
        rel.dot(right).abs() <= self.footprint.x && rel.dot(forward).abs() <= self.footprint.y
    }

    /// Separating-axis overlap check between two oriented town lots (matches `VillageLot::overlaps`).
    pub fn overlaps(&self, other: &TownLot) -> bool {
        let corners_a = self.corners();
        let corners_b = other.corners();
        let axes = [
            (corners_a[1] - corners_a[0]).normalize_or_zero(),
            (corners_a[3] - corners_a[0]).normalize_or_zero(),
            (corners_b[1] - corners_b[0]).normalize_or_zero(),
            (corners_b[3] - corners_b[0]).normalize_or_zero(),
        ];
        // new: DECISIONS 2026-11-25
        const AXIS_EPS_SQ: f32 = 1e-12;
        for axis in axes {
            if axis.length_squared() <= AXIS_EPS_SQ {
                continue;
            }
            let (min_a, max_a) = project_onto_axis(&corners_a, axis);
            let (min_b, max_b) = project_onto_axis(&corners_b, axis);
            if max_a < min_b || max_b < min_a {
                return false;
            }
        }
        true
    }
}

fn project_onto_axis(corners: &[Vec2; 4], axis: Vec2) -> (f32, f32) {
    let mut min = corners[0].dot(axis);
    let mut max = min;
    for &c in &corners[1..] {
        let d = c.dot(axis);
        if d < min {
            min = d;
        }
        if d > max {
            max = d;
        }
    }
    (min, max)
}
