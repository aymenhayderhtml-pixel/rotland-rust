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

    pub fn with_items(items: Vec<WorldItem>) -> Self {
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
    pub fn nearest_within_mut(&mut self, pos: Vec3, radius: f32) -> Option<(usize, &mut WorldItem)> {
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
    pub fn nearest_index(&self, pos: Vec3, radius: f32) -> Option<usize> {
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
}
