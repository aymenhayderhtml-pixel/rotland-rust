//! Persistence for fallen trees.
//! Port of `scripts/World/WorldState.cs` fallen tree save/load.

use std::collections::HashSet;
use serde_json::{json, Value};

/// Godot system id for WorldState, which owns fallen trees.
pub const TREE_WORLDSTATE_SAVE_ID: &str = "worldstate";
/// Exact Godot key within WorldState save data for fallen tree ids.
pub const TREE_FALLEN_KEY: &str = "fallen_trees";

/// Serializes a set of fallen tree ids into the Godot-compatible WorldState JSON structure.
// Godot: WorldState.cs Save() -> ["fallen_trees"]
pub fn to_save_fallen_trees(fallen: &HashSet<String>) -> Value {
    let mut list: Vec<String> = fallen.iter().cloned().collect();
    list.sort();
    json!({
        TREE_FALLEN_KEY: list,
    })
}

/// Loads fallen tree ids from a Godot save value.
/// Accepts either a dictionary with key "fallen_trees" or directly an array of tree id strings.
// Godot: WorldState.cs Load() -> SaveData.GetList(data, "fallen_trees")
pub fn from_save_fallen_trees(data: &Value) -> HashSet<String> {
    let mut set = HashSet::new();
    let items_opt = if let Some(arr) = data.get(TREE_FALLEN_KEY).and_then(|v| v.as_array()) {
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
