//! The HUD numbers for one frame.

use super::PlayState;
use crate::hud::{HotbarSlot, HudState};

impl PlayState {
    /// The HUD's numbers for this frame.
    #[allow(clippy::too_many_arguments)]
    pub fn hud(&self, health: f32, max_health: f32, stamina: f32, max_stamina: f32, hour: f32, yaw: f32, fps: f32, dead: bool) -> HudState {
        let selected = self.hotbar.selected_item();
        let hotbar = (1..=self.hotbar.slot_count())
            .map(|key| match self.hotbar.slot_entry(key, &self.inventory) {
                Some(stack) => HotbarSlot {
                    label: self.items.get(&stack.item_id).map_or(stack.item_id.clone(), |d| d.hotbar_label().to_string()),
                    count: stack.count,
                    selected: selected == Some(stack.item_id.as_str()),
                },
                None => HotbarSlot { label: String::new(), count: 0, selected: false },
            })
            .collect();
        let hotbar_item_ids = (1..=self.hotbar.slot_count())
            .map(|key| self.hotbar.slot_entry(key, &self.inventory).map(|st| st.item_id.clone()))
            .collect();
        let hotbar_conditions = (1..=self.hotbar.slot_count())
            .map(|key| {
                self.hotbar.slot_entry(key, &self.inventory).and_then(|st| {
                    self.items
                        .get(&st.item_id)
                        .filter(|d| d.is_weapon() && self.wear_config.weapons.contains_key(&d.weapon_id))
                        .map(|_| st.condition())
                })
            })
            .collect();
        HudState {
            health,
            max_health,
            stamina,
            max_stamina,
            hour,
            day: 1,
            heading_yaw: yaw,
            hotbar,
            ammo: self.gun.as_ref().map(|_| self.last_shot.ammo),
            fps,
            food: Some((self.hunger.current() / self.hunger.max().max(1.0), self.hunger.is_low())),
            water: Some((self.thirst.current() / self.thirst.max().max(1.0), self.thirst.is_low())),
            message: self.toast.as_ref().map(|(t, _)| t.clone()).or_else(|| self.prompt.clone()),
            dead,
            hotbar_item_ids: Some(hotbar_item_ids),
            hotbar_conditions: Some(hotbar_conditions),
            ..Default::default()
        }
    }
}
