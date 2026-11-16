//! Which bag items the dog eats (feedItems in dog.json).
// Godot: DogFood.cs

use crate::items::Inventory;

/// Functions for finding and consuming dog food items from inventory.
// Godot: DogFood.cs
pub struct DogFood;

impl DogFood {
    /// The first food item the bag holds, or None when there is none.
    // Godot: DogFood.cs Find
    pub fn find(bag: &Inventory, feed_items: &[String]) -> Option<String> {
        for id in feed_items {
            if bag.has(id, 1) {
                return Some(id.clone());
            }
        }
        None
    }

    /// Removes one food item from the bag. Returns false when there is none.
    // Godot: DogFood.cs Take
    pub fn take(bag: &mut Inventory, feed_items: &[String]) -> bool {
        if let Some(id) = Self::find(bag, feed_items) {
            bag.remove(&id, 1)
        } else {
            false
        }
    }
}
