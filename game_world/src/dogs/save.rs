//! Serialization and deserialization matching Godot DogManager save keys.
// Godot: DogManager.cs Save & Load

use glam::Vec3;
use game_core::items::InventorySaveData;
use super::dog::Dog;
use super::growth::DogGrowth;
use super::manager::Dogs;
use super::mode::DogMode;

/// Godot save system id for DogManager.
// Godot: DogManager.cs SaveId => "dog"
pub const DOG_SAVE_ID: &str = "dog";

impl Dogs {
    /// Serializes DogManager state using Godot save keys.
    // Godot: DogManager.cs Save
    pub fn to_save(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert("dead_count".to_string(), serde_json::json!(self.dead_count));
        let tamed_vec: Vec<serde_json::Value> = self
            .tamed
            .iter()
            .map(|s| serde_json::Value::String(s.clone()))
            .collect();
        map.insert("tamed".to_string(), serde_json::Value::Array(tamed_vec));

        if let Some(dog) = &self.dog
            && !dog.health.is_dead() {
                let pack_val = serde_json::to_value(dog.pack.save())
                    .unwrap_or_else(|_| serde_json::json!({"stacks": []}));
                let dog_obj = serde_json::json!({
                    "name": dog.name,
                    "health": dog.health.current(),
                    "mode": dog.mode.to_string(),
                    "x": dog.position.x,
                    "y": dog.position.y,
                    "z": dog.position.z,
                    "bought_day": dog.bought_day,
                    "growth_start": dog.growth_start,
                    "pack": pack_val,
                });
                map.insert("dog".to_string(), dog_obj);
            }
        serde_json::Value::Object(map)
    }

    /// Restores DogManager state from Godot save data.
    // Godot: DogManager.cs Load
    pub fn from_save(&mut self, data: &serde_json::Value) {
        if let Some(count) = data.get("dead_count").and_then(|v| v.as_i64()) {
            self.dead_count = count as i32;
        }
        self.tamed.clear();
        if let Some(arr) = data.get("tamed").and_then(|v| v.as_array()) {
            for val in arr {
                if let Some(s) = val.as_str() {
                    self.tamed.insert(s.to_string());
                }
            }
        }
        self.dog = None;
        let Some(saved) = data.get("dog") else {
            return;
        };

        let name = saved
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&self.config.default_name)
            .to_string();
        let health = saved
            .get("health")
            .and_then(|v| v.as_f64())
            .unwrap_or(self.config.max_health as f64) as f32;
        let mode_str = saved.get("mode").and_then(|v| v.as_str()).unwrap_or("Follow");
        let mode = DogMode::from_str_loose(mode_str);
        let x = saved.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let y = saved.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let z = saved.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let bought_day = saved.get("bought_day").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
        let growth_start = saved
            .get("growth_start")
            .and_then(|v| v.as_f64())
            .unwrap_or(DogGrowth::NO_START);

        let id = self.next_id;
        self.next_id += 1;
        let mut dog = Dog::new(id, name, mode, Vec3::new(x, y, z), &self.config);
        dog.bought_day = bought_day;
        dog.growth_start = growth_start;
        dog.health.set_current(health);
        if let Some(pack_val) = saved.get("pack")
            && let Ok(pack_save) = serde_json::from_value::<InventorySaveData>(pack_val.clone()) {
                dog.pack.load_save(&pack_save);
            }
        self.dog = Some(dog);
    }
}
