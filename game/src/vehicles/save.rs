use std::collections::HashMap;
use crate::vehicles::car::Car;

pub const CAR_SYSTEM_ID: &str = "cars";

/// Serializes cars data matching Godot CarManager.cs Save (system id "cars").
pub fn to_save(records: &HashMap<String, serde_json::Value>, live: &HashMap<String, Car>) -> serde_json::Value {
    let mut all_records = records.clone();
    for (id, car) in live {
        all_records.insert(id.clone(), car.to_record());
    }
    let list: Vec<serde_json::Value> = all_records.into_values().collect();
    serde_json::json!({
        "cars": list
    })
}

/// Deserializes cars data matching Godot CarManager.cs Load.
pub fn from_save(
    records: &mut HashMap<String, serde_json::Value>,
    live: &mut HashMap<String, Car>,
    data: &serde_json::Value,
) {
    live.clear();
    records.clear();
    if let Some(list) = data.get("cars").and_then(|v| v.as_array()) {
        for item in list {
            if let Some(id) = item.get("id").and_then(|v| v.as_str())
                && !id.is_empty()
            {
                records.insert(id.to_string(), item.clone());
            }
        }
    }
}
