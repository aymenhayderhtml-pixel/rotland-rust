use std::sync::Arc;

use crate::items::def::ItemDef;
use crate::items::registry::ItemRegistry;
use crate::items::save::InventorySaveData;
use crate::items::stack::{
    InventoryEvent, ItemStack, DEFAULT_MAX_WEIGHT, DEFAULT_SLOT_COUNT, WEIGHT_EPSILON,
};

/// The player's bag: fixed slots holding stacks of items, limited by weight (port of scripts/Items/Inventory.cs).
#[derive(Clone, Debug)]
pub struct Inventory {
    slots: Vec<Option<ItemStack>>,
    max_weight: f32,
    registry: Arc<ItemRegistry>,
    changed: bool,
    events: Vec<InventoryEvent>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

impl Inventory {
    /// Creates an inventory with default slots (24), default weight (40 kg), and loads items.json from the repo.
    pub fn new() -> Self {
        let registry = ItemRegistry::from_repo().expect("data/items.json must load for Inventory::new");
        Self::with_registry(Arc::new(registry))
    }
    pub fn with_registry(registry: Arc<ItemRegistry>) -> Self {
        Self::with_capacity_and_registry(DEFAULT_SLOT_COUNT, DEFAULT_MAX_WEIGHT, registry)
    }
    pub fn from_registry(registry: ItemRegistry) -> Self {
        Self::with_registry(Arc::new(registry))
    }
    pub fn with_capacity(slot_count: usize, max_weight: f32) -> Self {
        let registry = ItemRegistry::from_repo().expect("data/items.json must load for Inventory::with_capacity");
        Self::with_capacity_and_registry(slot_count, max_weight, Arc::new(registry))
    }
    pub fn with_capacity_and_registry(slot_count: usize, max_weight: f32, registry: Arc<ItemRegistry>) -> Self {
        Self {
            slots: vec![None; slot_count],
            max_weight,
            registry,
            changed: false,
            events: Vec::new(),
        }
    }

    pub fn slots(&self) -> &[Option<ItemStack>] { &self.slots }
    pub fn slots_mut(&mut self) -> &mut [Option<ItemStack>] { &mut self.slots }
    pub fn slot(&self, index: usize) -> Option<&ItemStack> { self.slots.get(index).and_then(|s| s.as_ref()) }
    pub fn slot_count(&self) -> usize { self.slots.len() }
    pub fn max_weight(&self) -> f32 { self.max_weight }
    pub fn set_max_weight(&mut self, max_weight: f32) { self.max_weight = max_weight; }
    pub fn set_slot_count(&mut self, new_count: usize) { self.slots.resize(new_count, None); }
    pub fn registry(&self) -> &ItemRegistry { &self.registry }
    pub fn set_registry(&mut self, registry: Arc<ItemRegistry>) { self.registry = registry; }

    /// Total weight of everything carried, in kg.
    pub fn total_weight(&self) -> f32 {
        self.total_weight_with(&self.registry)
    }

    /// Total weight calculated with an explicitly provided ItemRegistry.
    pub fn total_weight_with(&self, registry: &ItemRegistry) -> f32 {
        let mut total = 0.0;
        for slot in &self.slots {
            if let Some(stack) = slot
                && let Some(def) = registry.get(&stack.item_id) {
                    total += def.weight * (stack.count as f32);
                }
        }
        total
    }

    /// Returns the total count of an item across all slots.
    pub fn count(&self, id: &str) -> u32 {
        let mut total = 0;
        for slot in &self.slots {
            if let Some(stack) = slot
                && stack.item_id == id {
                    total += stack.count;
                }
        }
        total
    }

    pub fn has(&self, id: &str, n: u32) -> bool {
        self.count(id) >= n
    }

    /// True if Add would take all of this count without overflow. Does not modify anything.
    pub fn can_add(&self, id: &str, count: u32) -> bool {
        self.can_add_with(id, count, &self.registry)
    }

    /// True if Add would take all of this count using an explicit ItemRegistry.
    pub fn can_add_with(&self, id: &str, count: u32, registry: &ItemRegistry) -> bool {
        if count == 0 {
            return true;
        }
        let Some(def) = registry.get(id) else { return false; };
        let mut capacity = 0;
        for slot in &self.slots {
            match slot {
                None => capacity += def.max_stack,
                Some(stack) if stack.item_id == id => {
                    capacity += def.max_stack.saturating_sub(stack.count);
                }
                _ => {}
            }
        }
        count <= capacity.min(self.max_fit_by_weight_with(def, registry))
    }

    /// How many of this item the weight limit still allows.
    pub fn max_fit_by_weight(&self, def: &ItemDef) -> u32 {
        self.max_fit_by_weight_with(def, &self.registry)
    }

    pub fn max_fit_by_weight_with(&self, def: &ItemDef, registry: &ItemRegistry) -> u32 {
        if def.weight <= 0.0 {
            return u32::MAX;
        }
        let room = self.max_weight - self.total_weight_with(registry) + WEIGHT_EPSILON;
        if room <= 0.0 { 0 } else { (room / def.weight).floor() as u32 }
    }

    /// Adds items, filling existing stacks first, then empty slots. Returns leftover count.
    pub fn add(&mut self, id: &str, count: u32) -> u32 {
        let registry = Arc::clone(&self.registry);
        self.add_with_registry(id, count, &registry)
    }

    /// Adds items using an explicit ItemRegistry. Returns leftover.
    pub fn add_with_registry(&mut self, id: &str, count: u32, registry: &ItemRegistry) -> u32 {
        let leftover = self.fill_stacks(id, count, registry);
        if leftover < count {
            self.raise_changed();
        }
        leftover
    }

    fn fill_stacks(&mut self, id: &str, count: u32, registry: &ItemRegistry) -> u32 {
        if count == 0 {
            return 0;
        }
        let Some(def) = registry.get(id) else { return count; };
        let budget = count.min(self.max_fit_by_weight_with(def, registry));
        let mut left = budget;

        // Fill existing stacks first
        for slot in &mut self.slots {
            if left == 0 { break; }
            if let Some(stack) = slot
                && stack.item_id == id && stack.count < def.max_stack {
                    let take = left.min(def.max_stack - stack.count);
                    stack.count += take;
                    left -= take;
                }
        }
        // Fill empty slots
        for slot in &mut self.slots {
            if left == 0 { break; }
            if slot.is_none() {
                let take = left.min(def.max_stack);
                *slot = Some(ItemStack::new(id, take));
                left -= take;
            }
        }

        count - (budget - left)
    }

    /// All or nothing: removes from the last stacks first. False and no change if there are not enough.
    pub fn remove(&mut self, id: &str, count: u32) -> bool {
        if count == 0 || self.count(id) < count {
            return false;
        }

        let mut left = count;
        for i in (0..self.slots.len()).rev() {
            if left == 0 { break; }
            if let Some(stack) = &mut self.slots[i]
                && stack.item_id == id {
                    let take = left.min(stack.count);
                    left -= take;
                    if stack.count > take {
                        stack.count -= take;
                    } else {
                        self.slots[i] = None;
                    }
                }
        }
        self.raise_changed();
        true
    }

    /// Empties one slot and returns what was in it, or None if it was empty.
    pub fn take_slot(&mut self, index: usize) -> Option<ItemStack> {
        if index >= self.slots.len() {
            return None;
        }
        let taken = self.slots[index].take();
        if taken.is_some() {
            self.raise_changed();
        }
        taken
    }

    /// Removes items from one slot only. Count 0 takes the whole stack. Returns how many were removed.
    pub fn remove_from_slot(&mut self, index: usize, count: u32) -> u32 {
        if index >= self.slots.len() {
            return 0;
        }
        let Some(stack) = &mut self.slots[index] else { return 0; };
        let take = if count == 0 { stack.count } else { count.min(stack.count) };
        if stack.count > take {
            stack.count -= take;
        } else {
            self.slots[index] = None;
        }
        self.raise_changed();
        take
    }

    /// Swaps two slots. If both hold the same item and there is room, the second stack merges
    /// into the first, up to maxStack.
    pub fn swap_slots(&mut self, a: usize, b: usize) {
        if a >= self.slots.len() || b >= self.slots.len() || a == b {
            return;
        }

        if let (Some(first), Some(second)) = (&self.slots[a], &self.slots[b])
            && first.item_id == second.item_id
                && let Some(def) = self.registry.get(&first.item_id) {
                    let room = def.max_stack.saturating_sub(first.count);
                    let move_count = room.min(second.count);
                    if move_count > 0 {
                        let new_first_count = first.count + move_count;
                        let new_second_count = second.count - move_count;
                        let item_id = first.item_id.clone();
                        self.slots[a] = Some(ItemStack::new(&item_id, new_first_count));
                        self.slots[b] = if new_second_count > 0 {
                            Some(ItemStack::new(&item_id, new_second_count))
                        } else {
                            None
                        };
                        self.raise_changed();
                        return;
                    }
                }

        self.slots.swap(a, b);
        self.raise_changed();
    }

    /// Empties every slot.
    pub fn clear(&mut self) {
        self.slots.fill(None);
        self.raise_changed();
    }

    pub fn is_changed(&self) -> bool { self.changed }
    pub fn clear_changed(&mut self) { self.changed = false; }
    pub fn set_changed(&mut self) { self.raise_changed(); }

    /// Drains all accumulated inventory events and resets the changed flag.
    pub fn drain_events(&mut self) -> Vec<InventoryEvent> {
        self.changed = false;
        std::mem::take(&mut self.events)
    }

    fn raise_changed(&mut self) {
        self.changed = true;
        self.events.push(InventoryEvent::Changed);
    }

    /// Saves the slots into an InventorySaveData structure.
    pub fn save(&self) -> InventorySaveData {
        InventorySaveData::write_from_slots(&self.slots)
    }

    /// Replaces the contents with saved stacks. Unknown items or invalid slots are skipped.
    pub fn load_save(&mut self, save: &InventorySaveData) {
        save.read_into_inventory(self);
    }
}
