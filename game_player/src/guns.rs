//! Gun state for one held gun: rounds in the magazine, fire cooldown and reloading.
//! Port of the per-gun parts of scripts/Combat/GunHandler.cs. Ammo in the bag is the caller's
//! (the inventory); this module only reports how many rounds moved.

use game_core::weapons::WeaponDef;

/// A reload finished: the magazine took `rounds_taken` rounds. The caller removes that many from the bag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReloadDone {
    pub rounds_taken: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Reload {
    time_left: f32,
    rounds_taken: u32,
}

/// One held gun. The caller keeps one per weapon id so magazines survive switching weapons (as in C#).
#[derive(Clone, Debug, PartialEq)]
pub struct Gun {
    rounds: u32,
    cooldown: f32,
    reload: Option<Reload>,
}

impl Gun {
    /// A gun with this many rounds in the magazine, ready to fire.
    pub fn new(rounds: u32) -> Self {
        Self {
            rounds,
            cooldown: 0.0,
            reload: None,
        }
    }

    pub fn rounds(&self) -> u32 {
        self.rounds
    }

    /// True when the magazine is empty. The caller may then start a reload (C# does this on a dry fire).
    pub fn is_empty(&self) -> bool {
        self.rounds == 0
    }

    pub fn is_reloading(&self) -> bool {
        self.reload.is_some()
    }

    /// Fires one round if the trigger asks for it. Returns false when the gun is cooling down,
    /// reloading, empty, or the trigger is not pressed.
    /// Automatic guns fire while `held`; semi-automatic guns only on the press (`pressed`).
    /// The caller then calls `fire_rays` for the shot's direction.
    pub fn try_fire(&mut self, def: &WeaponDef, pressed: bool, held: bool) -> bool {
        if self.reload.is_some() || self.cooldown > 0.0 {
            return false;
        }
        let wants_fire = if def.auto { pressed || held } else { pressed };
        if !wants_fire || self.rounds == 0 {
            return false;
        }
        self.rounds -= 1;
        self.cooldown = def.fire_interval;
        true
    }

    /// Starts a reload from the bag's ammo. Fails if already reloading, the magazine is full, or the bag
    /// has none. The rounds that will move are fixed now, so the caller should not change the bag
    /// during the reload.
    pub fn start_reload(&mut self, def: &WeaponDef, ammo_in_bag: u32) -> bool {
        if self.reload.is_some() || self.rounds >= def.magazine_size || ammo_in_bag == 0 {
            return false;
        }
        let room = def.magazine_size - self.rounds;
        self.reload = Some(Reload {
            time_left: def.reload_time,
            rounds_taken: room.min(ammo_in_bag),
        });
        true
    }

    /// Ends a reload early without moving any ammo (used when the player switches weapon).
    pub fn cancel_reload(&mut self) {
        self.reload = None;
    }

    /// Advances the cooldown and any reload by `dt` seconds. Returns the reload when it finishes.
    pub fn update(&mut self, dt: f32) -> Option<ReloadDone> {
        self.cooldown = (self.cooldown - dt).max(0.0);
        let reload = self.reload.as_mut()?;
        reload.time_left -= dt;
        if reload.time_left > 0.0 {
            return None;
        }
        let taken = reload.rounds_taken;
        self.reload = None;
        self.rounds += taken;
        Some(ReloadDone { rounds_taken: taken })
    }
}
