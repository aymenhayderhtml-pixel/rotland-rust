//! Definitions and configuration data for base building parts.

use std::collections::HashMap;
use std::path::Path;

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Snap target of a build part: either placed in a cell or along a cell edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapTarget {
    Cell,
    Edge,
}

/// Definition of one placeable building part from data/build_parts.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPartDef {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub snap: SnapTarget,
    #[serde(default = "default_footprint")]
    pub footprint: Vec<[i32; 2]>,
    pub height: f32,
    pub hp: f32,
    #[serde(default)]
    pub cost: HashMap<String, u32>,
    #[serde(default)]
    pub blocks_zombies: bool,
    #[serde(default)]
    pub is_door: bool,
    #[serde(default)]
    pub is_floor: bool,
    #[serde(default)]
    pub is_roof: bool,
}

fn default_footprint() -> Vec<[i32; 2]> {
    vec![[0, 0]]
}

// data: build_parts.json gridSize
fn default_grid_size() -> f32 {
    2.0
}

// data: build_parts.json floorHeight
fn default_floor_height() -> f32 {
    3.0
}

// data: build_parts.json maxFloors
fn default_max_floors() -> u32 {
    2
}

fn default_max_slope() -> f32 {
    // data: build_parts.json maxSlope
    0.8
}

fn default_camp_radius() -> f32 {
    // data: build_parts.json campRadius
    60.0
}

// data: build_parts.json refundShare
fn default_refund_share() -> f32 {
    0.5
}

/// Overall building configuration from data/build_parts.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildConfig {
    #[serde(default = "default_grid_size")]
    pub grid_size: f32,
    #[serde(default = "default_floor_height")]
    pub floor_height: f32,
    #[serde(default = "default_max_floors")]
    pub max_floors: u32,
    #[serde(default = "default_max_slope")]
    pub max_slope: f32,
    #[serde(default)]
    pub only_near_camp: bool,
    #[serde(default = "default_camp_radius")]
    pub camp_radius: f32,
    #[serde(default)]
    pub camp_pos: [f32; 2],
    #[serde(default = "default_refund_share")]
    pub refund_share: f32,
    #[serde(default)]
    pub parts: HashMap<String, BuildPartDef>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            grid_size: default_grid_size(),
            floor_height: default_floor_height(),
            max_floors: default_max_floors(),
            max_slope: default_max_slope(),
            only_near_camp: false,
            camp_radius: default_camp_radius(),
            camp_pos: [0.0, 0.0],
            refund_share: default_refund_share(),
            parts: HashMap::new(),
        }
    }
}

impl BuildConfig {
    /// Gets a building part definition by its ID.
    pub fn get(&self, part_id: &str) -> Option<&BuildPartDef> {
        self.parts.get(part_id)
    }

    /// Camp position as a 2D vector (X, Z).
    pub fn camp_pos_vec2(&self) -> Vec2 {
        Vec2::new(self.camp_pos[0], self.camp_pos[1])
    }

    /// Loads configuration from a JSON file.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let mut cfg: Self = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        for (key, part) in &mut cfg.parts {
            if part.id.is_empty() {
                part.id = key.clone();
            }
        }
        Ok(cfg)
    }

    /// Parses configuration from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let mut cfg: Self = serde_json::from_str(json)?;
        for (key, part) in &mut cfg.parts {
            if part.id.is_empty() {
                part.id = key.clone();
            }
        }
        Ok(cfg)
    }

    /// Loads the default build_parts.json using paths::repo_path.
    pub fn load_default() -> Result<Self, std::io::Error> {
        let path = paths::repo_path("data/build_parts.json");
        Self::load_from_file(path)
    }
}
