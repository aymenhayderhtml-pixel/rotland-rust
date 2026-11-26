use serde::{Deserialize, Serialize};

/// One stack entry in inventory save data.
/// Matches `scripts/Items/InventorySave.cs`.
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

impl SavedSlot {
    pub fn new(slot: usize, id: impl Into<String>, count: u32) -> Self {
        Self {
            slot,
            id: id.into(),
            count,
            wear: None,
            mods: Vec::new(),
        }
    }
}

/// Inventory contents: backpack and hotbar (the first 6 slots, 0..5).
/// Port of `scripts/Items/Inventory.cs` and `InventorySave.cs` (SaveId: "inventory").
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InventorySaveData {
    #[serde(default)]
    pub stacks: Vec<SavedSlot>,
}

impl InventorySaveData {
    pub fn new(stacks: Vec<SavedSlot>) -> Self {
        Self { stacks }
    }
}
