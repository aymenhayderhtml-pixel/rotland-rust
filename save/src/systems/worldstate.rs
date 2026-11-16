use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Item dropped on the ground in the open world.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DroppedItemData {
    pub id: String,
    pub count: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl DroppedItemData {
    pub fn new(id: impl Into<String>, count: u32, x: f32, y: f32, z: f32) -> Self {
        Self {
            id: id.into(),
            count,
            x,
            y,
            z,
        }
    }
}

fn default_campfire_hours() -> f32 {
    5.0
}

/// Campfire placed in the open world (burns for up to 5 hours).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CampfireData {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    #[serde(default = "default_campfire_hours")]
    pub hours: f32,
}

impl CampfireData {
    pub fn new(x: f32, y: f32, z: f32, hours: f32) -> Self {
        Self { x, y, z, hours }
    }
}

/// Changes to the open world: looted crates, dropped items, fallen trees, campfires, visited POIs.
/// Port of `scripts/World/WorldState.cs` (SaveId: "worldstate").
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WorldStateSaveData {
    #[serde(default)]
    pub looted: Vec<String>,
    #[serde(default)]
    pub dropped: HashMap<String, Vec<DroppedItemData>>,
    #[serde(default, rename = "fallen_trees")]
    pub fallen_trees: Vec<String>,
    #[serde(default)]
    pub campfires: HashMap<String, Vec<CampfireData>>,
    #[serde(default, rename = "visited_pois")]
    pub visited_pois: Vec<String>,
}
