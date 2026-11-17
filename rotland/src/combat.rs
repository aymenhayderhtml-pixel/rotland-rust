//! The player's side of the fight: health, the melee swing on the left mouse button, being bitten,
//! dying and coming back at the camp. Zombies live in zombies.rs.

use game::melee::{gun_bash, Attacker, MeleeSwing, SwingState};
use game::zombie::Noise;
use game::{DamageInfo, Health, Player, Stamina, WeaponDef, WeaponRegistry};
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
    pub bash_weapon: Option<WeaponDef>,
    pending_swing_started: Option<String>,
    pub dead_for: Option<f32>,
    pub kills: u32,
    /// Gun shots fired (stats only).
    pub shots: u32,
    /// Sounds made this frame (swings), heard by zombies next frame.
    pub noises: Vec<Noise>,
    /// Melee hits that landed this frame (for the hit sound).
    pub hits: u32,
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
    /// A swing landed this frame (hit or miss): the weapon and the flat swing direction, for trees.
    pub swung: Option<(WeaponDef, Vec3)>,
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
            weapon,
            bash_weapon: None,
            pending_swing_started: None,
            dead_for: None,
            kills: 0,
            shots: 0,
            noises: Vec::new(),
            hits: 0,
        }
    }

    /// The weapon in hand changes (hotbar). None: nothing to swing.
    pub fn set_weapon(&mut self, weapon: Option<WeaponDef>) {
        self.swing.cancel();
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

    /// Swings an empty gun as a club when out of ammo (GunBash.cs).
    /// Builds a melee WeaponDef via gun_bash(gun), deducting stamina and resolving
    /// the swing through the same MeleeWorld path as a melee weapon.
    pub fn bash(&mut self, gun: &WeaponDef, stamina: &mut Stamina) -> bool {
        if self.is_dead() || !gun.is_gun() {
            return false;
        }
        let bash_def = gun_bash(gun);
        if !self.swing.try_start(&bash_def, stamina) {
            return false;
        }
        self.pending_swing_started = bash_def.anim.attack.clone();
        self.bash_weapon = Some(bash_def);
        true
    }

    /// True if a gun bash is active (winding up or cooling down).
    #[cfg(test)]
    pub fn is_bashing(&self) -> bool {
        self.bash_weapon.is_some()
    }

    /// Current state of the melee / bash swing.
    #[cfg(test)]
    pub fn swing_state(&self) -> SwingState {
        self.swing.state()
    }


    /// One frame: start a swing if asked, resolve hits on zombies, take bites, count down a death.
    pub fn update(&mut self, dt: f32, attack_pressed: bool, player: &mut Player, zombies: &mut impl MeleeWorld, physics: &PhysicsWorld) -> CombatEvents {
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

        if let Some(weapon) = &self.weapon
            && attack_pressed
            && self.swing.try_start(weapon, &mut player.stamina)
        {
            ev.swing_started = weapon.anim.attack.clone();
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

    /// Kills the player now (test option --die-at). Returns true if they were alive.
    pub fn kill(&mut self) -> bool {
        if self.is_dead() {
            return false;
        }
        self.health.kill();
        self.dead_for = Some(0.0);
        self.swing.cancel();
        self.bash_weapon = None;
        self.pending_swing_started = None;
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
                self.bash_weapon = None;
                self.pending_swing_started = None;
                break;
            }
        }
        ev
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
mod tests {
    use super::*;
    use game::{Health, Stamina, WeaponRegistry};
    use glam::IVec2;
    use physics::CHUNK_SAMPLES;

    struct MockZombie {
        id: u64,
        position: Vec3,
        health: Health,
        knockback: Vec3,
    }

    struct MockMeleeWorld {
        zombies: Vec<MockZombie>,
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

    fn flat_world() -> PhysicsWorld {
        let mut world = PhysicsWorld::new();
        let heights = vec![0.0f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
        world.add_ground_chunk(IVec2::new(0, 0), &heights);
        world
    }

    fn load_weapons() -> WeaponRegistry {
        WeaponRegistry::load(&paths::repo_path("data/weapons.json"))
            .expect("data/weapons.json must load")
    }

    #[test]
    fn empty_pistol_click_starts_bash_with_weapons_json_numbers() {
        let registry = load_weapons();
        let pistol = registry.get("pistol").expect("pistol weapon def");

        let mut asserts = 0;
        // Verify numbers directly from data/weapons.json
        assert_eq!(pistol.bash_damage, 15.0);
        asserts += 1;
        assert_eq!(pistol.bash_impact, 0.30);
        asserts += 1;
        assert_eq!(pistol.anim.bash.as_deref(), Some("gun_bash"));
        asserts += 1;

        let mut combat = Combat::new("bat");
        // Holding a gun clears melee weapon
        combat.set_weapon(None);
        assert_eq!(combat.weapon_id(), None);
        asserts += 1;

        let mut stamina = Stamina::default();
        let initial_stamina = stamina.current; // 100.0
        assert_eq!(initial_stamina, 100.0);
        asserts += 1;

        // Start gun bash
        let started = combat.bash(pistol, &mut stamina);
        assert!(started);
        asserts += 1;
        assert!(combat.is_bashing());
        asserts += 1;
        assert_eq!(combat.swing_state(), SwingState::WindingUp);
        asserts += 1;
        assert_eq!(combat.weapon_id(), Some("pistol"));
        asserts += 1;

        // Stamina deducted by BASH_STAMINA_COST (10.0)
        assert_eq!(stamina.current, 90.0);
        asserts += 1;
        assert_eq!(stamina.time_since_use, 0.0);
        asserts += 1;

        // Verify bash weapon definition matches GunBash.cs and weapons.json specifications
        let bash_def = combat.bash_weapon.as_ref().expect("bash weapon def");
        assert_eq!(bash_def.id, "pistol");
        asserts += 1;
        assert_eq!(bash_def.damage, 15.0); // pistol.bash_damage
        asserts += 1;
        assert_eq!(bash_def.windup, 0.30); // pistol.bash_impact
        asserts += 1;
        assert_eq!(bash_def.cooldown, 0.7); // BASH_COOLDOWN
        asserts += 1;
        assert_eq!(bash_def.stamina_cost, 10.0); // BASH_STAMINA_COST
        asserts += 1;
        assert_eq!(bash_def.range, 1.9); // BASH_RANGE
        asserts += 1;
        assert_eq!(bash_def.arc_deg, 80.0); // BASH_ARC_DEG
        asserts += 1;
        assert_eq!(bash_def.knockback, 2.0); // BASH_KNOCKBACK
        asserts += 1;
        assert_eq!(bash_def.stagger, 0.3); // BASH_STAGGER
        asserts += 1;
        assert_eq!(bash_def.max_targets, 1);
        asserts += 1;
        assert_eq!(bash_def.noise_radius, 6.0); // BASH_NOISE_RADIUS
        asserts += 1;
        assert_eq!(bash_def.anim.attack.as_deref(), Some("gun_bash"));
        asserts += 1;

        // Cannot start another bash while winding up
        assert!(!combat.bash(pistol, &mut stamina));
        asserts += 1;
        assert!(asserts >= 20);
    }

    #[test]
    fn zombie_in_front_takes_bash_damage() {
        let registry = load_weapons();
        let pistol = registry.get("pistol").expect("pistol weapon def");
        let physics = flat_world();

        let mut combat = Combat::new("bat");
        combat.set_weapon(None);

        let mut player = Player::new(Vec3::ZERO);
        player.yaw = 0.0; // facing Vec3::NEG_Z (forward is (0, 0, -1))

        // Zombie 1.2m ahead at (0, 0, -1.2) with 100.0 health (in range: 1.2 < 1.9 BASH_RANGE)
        let mut world = MockMeleeWorld {
            zombies: vec![MockZombie {
                id: 42,
                position: Vec3::new(0.0, 0.0, -1.2),
                health: Health::new(100.0),
                knockback: Vec3::ZERO,
            }],
        };

        let mut asserts = 0;
        let started = combat.bash(pistol, &mut player.stamina);
        assert!(started);
        asserts += 1;

        // Frame 1: advance 0.15s (< 0.30s bash impact). No hit yet.
        let ev1 = combat.update(0.15, false, &mut player, &mut world, &physics);
        assert_eq!(combat.hits, 0);
        asserts += 1;
        assert_eq!(world.zombies[0].health.current(), 100.0);
        asserts += 1;
        assert_eq!(ev1.swing_started.as_deref(), Some("gun_bash"));
        asserts += 1;
        assert!(ev1.swung.is_none());
        asserts += 1;
        assert_eq!(combat.swing_state(), SwingState::WindingUp);
        asserts += 1;

        // Frame 2: advance 0.16s (total elapsed 0.31s >= 0.30s bash impact). Hit resolves!
        let ev2 = combat.update(0.16, false, &mut player, &mut world, &physics);
        assert_eq!(combat.hits, 1);
        asserts += 1;
        // Zombie takes exactly pistol.bash_damage (15.0). 100.0 - 15.0 = 85.0.
        assert_eq!(world.zombies[0].health.current(), 85.0);
        asserts += 1;
        // Knockback along attack direction (0, 0, -1) * 2.0 knockback
        assert_eq!(world.zombies[0].knockback, Vec3::new(0.0, 0.0, -2.0));
        asserts += 1;
        // Bash noise emitted at player position with radius 6.0
        assert_eq!(combat.noises.len(), 1);
        asserts += 1;
        assert_eq!(combat.noises[0].radius, 6.0);
        asserts += 1;
        assert!(ev2.swung.is_some());
        asserts += 1;
        // State transitions to CoolingDown
        assert_eq!(combat.swing_state(), SwingState::CoolingDown);
        asserts += 1;

        // Frame 3: advance past cooldown (0.7s total cooldown, 0.31s elapsed -> remaining ~0.39s)
        combat.update(0.40, false, &mut player, &mut world, &physics);
        assert_eq!(combat.swing_state(), SwingState::Idle);
        asserts += 1;
        assert!(!combat.is_bashing());
        asserts += 1;
        assert!(asserts >= 12);
    }

    #[test]
    fn ak47_bash_deals_30_damage_matching_godot_gun_bash_when_empty() {
        let registry = load_weapons();
        let ak47 = registry.get("ak47").expect("ak47 weapon def");
        let physics = flat_world();

        let mut combat = Combat::new("bat");
        combat.set_weapon(None);

        let mut player = Player::new(Vec3::ZERO);
        player.yaw = 0.0;

        // Zombie 1.5m in front with 60.0 health (exact setup from GunBashWhenEmpty.cs)
        let mut world = MockMeleeWorld {
            zombies: vec![MockZombie {
                id: 10,
                position: Vec3::new(0.0, 0.0, -1.5),
                health: Health::new(60.0),
                knockback: Vec3::ZERO,
            }],
        };

        let mut asserts = 0;
        assert_eq!(ak47.bash_damage, 30.0);
        asserts += 1;
        assert_eq!(ak47.bash_impact, 0.30);
        asserts += 1;

        assert!(combat.bash(ak47, &mut player.stamina));
        asserts += 1;

        // Advance 0.35s past impact (0.30s)
        let ev = combat.update(0.35, false, &mut player, &mut world, &physics);
        assert_eq!(combat.hits, 1);
        asserts += 1;
        // Exact assertion from GunBashWhenEmpty.cs: 60 - 30 = 30
        assert_eq!(world.zombies[0].health.current(), 30.0);
        asserts += 1;
        assert_eq!(combat.noises[0].radius, 6.0);
        asserts += 1;
        assert!(ev.swung.is_some());
        asserts += 1;
        assert!(asserts >= 7);
    }

    #[test]
    fn bash_fails_when_insufficient_stamina_or_dead() {
        let registry = load_weapons();
        let pistol = registry.get("pistol").expect("pistol weapon def");

        let mut combat = Combat::new("bat");
        combat.set_weapon(None);

        let mut asserts = 0;
        // Stamina less than 10.0 (BASH_STAMINA_COST)
        let mut stamina = Stamina { current: 5.0, ..Default::default() };
        assert!(!combat.bash(pistol, &mut stamina));
        asserts += 1;
        assert_eq!(stamina.current, 5.0); // stamina unchanged
        asserts += 1;
        assert!(!combat.is_bashing());
        asserts += 1;

        // Dead player cannot bash
        combat.kill();
        let mut full_stamina = Stamina::default();
        assert!(!combat.bash(pistol, &mut full_stamina));
        asserts += 1;
        assert_eq!(full_stamina.current, 100.0);
        asserts += 1;
        assert!(asserts >= 5);
    }
}
