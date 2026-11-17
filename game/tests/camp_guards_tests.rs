//! Integration tests for camp guards shooting logic (port of SettlementGuard.cs).

use std::f32::consts::{FRAC_PI_2, PI};
use game::{
    CampGuardConfig, CampGuards, GuardFrame, GuardPost, GuardRifle, GuardTarget,
};
use glam::Vec3;

#[test]
fn test_config_values_match_data_file() {
    let cfg = CampGuardConfig::from_repo().expect("settlements.json must load");
    // Numbers traced from data/settlements.json
    assert_eq!(cfg.guard_range, 40.0);
    assert_eq!(cfg.guard_sight_interval, 0.5);
    assert_eq!(cfg.guard_fire_interval, 1.2);
    assert_eq!(cfg.guard_damage, 60.0);
    assert!((cfg.guard_miss_chance - 0.15).abs() < 1e-4);
    assert_eq!(cfg.guard_eye_height, 1.6);
}

#[test]
fn test_zombie_in_range_shot_at_fire_rate() {
    let mut guards = CampGuards::from_repo()
        .expect("settlements.json must load")
        .with_camp_center(Vec3::ZERO)
        .with_miss_chance(0.0); // 0 miss chance to test exact fire rate

    let guard = GuardPost::new("guard_1", Vec3::ZERO, 0.0);
    let target = GuardTarget::new(100, Vec3::new(10.0, 0.0, 0.0), true);

    // Frame 1 (dt = 0.1): scan_timer and fire_timer both start at 0, so guard scans and fires on frame 1
    let frame1: GuardFrame = guards.update_clear(0.1, &[guard.clone()], &[target.clone()]);
    assert_eq!(frame1.shots.len(), 1, "Guard must fire on first acquisition");
    assert_eq!(frame1.sounds.len(), 1, "Rifle sound must play on fire");
    assert_eq!(frame1.sounds[0].0, GuardRifle::SHOT_SOUND);
    assert_eq!(frame1.sounds[0].1, Vec3::new(0.0, 1.6, 0.0));

    let shot = &frame1.shots[0];
    assert_eq!(shot.guard_id, "guard_1");
    assert_eq!(shot.target_id, 100);
    assert_eq!(shot.from, Vec3::new(0.0, 1.6, 0.0));
    assert_eq!(shot.to, Vec3::new(10.0, 1.2, 0.0));
    assert_eq!(shot.damage.amount, 60.0);
    assert_eq!(shot.damage.damage_type, GuardRifle::DAMAGE_TYPE);

    // Aim yaw: target is at +X (East), so yaw must be -PI/2
    assert_eq!(frame1.yaws.len(), 1);
    assert!((frame1.yaws[0].1 - (-FRAC_PI_2)).abs() < 1e-4);
    assert_eq!(guards.shots_fired("guard_1"), 1);

    // Next 11 frames of dt = 0.1 (total elapsed = 1.2s since start, 1.1s since shot): fire timer has not expired yet
    for _ in 0..11 {
        let frame: GuardFrame = guards.update_clear(0.1, &[guard.clone()], &[target.clone()]);
        assert_eq!(frame.shots.len(), 0, "Guard must wait fire_interval before shooting again");
    }

    // Frame 13 (total elapsed = 1.3s, 1.2s since shot): fire_timer (1.2s) reaches 0 and fires second shot
    let frame13: GuardFrame = guards.update_clear(0.1, &[guard.clone()], &[target]);
    assert_eq!(frame13.shots.len(), 1, "Guard must fire second shot after fire_interval (1.2s)");
    assert_eq!(guards.shots_fired("guard_1"), 2);
}

#[test]
fn test_out_of_range_zombie_not_shot() {
    let mut guards = CampGuards::from_repo()
        .expect("settlements.json must load")
        .with_camp_center(Vec3::ZERO)
        .with_miss_chance(0.0);

    let guard = GuardPost::new("guard_1", Vec3::ZERO, 0.0);
    // Zombie is at 45m, beyond guard_range of 40m
    let target = GuardTarget::new(100, Vec3::new(45.0, 0.0, 0.0), true);

    let frame: GuardFrame = guards.update_clear(0.1, &[guard], &[target]);
    assert!(frame.shots.is_empty(), "Zombie beyond 40m guard range must not be shot");
    assert!(frame.sounds.is_empty());
    assert_eq!(guards.shots_fired("guard_1"), 0);
}

#[test]
fn test_blocked_by_obstacle_not_shot() {
    let mut guards = CampGuards::from_repo()
        .expect("settlements.json must load")
        .with_camp_center(Vec3::ZERO)
        .with_miss_chance(0.0);

    let guard = GuardPost::new("guard_1", Vec3::ZERO, 0.0);
    let target = GuardTarget::new(100, Vec3::new(10.0, 0.0, 0.0), true);

    // Line of sight blocked by wall
    let blocked_los = |_from: Vec3, _to: Vec3| false;

    let frame: GuardFrame = guards.update(0.1, &[guard], &[target], &blocked_los);
    assert!(frame.shots.is_empty(), "Target without clear line of sight must not be shot");
    assert!(frame.sounds.is_empty());
    assert_eq!(guards.shots_fired("guard_1"), 0);
}

#[test]
fn test_dead_target_ignored() {
    let mut guards = CampGuards::from_repo()
        .expect("settlements.json must load")
        .with_camp_center(Vec3::ZERO)
        .with_miss_chance(0.0);

    let guard = GuardPost::new("guard_1", Vec3::ZERO, 0.0);
    // Target is dead
    let target = GuardTarget::new(100, Vec3::new(10.0, 0.0, 0.0), false);

    let frame: GuardFrame = guards.update_clear(0.1, &[guard], &[target]);
    assert!(frame.shots.is_empty(), "Dead targets must be ignored");
    assert!(frame.sounds.is_empty());
    assert_eq!(guards.shots_fired("guard_1"), 0);
}

#[test]
fn test_nearest_target_picked() {
    let mut guards = CampGuards::from_repo()
        .expect("settlements.json must load")
        .with_camp_center(Vec3::ZERO)
        .with_miss_chance(0.0);

    let guard = GuardPost::new("guard_1", Vec3::ZERO, 0.0);
    let targets = vec![
        GuardTarget::new(101, Vec3::new(25.0, 0.0, 0.0), true),
        GuardTarget::new(102, Vec3::new(8.0, 0.0, 0.0), true), // Nearest
        GuardTarget::new(103, Vec3::new(18.0, 0.0, 0.0), true),
    ];

    let frame: GuardFrame = guards.update_clear(0.1, &[guard], &targets);
    assert_eq!(frame.shots.len(), 1);
    assert_eq!(
        frame.shots[0].target_id, 102,
        "Nearest target (id 102 at 8m) must be chosen over farther targets"
    );
}

#[test]
fn test_guard_aim_yaw_directions() {
    let mut guards = CampGuards::from_repo()
        .expect("settlements.json must load")
        .with_camp_center(Vec3::ZERO)
        .with_miss_chance(0.0);

    let guard = GuardPost::new("guard_1", Vec3::ZERO, 0.0);

    // East (+X): yaw = atan2(-10, -0) = -PI/2
    let target_east = GuardTarget::new(1, Vec3::new(10.0, 0.0, 0.0), true);
    let frame_east = guards.update_clear(0.1, &[guard.clone()], &[target_east]);
    assert!((frame_east.yaws[0].1 - (-FRAC_PI_2)).abs() < 1e-4);

    // West (-X): yaw = atan2(10, -0) = +PI/2
    let mut guards_west = CampGuards::from_repo()
        .expect("settlements.json must load")
        .with_camp_center(Vec3::ZERO)
        .with_miss_chance(0.0);
    let target_west = GuardTarget::new(2, Vec3::new(-10.0, 0.0, 0.0), true);
    let frame_west = guards_west.update_clear(0.1, &[guard.clone()], &[target_west]);
    assert!((frame_west.yaws[0].1 - FRAC_PI_2).abs() < 1e-4);

    // North (-Z, forward in Godot): yaw = atan2(-0, 10) = 0.0
    let mut guards_north = CampGuards::from_repo()
        .expect("settlements.json must load")
        .with_camp_center(Vec3::ZERO)
        .with_miss_chance(0.0);
    let target_north = GuardTarget::new(3, Vec3::new(0.0, 0.0, -10.0), true);
    let frame_north = guards_north.update_clear(0.1, &[guard.clone()], &[target_north]);
    assert!(frame_north.yaws[0].1.abs() < 1e-4);

    // South (+Z, backward in Godot): yaw = atan2(-0, -10) = PI or -PI
    let mut guards_south = CampGuards::from_repo()
        .expect("settlements.json must load")
        .with_camp_center(Vec3::ZERO)
        .with_miss_chance(0.0);
    let target_south = GuardTarget::new(4, Vec3::new(0.0, 0.0, 10.0), true);
    let frame_south = guards_south.update_clear(0.1, &[guard], &[target_south]);
    assert!((frame_south.yaws[0].1.abs() - PI).abs() < 1e-4);
}

#[test]
fn test_target_death_switches_to_next_target() {
    let mut guards = CampGuards::from_repo()
        .expect("settlements.json must load")
        .with_camp_center(Vec3::ZERO)
        .with_miss_chance(0.0);

    let guard = GuardPost::new("guard_1", Vec3::ZERO, 0.0);
    let target_a = GuardTarget::new(101, Vec3::new(10.0, 0.0, 0.0), true);

    // Frame 1: fires at target A
    let f1 = guards.update_clear(0.1, &[guard.clone()], &[target_a]);
    assert_eq!(f1.shots.len(), 1);
    assert_eq!(f1.shots[0].target_id, 101);

    // Target A dies, target B appears
    let dead_target_a = GuardTarget::new(101, Vec3::new(10.0, 0.0, 0.0), false);
    let target_b = GuardTarget::new(102, Vec3::new(12.0, 0.0, 0.0), true);

    // Advance 0.5s to trigger scan_timer
    let _ = guards.update_clear(0.5, &[guard.clone()], &[dead_target_a, target_b.clone()]);

    // Guard acquires target B
    assert_eq!(guards.current_target("guard_1"), Some(102));
}
