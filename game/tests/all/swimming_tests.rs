use game::swimming::{
    evaluate_water, speed_multiplier_for_zone, try_gather_dirty_water, SwimmingConfig,
};
use glam::Vec3;
use world::LakeZone;

#[test]
fn zones_to_speed_multipliers() {
    let cfg = SwimmingConfig::default();
    assert_eq!(speed_multiplier_for_zone(LakeZone::Land, &cfg), 1.0);
    assert_eq!(speed_multiplier_for_zone(LakeZone::Beach, &cfg), 1.0);
    assert_eq!(speed_multiplier_for_zone(LakeZone::Shallow, &cfg), 0.6);
    assert_eq!(speed_multiplier_for_zone(LakeZone::Deep, &cfg), 0.45);
}

#[test]
fn drowning_and_deep_water_effects() {
    let cfg = SwimmingConfig::default();
    let pos = Vec3::new(100.0, 10.0, 100.0);
    let water_level = 12.0;
    let dt = 0.1;

    // 1. Deep water with stamina: drains stamina, no drowning damage yet
    let res_stamina = evaluate_water(pos, LakeZone::Deep, water_level, 50.0, false, &cfg, dt);
    assert_eq!(res_stamina.speed_multiplier, 0.45);
    assert!(!res_stamina.can_sprint);
    assert!(!res_stamina.can_jump);
    assert!(res_stamina.disarm_weapons);
    assert!(res_stamina.stamina_drain > 0.0);
    assert_eq!(res_stamina.drowning_damage, 0.0);
    assert_eq!(res_stamina.float_y, Some(water_level - cfg.chest_height));

    // 2. Deep water with 0 stamina: takes drowning damage
    let res_drown = evaluate_water(pos, LakeZone::Deep, water_level, 0.0, false, &cfg, dt);
    assert_eq!(res_drown.drowning_damage, cfg.drowning_damage_per_sec * dt);
    assert!(res_drown.drowning_damage > 0.0);

    // 3. Shallow water: slow, but can sprint, can jump, weapons kept, no drowning even with 0 stamina
    let res_shallow = evaluate_water(pos, LakeZone::Shallow, water_level, 0.0, false, &cfg, dt);
    assert_eq!(res_shallow.speed_multiplier, 0.6);
    assert!(res_shallow.can_sprint);
    assert!(res_shallow.can_jump);
    assert!(!res_shallow.disarm_weapons);
    assert_eq!(res_shallow.drowning_damage, 0.0);
    assert_eq!(res_shallow.float_y, None);

    // 4. Land: full speed, no water effects
    let res_land = evaluate_water(pos, LakeZone::Land, water_level, 0.0, false, &cfg, dt);
    assert_eq!(res_land.speed_multiplier, 1.0);
    assert!(res_land.can_sprint);
    assert!(res_land.can_jump);
    assert!(!res_land.disarm_weapons);
    assert_eq!(res_land.drowning_damage, 0.0);
}

#[test]
fn dock_prevents_submersion() {
    let cfg = SwimmingConfig::default();
    let pos = Vec3::new(100.0, 12.0, 100.0);
    let water_level = 12.0;
    let dt = 0.1;

    let res = evaluate_water(pos, LakeZone::Deep, water_level, 0.0, true, &cfg, dt);
    assert_eq!(res.speed_multiplier, 1.0);
    assert!(res.can_sprint);
    assert!(res.can_jump);
    assert!(!res.disarm_weapons);
    assert_eq!(res.drowning_damage, 0.0);
    assert_eq!(res.float_y, None);
}

#[test]
fn player_applies_water_movement_and_floating() {
    let cfg = SwimmingConfig::default();
    let mut player = game::Player::new(Vec3::new(0.0, 5.0, 0.0));
    let water_level = 10.0;
    let dt = 0.1;

    let res = evaluate_water(
        player.position,
        LakeZone::Deep,
        water_level,
        player.stamina.current,
        false,
        &cfg,
        dt,
    );
    player.apply_water_movement(&res);

    assert_eq!(player.speed_multiplier, 0.45);
    assert!(!player.can_sprint);
    assert!(!player.can_jump);
    assert!(player.in_deep_water);
    let expected_float = water_level - cfg.chest_height;
    assert_eq!(player.water_float_y, Some(expected_float));
}

#[test]
fn dirty_water_gathering_in_shallow_water() {
    let mut cooldown = 0.0;
    let cooldown_sec = 2.0;

    // Can gather in shallow water when cooldown is 0
    assert!(try_gather_dirty_water(LakeZone::Shallow, &mut cooldown, cooldown_sec));
    assert_eq!(cooldown, 2.0);

    // Cannot gather while cooldown is active
    assert!(!try_gather_dirty_water(LakeZone::Shallow, &mut cooldown, cooldown_sec));

    // Cannot gather on land or deep
    let mut zero_cd = 0.0;
    assert!(!try_gather_dirty_water(LakeZone::Land, &mut zero_cd, cooldown_sec));
    assert!(!try_gather_dirty_water(LakeZone::Deep, &mut zero_cd, cooldown_sec));
}

#[test]
fn swimming_config_loads_from_repo() {
    let cfg = SwimmingConfig::from_repo().expect("swimming.json must load");
    assert_eq!(cfg.shallow_speed_multiplier, 0.6);
    assert_eq!(cfg.deep_speed_multiplier, 0.45);
    assert_eq!(cfg.chest_height, 1.2);
    assert_eq!(cfg.dirty_water_cooldown_sec, 2.0);
}
