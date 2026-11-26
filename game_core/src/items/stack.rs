use serde::{Deserialize, Serialize};

/// Default backpack slot count (24 slots, from scripts/Items/Inventory.cs).
pub const DEFAULT_SLOT_COUNT: usize = 24;

/// Default carry capacity in kg (40 kg, from scripts/Items/Inventory.cs).
pub const DEFAULT_MAX_WEIGHT: f32 = 40.0;

/// Float rounding must not stop a load that exactly fills the weight limit.
pub const WEIGHT_EPSILON: f32 = 0.0001;

/// A number of one item type sitting in one inventory slot (port of scripts/Items/ItemStack.cs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ItemStack {
    pub item_id: String,
    pub count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wear: Option<f32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mods: Vec<String>,
}

impl ItemStack {
    pub fn new(item_id: impl Into<String>, count: u32) -> Self {
        Self {
            item_id: item_id.into(),
            count,
            wear: None,
            mods: Vec::new(),
        }
    }

    /// Weapon condition in 0.0..=1.0; `None` means brand new (1.0).
    pub fn condition(&self) -> f32 {
        match self.wear {
            Some(w) if w.is_finite() => w.clamp(0.0, 1.0),
            _ => 1.0,
        }
    }

    /// True if the given attachment id is fitted to this stack.
    pub fn has_mod(&self, id: &str) -> bool {
        self.mods.iter().any(|m| m == id)
    }

    /// True if the stack has no per-instance wear or attachments and may merge with identical plain stacks.
    pub fn is_plain(&self) -> bool {
        self.wear.is_none() && self.mods.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryEvent {
    Changed,
}
