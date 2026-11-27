//! Small reads and writes on the play state: HUD prompt and toast, the map, and save data.

use super::PlayState;
use glam::Vec3;

use super::TOAST_SECONDS;

impl PlayState {
    /// The interaction hint at the bottom of the screen ("[E] Trade").
    pub fn set_prompt(&mut self, text: String) {
        self.prompt = Some(text);
    }

    /// True while the gun in hand is reloading.
    pub fn gun_reloading(&self) -> bool {
        self.shooting.is_reloading()
    }

    pub fn spawner_hard_cap(&self) -> usize {
        if self.spawner.config.hard_cap > 0 {
            self.spawner.config.hard_cap
        } else {
            60 // data: spawning.json hardCap
        }
    }

    pub fn has_prompt(&self) -> bool {
        self.prompt.as_ref().is_some_and(|p| !p.is_empty())
    }

    pub fn brute_at_army_line(&self) -> bool {
        self.spawner
            .config
            .metro
            .as_ref()
            .is_some_and(|m| m.brute_at_army_line())
    }

    /// A message in the middle of the screen for a few seconds.
    /// Notes the POI buildings within reveal range on the map, dimmed once all their crates are looted.
    pub fn note_buildings<'a>(
        &mut self,
        player: Vec3,
        chunks: impl Iterator<Item = (&'a [world::PoiInstance], &'a [world::LootSpot])>,
        config: &world::PoiConfigData,
    ) {
        let looted = self.loot_state.all_looted();
        for (pois, crates) in chunks {
            crate::poi_links::note_nearby_pois(&mut self.map, player, pois, crates, config, |id| looted.contains(id));
        }
    }

    /// Puts items on the ground as a world item (felled wood).
    pub fn drop_on_ground(&mut self, item_id: &str, count: u32, at: Vec3) {
        self.world_items.add(item_id, count, at);
    }

    pub fn show_toast(&mut self, text: String) {
        self.toast = Some((text, TOAST_SECONDS));
    }

    /// Takes out the first hotbar item at the start of a game, like Godot's starting bat.
    /// The backpack's stacks and the looted crates, for a save file.
    pub fn save_parts(&self) -> (game::items::InventorySaveData, Vec<String>) {
        let mut looted: Vec<String> = self.loot_state.all_looted().iter().cloned().collect();
        looted.sort();
        (game::items::InventorySaveData::write_from_slots(self.inventory.slots()), looted)
    }

    /// Puts a save file's backpack and looted crates back.
    pub fn load_parts(&mut self, stacks: &game::items::InventorySaveData, looted: &[String]) {
        stacks.read_into_inventory(&mut self.inventory);
        self.hotbar.on_inventory_changed(&self.inventory);
        self.loot_state.clear();
        for id in looted {
            self.loot_state.mark_looted(id.clone());
        }
    }
}
