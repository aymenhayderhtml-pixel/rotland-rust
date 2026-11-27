//! Using items from the hotbar and the bag screen (food, drink, bandages, special items).

use super::PlayState;
use glam::Vec3;

use super::{HandChange, USE_COOLDOWN};

impl PlayState {
    /// Eats, drinks or uses a bandage: one item is used up (Godot ItemUse.TryUse).
    pub(super) fn use_item(&mut self, item_id: &str) -> Option<HandChange> {
        match item_id {
            "night_vision" => return Some(HandChange::ToggleNightVision),
            "adrenaline_shot" => return Some(HandChange::UseAdrenaline),
            "surgery_kit" => {
                return (!self.health_full && self.inventory.has("surgery_kit", 1))
                    .then_some(HandChange::StartSurgery);
            }
            "car_parts" => return Some(HandChange::RepairCarParts),
            "antibiotics" => {
                if self.use_cooldown <= 0.0 && self.inventory.remove("antibiotics", 1) {
                    self.boosts.start_antibiotics(&self.boosts_cfg.antibiotics);
                    self.hotbar.on_inventory_changed(&self.inventory);
                    self.use_cooldown = USE_COOLDOWN;
                    self.show_toast("Antibiotics taken".into());
                }
                return None;
            }
            "gun_parts" => {
                let has_gun = self.gun.is_some();
                let has_parts = self.inventory.has("gun_parts", 1);
                if has_gun && has_parts && let Some(stack) = self.equipped_stack_mut()
                    && stack.condition() < 1.0
                    && game::repair_with_parts(stack, game::GUN_PARTS_REPAIR_AMOUNT)
                {
                    self.inventory.remove("gun_parts", 1);
                    self.inventory.set_changed();
                    self.hotbar.on_inventory_changed(&self.inventory);
                    self.show_toast("Gun repaired (+50%)".into());
                }
                return None;
            }
            _ => {}
        }
        if item_id == "hand_warmer" {
            // Tool, not food: the warmth system starts its timer and removes the item (R32-05).
            if self.use_cooldown <= 0.0 && self.inventory.count(item_id) > 0 {
                self.hand_warmer_pending = true;
                self.use_cooldown = USE_COOLDOWN;
            }
            return None;
        }
        let def = self.items.get(item_id)?.clone();
        use game::items::ItemCategory;
        if self.use_cooldown > 0.0 || !matches!(def.category, ItemCategory::Food | ItemCategory::Drink | ItemCategory::Medical) {
            return None;
        }
        if def.category == ItemCategory::Medical && self.health_full {
            return None;
        }
        if self.inventory.count(item_id) == 0 {
            return None;
        }
        // Cooking skill bonus comes with the skills wiring; 1.0 until then
        let health = game::apply_item_effects(&def.effects, &mut self.hunger, &mut self.thirst, 1.0);
        self.inventory.remove(item_id, 1);
        self.hotbar.on_inventory_changed(&self.inventory);
        self.use_cooldown = USE_COOLDOWN;
        self.anim_cue = Some(match def.category {
            ItemCategory::Food => humanoid::ActionCue::Eat,
            ItemCategory::Drink => humanoid::ActionCue::Drink,
            _ => humanoid::ActionCue::Bandage,
        });
        (health != 0.0).then_some(HandChange::Health(health))
    }

    /// A button on the bag screen (hub_ui). Returns a hand change and a line for the screen's status.
    /// Dropped items land at `feet` and can be picked up again with E.
    pub fn bag_action(&mut self, action: &crate::hub_ui::HubAction, split: u32, feet: Vec3) -> (Option<HandChange>, Option<String>) {
        use crate::hub_ui::HubAction;
        let name = |s: &Self, slot: usize| s.inventory.slot(slot).map(|st| s.items.get(&st.item_id).map_or(st.item_id.clone(), |d| d.name.clone()));
        let out = match action {
            HubAction::MoveSlot { from, to } => {
                self.inventory.swap_slots(*from, *to);
                (None, None)
            }
            HubAction::AssignHotbar { slot, key } if (1..=self.hotbar.slot_count()).contains(key) => {
                self.inventory.swap_slots(*slot, key - 1);
                (None, Some(format!("On key {key}")))
            }
            HubAction::UseItem { slot } => {
                let Some(id) = self.inventory.slot(*slot).map(|s| s.item_id.clone()) else { return (None, None) };
                let label = name(self, *slot);
                if id == "flashlight" {
                    (Some(HandChange::ToggleFlashlight), None)
                } else if id == "night_vision" {
                    (Some(HandChange::ToggleNightVision), None)
                } else if id == "campfire_kit" {
                    (Some(HandChange::PlaceCampfire), None)
                } else if self.items.get(&id).is_some_and(|d| d.holdable) {
                    // A holdable tool (fishing rod) goes in hand instead of being used up
                    if self.hotbar.toggle_equip_item(&id, &self.inventory, &self.items) {
                        (Some(self.equipped_hand_change()), None)
                    } else {
                        (None, Some("Can't hold that".into()))
                    }
                } else {
                    let before = self.inventory.count(&id);
                    let change = self.use_item(&id);
                    let used = self.inventory.count(&id) < before;
                    (change, Some(if used { format!("Used {}", label.unwrap_or(id)) } else { "Can't use that now".into() }))
                }
            }
            HubAction::EquipItem { slot } => {
                let Some(id) = self.inventory.slot(*slot).map(|s| s.item_id.clone()) else { return (None, None) };
                if self.hotbar.toggle_equip_item(&id, &self.inventory, &self.items) {
                    (Some(self.equipped_hand_change()), None)
                } else {
                    (None, Some("Not a weapon".into()))
                }
            }
            HubAction::DropItem { slot, count } => {
                let Some(stack) = self.inventory.slot(*slot).cloned() else { return (None, None) };
                let n = if *count == 0 { stack.count } else { (*count).min(stack.count) };
                let label = name(self, *slot).unwrap_or_default();
                let removed = self.inventory.remove_from_slot(*slot, n);
                if removed > 0 {
                    self.world_items.add(stack.item_id, removed, feet);
                }
                (None, Some(format!("Dropped {removed} {label}")))
            }
            HubAction::SplitStack { slot } => {
                let Some(stack) = self.inventory.slot(*slot).cloned() else { return (None, None) };
                let n = split.clamp(1, stack.count.saturating_sub(1).max(1));
                let free = self.inventory.slots().iter().position(Option::is_none);
                match free {
                    Some(free) if stack.count > 1 => {
                        self.inventory.remove_from_slot(*slot, n);
                        self.inventory.slots_mut()[free] = Some(game::items::ItemStack::new(stack.item_id, n));
                        self.inventory.set_changed();
                        (None, Some(format!("Split off {n}")))
                    }
                    _ => (None, Some("No free slot".into())),
                }
            }
            _ => (None, None),
        };
        self.hotbar.on_inventory_changed(&self.inventory);
        out
    }
}
