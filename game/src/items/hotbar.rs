use crate::items::def::ItemCategory;
use crate::items::inventory::{Inventory, ItemStack};
use crate::items::registry::ItemRegistry;
use crate::weapons::{WeaponDef, WeaponRegistry};

/// The first six backpack slots, bound to keys 1–6 (from scripts/Player/Hotbar.cs).
pub const HOTBAR_SIZE: usize = 6;

/// Result of pressing a hotbar slot key.
#[derive(Clone, Debug, PartialEq)]
pub enum HotbarAction {
    EquipWeapon { item_id: String, weapon_id: String },
    UnequipWeapon { item_id: String },
    UseItem { item_id: String },
    Empty,
    InvalidKey,
}

/// The hotbar: manages the first 1..N backpack slots, selecting items and equipping weapons (port of scripts/Player/Hotbar.cs).
#[derive(Clone, Debug)]
pub struct Hotbar {
    slot_count: usize,
    equipped_item_id: Option<String>,
}

impl Default for Hotbar {
    fn default() -> Self {
        Self::new()
    }
}

impl Hotbar {
    /// Creates a hotbar with default size (6 slots).
    pub fn new() -> Self {
        Self::with_size(HOTBAR_SIZE)
    }

    /// Creates a hotbar with a custom slot count.
    pub fn with_size(slot_count: usize) -> Self {
        Self {
            slot_count,
            equipped_item_id: None,
        }
    }

    /// Number of hotbar slots (default 6).
    pub fn slot_count(&self) -> usize {
        self.slot_count
    }

    /// The inventory item id of the weapon currently in hand, or None.
    pub fn equipped_item_id(&self) -> Option<&str> {
        self.equipped_item_id.as_deref()
    }

    /// Alias for equipped_item_id: the currently selected weapon item.
    pub fn selected_item(&self) -> Option<&str> {
        self.equipped_item_id.as_deref()
    }

    /// Returns the inventory entry shown by hotbar key 1..N (slot index key - 1).
    pub fn slot_entry<'a>(&self, key: usize, inventory: &'a Inventory) -> Option<&'a ItemStack> {
        if key < 1 || key > self.slot_count {
            return None;
        }
        inventory.slots().get(key - 1).and_then(|s| s.as_ref())
    }

    /// Slice of inventory slots shown on the hotbar.
    pub fn entries<'a>(&self, inventory: &'a Inventory) -> &'a [Option<ItemStack>] {
        let count = self.slot_count.min(inventory.slots().len());
        &inventory.slots()[..count]
    }

    /// Acts on hotbar key 1..N (slot index key - 1). Weapons are toggled; usable items are used.
    pub fn press(
        &mut self,
        key: usize,
        inventory: &Inventory,
        registry: &ItemRegistry,
    ) -> HotbarAction {
        if key < 1 || key > self.slot_count || key - 1 >= inventory.slots().len() {
            return HotbarAction::InvalidKey;
        }
        let Some(stack) = &inventory.slots()[key - 1] else {
            return HotbarAction::Empty;
        };
        let Some(def) = registry.get(&stack.item_id) else {
            return HotbarAction::Empty;
        };

        match def.category {
            ItemCategory::Weapon => {
                if self.equipped_item_id.as_deref() == Some(&stack.item_id) {
                    let item_id = self.equipped_item_id.take().unwrap();
                    HotbarAction::UnequipWeapon { item_id }
                } else {
                    self.equipped_item_id = Some(stack.item_id.clone());
                    HotbarAction::EquipWeapon {
                        item_id: stack.item_id.clone(),
                        weapon_id: def.weapon_id.clone(),
                    }
                }
            }
            ItemCategory::Food
            | ItemCategory::Drink
            | ItemCategory::Medical
            | ItemCategory::Tool
            | ItemCategory::Placeable => HotbarAction::UseItem {
                item_id: stack.item_id.clone(),
            },
            _ => HotbarAction::Empty,
        }
    }

    /// Selects key 1..N. Returns the selected weapon item id if equipped, or None if holstered or empty.
    pub fn select(
        &mut self,
        key: usize,
        inventory: &Inventory,
        registry: &ItemRegistry,
    ) -> Option<&str> {
        self.press(key, inventory, registry);
        self.selected_item()
    }

    /// Selects by 0-based slot index (slot_index 0 = key 1).
    pub fn select_slot(
        &mut self,
        slot_index: usize,
        inventory: &Inventory,
        registry: &ItemRegistry,
    ) -> Option<&str> {
        self.select(slot_index + 1, inventory, registry)
    }

    /// Puts a carried weapon in hand, or away if it is already in hand. False when not a carried weapon.
    pub fn toggle_equip_item(
        &mut self,
        item_id: &str,
        inventory: &Inventory,
        registry: &ItemRegistry,
    ) -> bool {
        if !inventory.has(item_id, 1) {
            return false;
        }
        let Some(def) = registry.get(item_id) else {
            return false;
        };
        if def.category != ItemCategory::Weapon {
            return false;
        }

        if self.equipped_item_id.as_deref() == Some(item_id) {
            self.holster();
        } else {
            self.equipped_item_id = Some(item_id.to_string());
        }
        true
    }

    /// Puts away any equipped weapon.
    pub fn holster(&mut self) {
        self.equipped_item_id = None;
    }

    /// Alias for holster().
    pub fn unequip(&mut self) {
        self.holster();
    }

    /// A weapon whose item was used up or removed can no longer stay in hand.
    pub fn on_inventory_changed(&mut self, inventory: &Inventory) {
        if let Some(id) = &self.equipped_item_id {
            if !inventory.has(id, 1) {
                self.holster();
            }
        }
    }

    /// Resolves the equipped weapon in hand through WeaponRegistry.
    pub fn selected_weapon<'a>(
        &self,
        registry: &ItemRegistry,
        weapons: &'a WeaponRegistry,
    ) -> Option<&'a WeaponDef> {
        let item_id = self.selected_item()?;
        let item_def = registry.get(item_id)?;
        item_def.weapon(weapons)
    }
}
