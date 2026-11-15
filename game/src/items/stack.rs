use serde::{Deserialize, Serialize};

/// Default backpack slot count (24 slots, from scripts/Items/Inventory.cs).
pub const DEFAULT_SLOT_COUNT: usize = 24;

/// Default carry capacity in kg (40 kg, from scripts/Items/Inventory.cs).
pub const DEFAULT_MAX_WEIGHT: f32 = 40.0;

/// Float rounding must not stop a load that exactly fills the weight limit.
pub const WEIGHT_EPSILON: f32 = 0.0001;

/// A number of one item type sitting in one inventory slot (port of scripts/Items/ItemStack.cs).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemStack {
    pub item_id: String,
    pub count: u32,
}

impl ItemStack {
    pub fn new(item_id: impl Into<String>, count: u32) -> Self {
        Self {
            item_id: item_id.into(),
            count,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryEvent {
    Changed,
}
