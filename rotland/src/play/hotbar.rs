//! Hotbar keys: taking an item out in hand or putting it away.

use super::PlayState;
use engine::Input;
use game::items::HotbarAction;
use winit::keyboard::KeyCode;

use super::HandChange;

impl PlayState {
    pub fn equip_first(&mut self) -> Option<HandChange> {
        self.press(1)
    }

    /// Hotbar keys 1-6: take out or put away the item in that slot.
    pub fn hotbar_keys(&mut self, input: &Input, keys: &[KeyCode; 6]) -> Option<HandChange> {
        let key = keys.iter().position(|k| input.pressed(*k))? + 1;
        self.press(key)
    }

    pub fn press(&mut self, key: usize) -> Option<HandChange> {
        match self.hotbar.press(key, &self.inventory, &self.items) {
            HotbarAction::EquipWeapon { weapon_id, item_id } if weapon_id.is_empty() => Some(HandChange::Tool(item_id)),
            HotbarAction::EquipWeapon { weapon_id, .. } => {
                let def = self.weapons.get(&weapon_id).map(|w| self.effective_weapon(w));
                Some(HandChange::Weapon(def))
            }
            HotbarAction::UnequipWeapon { .. } => Some(HandChange::Weapon(None)),
            // Godot's ItemUse: using the flashlight item switches it on and off
            HotbarAction::UseItem { item_id } if item_id == "flashlight" => Some(HandChange::ToggleFlashlight),
            HotbarAction::UseItem { item_id } if item_id == "campfire_kit" => Some(HandChange::PlaceCampfire),
            HotbarAction::UseItem { item_id } => self.use_item(&item_id),
            _ => None,
        }
    }
}
