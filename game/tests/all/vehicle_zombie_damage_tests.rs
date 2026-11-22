use glam::{Quat, Vec3};
use game::items::Inventory;
use game::vehicles::{
    calculate_zombie_damage, count_zombies_in_reach, is_zombie_in_reach, CarDamageBox,
    CarHealth, Cars, VehicleConfigData,
};

#[test]
fn test_zombie_reach_checks_horizontal_and_vertical() {
    let car_pos = Vec3::new(10.0, 0.0, 20.0);
    let car_rot = Quat::IDENTITY;
    let half_size = Vec3::new(0.8, 1.2, 1.9);
    let reach = 1.5;

    // 1. Zombie right beside car side (0.5m away from hull): in reach
    let near_side = Vec3::new(10.0 + 0.8 + 0.5, 0.0, 20.0);
    assert!(is_zombie_in_reach(car_pos, car_rot, half_size, reach, near_side));

    // 2. Zombie right behind car (0.5m behind rear): in reach
    let near_rear = Vec3::new(10.0, 0.0, 20.0 + 1.9 + 0.5);
    assert!(is_zombie_in_reach(car_pos, car_rot, half_size, reach, near_rear));

    // 3. Zombie 5m away horizontally: out of reach
    let far_away = Vec3::new(10.0 + 5.0, 0.0, 20.0);
    assert!(!is_zombie_in_reach(car_pos, car_rot, half_size, reach, far_away));

    // 4. Zombie near horizontally but 10m high in the air: out of reach
    let high_up = Vec3::new(10.0 + 0.8 + 0.5, 10.0, 20.0);
    assert!(!is_zombie_in_reach(car_pos, car_rot, half_size, reach, high_up));

    // 5. Zombie 5m below ground: out of reach
    let deep_down = Vec3::new(10.0 + 0.8 + 0.5, -5.0, 20.0);
    assert!(!is_zombie_in_reach(car_pos, car_rot, half_size, reach, deep_down));
}

#[test]
fn test_zombie_reach_rotated_car() {
    // Car rotated 90 degrees around Y: local Z is world -X, local X is world Z
    let car_pos = Vec3::ZERO;
    let car_rot = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let half_size = Vec3::new(0.8, 1.2, 1.9);
    let reach = 1.5;

    // In local space, (0.0, 0.0, 2.2) is 0.3m behind rear (half_length 1.9).
    // In world space rotated 90 deg, that point is (-2.2, 0.0, 0.0).
    let behind_turned_car = Vec3::new(-2.2, 0.0, 0.0);
    assert!(is_zombie_in_reach(car_pos, car_rot, half_size, reach, behind_turned_car));

    // A point at (+10.0, 0.0, 0.0) is far in front: out of reach
    let far_point = Vec3::new(10.0, 0.0, 0.0);
    assert!(!is_zombie_in_reach(car_pos, car_rot, half_size, reach, far_point));
}

#[test]
fn test_zombie_damage_counts_and_amounts() {
    let half_size = Vec3::new(0.8, 1.2, 1.9);
    let car_box = CarDamageBox::new("car_1", Vec3::ZERO, Quat::IDENTITY, half_size, false);
    let cars = vec![car_box];
    let hit_damage = 2.0;
    let hit_seconds = 1.2;
    let reach = 1.5;

    // 0 zombies in reach: 0.0 damage
    assert_eq!(count_zombies_in_reach(Vec3::ZERO, Quat::IDENTITY, half_size, reach, &[]), 0);
    let dmg_0 = calculate_zombie_damage(&cars, &[], hit_damage, hit_seconds, reach, 1.2);
    assert_eq!(dmg_0.len(), 1);
    assert_eq!(dmg_0[0].1, 0.0);

    // Zombies out of reach: 0.0 damage
    let distant = vec![Vec3::new(10.0, 0.0, 0.0), Vec3::new(-20.0, 0.0, 5.0)];
    assert_eq!(count_zombies_in_reach(Vec3::ZERO, Quat::IDENTITY, half_size, reach, &distant), 0);
    let dmg_distant = calculate_zombie_damage(&cars, &distant, hit_damage, hit_seconds, reach, 1.2);
    assert_eq!(dmg_distant[0].1, 0.0);

    // 1 zombie in reach: 2.0 damage over 1.2s (1 hit)
    let one_zombie = vec![Vec3::new(1.0, 0.0, 0.0)];
    assert_eq!(count_zombies_in_reach(Vec3::ZERO, Quat::IDENTITY, half_size, reach, &one_zombie), 1);
    let dmg_1 = calculate_zombie_damage(&cars, &one_zombie, hit_damage, hit_seconds, reach, 1.2);
    assert!((dmg_1[0].1 - 2.0).abs() < 1e-4);

    // 3 zombies in reach: 6.0 damage over 1.2s (3 hits)
    let three_zombies = vec![
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(-1.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 2.2),
    ];
    assert_eq!(count_zombies_in_reach(Vec3::ZERO, Quat::IDENTITY, half_size, reach, &three_zombies), 3);
    let dmg_3 = calculate_zombie_damage(&cars, &three_zombies, hit_damage, hit_seconds, reach, 1.2);
    assert!((dmg_3[0].1 - 6.0).abs() < 1e-4);

    // 3 zombies over 60 seconds (1 minute): exactly 300 damage (wrecks a 300 HP car)
    let dmg_60s = calculate_zombie_damage(&cars, &three_zombies, hit_damage, hit_seconds, reach, 60.0);
    assert!((dmg_60s[0].1 - 300.0).abs() < 1e-4);
}

#[test]
fn test_wrecked_car_takes_no_damage_and_clamps_at_zero() {
    let half_size = Vec3::new(0.8, 1.2, 1.9);
    // Already wrecked car
    let wrecked_box = CarDamageBox::new("wrecked_car", Vec3::ZERO, Quat::IDENTITY, half_size, true);
    let three_zombies = vec![
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(-1.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 2.2),
    ];

    // calculate_zombie_damage returns 0.0 for wrecked cars
    let dmg = calculate_zombie_damage(&[wrecked_box], &three_zombies, 2.0, 1.2, 1.5, 60.0);
    assert_eq!(dmg[0].1, 0.0, "wrecked car must take no further damage");

    // CarHealth clamps at 0.0 (no damage below 0)
    let mut health = CarHealth::new(300.0);
    health.damage(295.0);
    assert_eq!(health.current(), 5.0);
    assert!(!health.is_wrecked());

    // Taking 10.0 damage when at 5.0 drops to 0.0 and becomes wrecked
    health.damage(10.0);
    assert_eq!(health.current(), 0.0);
    assert!(health.is_wrecked());

    // Taking further damage stays at 0.0 (never negative)
    health.damage(50.0);
    assert_eq!(health.current(), 0.0);
}

#[test]
fn test_manager_zombie_damage_and_repair_loop() {
    let cfg = VehicleConfigData::default();
    let mut manager = Cars::new(cfg);
    manager.spawn_car("test_car", "red", Vec3::ZERO, 0.0, 40.0, 300.0);

    assert_eq!(manager.get("test_car").unwrap().condition.current(), 300.0);
    assert!(!manager.get("test_car").unwrap().condition.is_wrecked());

    // 3 zombies attack for 60 seconds
    let three_zombies = vec![
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(-1.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 2.2),
    ];
    let hits = manager.apply_zombie_damage(&three_zombies, 60.0);
    assert_eq!(hits.len(), 1);
    assert!((hits[0].1 - 300.0).abs() < 1e-4);

    let car = manager.get("test_car").unwrap();
    assert_eq!(car.condition.current(), 0.0);
    assert!(car.condition.is_wrecked());

    // Subsequent damage to wrecked car is 0.0
    let hits_after = manager.apply_zombie_damage(&three_zombies, 1.2);
    assert_eq!(hits_after[0].1, 0.0);

    // [T] Repair with scrap metal revives the wrecked parked car (+50 HP)
    let mut inventory = Inventory::new();
    inventory.add("scrap_metal", 2);

    assert!(manager.repair("test_car", &mut inventory));
    let car_revived = manager.get("test_car").unwrap();
    assert_eq!(car_revived.condition.current(), 50.0);
    assert!(!car_revived.condition.is_wrecked());
    assert_eq!(inventory.count("scrap_metal"), 1);
}
