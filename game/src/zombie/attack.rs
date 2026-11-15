//! The bite timing (port of the timing and reach rules in ZombieAttack.cs). No animation here.

use super::defs::ZombieDef;

/// Extra reach past `attack_range`, so a bite does not miss by a few centimetres.
pub const BITE_REACH_MARGIN: f32 = 0.3;

/// A bite that landed. The caller applies the damage to the target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttackHit {
    pub damage: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Idle,
    /// Counting down to the bite.
    Windup,
    /// After the bite: the zombie cannot start another attack until this ends.
    Cooldown,
}

/// One zombie's bite. `start` begins an attack; `update` times it and reports a bite.
#[derive(Clone, Debug)]
pub struct ZombieAttack {
    windup: f32,
    cooldown: f32,
    reach: f32,
    damage: f32,
    phase: Phase,
    timer: f32,
}

impl ZombieAttack {
    pub fn new(def: &ZombieDef) -> Self {
        Self {
            windup: def.attack_windup,
            cooldown: def.attack_cooldown,
            reach: def.attack_range + BITE_REACH_MARGIN,
            damage: def.attack_damage,
            phase: Phase::Idle,
            timer: 0.0,
        }
    }

    /// Starts a new attack. The bite lands once the windup time has passed.
    pub fn start(&mut self) {
        self.phase = Phase::Windup;
        self.timer = self.windup;
    }

    /// True from `start` until the cooldown after the bite has ended.
    pub fn is_active(&self) -> bool {
        self.phase != Phase::Idle
    }

    /// Advance by `dt` seconds. `distance_to_target` is the horizontal distance in metres (height is
    /// ignored, as in the Godot code). Returns a hit only on the frame the windup ends with the target
    /// still in reach; a bite that misses still starts the cooldown.
    pub fn update(&mut self, dt: f32, distance_to_target: f32) -> Option<AttackHit> {
        match self.phase {
            Phase::Idle => None,
            Phase::Windup => {
                self.timer -= dt;
                if self.timer > 0.0 {
                    return None;
                }
                self.phase = Phase::Cooldown;
                self.timer = self.cooldown;
                (distance_to_target <= self.reach).then_some(AttackHit {
                    damage: self.damage,
                })
            }
            Phase::Cooldown => {
                self.timer -= dt;
                if self.timer <= 0.0 {
                    self.phase = Phase::Idle;
                }
                None
            }
        }
    }
}
