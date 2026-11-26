//! The player's side of the fight: health, the melee swing on the left mouse button, being bitten,
//! dying and coming back at the camp. Zombies live in zombies.rs.

#[path = "bow.rs"]
pub mod bow;
#[path = "projectiles_view.rs"]
pub mod projectiles_view;

pub use bow::BowState;

use game::melee::{Attacker, MeleeSwing, SwingState};
use game::projectiles::ProjectileSim;
use game::zombie::Noise;
use game::{Health, Player, WeaponDef, WeaponRegistry};
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
    pub(crate) swing: MeleeSwing,
    combo: game::MeleeComboState,
    weapon: Option<WeaponDef>,
    pub bash_weapon: Option<WeaponDef>,
    pub(crate) pending_swing_started: Option<String>,
    pub dead_for: Option<f32>,
    pub kills: u32,
    /// Gun shots fired (stats only).
    pub shots: u32,
    /// Sounds made this frame (swings), heard by zombies next frame.
    pub noises: Vec<Noise>,
    /// Melee hits that landed this frame (for the hit sound).
    pub hits: u32,
    pub projectiles: ProjectileSim,
    pub bow: BowState,
}

/// What the game should do after a combat frame.
#[derive(Default)]
pub struct CombatEvents {
    /// The swing started this frame: play the attack clip.
    pub swing_started: Option<String>,
    /// Play speed for the swing clip (0 = use 1.0).
    pub swing_speed: f32,
    pub bitten: bool,
    pub died: bool,
    /// Transitioned from standing to knocked down this frame.
    pub downed_now: bool,
    /// Time to wake up at the camp.
    pub respawn: bool,
    /// A swing landed this frame (hit or miss): the weapon and the flat swing direction, for trees.
    pub swung: Option<(WeaponDef, Vec3)>,
}

fn melee_moves() -> &'static game::MeleeMovesConfig {
    static CFG: std::sync::OnceLock<game::MeleeMovesConfig> = std::sync::OnceLock::new();
    CFG.get_or_init(|| game::MeleeMovesConfig::from_repo().unwrap_or_default())
}

impl Combat {
    pub fn new(weapon_id: &str) -> Self {
        let weapon = WeaponRegistry::load(&paths::repo_path("data/weapons.json")).ok().and_then(|r| r.get(weapon_id).cloned());
        if weapon.is_none() {
            log::warn!("weapon {weapon_id} not found in data/weapons.json");
        }
        Self {
            health: Health::new(PLAYER_HEALTH),
            swing: MeleeSwing::new(),
            combo: game::MeleeComboState::default(),
            weapon,
            bash_weapon: None,
            pending_swing_started: None,
            dead_for: None,
            kills: 0,
            shots: 0,
            noises: Vec::new(),
            hits: 0,
            projectiles: ProjectileSim::new(0x5052_4F4A_4543_5449),
            bow: BowState::new(),
        }
    }

    /// The weapon in hand changes (hotbar). None: nothing to swing.
    pub fn set_weapon(&mut self, weapon: Option<WeaponDef>) {
        self.swing.cancel();
        self.combo.reset();
        self.bow.cancel();
        self.bash_weapon = None;
        self.pending_swing_started = None;
        self.weapon = weapon;
    }

    /// The melee weapon in hand (first-person view model).
    pub fn weapon_def(&self) -> Option<&WeaponDef> {
        self.weapon.as_ref()
    }

    /// The melee weapon in hand (for its swing sound).
    pub fn weapon_id(&self) -> Option<&str> {
        self.weapon.as_ref().or(self.bash_weapon.as_ref()).map(|w| w.id.as_str())
    }

    pub fn is_dead(&self) -> bool {
        self.dead_for.is_some()
    }

    /// True while a melee swing or bash is winding up.
    pub fn swing_busy(&self) -> bool {
        self.swing.state() == SwingState::WindingUp
    }

    /// Current state of the melee / bash swing.
    #[cfg(test)]
    pub fn swing_state(&self) -> SwingState {
        self.swing.state()
    }

    /// One frame: start a swing if asked, resolve hits on zombies, take bites, count down a death.
    pub fn update(&mut self, dt: f32, attack_pressed: bool, player: &mut Player, zombies: &mut impl MeleeWorld, physics: &PhysicsWorld) -> CombatEvents {
        self.update_with_wear(dt, attack_pressed, player, zombies, physics, 1.0)
    }

    /// One frame with melee damage scaled by weapon condition (`melee_dmg_mult`).
    pub fn update_with_wear(&mut self, dt: f32, attack_pressed: bool, player: &mut Player, zombies: &mut impl MeleeWorld, physics: &PhysicsWorld, melee_dmg_mult: f32) -> CombatEvents {
        let mut ev = CombatEvents::default();
        self.noises.clear();
        self.hits = 0;
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

        let moves = melee_moves();
        self.combo.tick(dt);
        if let Some(weapon) = &self.weapon
            && !weapon.is_bow()
            && self.combo.wants_swing(attack_pressed, self.swing.state() == SwingState::Idle, moves)
        {
            let mut mv = self.combo.next_move(weapon, moves);
            mv.weapon.damage *= melee_dmg_mult.clamp(0.0, 1.0);
            if self.swing.try_start(&mv.weapon, &mut player.stamina) {
                self.combo.on_started(&mv, moves);
                ev.swing_started = Some(mv.clip);
                ev.swing_speed = mv.clip_speed;
            } else {
                self.combo.reset();
            }
        }

        if ev.swing_started.is_none() {
            ev.swing_started = self.pending_swing_started.take();
        }

        let current_weapon = self.weapon.as_ref().or(self.bash_weapon.as_ref());
        if current_weapon.is_some() || self.swing.state() != SwingState::Idle {
            let facing = Vec3::new(-player.yaw.sin(), 0.0, -player.yaw.cos());
            let attacker = Attacker { id: PLAYER_ID, position: player.position, facing, eye: player.position + Vec3::Y * 1.5 };
            for hit in self.swing.update(dt, attacker, &zombies.melee_targets(), physics) {
                self.hits += 1;
                if zombies.melee_hit(hit.target, hit.damage, hit.knockback) {
                    self.kills += 1;
                }
            }
            if let Some(noise) = self.swing.take_noise() {
                self.noises.push(noise);
                if self.bash_weapon.is_none()
                    && let Some(w) = self.weapon.as_ref()
                {
                    self.combo.on_resolved(self.hits > 0, &w.id, moves);
                }
                if let Some(weapon) = current_weapon {
                    ev.swung = Some((weapon.clone(), facing));
                }
            }
        }

        if self.swing.state() == SwingState::Idle {
            self.bash_weapon = None;
        }

        ev
    }

    /// Cancels any active melee swing, combo, bow draw, or gun bash.
    pub(crate) fn cancel_actions(&mut self) {
        self.swing.cancel();
        self.combo.reset();
        self.bow.cancel();
        self.bash_weapon = None;
        self.pending_swing_started = None;
    }
}

/// What a swing can hit (zombies alone, or zombies and animals: wildlife::Prey).
pub trait MeleeWorld {
    fn melee_targets(&self) -> Vec<game::melee::MeleeTarget>;
    fn melee_hit(&mut self, id: u64, damage: game::DamageInfo, knockback: Vec3) -> bool;
}

impl MeleeWorld for Zombies {
    fn melee_targets(&self) -> Vec<game::melee::MeleeTarget> {
        self.targets()
    }

    fn melee_hit(&mut self, id: u64, damage: game::DamageInfo, knockback: Vec3) -> bool {
        self.hit(id, damage, knockback)
    }
}

#[cfg(test)]
pub(crate) mod test_helpers {
    use super::MeleeWorld;
    use game::{Health, WeaponRegistry};
    use glam::{IVec2, Vec3};
    use physics::{PhysicsWorld, CHUNK_SAMPLES};

    pub(crate) struct MockZombie {
        pub id: u64,
        pub position: Vec3,
        pub health: Health,
        pub knockback: Vec3,
    }

    pub(crate) struct MockMeleeWorld {
        pub zombies: Vec<MockZombie>,
    }

    impl MeleeWorld for MockMeleeWorld {
        fn melee_targets(&self) -> Vec<game::melee::MeleeTarget> {
            self.zombies
                .iter()
                .filter(|z| !z.health.is_dead())
                .map(|z| game::melee::MeleeTarget {
                    id: z.id,
                    position: z.position,
                    radius: 0.4,
                })
                .collect()
        }

        fn melee_hit(&mut self, id: u64, damage: game::DamageInfo, knockback: Vec3) -> bool {
            let Some(z) = self.zombies.iter_mut().find(|z| z.id == id) else {
                return false;
            };
            z.knockback += knockback;
            let res = z.health.take_damage(damage);
            res.died_now
        }
    }

    pub(crate) fn flat_world() -> PhysicsWorld {
        let mut world = PhysicsWorld::new();
        let heights = vec![0.0f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
        world.add_ground_chunk(IVec2::new(0, 0), &heights);
        world
    }

    pub(crate) fn load_weapons() -> WeaponRegistry {
        WeaponRegistry::load(&paths::repo_path("data/weapons.json"))
            .expect("data/weapons.json must load")
    }
}

#[cfg(test)]
mod tests {
    use super::test_helpers::{flat_world, load_weapons, MockMeleeWorld};
    use super::*;

    #[test]
    fn bow_equipped_does_not_start_melee_swing() {
        let registry = load_weapons();
        let bow = registry.get("bow").expect("bow weapon def");

        let mut combat = Combat::new("bow");
        combat.set_weapon(Some(bow.clone()));
        assert_eq!(combat.weapon_id(), Some("bow"));

        let mut player = Player::new(Vec3::ZERO);
        let physics = flat_world();
        let mut world = MockMeleeWorld { zombies: Vec::new() };

        let ev = combat.update(0.1, true, &mut player, &mut world, &physics);
        assert_eq!(ev.swing_started, None);
        assert_eq!(combat.swing_state(), SwingState::Idle);
    }

    #[test]
    fn dagger_and_spear_swing_as_melee() {
        let registry = load_weapons();
        let dagger = registry.get("dagger").expect("dagger def");
        let spear = registry.get("spear").expect("spear def");

        let mut combat = Combat::new("dagger");
        combat.set_weapon(Some(dagger.clone()));
        let mut player = Player::new(Vec3::ZERO);
        let physics = flat_world();
        let mut world = MockMeleeWorld { zombies: Vec::new() };

        let ev1 = combat.update(0.01, true, &mut player, &mut world, &physics);
        assert!(ev1.swing_started.is_some());
        assert_eq!(combat.swing_state(), SwingState::WindingUp);

        // A fresh fighter: the dagger swing above is still winding up, so it can't swing again yet
        let mut combat = Combat::new("spear");
        combat.set_weapon(Some(spear.clone()));
        let ev2 = combat.update(0.01, true, &mut player, &mut world, &physics);
        assert!(ev2.swing_started.is_some());
        assert_eq!(combat.swing_state(), SwingState::WindingUp);
    }

    #[test]
    fn melee_damage_scales_with_wear_multiplier() {
        let registry = load_weapons();
        let bat = registry.get("bat").expect("bat def");
        let physics = flat_world();
        let mut player = Player::new(Vec3::ZERO);
        let mut world = MockMeleeWorld {
            zombies: vec![test_helpers::MockZombie {
                id: 1,
                position: Vec3::new(0.0, 0.0, -1.0),
                health: Health::new(100.0),
                knockback: Vec3::ZERO,
            }],
        };

        let mut combat = Combat::new("bat");
        combat.set_weapon(Some(bat.clone()));
        let _ = combat.update_with_wear(0.01, true, &mut player, &mut world, &physics, 0.5);
        let _ = combat.update_with_wear(0.25, false, &mut player, &mut world, &physics, 0.5);
        assert_eq!(combat.hits, 1);
        assert!((world.zombies[0].health.current() - 87.5).abs() < 1e-4);
    }
}
