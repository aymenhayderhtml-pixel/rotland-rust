//! Shooting unit tests: automatic fire cadence, aim FOV and the scoped crossbow.

use game::items::Inventory;
use glam::Vec3;

use super::aim::{aim_fov, FIRST_PERSON_AIM_FOV, THIRD_PERSON_AIM_FOV};
use super::tests_helpers::{flat_world, load_weapons, MockWorld};
use super::{ShootInput, Shooting};

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

#[test]
fn scoped_crossbow_returns_scope_aim_fov() {
    let registry = load_weapons();
    let crossbow = registry.get("crossbow").expect("crossbow def");
    let atts = game::AttachmentRegistry::from_repo().expect("attachments");
    assert_eq!(aim_fov(crossbow, false), None);

    let scoped = game::effective_stats(crossbow, &["scope_2x".to_string()], &atts);
    assert_eq!(aim_fov(&scoped, false), Some(40.0));
    assert_eq!(aim_fov(&scoped, true), Some(40.0));
}
