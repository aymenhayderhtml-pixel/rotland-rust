use crate::bounties::def::{BountyDef, BountySaveError};
use crate::bounties::manager::BountyManager;

/// Serializes a BountyDef into a Godot save dictionary format.
// Godot: BountyDef.cs ToDictionary
pub fn to_save_bounty(bounty: &BountyDef) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("id".to_string(), serde_json::Value::String(bounty.id.clone()));
    map.insert(
        "type".to_string(),
        serde_json::Value::String(bounty.bounty_type.clone()),
    );
    map.insert(
        "text".to_string(),
        serde_json::Value::String(bounty.text.clone()),
    );
    map.insert("target".to_string(), serde_json::json!(bounty.target));
    map.insert("progress".to_string(), serde_json::json!(bounty.progress));
    map.insert("dollars".to_string(), serde_json::json!(bounty.dollars));
    map.insert("xp".to_string(), serde_json::json!(bounty.xp));
    map.insert(
        "skill".to_string(),
        serde_json::Value::String(bounty.skill.clone()),
    );
    map.insert(
        "expiresOn".to_string(),
        serde_json::json!(bounty.expires_on),
    );
    map.insert("weekly".to_string(), serde_json::Value::Bool(bounty.weekly));
    map.insert("targetX".to_string(), serde_json::json!(bounty.target_x));
    map.insert("targetZ".to_string(), serde_json::json!(bounty.target_z));

    if let Some(z) = &bounty.zombie {
        map.insert("zombie".to_string(), serde_json::Value::String(z.clone()));
    }
    if let Some(a) = &bounty.animal {
        map.insert("animal".to_string(), serde_json::Value::String(a.clone()));
    }
    if let Some(i) = &bounty.item {
        map.insert("item".to_string(), serde_json::Value::String(i.clone()));
    }
    if let Some(b) = &bounty.building {
        map.insert("building".to_string(), serde_json::Value::String(b.clone()));
    }
    if let Some(poi) = &bounty.poi_id {
        map.insert("poi".to_string(), serde_json::Value::String(poi.clone()));
    }
    serde_json::Value::Object(map)
}

/// Deserializes a BountyDef from a Godot save dictionary format.
// Godot: BountyDef.cs FromDictionary
pub fn from_save_bounty(data: &serde_json::Value) -> Result<BountyDef, BountySaveError> {
    let obj = data.as_object().ok_or_else(|| {
        BountySaveError::InvalidValue("expected bounty json object".to_string())
    })?;

    let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let bounty_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let text = obj.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let target = obj.get("target").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
    let progress = obj.get("progress").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let dollars = obj.get("dollars").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let xp = obj.get("xp").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let skill = obj.get("skill").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let expires_on = obj.get("expiresOn").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let weekly = obj.get("weekly").and_then(|v| v.as_bool()).unwrap_or(false);
    let target_x = obj.get("targetX").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let target_z = obj.get("targetZ").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

    let zombie = obj.get("zombie").and_then(|v| v.as_str()).map(|s| s.to_string());
    let animal = obj.get("animal").and_then(|v| v.as_str()).map(|s| s.to_string());
    let item = obj.get("item").and_then(|v| v.as_str()).map(|s| s.to_string());
    let building = obj.get("building").and_then(|v| v.as_str()).map(|s| s.to_string());
    let poi_id = obj.get("poi").and_then(|v| v.as_str()).map(|s| s.to_string());

    Ok(BountyDef {
        id,
        bounty_type,
        text,
        target,
        progress,
        dollars,
        xp,
        skill,
        zombie,
        animal,
        item,
        building,
        poi_id,
        target_x,
        target_z,
        expires_on,
        weekly,
    })
}

/// Serializes BountyManager state into Godot save dictionary format.
// Godot: BountyManager.cs Save
pub fn to_save_manager(manager: &BountyManager) -> serde_json::Value {
    serde_json::json!({
        "offered": manager.offered().iter().map(|b| b.to_save()).collect::<Vec<_>>(),
        "active": manager.active().iter().map(|b| b.to_save()).collect::<Vec<_>>(),
        "claimed": manager.claimed().iter().cloned().collect::<Vec<_>>(),
        "dailyDay": manager.daily_day(),
        "week": manager.week(),
    })
}

/// Deserializes BountyManager state from Godot save dictionary format.
// Godot: BountyManager.cs Load
pub fn from_save_manager(
    manager: &mut BountyManager,
    data: &serde_json::Value,
) -> Result<(), BountySaveError> {
    let obj = data.as_object().ok_or_else(|| {
        BountySaveError::InvalidValue("expected bounty manager save object".to_string())
    })?;

    let mut offered = Vec::new();
    let mut active = Vec::new();
    let mut claimed = std::collections::HashSet::new();

    if let Some(offered_arr) = obj.get("offered").and_then(|v| v.as_array()) {
        for entry in offered_arr {
            offered.push(BountyDef::from_save(entry)?);
        }
    }

    if let Some(active_arr) = obj.get("active").and_then(|v| v.as_array()) {
        for entry in active_arr {
            active.push(BountyDef::from_save(entry)?);
        }
    }

    if let Some(claimed_arr) = obj.get("claimed").and_then(|v| v.as_array()) {
        for entry in claimed_arr {
            if let Some(id) = entry.as_str() {
                claimed.insert(id.to_string());
            }
        }
    }

    let daily_day = obj
        .get("dailyDay")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1) as i32;
    let week = obj
        .get("week")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1) as i32;

    manager.set_save_state(offered, active, claimed, daily_day, week);
    Ok(())
}
