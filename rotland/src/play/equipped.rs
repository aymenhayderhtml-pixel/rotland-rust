//! The item in hand: its stack, attachments, condition, wear and the weapon stats it gives.

use super::PlayState;
use game::WeaponDef;

use super::HandChange;

impl PlayState {
    /// Inventory slot index of the currently equipped hotbar/bag item, if any.
    pub(super) fn equipped_slot_index(&self) -> Option<usize> {
        let id = self.hotbar.equipped_item_id()?;
        let hb = self.hotbar.slot_count().min(self.inventory.slot_count());
        (0..hb)
            .find(|&i| self.inventory.slot(i).is_some_and(|s| s.item_id == id))
            .or_else(|| {
                (hb..self.inventory.slot_count())
                    .find(|&i| self.inventory.slot(i).is_some_and(|s| s.item_id == id))
            })
    }

    /// Currently equipped inventory stack, if any.
    pub(super) fn equipped_stack(&self) -> Option<&game::items::ItemStack> {
        self.equipped_slot_index()
            .and_then(|i| self.inventory.slot(i))
    }

    /// Mutable reference to the currently equipped inventory stack, if any.
    pub(super) fn equipped_stack_mut(&mut self) -> Option<&mut game::items::ItemStack> {
        let idx = self.equipped_slot_index()?;
        self.inventory.slots_mut().get_mut(idx)?.as_mut()
    }

    /// Attachments fitted to the currently equipped item stack.
    pub fn equipped_mods(&self) -> &[String] {
        self.equipped_stack().map_or(&[], |s| s.mods.as_slice())
    }

    /// Condition (`0.0..=1.0`) of the currently equipped item stack (`1.0` when new or none).
    pub fn equipped_condition(&self) -> f32 {
        self.equipped_stack().map_or(1.0, |s| s.condition())
    }

    /// Computes effective weapon stats using the currently equipped stack's attachments.
    pub(super) fn effective_weapon(&self, base: &WeaponDef) -> WeaponDef {
        game::effective_stats(base, self.equipped_mods(), &self.attachments)
    }

    /// Applies one melee hit's wear to the currently equipped weapon stack.
    pub fn apply_equipped_hit_wear(&mut self) {
        let wear_cfg = self.wear_config.clone();
        if let Some(stack) = self.equipped_stack_mut() {
            game::apply_hit_wear(stack, &wear_cfg);
            self.inventory.set_changed();
        }
    }

    /// Applies `shots` fired shots of wear to the currently equipped weapon stack.
    pub fn apply_equipped_shot_wear(&mut self, shots: u32) {
        if shots == 0 {
            return;
        }
        let wear_cfg = self.wear_config.clone();
        if let Some(stack) = self.equipped_stack_mut() {
            for _ in 0..shots {
                game::apply_shot_wear(stack, &wear_cfg);
            }
            self.inventory.set_changed();
        }
    }

    /// Refreshes `self.gun`, `shooting` mag capacity, and `combat` weapon after fitting/removing mods.
    pub fn refresh_equipped_stats(&mut self, combat: Option<&mut crate::combat::Combat>) {
        let eff = self.equipped_weapon();
        let is_gun = eff.as_ref().is_some_and(|d| d.is_gun());
        self.gun = if is_gun { eff.clone() } else { None };
        if let Some(g) = &self.gun {
            self.shooting.sync_mag_cap(&g.id, g.magazine_size);
        }
        if let Some(c) = combat {
            c.set_weapon(if is_gun { None } else { eff });
        }
    }

    /// Returns the currently equipped weapon definition with fitted attachments applied, or None.
    pub fn equipped_weapon(&self) -> Option<game::weapons::WeaponDef> {
        let base = self
            .hotbar
            .equipped_item_id()
            .and_then(|id| self.items.get(id).and_then(|d| d.weapon(&self.weapons)).or_else(|| self.weapons.get(id)))?;
        Some(self.effective_weapon(base))
    }

    /// Hand change representing the current hotbar equipped item (a holdable tool or a weapon).
    pub fn equipped_hand_change(&self) -> HandChange {
        match self.hotbar.equipped_item_id() {
            Some(id) if self.items.get(id).is_some_and(|d| d.holdable) => HandChange::Tool(id.to_string()),
            _ => HandChange::Weapon(self.equipped_weapon()),
        }
    }
}
