//! Player shooting and gun handling (port of scripts/Combat/GunHandler.cs and WeaponHolder.cs).
//!
//! Manages per-gun magazine states surviving weapon switches, ammo consumption from the inventory,
//! firing cadence (semi/auto), raycast spread, hit resolution against zombies, reload timing,
//! out-of-ammo bash routing, and camera aim field of view.
mod aim;
mod events;
mod targets;
mod update;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_helpers;
#[cfg(test)]
mod tests_auto;
#[cfg(test)]
mod tests_gun;

pub use aim::{aim_fov, FIRST_PERSON_AIM_FOV, THIRD_PERSON_AIM_FOV};
pub use events::{ShootEvents, ShootInput};
pub use targets::TargetWorld;

use std::collections::HashMap;

use game::guns::Gun;
use game::guns_fire::GunRng;

/// The player's shooting state: owns magazine counts for all guns used, and the spread RNG.
pub struct Shooting {
    guns: HashMap<String, Gun>,
    shot_cooldowns: HashMap<String, f32>,
    rng: GunRng,
    active_weapon: Option<String>,
}

impl Default for Shooting {
    fn default() -> Self {
        Self::new(0x9E37_79B9_7F4A_7C15)
    }
}

impl Shooting {
    pub fn new(seed: u64) -> Self {
        Self {
            guns: HashMap::new(),
            shot_cooldowns: HashMap::new(),
            rng: GunRng::new(seed),
            active_weapon: None,
        }
    }

    /// Clamps a cached magazine if an attachment removal reduced `max_mag`.
    pub fn sync_mag_cap(&mut self, weapon_id: &str, max_mag: u32) {
        if let Some(gun) = self.guns.get_mut(weapon_id)
            && !gun.is_reloading()
            && gun.rounds() > max_mag
        {
            *gun = Gun::new(max_mag);
        }
    }

    /// True while the gun in hand is reloading (feeds first-person motion).
    pub fn is_reloading(&self) -> bool {
        self.active_weapon
            .as_deref()
            .and_then(|id| self.guns.get(id))
            .is_some_and(Gun::is_reloading)
    }

    /// Read-only access to a gun's state.
    #[cfg(test)]
    pub fn gun(&self, weapon_id: &str) -> Option<&Gun> {
        self.guns.get(weapon_id)
    }


    /// Sets the rounds in a gun's magazine (test hook and save loading).
    #[cfg(test)]
    pub fn set_magazine(&mut self, weapon_id: &str, rounds: u32) {
        if let Some(gun) = self.guns.get_mut(weapon_id) {
            *gun = Gun::new(rounds);
        } else {
            self.guns.insert(weapon_id.to_string(), Gun::new(rounds));
        }
    }
}
