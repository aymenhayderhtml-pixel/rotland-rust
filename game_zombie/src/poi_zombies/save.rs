//! Persistence for visited POI buildings.
//! Port of `scripts/World/WorldState.cs` and `scripts/World/PoiManager.cs` save/load.

use std::collections::HashSet;
use serde_json::{json, Value};

/// Godot system id for WorldState, which owns visited POIs.
// Godot: WorldState.cs SaveId => "worldstate"
pub const POI_ZOMBIES_SAVE_ID: &str = "worldstate";

/// Exact Godot key within WorldState save data for visited POI ids.
// Godot: WorldState.cs Save() -> ["visited_pois"]
pub const POI_ZOMBIES_SAVE_KEY: &str = "visited_pois";

/// Serializes a set of visited POI ids into the Godot-compatible WorldState JSON structure.
// Godot: WorldState.cs Save() -> ["visited_pois"] = _pois.VisitedIds().Cast<object>().ToList()
pub fn to_save_visited_pois(visited: &HashSet<String>) -> Value {
    let mut list: Vec<String> = visited.iter().cloned().collect();
    list.sort();
    json!({
        POI_ZOMBIES_SAVE_KEY: list,
    })
}

/// Loads visited POI ids from a Godot save value.
/// Accepts either a dictionary with key "visited_pois" or directly an array of POI id strings.
// Godot: WorldState.cs Load() -> _pois.LoadVisited(SaveData.GetList(data, "visited_pois"))
pub fn from_save_visited_pois(data: &Value) -> HashSet<String> {
    let mut set = HashSet::new();
    let items_opt = if let Some(arr) = data.get(POI_ZOMBIES_SAVE_KEY).and_then(|v| v.as_array()) {
        Some(arr)
    } else {
        data.as_array()
    };

    if let Some(arr) = items_opt {
        for item in arr {
            if let Some(s) = item.as_str() {
                set.insert(s.to_string());
            }
        }
    }
    set
}
