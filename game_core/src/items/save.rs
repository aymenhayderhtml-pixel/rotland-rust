use serde::{Deserialize, Serialize};

use crate::items::inventory::Inventory;
use crate::items::stack::ItemStack;

/// One stack entry in save data (port of scripts/Items/InventorySave.cs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedSlot {
    pub slot: usize,
    pub id: String,
    pub count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wear: Option<f32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mods: Vec<String>,
}

/// Serialized inventory data matching the structure of InventorySave.cs.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InventorySaveData {
    pub stacks: Vec<SavedSlot>,
}

impl InventorySaveData {
    /// Extracts save data from an inventory's slots (matching InventorySave.Write).
    pub fn write_from_slots(slots: &[Option<ItemStack>]) -> Self {
        let mut stacks = Vec::new();
        for (i, slot) in slots.iter().enumerate() {
            if let Some(stack) = slot {
                stacks.push(SavedSlot {
                    slot: i,
                    id: stack.item_id.clone(),
                    count: stack.count,
                    wear: stack.wear,
                    mods: stack.mods.clone(),
                });
            }
        }
        Self { stacks }
    }

    /// Reads saved stacks into an inventory (matching InventorySave.Read).
    pub fn read_into_inventory(&self, inventory: &mut Inventory) {
        inventory.clear();
        for entry in &self.stacks {
            if entry.slot >= inventory.slot_count() || entry.count == 0 {
                continue;
            }
            if let Some(def) = inventory.registry().get(&entry.id) {
                let count = entry.count.min(def.max_stack);
                let wear = entry.wear.map(|w| if w.is_finite() { w.clamp(0.0, 1.0) } else { 1.0 });
                if let Some(slot) = inventory.slots_mut().get_mut(entry.slot) {
                    *slot = Some(ItemStack {
                        item_id: entry.id.clone(),
                        count,
                        wear,
                        mods: entry.mods.clone(),
                    });
                }
            }
        }
        inventory.set_changed();
    }
}
