//! Shooting unit tests: empty-gun bash, bows, broken guns and gun-shop style mods.

use game::items::Inventory;
use glam::Vec3;

use super::tests_helpers::{flat_world, load_weapons, MockWorld};
use super::{ShootInput, Shooting};

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
fn gun_with_ammo_fires_instead_of_bashing() {
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
    // Case 1: Gun with rounds in magazine fires a bullet instead of bashing
    shooting.set_magazine("pistol", 12);
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
    assert!(!ev1.bash);
    asserts += 1;
    assert!(!ev1.empty_click);
    asserts += 1;
    assert_eq!(ev1.ammo, (11, 20));
    asserts += 1;

    // Case 2: Gun with 0 in magazine but ammo in inventory dry-fires and starts reload (no bash)
    shooting.set_magazine("pistol", 0);
    let ev2 = shooting.update_targets(
        0.3, // past fire interval
        ShootInput { fire_pressed: true, ..Default::default() },
        pistol,
        eye,
        aim_dir,
        &mut inventory,
        &mut targets,
        &physics,
    );
    assert_eq!(ev2.shots_fired, 0);
    asserts += 1;
    assert!(!ev2.bash);
    asserts += 1;
    assert!(ev2.empty_click);
    asserts += 1;
    assert!(ev2.reload_started);
    asserts += 1;

    // Case 3: Gun with 0 in magazine and 0 in inventory produces bash event (no dry fire click)
    let mut empty_inventory = Inventory::new();
    let mut empty_shooting = Shooting::default();
    empty_shooting.set_magazine("pistol", 0);
    let ev3 = empty_shooting.update_targets(
        0.0,
        ShootInput { fire_pressed: true, ..Default::default() },
        pistol,
        eye,
        aim_dir,
        &mut empty_inventory,
        &mut targets,
        &physics,
    );
    assert_eq!(ev3.shots_fired, 0);
    asserts += 1;
    assert!(ev3.bash);
    asserts += 1;
    assert!(!ev3.empty_click);
    asserts += 1;
    assert!(!ev3.reload_started);
    asserts += 1;
    assert_eq!(ev3.ammo, (0, 0));
    asserts += 1;

    assert!(asserts >= 13);
}

#[test]
fn bow_reports_arrows_in_bag_as_ammo() {
    let registry = load_weapons();
    let bow = registry.get("bow").expect("bow def");
    let physics = flat_world();
    let mut shooting = Shooting::default();
    let mut inventory = Inventory::new();
    inventory.add("arrow", 25);
    let mut targets = MockWorld { zombies: Vec::new() };

    let eye = Vec3::new(0.0, 1.65, 0.0);
    let aim_dir = Vec3::new(0.0, 0.0, -1.0);

    let ev = shooting.update_targets(
        0.0,
        ShootInput::default(),
        bow,
        eye,
        aim_dir,
        &mut inventory,
        &mut targets,
        &physics,
    );
    assert_eq!(ev.ammo, (25, u32::MAX));
    assert_eq!(ev.shots_fired, 0);
}

#[test]
fn broken_gun_refuses_to_fire_and_clicks() {
    let registry = load_weapons();
    let pistol = registry.get("pistol").expect("pistol def");
    let wear_cfg = game::WearConfig::from_repo().expect("wear config");
    let physics = flat_world();
    let mut shooting = Shooting::default();
    let mut inventory = Inventory::new();
    inventory.add("pistol_ammo", 12);
    let mut targets = MockWorld { zombies: Vec::new() };

    let ev = shooting.update_targets_with_wear(
        0.0,
        ShootInput { fire_pressed: true, ..Default::default() },
        pistol,
        Vec3::new(0.0, 1.65, 0.0),
        Vec3::NEG_Z,
        &mut inventory,
        &mut targets,
        &physics,
        0.0,
        &wear_cfg,
    );
    assert!(ev.broken);
    assert!(ev.empty_click);
    assert_eq!(ev.shots_fired, 0);
    assert_eq!(ev.ammo.0, pistol.magazine_size);
}
