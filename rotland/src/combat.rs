//! The player's side of the fight: health, the melee swing on the left mouse button, being bitten,
//! dying and coming back at the camp. Zombies live in zombies.rs.

use game::melee::{Attacker, MeleeSwing};
use game::zombie::Noise;
use game::{DamageInfo, Health, Player, WeaponDef, WeaponRegistry};
use glam::Vec3;
use physics::PhysicsWorld;

use crate::zombies::Zombies;

/// The player's health (Player.tscn's Health node).
const PLAYER_HEALTH: f32 = 100.0;
/// Seconds on the ground before waking up at the camp.
const RESPAWN_SECONDS: f32 = 4.0;
const PLAYER_ID: u64 = 1;

pub struct Combat {
    pub health: Health,
    swing: MeleeSwing,
    weapon: Option<WeaponDef>,
    pub dead_for: Option<f32>,
    pub kills: u32,
    /// Sounds made this frame (swings), heard by zombies next frame.
    pub noises: Vec<Noise>,
}

/// What the game should do after a combat frame.
#[derive(Default)]
pub struct CombatEvents {
    /// The swing started this frame: play the attack clip.
    pub swing_started: Option<String>,
    pub bitten: bool,
    pub died: bool,
    /// Time to wake up at the camp.
    pub respawn: bool,
}

impl Combat {
    pub fn new(weapon_id: &str) -> Self {
        let weapon = WeaponRegistry::load(&paths::repo_path("data/weapons.json")).ok().and_then(|r| r.get(weapon_id).cloned());
        if weapon.is_none() {
            log::warn!("weapon {weapon_id} not found in data/weapons.json");
        }
        Self { health: Health::new(PLAYER_HEALTH), swing: MeleeSwing::new(), weapon, dead_for: None, kills: 0, noises: Vec::new() }
    }

    pub fn is_dead(&self) -> bool {
        self.dead_for.is_some()
    }

    /// One frame: start a swing if asked, resolve hits on zombies, take bites, count down a death.
    pub fn update(&mut self, dt: f32, attack_pressed: bool, player: &mut Player, zombies: &mut Zombies, physics: &PhysicsWorld) -> CombatEvents {
        let mut ev = CombatEvents::default();
        self.noises.clear();
        self.health.tick(dt);

        if let Some(t) = self.dead_for.as_mut() {
            *t += dt;
            if *t >= RESPAWN_SECONDS {
                self.dead_for = None;
                self.health.revive(None);
                ev.respawn = true;
            }
            return ev;
        }

        if let Some(weapon) = &self.weapon {
            if attack_pressed && self.swing.try_start(weapon, &mut player.stamina) {
                ev.swing_started = weapon.anim.attack.clone();
            }
            let facing = Vec3::new(-player.yaw.sin(), 0.0, -player.yaw.cos());
            let attacker = Attacker { id: PLAYER_ID, position: player.position, facing, eye: player.position + Vec3::Y * 1.5 };
            for hit in self.swing.update(dt, attacker, &zombies.targets(), physics) {
                if zombies.hit(hit.target, hit.damage, hit.knockback) {
                    self.kills += 1;
                }
            }
            if let Some(noise) = self.swing.take_noise() {
                self.noises.push(noise);
            }
        }
        ev
    }

    /// Kills the player now (test option --die-at). Returns true if they were alive.
    pub fn kill(&mut self) -> bool {
        if self.is_dead() {
            return false;
        }
        self.health.kill();
        self.dead_for = Some(0.0);
        self.swing.cancel();
        true
    }

    /// Bites from this frame's zombies. Returns events (bitten / died).
    pub fn take_bites(&mut self, damage: &[f32]) -> CombatEvents {
        let mut ev = CombatEvents::default();
        if self.is_dead() {
            return ev;
        }
        for &amount in damage {
            let r = self.health.take_damage(DamageInfo::new(amount));
            ev.bitten |= r.dealt > 0.0;
            if r.died_now {
                ev.died = true;
                self.dead_for = Some(0.0);
                self.swing.cancel();
                break;
            }
        }
        ev
    }
}
