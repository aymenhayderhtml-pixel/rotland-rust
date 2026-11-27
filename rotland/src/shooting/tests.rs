//! Shooting unit tests: hit resolution, magazine counts and reload timing.

use game::items::Inventory;
use game::Health;
use glam::Vec3;

use super::{ShootInput, Shooting};
use super::tests_helpers::{flat_world, load_weapons, MockWorld, MockZombie};

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
    assert_eq!(ev.fired_weapon.as_deref(), Some("pistol"));
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
