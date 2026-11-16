use crate::bounties::config::{
    CLEAR_POI, DELIVER_ITEMS, HUNT_ANIMALS, KILL_AT_NIGHT, KILL_ZOMBIES, LOOT_CRATES,
};
use crate::bounties::def::BountyDef;
use crate::bounties::manager::BountyManager;

impl BountyManager {
    /// Plain call when a zombie is killed.
    // Godot: BountyManager.cs OnZombieKilled
    pub fn on_zombie_killed(&mut self, kind: &str) -> usize {
        self.on_zombie_killed_opt(Some(kind), false)
    }

    /// Plain call when a zombie is killed with night indicator.
    pub fn on_zombie_killed_night(&mut self, kind: &str, is_night: bool) -> usize {
        self.on_zombie_killed_opt(Some(kind), is_night)
    }

    /// Plain call for zombie kills with optional kind filter and night status.
    pub fn on_zombie_killed_opt(&mut self, kind: Option<&str>, is_night: bool) -> usize {
        let mut changed = 0;
        for bounty in &mut self.active {
            let counts = (bounty.bounty_type == KILL_ZOMBIES
                && (bounty.zombie.is_none() || bounty.zombie.as_deref() == kind))
                || (bounty.bounty_type == KILL_AT_NIGHT && is_night);

            if counts && Self::set_bounty_progress(bounty, bounty.progress + 1) {
                changed += 1;
            }
        }
        changed
    }

    /// Plain call when items are delivered or acquired.
    pub fn on_item_delivered(&mut self, item_id: &str, count: i32) -> usize {
        let mut changed = 0;
        for bounty in &mut self.active {
            if bounty.bounty_type == DELIVER_ITEMS
                && bounty.item.as_deref() == Some(item_id)
                && Self::set_bounty_progress(bounty, bounty.progress + count)
            {
                changed += 1;
            }
        }
        changed
    }

    /// Sets item delivery progress based on current bag inventory count.
    // Godot: BountyManager.cs UpdateProgress
    pub fn set_item_count(&mut self, item_id: &str, count: i32) -> usize {
        let mut changed = 0;
        for bounty in &mut self.active {
            if bounty.bounty_type == DELIVER_ITEMS
                && bounty.item.as_deref() == Some(item_id)
                && Self::set_bounty_progress(bounty, count)
            {
                changed += 1;
            }
        }
        changed
    }

    /// Plain call when a wild animal is killed.
    // Godot: BountyManager.cs OnAnimalKilled
    pub fn on_animal_killed(&mut self, species: &str) -> usize {
        let mut changed = 0;
        for bounty in &mut self.active {
            if bounty.bounty_type == HUNT_ANIMALS
                && (bounty.animal.is_none() || bounty.animal.as_deref() == Some(species))
                && Self::set_bounty_progress(bounty, bounty.progress + 1)
            {
                changed += 1;
            }
        }
        changed
    }

    /// Plain call when a loot crate is searched.
    // Godot: BountyManager.cs OnCrateLooted
    pub fn on_crate_looted(&mut self) -> usize {
        let mut changed = 0;
        for bounty in &mut self.active {
            if bounty.bounty_type == LOOT_CRATES
                && Self::set_bounty_progress(bounty, bounty.progress + 1)
            {
                changed += 1;
            }
        }
        changed
    }

    /// Plain call when a specific POI site is cleared of zombies.
    // Godot: BountyManager.cs UpdateProgress
    pub fn on_poi_cleared(&mut self, poi_id: &str) -> usize {
        let mut changed = 0;
        for bounty in &mut self.active {
            if bounty.bounty_type == CLEAR_POI
                && bounty.poi_id.as_deref() == Some(poi_id)
                && Self::set_bounty_progress(bounty, 1)
            {
                changed += 1;
            }
        }
        changed
    }

    /// Plain call when a building type is cleared of zombies.
    pub fn on_building_cleared(&mut self, building: &str) -> usize {
        let mut changed = 0;
        for bounty in &mut self.active {
            if bounty.bounty_type == CLEAR_POI
                && bounty.building.as_deref() == Some(building)
                && Self::set_bounty_progress(bounty, 1)
            {
                changed += 1;
            }
        }
        changed
    }

    // Godot: BountyManager.cs SetProgress
    pub(crate) fn set_bounty_progress(bounty: &mut BountyDef, value: i32) -> bool {
        let clamped = value.clamp(0, bounty.target);
        if clamped == bounty.progress {
            false
        } else {
            bounty.progress = clamped;
            true
        }
    }
}
