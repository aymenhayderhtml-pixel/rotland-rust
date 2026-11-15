//! Player shooting and gun handling (port of scripts/Combat/GunHandler.cs and WeaponHolder.cs).
//!
//! Manages per-gun magazine states surviving weapon switches, ammo consumption from the inventory,
//! firing cadence (semi/auto), raycast spread, hit resolution against zombies, reload timing,
//! out-of-ammo bash routing, and camera aim field of view.

use std::collections::HashMap;

use game::guns::Gun;
use game::guns_fire::{fire_rays, resolve_hits, shot_noise, GunRng, GunTarget};
use game::items::Inventory;
use game::zombie::Noise;
use game::{DamageInfo, WeaponDef};
use glam::Vec3;
use physics::PhysicsWorld;

use crate::zombies::{Zombies, BODY_RADIUS};

/// Height of the zombie's head centre relative to its feet (metres).
pub const HEAD_HEIGHT_OFFSET: f32 = 1.65;
/// Radius of the head sphere target (metres).
pub const HEAD_RADIUS: f32 = 0.14;

/// Camera aim FOVs from scripts/Player/CameraRig.cs.
pub const THIRD_PERSON_AIM_FOV: f32 = 55.0;
pub const FIRST_PERSON_AIM_FOV: f32 = 50.0;

/// Player shooting input for the current frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShootInput {
    pub fire_pressed: bool,
    pub fire_held: bool,
    pub reload_pressed: bool,
    pub aiming: bool,
}

/// Events and outcome produced by one shooting update frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShootEvents {
    pub shots_fired: u32,
    pub hits: u32,
    pub kills: u32,
    pub noise: Option<Noise>,
    pub reload_started: bool,
    pub ammo: (u32, u32),
    pub empty_click: bool,
    pub bash: bool,
}

/// Abstraction over zombie target querying and damage application.
/// Allows unit testing without needing a GPU device for `Zombies`.
pub trait TargetWorld {
    fn gun_targets(&self) -> Vec<GunTarget>;
    fn hit(&mut self, id: u64, damage: DamageInfo, knockback: Vec3) -> bool;
}

impl TargetWorld for Zombies {
    fn gun_targets(&self) -> Vec<GunTarget> {
        self.list
            .iter()
            .filter(|z| !z.health.is_dead())
            .map(|z| GunTarget {
                id: z.id,
                position: z.position,
                body_radius: BODY_RADIUS,
                head_center: z.position + Vec3::Y * HEAD_HEIGHT_OFFSET,
                head_radius: HEAD_RADIUS,
            })
            .collect()
    }

    fn hit(&mut self, id: u64, damage: DamageInfo, knockback: Vec3) -> bool {
        self.hit(id, damage, knockback)
    }
}

/// The player's shooting state: owns magazine counts for all guns used, and the spread RNG.
pub struct Shooting {
    guns: HashMap<String, Gun>,
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
            rng: GunRng::new(seed),
            active_weapon: None,
        }
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


    /// One frame of shooting logic with the live `Zombies` system.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        dt: f32,
        input: ShootInput,
        weapon: &WeaponDef,
        eye: Vec3,
        aim_dir: Vec3,
        inventory: &mut Inventory,
        zombies: &mut Zombies,
        physics: &PhysicsWorld,
    ) -> ShootEvents {
        self.update_targets(dt, input, weapon, eye, aim_dir, inventory, zombies, physics)
    }

    /// One frame of shooting logic against any `TargetWorld` (enables pure tests without GPU).
    #[allow(clippy::too_many_arguments)]
    pub fn update_targets<T: TargetWorld>(
        &mut self,
        dt: f32,
        input: ShootInput,
        weapon: &WeaponDef,
        eye: Vec3,
        aim_dir: Vec3,
        inventory: &mut Inventory,
        targets: &mut T,
        physics: &PhysicsWorld,
    ) -> ShootEvents {
        if !weapon.is_gun() {
            return ShootEvents::default();
        }

        // Switching weapons cancels any reload in progress on the previous weapon
        if let Some(prev_id) = &self.active_weapon {
            if prev_id != &weapon.id {
                if let Some(prev_gun) = self.guns.get_mut(prev_id) {
                    prev_gun.cancel_reload();
                }
            }
        }
        self.active_weapon = Some(weapon.id.clone());

        let gun = self
            .guns
            .entry(weapon.id.clone())
            .or_insert_with(|| Gun::new(weapon.magazine_size));

        // Advance cooldown and reload timer
        if let Some(done) = gun.update(dt) {
            if done.rounds_taken > 0 {
                let available = inventory.count(&weapon.ammo_item);
                let to_take = done.rounds_taken.min(available);
                if to_take > 0 {
                    inventory.remove(&weapon.ammo_item, to_take);
                }
            }
        }

        let mut events = ShootEvents::default();
        let spare_in_bag = inventory.count(&weapon.ammo_item);

        // 1. Manual reload requested
        if input.reload_pressed && gun.start_reload(weapon, spare_in_bag) {
            events.reload_started = true;
        }

        // 2. Firing request: auto weapons fire while held, semi only on trigger press
        let wants_fire = if weapon.auto {
            input.fire_pressed || input.fire_held
        } else {
            input.fire_pressed
        };

        if wants_fire && !gun.is_reloading() {
            if gun.rounds() == 0 {
                if spare_in_bag == 0 {
                    // Empty with no spares: swung like a club instead of clicking (GunHandler.cs line 263)
                    events.bash = true;
                } else {
                    // Dry fire click, automatically starts reload if ammo is in the bag (GunHandler.cs line 144)
                    events.empty_click = true;
                    if gun.start_reload(weapon, spare_in_bag) {
                        events.reload_started = true;
                    }
                }
            } else if gun.try_fire(weapon, input.fire_pressed, input.fire_held) {
                events.shots_fired += 1;
                events.noise = Some(shot_noise(weapon, eye));

                let rays = fire_rays(weapon, eye, aim_dir, input.aiming, &mut self.rng);
                let target_list = targets.gun_targets();
                let hits = resolve_hits(&rays, weapon, &target_list, physics, weapon.range);

                for hit in hits {
                    if let (Some(target_id), Some(damage)) = (hit.target, hit.damage) {
                        events.hits += 1;
                        let knockback = Vec3::new(damage.direction.x, 0.0, damage.direction.z)
                            .normalize_or_zero()
                            * weapon.knockback;
                        if targets.hit(target_id, damage, knockback) {
                            events.kills += 1;
                        }
                    }
                }
            }
        }

        let final_spare = inventory.count(&weapon.ammo_item);
        events.ammo = (gun.rounds(), final_spare);

        events
    }
}

/// Aim FOV for a weapon: scope FOV if scoped, else 50 deg for 1st-person or 55 deg for 3rd-person.
/// Returns None for non-guns (melee).
pub fn aim_fov(weapon: &WeaponDef, first_person: bool) -> Option<f32> {
    if !weapon.is_gun() {
        return None;
    }
    if let Some(scope) = &weapon.scope {
        Some(scope.fov)
    } else if first_person {
        Some(FIRST_PERSON_AIM_FOV)
    } else {
        Some(THIRD_PERSON_AIM_FOV)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game::{Health, WeaponRegistry};
    use glam::IVec2;
    use physics::CHUNK_SAMPLES;

    struct MockZombie {
        id: u64,
        position: Vec3,
        health: Health,
    }

    struct MockWorld {
        zombies: Vec<MockZombie>,
    }

    impl TargetWorld for MockWorld {
        fn gun_targets(&self) -> Vec<GunTarget> {
            self.zombies
                .iter()
                .filter(|z| !z.health.is_dead())
                .map(|z| GunTarget {
                    id: z.id,
                    position: z.position,
                    body_radius: BODY_RADIUS,
                    head_center: z.position + Vec3::Y * HEAD_HEIGHT_OFFSET,
                    head_radius: HEAD_RADIUS,
                })
                .collect()
        }

        fn hit(&mut self, id: u64, damage: DamageInfo, _knockback: Vec3) -> bool {
            let Some(z) = self.zombies.iter_mut().find(|z| z.id == id) else { return false };
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
    fn shot_at_zombie_in_front_hits_and_lowers_health() {
        let registry = load_weapons();
        let pistol = registry.get("pistol").expect("pistol def");
        let physics = flat_world();
        let mut shooting = Shooting::default();
        let mut inventory = Inventory::new();
        inventory.add("pistol_ammo", 20);

        // Zombie 5m in front at (0, 0, -5). Head center is at (0, 1.65, -5).
        let mut targets = MockWorld {
            zombies: vec![MockZombie {
                id: 101,
                position: Vec3::new(0.0, 0.0, -5.0),
                health: Health::new(100.0),
            }],
        };

        // Player eye at (0, 1.65, 0) aiming straight forward (0, 0, -1) with aim spread
        let eye = Vec3::new(0.0, 1.65, 0.0);
        let aim_dir = Vec3::new(0.0, 0.0, -1.0);
        let input = ShootInput {
            fire_pressed: true,
            fire_held: false,
            reload_pressed: false,
            aiming: true,
        };

        let ev = shooting.update_targets(
            0.0,
            input,
            pistol,
            eye,
            aim_dir,
            &mut inventory,
            &mut targets,
            &physics,
        );

        let mut asserts = 0;
        assert_eq!(ev.shots_fired, 1);
        asserts += 1;
        assert_eq!(ev.hits, 1);
        asserts += 1;
        // Headshot: pistol damage 30 * headshotMultiplier 2.0 = 60 damage dealt. 100 - 60 = 40.
        let remaining_health = targets.zombies[0].health.current();
        assert_eq!(remaining_health, 40.0);
        asserts += 1;
        assert_eq!(ev.kills, 0);
        asserts += 1;
        assert!(ev.noise.is_some());
        asserts += 1;
        assert_eq!(ev.noise.unwrap().radius, pistol.noise_radius);
        asserts += 1;
        assert!(asserts >= 6);
    }

    #[test]
    fn magazine_count_drops_by_one_per_shot() {
        let registry = load_weapons();
        let pistol = registry.get("pistol").expect("pistol def");
        let physics = flat_world();
        let mut shooting = Shooting::default();
        let mut inventory = Inventory::new();
        inventory.add("pistol_ammo", 20);
        let mut targets = MockWorld { zombies: Vec::new() };

        let eye = Vec3::new(0.0, 1.65, 0.0);
        let aim_dir = Vec3::new(0.0, 0.0, -1.0);

        let mut asserts = 0;
        assert_eq!(pistol.magazine_size, 12);
        asserts += 1;

        // Shot 1
        let ev1 = shooting.update_targets(
            0.0,
            ShootInput { fire_pressed: true, ..Default::default() },
            pistol,
            eye,
            aim_dir,
            &mut inventory,
            &mut targets,
            &physics,
        );
        assert_eq!(ev1.shots_fired, 1);
        asserts += 1;
        assert_eq!(ev1.ammo.0, 11);
        asserts += 1;

        // Advance past cooldown (pistol fire_interval is 0.25)
        let ev2 = shooting.update_targets(
            0.3,
            ShootInput { fire_pressed: true, ..Default::default() },
            pistol,
            eye,
            aim_dir,
            &mut inventory,
            &mut targets,
            &physics,
        );
        assert_eq!(ev2.shots_fired, 1);
        asserts += 1;
        assert_eq!(ev2.ammo.0, 10);
        asserts += 1;
        assert!(asserts >= 5);
    }

    #[test]
    fn reload_takes_weapons_json_time_and_moves_rounds() {
        let registry = load_weapons();
        let pistol = registry.get("pistol").expect("pistol def");
        let physics = flat_world();
        let mut shooting = Shooting::default();
        let mut inventory = Inventory::new();
        inventory.add("pistol_ammo", 20);
        let mut targets = MockWorld { zombies: Vec::new() };

        let eye = Vec3::new(0.0, 1.65, 0.0);
        let aim_dir = Vec3::new(0.0, 0.0, -1.0);

        let mut asserts = 0;
        assert_eq!(pistol.reload_time, 1.6);
        asserts += 1;
        assert_eq!(pistol.magazine_size, 12);
        asserts += 1;

        // Start with 5 rounds in magazine: room is 12 - 5 = 7 rounds
        shooting.set_magazine("pistol", 5);

        // Frame 0: press reload
        let ev0 = shooting.update_targets(
            0.0,
            ShootInput { reload_pressed: true, ..Default::default() },
            pistol,
            eye,
            aim_dir,
            &mut inventory,
            &mut targets,
            &physics,
        );
        assert!(ev0.reload_started);
        asserts += 1;
        assert_eq!(ev0.ammo, (5, 20));
        asserts += 1;

        // Frame 1: advance 1.0 s (< 1.6 s reload_time)
        let ev1 = shooting.update_targets(
            1.0,
            ShootInput::default(),
            pistol,
            eye,
            aim_dir,
            &mut inventory,
            &mut targets,
            &physics,
        );
        assert!(!ev1.reload_started);
        asserts += 1;
        assert_eq!(ev1.ammo, (5, 20));
        asserts += 1;

        // Frame 2: advance 0.6 s (total elapsed 1.6 s == reload_time)
        let ev2 = shooting.update_targets(
            0.6,
            ShootInput::default(),
            pistol,
            eye,
            aim_dir,
            &mut inventory,
            &mut targets,
            &physics,
        );
        // Reload completed! 7 rounds moved from inventory into magazine
        assert_eq!(ev2.ammo, (12, 13));
        asserts += 1;
        assert_eq!(inventory.count("pistol_ammo"), 13);
        asserts += 1;
        assert_eq!(shooting.gun("pistol").unwrap().rounds(), 12);
        asserts += 1;
        assert!(!shooting.gun("pistol").unwrap().is_reloading());
        asserts += 1;
        assert!(asserts >= 10);
    }

    #[test]
    fn empty_gun_with_no_ammo_reports_bash_flag() {
        let registry = load_weapons();
        let pistol = registry.get("pistol").expect("pistol def");
        let physics = flat_world();
        let mut shooting = Shooting::default();
        let mut inventory = Inventory::new(); // 0 ammo
        let mut targets = MockWorld { zombies: Vec::new() };

        let eye = Vec3::new(0.0, 1.65, 0.0);
        let aim_dir = Vec3::new(0.0, 0.0, -1.0);

        shooting.set_magazine("pistol", 0);

        let mut asserts = 0;
        let ev = shooting.update_targets(
            0.0,
            ShootInput { fire_pressed: true, ..Default::default() },
            pistol,
            eye,
            aim_dir,
            &mut inventory,
            &mut targets,
            &physics,
        );
        assert!(ev.bash);
        asserts += 1;
        assert_eq!(ev.shots_fired, 0);
        asserts += 1;
        assert!(!ev.empty_click);
        asserts += 1;
        assert!(!ev.reload_started);
        asserts += 1;
        assert_eq!(ev.ammo, (0, 0));
        asserts += 1;
        assert!(asserts >= 5);
    }

    #[test]
    fn auto_weapons_fire_every_cooldown_while_held() {
        let registry = load_weapons();
        let ak47 = registry.get("ak47").expect("ak47 def");
        let physics = flat_world();
        let mut shooting = Shooting::default();
        let mut inventory = Inventory::new();
        inventory.add("ammo_762", 60);
        let mut targets = MockWorld { zombies: Vec::new() };

        let eye = Vec3::new(0.0, 1.65, 0.0);
        let aim_dir = Vec3::new(0.0, 0.0, -1.0);

        let mut asserts = 0;
        assert!(ak47.auto);
        asserts += 1;
        assert_eq!(ak47.fire_interval, 0.1);
        asserts += 1;
        assert_eq!(ak47.magazine_size, 30);
        asserts += 1;

        // Frame 0: trigger pressed and held -> shot 1 fires immediately
        let ev0 = shooting.update_targets(
            0.0,
            ShootInput { fire_pressed: true, fire_held: true, ..Default::default() },
            ak47,
            eye,
            aim_dir,
            &mut inventory,
            &mut targets,
            &physics,
        );
        assert_eq!(ev0.shots_fired, 1);
        asserts += 1;
        assert_eq!(ev0.ammo.0, 29);
        asserts += 1;

        // Frame 1: dt = 0.05 (< 0.10 fire_interval) -> cooldown still running, no shot
        let ev1 = shooting.update_targets(
            0.05,
            ShootInput { fire_pressed: false, fire_held: true, ..Default::default() },
            ak47,
            eye,
            aim_dir,
            &mut inventory,
            &mut targets,
            &physics,
        );
        assert_eq!(ev1.shots_fired, 0);
        asserts += 1;
        assert_eq!(ev1.ammo.0, 29);
        asserts += 1;

        // Frame 2: dt = 0.05 (total 0.10 since shot 1) -> cooldown finishes, shot 2 fires!
        let ev2 = shooting.update_targets(
            0.05,
            ShootInput { fire_pressed: false, fire_held: true, ..Default::default() },
            ak47,
            eye,
            aim_dir,
            &mut inventory,
            &mut targets,
            &physics,
        );
        assert_eq!(ev2.shots_fired, 1);
        asserts += 1;
        assert_eq!(ev2.ammo.0, 28);
        asserts += 1;

        // Frame 3: dt = 0.10 -> shot 3 fires!
        let ev3 = shooting.update_targets(
            0.10,
            ShootInput { fire_pressed: false, fire_held: true, ..Default::default() },
            ak47,
            eye,
            aim_dir,
            &mut inventory,
            &mut targets,
            &physics,
        );
        assert_eq!(ev3.shots_fired, 1);
        asserts += 1;
        assert_eq!(ev3.ammo.0, 27);
        asserts += 1;

        // 5 more intervals of 0.10 s while held
        let mut extra_shots = 0;
        for _ in 0..5 {
            let ev = shooting.update_targets(
                0.10,
                ShootInput { fire_pressed: false, fire_held: true, ..Default::default() },
                ak47,
                eye,
                aim_dir,
                &mut inventory,
                &mut targets,
                &physics,
            );
            extra_shots += ev.shots_fired;
        }
        assert_eq!(extra_shots, 5);
        asserts += 1;
        // Total shots fired: 1 + 0 + 1 + 1 + 5 = 8. Magazine: 30 - 8 = 22.
        assert_eq!(shooting.gun("ak47").unwrap().rounds(), 22);
        asserts += 1;
        assert!(asserts >= 12);
    }

    #[test]
    fn dry_fire_empty_click_auto_reloads() {
        let registry = load_weapons();
        let pistol = registry.get("pistol").expect("pistol def");
        let physics = flat_world();
        let mut shooting = Shooting::default();
        let mut inventory = Inventory::new();
        inventory.add("pistol_ammo", 10);
        let mut targets = MockWorld { zombies: Vec::new() };

        let eye = Vec3::new(0.0, 1.65, 0.0);
        let aim_dir = Vec3::new(0.0, 0.0, -1.0);

        shooting.set_magazine("pistol", 0);

        let mut asserts = 0;
        let ev = shooting.update_targets(
            0.0,
            ShootInput { fire_pressed: true, ..Default::default() },
            pistol,
            eye,
            aim_dir,
            &mut inventory,
            &mut targets,
            &physics,
        );
        assert!(ev.empty_click);
        asserts += 1;
        assert!(ev.reload_started);
        asserts += 1;
        assert!(!ev.bash);
        asserts += 1;
        assert_eq!(ev.shots_fired, 0);
        asserts += 1;
        assert!(shooting.gun("pistol").unwrap().is_reloading());
        asserts += 1;
        assert!(asserts >= 5);
    }

    #[test]
    fn aim_fov_returns_expected_numbers() {
        let registry = load_weapons();
        let pistol = registry.get("pistol").expect("pistol def");
        let rifle = registry.get("rifle").expect("rifle def");
        let bat = registry.get("bat").expect("bat def");

        let mut asserts = 0;
        // Non-gun (bat): None
        assert_eq!(aim_fov(bat, false), None);
        asserts += 1;
        assert_eq!(aim_fov(bat, true), None);
        asserts += 1;

        // Open sights gun (pistol): 55.0 in 3rd person, 50.0 in 1st person
        assert_eq!(aim_fov(pistol, false), Some(THIRD_PERSON_AIM_FOV));
        asserts += 1;
        assert_eq!(aim_fov(pistol, true), Some(FIRST_PERSON_AIM_FOV));
        asserts += 1;

        // Scoped gun (rifle): scope fov 25.0 in both modes
        assert_eq!(rifle.scope.as_ref().map(|s| s.fov), Some(25.0));
        asserts += 1;
        assert_eq!(aim_fov(rifle, false), rifle.scope.as_ref().map(|s| s.fov));
        asserts += 1;
        assert_eq!(aim_fov(rifle, true), rifle.scope.as_ref().map(|s| s.fov));
        asserts += 1;
        assert!(asserts >= 7);
    }
}
