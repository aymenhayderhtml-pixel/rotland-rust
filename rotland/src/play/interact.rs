//! The E key: opening crates and picking up items in front of the player.

use super::PlayState;
use glam::Vec3;
use world::LootSpot;

use super::TOAST_SECONDS;
use crate::crates::{interact, interact_prompt, InteractResult};

impl PlayState {
    /// E: open the crate or pick up the item in front of the player; also refreshes the HUD prompt.
    pub fn interact_tick(&mut self, dt: f32, pressed: bool, player: Vec3, facing: Vec3, crates: &[LootSpot]) {
        if let Some((_, t)) = self.toast.as_mut() {
            *t -= dt;
        }
        if self.toast.as_ref().is_some_and(|(_, t)| *t <= 0.0) {
            self.toast = None;
        }
        self.picked_up = false;
        if pressed {
            let result = interact(
                player,
                facing,
                &mut self.inventory,
                &mut self.loot_state,
                crates,
                &mut self.world_items,
                self.world_seed,
                &self.loot_tables,
                &self.items,
                self.loot_multiplier,
            );
            let text = match result {
                InteractResult::OpenedCrate { toast, cash, items, leftovers, .. } => {
                    // Crate cash goes to the wallet; opening one counts for loot bounties
                    self.camp.wallet.add(cash);
                    self.camp.bounties.on_crate_looted();
                    self.picked_up = true;
                    self.anim_cue = Some(humanoid::ActionCue::OpenCrate);
                    for stack in items {
                        let leftover = leftovers.iter().find(|l| l.item_id == stack.item_id).map_or(0, |l| l.count);
                        let added = stack.count.saturating_sub(leftover);
                        if added > 0 {
                            self.pending_pickups.push((stack.item_id, added));
                        }
                    }
                    Some(toast)
                }
                InteractResult::PickedUpItem { toast, item_id, count, .. } => {
                    self.picked_up = true;
                    self.anim_cue = Some(humanoid::ActionCue::PickUp);
                    if count > 0 {
                        self.pending_pickups.push((item_id, count));
                    }
                    toast
                }
                InteractResult::None => None,
            };
            if let Some(t) = text {
                self.toast = Some((t, TOAST_SECONDS));
            }
        }
        self.prompt = interact_prompt(player, facing, crates, &self.world_items, &self.loot_state, Some(&self.items));
    }
}
