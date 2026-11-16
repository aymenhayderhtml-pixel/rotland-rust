use glam::{IVec2, Vec3};
use serde_json::{json, Map, Value};

use super::fire::{Campfire, BURN_HOURS};

/// Godot SaveId for WorldState, which holds placed campfires.
pub const WORLDSTATE_SAVE_ID: &str = "worldstate";
/// Key within worldstate save data for campfires.
pub const CAMPFIRES_KEY: &str = "campfires";

/// Record of an unloaded campfire.
/// Port of `scripts/World/StoredCampfire.cs`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StoredCampfire {
    pub position: Vec3,
    pub hours: f32,
}

impl StoredCampfire {
    pub fn new(position: Vec3, hours: f32) -> Self {
        Self { position, hours }
    }

    pub fn to_campfire(&self) -> Campfire {
        Campfire::restore(self.position, self.hours, false)
    }

    pub fn from_campfire(fire: &Campfire) -> Self {
        Self {
            position: fire.position,
            hours: fire.remaining_hours,
        }
    }
}

/// Converts world position to 64m chunk coordinate.
pub fn world_to_chunk(pos: Vec3) -> IVec2 {
    IVec2::new(
        (pos.x / 64.0).floor() as i32,
        (pos.z / 64.0).floor() as i32,
    )
}

/// Serializes non-permanent campfires to a Godot-compatible dictionary
/// mapping `"{x}_{z}"` chunk coordinate keys to lists of campfire records.
// Godot: WorldState.cs SaveCampfires
pub fn to_save_campfires(fires: &[Campfire]) -> Value {
    let mut map: std::collections::HashMap<String, Vec<Value>> = std::collections::HashMap::new();
    for fire in fires {
        if fire.permanent {
            continue;
        }
        let coord = world_to_chunk(fire.position);
        let key = format!("{}_{}", coord.x, coord.y);
        let entry = json!({
            "x": fire.position.x,
            "y": fire.position.y,
            "z": fire.position.z,
            "hours": fire.remaining_hours,
        });
        map.entry(key).or_default().push(entry);
    }

    let mut out = Map::new();
    for (k, v) in map {
        out.insert(k, Value::Array(v));
    }
    Value::Object(out)
}

/// Deserializes campfires from a Godot save dictionary under `campfires`.
// Godot: WorldState.cs LoadCampfires
pub fn from_save_campfires(data: &Value) -> Result<Vec<Campfire>, String> {
    let Some(obj) = data.as_object() else {
        return Ok(Vec::new());
    };

    let mut fires = Vec::new();
    for (_key, val) in obj {
        let Some(list) = val.as_array() else {
            continue;
        };
        for item in list {
            let Some(entry) = item.as_object() else {
                continue;
            };
            let x = entry.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let y = entry.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let z = entry.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let hours = entry
                .get("hours")
                .and_then(|v| v.as_f64())
                .unwrap_or(BURN_HOURS as f64) as f32;

            fires.push(Campfire::restore(Vec3::new(x, y, z), hours, false));
        }
    }
    Ok(fires)
}
