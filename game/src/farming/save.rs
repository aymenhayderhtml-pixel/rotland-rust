use glam::Vec3;
use serde_json::{json, Value};

use crate::farming::manager::FarmManager;
use crate::farming::record::PlotRecord;

/// Godot save ID for the farming system.
pub const SAVE_ID: &str = "farm";

/// Serializes one PlotRecord to a Godot-compatible dictionary.
// Godot: PlotRecord.cs ToDictionary
pub fn to_save_plot(plot: &PlotRecord) -> Value {
    json!({
        "id": plot.id,
        "x": plot.position.x,
        "y": plot.position.y,
        "z": plot.position.z,
        "camp": plot.camp,
        "crop": plot.crop.as_deref().unwrap_or(""),
        "growth": plot.growth,
        "dry": plot.dry_hours,
        "watered": plot.watered_at,
        "dead": plot.dead,
    })
}

/// Deserializes one PlotRecord from a Godot save dictionary.
// Godot: PlotRecord.cs FromDictionary
pub fn from_save_plot(data: &Value) -> Result<PlotRecord, String> {
    let obj = data
        .as_object()
        .ok_or_else(|| "expected plot JSON object".to_string())?;

    let id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let x = obj.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let y = obj.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let z = obj.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

    let camp = obj.get("camp").and_then(|v| v.as_bool()).unwrap_or(false);

    let crop = obj
        .get("crop")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let growth = obj.get("growth").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let dry_hours = obj.get("dry").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let watered_at = obj.get("watered").and_then(|v| v.as_f64()).unwrap_or(-1.0);
    let dead = obj.get("dead").and_then(|v| v.as_bool()).unwrap_or(false);

    Ok(PlotRecord {
        id,
        position: Vec3::new(x, y, z),
        camp,
        crop,
        growth,
        dry_hours,
        watered_at,
        dead,
    })
}

/// Serializes the entire FarmManager state into Godot save JSON format.
// Godot: FarmManager.Save.cs Save
pub fn to_save_manager(manager: &FarmManager) -> Value {
    let plots: Vec<Value> = manager.plots.iter().map(to_save_plot).collect();
    json!({
        "plots": plots,
        "synced": manager.synced_hours,
        "charges": manager.charges,
        "nextId": manager.next_id,
    })
}

/// Deserializes and loads FarmManager state from Godot save JSON format.
// Godot: FarmManager.Save.cs Load
pub fn from_save_manager(manager: &mut FarmManager, data: &Value) -> Result<(), String> {
    let obj = data
        .as_object()
        .ok_or_else(|| "expected farm save JSON object".to_string())?;

    let mut new_plots = Vec::new();
    if let Some(plots_arr) = obj.get("plots").and_then(|v| v.as_array()) {
        for item in plots_arr {
            new_plots.push(from_save_plot(item)?);
        }
    }

    let synced = obj.get("synced").and_then(|v| v.as_f64()).unwrap_or(-1.0);
    let default_cap = manager.config.can_capacity as i64;
    let charges_raw = obj
        .get("charges")
        .and_then(|v| v.as_i64())
        .unwrap_or(default_cap);
    let charges = charges_raw.clamp(0, default_cap) as u32;

    let next_id_raw = obj.get("nextId").and_then(|v| v.as_i64()).unwrap_or(1);
    let next_id = next_id_raw.max(1) as u32;

    manager.plots = new_plots;
    manager.synced_hours = synced;
    manager.charges = charges;
    manager.next_id = next_id;
    manager.camp_checked = false;
    manager.on_bed.clear();

    Ok(())
}
