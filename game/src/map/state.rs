//! Map state: explored cells, found buildings, last death spot, and player waypoint.
//!
//! Port of Godot C# scripts/World/MapBuilding.cs and scripts/World/MapState.cs.
//! Godot save system id: "map".

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::explore::MapExplore;

// Godot: MapBuilding.cs
/// A building the player has seen (within 60 m), kept on the map after its chunk unloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapBuilding {
    pub id: String,
    pub building_type: String,
    pub position: Vec2,
    pub looted: bool,
}

impl MapBuilding {
    pub fn new(id: impl Into<String>, building_type: impl Into<String>, position: Vec2, looted: bool) -> Self {
        Self {
            id: id.into(),
            building_type: building_type.into(),
            position,
            looted,
        }
    }
}

// Godot: MapState.cs
/// Everything the map remembers: seen cells, buildings seen, the last death spot and the waypoint.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MapState {
    pub explore: MapExplore,
    pub found: Vec<MapBuilding>,
    pub death_spot: Option<Vec2>,
    pub waypoint: Option<Vec2>,
}

impl MapState {
    // Godot: MapState.cs RevealRadius
    /// Cells whose centre lies this close to the player are revealed.
    pub const REVEAL_RADIUS: f32 = 40.0;

    // Godot: MapState.cs FoundRadius
    /// Buildings seen within this distance are added to the map.
    pub const FOUND_RADIUS: f32 = 60.0;

    pub fn new() -> Self {
        Self::default()
    }

    /// Reveals cells around the player's world XZ position.
    // Godot: MapState.cs Reveal
    pub fn reveal(&mut self, xz: Vec2) -> usize {
        self.explore.reveal(xz, Self::REVEAL_RADIUS)
    }

    /// Adds a building, or updates it (its looted flag) when it is already known.
    // Godot: MapState.cs NoteBuilding
    pub fn note_building(&mut self, building: MapBuilding) {
        if let Some(existing) = self.found.iter_mut().find(|b| b.id == building.id) {
            *existing = building;
        } else {
            self.found.push(building);
        }
    }

    /// True if the building with the given ID has already been recorded.
    // Godot: MapState.cs HasBuilding
    pub fn has_building(&self, id: &str) -> bool {
        self.found.iter().any(|b| b.id == id)
    }

    /// Sets the player's custom waypoint at world XZ.
    // Godot: WorldMap.cs SetWaypoint
    pub fn set_waypoint(&mut self, xz: Vec2) {
        self.waypoint = Some(xz);
    }

    /// Clears the player's custom waypoint.
    // Godot: WorldMap.cs ClearWaypoint
    pub fn clear_waypoint(&mut self) {
        self.waypoint = None;
    }

    /// Saves map data under Godot save keys: "explored", "found", "death", "waypoint".
    // Godot: MapState.cs ToDictionary
    pub fn to_save(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert("explored".to_string(), self.explore.to_save());

        let found_list: Vec<serde_json::Value> = self
            .found
            .iter()
            .map(|b| {
                serde_json::json!({
                    "id": b.id,
                    "type": b.building_type,
                    "x": b.position.x,
                    "z": b.position.y,
                    "looted": b.looted,
                })
            })
            .collect();
        map.insert("found".to_string(), serde_json::Value::Array(found_list));

        if let Some(death) = self.death_spot {
            map.insert("death".to_string(), serde_json::json!({ "x": death.x, "z": death.y }));
        }
        if let Some(waypoint) = self.waypoint {
            map.insert("waypoint".to_string(), serde_json::json!({ "x": waypoint.x, "z": waypoint.y }));
        }

        serde_json::Value::Object(map)
    }

    /// Restores the map from Godot save data.
    // Godot: MapState.cs Load
    pub fn from_save(&mut self, data: &serde_json::Value) {
        if let Some(explored_val) = data.get("explored") {
            self.explore.from_save(explored_val);
        } else {
            self.explore = MapExplore::new();
        }

        self.found.clear();
        if let Some(found_arr) = data.get("found").and_then(|v| v.as_array()) {
            for entry in found_arr {
                if let (Some(id), Some(b_type), Some(x), Some(z)) = (
                    entry.get("id").and_then(|v| v.as_str()),
                    entry.get("type").and_then(|v| v.as_str()),
                    entry.get("x").and_then(|v| v.as_f64()),
                    entry.get("z").and_then(|v| v.as_f64()),
                ) && !id.is_empty() {
                    let looted = entry.get("looted").and_then(|v| v.as_bool()).unwrap_or(false);
                    self.found.push(MapBuilding::new(
                        id,
                        b_type,
                        Vec2::new(x as f32, z as f32),
                        looted,
                    ));
                }
            }
        }

        self.death_spot = read_point(data.get("death"));
        self.waypoint = read_point(data.get("waypoint"));
    }
}

fn read_point(val: Option<&serde_json::Value>) -> Option<Vec2> {
    let obj = val?.as_object()?;
    let x = obj.get("x")?.as_f64()? as f32;
    let z = obj.get("z")?.as_f64()? as f32;
    Some(Vec2::new(x, z))
}
