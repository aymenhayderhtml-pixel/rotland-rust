use serde::{Deserialize, Serialize};

/// One stack entry in inventory save data.
/// Matches `scripts/Items/InventorySave.cs`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedSlot {
    pub slot: usize,
    pub id: String,
    pub count: u32,
}

impl SavedSlot {
    pub fn new(slot: usize, id: impl Into<String>, count: u32) -> Self {
        Self {
            slot,
            id: id.into(),
            count,
        }
    }
}

/// Inventory contents: backpack and hotbar (the first 6 slots, 0..5).
/// Port of `scripts/Items/Inventory.cs` and `InventorySave.cs` (SaveId: "inventory").
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventorySaveData {
    #[serde(default)]
    pub stacks: Vec<SavedSlot>,
}

impl InventorySaveData {
    pub fn new(stacks: Vec<SavedSlot>) -> Self {
        Self { stacks }
    }
}
