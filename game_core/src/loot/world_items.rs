use glam::Vec3;
use serde::{Deserialize, Serialize};

/// An item lying on the ground (port of scripts/World/DroppedItem.cs and scripts/Items/WorldItem.cs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldItem {
    pub item_id: String,
    pub count: u32,
    pub position: Vec3,
}

impl WorldItem {
    pub fn new(item_id: impl Into<String>, count: u32, position: Vec3) -> Self {
        Self {
            item_id: item_id.into(),
            count,
            position,
        }
    }

    pub fn id(&self) -> &str {
        &self.item_id
    }

    pub fn item_id(&self) -> &str {
        &self.item_id
    }
}

/// A list of dropped items in the world (port of WorldState dropped items).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WorldItems {
    items: Vec<WorldItem>,
}

impl WorldItems {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    fn with_items(items: Vec<WorldItem>) -> Self {
        Self { items }
    }

    /// Adds a dropped item to the world.
    pub fn add(&mut self, item_id: impl Into<String>, count: u32, position: Vec3) {
        if count > 0 {
            self.items.push(WorldItem::new(item_id, count, position));
        }
    }

    pub fn push(&mut self, item: WorldItem) {
        if item.count > 0 {
            self.items.push(item);
        }
    }

    /// Removes the item at index. Returns the removed item, or None if out of bounds.
    pub fn remove(&mut self, index: usize) -> Option<WorldItem> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    /// Finds the nearest dropped item within `radius` of `pos`.
    /// Returns its index and reference if found.
    pub fn nearest_within(&self, pos: Vec3, radius: f32) -> Option<(usize, &WorldItem)> {
        let r2 = radius * radius;
        let mut best: Option<(usize, f32)> = None;

        for (i, item) in self.items.iter().enumerate() {
            let d2 = item.position.distance_squared(pos);
            if d2 <= r2 {
                if let Some((_, best_d2)) = best {
                    if d2 < best_d2 {
                        best = Some((i, d2));
                    }
                } else {
                    best = Some((i, d2));
                }
            }
        }

        best.map(|(i, _)| (i, &self.items[i]))
    }

    /// Finds the nearest dropped item within `radius` of `pos` with mutable access.
    #[allow(dead_code)]
    fn nearest_within_mut(&mut self, pos: Vec3, radius: f32) -> Option<(usize, &mut WorldItem)> {
        let r2 = radius * radius;
        let mut best: Option<(usize, f32)> = None;

        for (i, item) in self.items.iter().enumerate() {
            let d2 = item.position.distance_squared(pos);
            if d2 <= r2 {
                if let Some((_, best_d2)) = best {
                    if d2 < best_d2 {
                        best = Some((i, d2));
                    }
                } else {
                    best = Some((i, d2));
                }
            }
        }

        best.map(move |(i, _)| (i, &mut self.items[i]))
    }

    /// Finds the nearest dropped item within `radius` of `pos`.
    pub fn nearest(&self, pos: Vec3, radius: f32) -> Option<&WorldItem> {
        self.nearest_within(pos, radius).map(|(_, item)| item)
    }

    /// Finds the index of the nearest dropped item within `radius` of `pos`.
    #[allow(dead_code)]
    fn nearest_index(&self, pos: Vec3, radius: f32) -> Option<usize> {
        self.nearest_within(pos, radius).map(|(i, _)| i)
    }

    pub fn get(&self, index: usize) -> Option<&WorldItem> {
        self.items.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut WorldItem> {
        self.items.get_mut(index)
    }

    pub fn items(&self) -> &[WorldItem] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut [WorldItem] {
        &mut self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &WorldItem> {
        self.items.iter()
    }

    /// Converts world position to 64m chunk coordinate key ("{chunk_x}_{chunk_z}").
    // Godot: WorldState.cs KeyOf / WorldToChunk
    pub fn chunk_key(pos: Vec3) -> String {
        let cx = (pos.x / 64.0).floor() as i32;
        let cz = (pos.z / 64.0).floor() as i32;
        format!("{cx}_{cz}")
    }

    /// Serializes dropped items to a Godot-compatible JSON Value mapping chunk keys to item lists.
    // Godot: WorldState.cs Save (dropped)
    pub fn to_save_json(&self) -> serde_json::Value {
        let mut map: std::collections::HashMap<String, Vec<serde_json::Value>> = std::collections::HashMap::new();
        for item in &self.items {
            let key = Self::chunk_key(item.position);
            map.entry(key).or_default().push(serde_json::json!({
                "id": item.item_id,
                "count": item.count,
                "x": item.position.x,
                "y": item.position.y,
                "z": item.position.z,
            }));
        }
        serde_json::to_value(map).unwrap_or_default()
    }

    /// Deserializes dropped items from a Godot-compatible JSON Value mapping chunk keys to item lists.
    // Godot: WorldState.cs Load (dropped)
    pub fn from_save_json(&mut self, data: &serde_json::Value) {
        self.clear();
        let Some(obj) = data.as_object() else { return };
        for (_key, val) in obj {
            let Some(list) = val.as_array() else { continue };
            for entry in list {
                let Some(item_obj) = entry.as_object() else { continue };
                let id = item_obj.get("id").and_then(|v| v.as_str()).unwrap_or_default();
                let count = item_obj.get("count").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let x = item_obj.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let y = item_obj.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let z = item_obj.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                if !id.is_empty() && count > 0 && x.is_finite() && y.is_finite() && z.is_finite() {
                    self.add(id, count, Vec3::new(x, y, z));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_key_64m_grid() {
        assert_eq!(WorldItems::chunk_key(Vec3::new(0.0, 0.0, 0.0)), "0_0");
        assert_eq!(WorldItems::chunk_key(Vec3::new(63.9, 0.0, 63.9)), "0_0");
        assert_eq!(WorldItems::chunk_key(Vec3::new(64.0, 0.0, 0.0)), "1_0");
        assert_eq!(WorldItems::chunk_key(Vec3::new(-1.0, 0.0, -1.0)), "-1_-1");
    }

    #[test]
    fn dropped_items_round_trip_json() {
        let mut items = WorldItems::new();
        items.add("canned_beans", 2, Vec3::new(10.0, 0.5, 15.0));
        items.add("bandage", 1, Vec3::new(70.0, 1.0, 5.0));

        let json = items.to_save_json();
        let mut loaded = WorldItems::new();
        loaded.from_save_json(&json);

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.items()[0].item_id, "canned_beans");
        assert_eq!(loaded.items()[0].count, 2);
        assert_eq!(loaded.items()[0].position, Vec3::new(10.0, 0.5, 15.0));
        assert_eq!(loaded.items()[1].item_id, "bandage");
        assert_eq!(loaded.items()[1].count, 1);
        assert_eq!(loaded.items()[1].position, Vec3::new(70.0, 1.0, 5.0));
    }
}
