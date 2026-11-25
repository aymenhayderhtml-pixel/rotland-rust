mod build;
mod index;

use std::path::Path;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::biome::layout::WorldLayout;
use crate::roads::build::build_network;
use crate::roads::index::RoadSpatialIndex;
use crate::towns::TownSite;

/// Surface and width classification of a road polyline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoadKind {
    /// 4 m dirt trail.
    Trail,
    /// 8 m asphalt highway.
    Highway,
}

impl RoadKind {
    /// Returns the nominal road width in meters from `RoadsData`.
    pub fn width(self, data: &RoadsData) -> f32 {
        match self {
            Self::Trail => data.trail_width,
            Self::Highway => data.highway_width,
        }
    }
}

/// A named road polyline in world XZ coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Road {
    pub kind: RoadKind,
    /// Polyline vertices in world XZ coordinates, sampled ~20 m apart.
    pub points: Vec<Vec2>,
    pub name: String,
}

/// Road generation parameters loaded from `data/roads.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoadsData {
    pub trailhead_margin: f32,
    pub trail_width: f32,
    pub highway_width: f32,
    pub point_spacing: f32,
    pub wiggle_m: f32,
    pub wiggle_frequency: f32,
    pub ring_radius_from_towns: f32,
}

impl RoadsData {
    /// Loads `data/roads.json` via `paths::try_repo_path`.
    pub fn load() -> Result<Self, String> {
        let path = paths::try_repo_path("data/roads.json")
            .map_err(|e| format!("data/roads.json: {e}"))?;
        Self::load_from_file(path)
    }

    /// Loads road parameters from an explicit file path.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&json)
    }

    /// Parses road parameters from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let data: Self =
            serde_json::from_str(json).map_err(|e| format!("failed to parse roads JSON: {e}"))?;
        if data.point_spacing <= 0.0 {
            return Err("roads.json point_spacing must be positive".to_string());
        }
        Ok(data)
    }
}

/// Procedural road network linking core trailheads and outer biome towns.
#[derive(Debug, Clone)]
pub struct RoadNetwork {
    trailheads: Vec<Vec2>,
    roads: Vec<Road>,
    index: RoadSpatialIndex,
}

impl RoadNetwork {
    /// Builds all trailheads, highways, ring trails, and the 64 m spatial grid index.
    pub fn build(
        data: &RoadsData,
        towns: &[TownSite],
        layout: &WorldLayout,
        seed: i32,
        origin: Vec2,
    ) -> Self {
        let (trailheads, roads) = build_network(data, towns, layout, seed, origin);
        let index = RoadSpatialIndex::build(&roads);
        Self {
            trailheads,
            roads,
            index,
        }
    }

    /// Returns all generated highways and ring trails.
    pub fn roads(&self) -> &[Road] {
        &self.roads
    }

    /// Returns the horizontal distance (m) and kind of the nearest road within 30 m of `p`.
    pub fn distance_to_road(&self, p: Vec2) -> Option<(f32, RoadKind)> {
        self.index.distance_to_road(p)
    }

    /// Returns the world XZ coordinates of the sector trailheads outside the blend band.
    pub fn trailheads(&self) -> &[Vec2] {
        &self.trailheads
    }
}
