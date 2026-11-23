use game::swim_hands::{SwimHands, SwimHandsChange};
use game::swimming::{
    evaluate_water, speed_multiplier_for_zone, try_gather_dirty_water, water_warning,
    SwimmingConfig, WaterWarning,
};
use glam::Vec3;
use world::LakeZone;

#[test]
fn zones_to_speed_multipliers() {
    let cfg = SwimmingConfig::default();
    assert_eq!(speed_multiplier_for_zone(LakeZone::Land, &cfg), 1.0);
    assert_eq!(speed_multiplier_for_zone(LakeZone::Beach, &cfg), 1.0);
    // data: swimming.json shallowSpeedMultiplier
    assert_eq!(speed_multiplier_for_zone(LakeZone::Shallow, &cfg), 0.6);
    // data: swimming.json deepSpeedMultiplier
    assert_eq!(speed_multiplier_for_zone(LakeZone::Deep, &cfg), 0.45);
}

#[test]
fn drowning_and_deep_water_effects() {
    let cfg = SwimmingConfig::default();
    // new: DECISIONS 2026-11-23
    let pos = Vec3::new(100.0, 10.0, 100.0);
    // new: DECISIONS 2026-11-23
    let water_level = 12.0;
    // new: DECISIONS 2026-11-23
    let dt = 0.1;

    // 1. Deep water with stamina: drains stamina, no drowning damage yet
    // new: DECISIONS 2026-11-23
    let mid_stamina = 50.0;
    let res_stamina = evaluate_water(pos, LakeZone::Deep, water_level, mid_stamina, false, &cfg, dt);
    // data: swimming.json deepSpeedMultiplier
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
    // data: swimming.json shallowSpeedMultiplier
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
    // new: DECISIONS 2026-11-23
    let pos = Vec3::new(100.0, 12.0, 100.0);
    // new: DECISIONS 2026-11-23
    let water_level = 12.0;
    // new: DECISIONS 2026-11-23
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
    // new: DECISIONS 2026-11-23
    let mut player = game::Player::new(Vec3::new(0.0, 5.0, 0.0));
    // new: DECISIONS 2026-11-23
    let water_level = 10.0;
    // new: DECISIONS 2026-11-23
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

    // data: swimming.json deepSpeedMultiplier
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
    // data: swimming.json dirtyWaterCooldownSec
    let cooldown_sec = 2.0;

    // Can gather in shallow water when cooldown is 0
    assert!(try_gather_dirty_water(LakeZone::Shallow, &mut cooldown, cooldown_sec));
    assert_eq!(cooldown, cooldown_sec);

    // Cannot gather while cooldown is active
    assert!(!try_gather_dirty_water(LakeZone::Shallow, &mut cooldown, cooldown_sec));

    // Cannot gather on land or deep
    let mut zero_cd = 0.0;
    assert!(!try_gather_dirty_water(LakeZone::Land, &mut zero_cd, cooldown_sec));
    assert!(!try_gather_dirty_water(LakeZone::Deep, &mut zero_cd, cooldown_sec));
}

#[test]
fn swim_hands_enter_leave_empty_and_clear() {
    // 1. Entering with an item stows once; leaving restores once
    let mut hands = SwimHands::new();
    assert_eq!(hands.update(false, Some("fishing_rod")), None);
    assert_eq!(hands.stowed_item(), None);

    assert_eq!(
        hands.update(true, Some("fishing_rod")),
        Some(SwimHandsChange::Stow)
    );
    assert_eq!(hands.stowed_item(), Some("fishing_rod"));
    // Staying in deep water does not fire Stow again
    assert_eq!(hands.update(true, None), None);
    assert_eq!(hands.update(true, Some("fishing_rod")), None);

    // Leaving deep water restores once
    assert_eq!(
        hands.update(false, None),
        Some(SwimHandsChange::Restore("fishing_rod".to_owned()))
    );
    assert_eq!(hands.stowed_item(), None);
    assert_eq!(hands.update(false, Some("fishing_rod")), None);

    // 2. Entering with empty hands produces no change on enter or leave
    assert_eq!(hands.update(true, None), None);
    assert_eq!(hands.update(true, None), None);
    assert_eq!(hands.update(false, None), None);

    // 3. Clearing on death forgets stowed item and resets deep-water state
    assert_eq!(hands.update(true, Some("axe")), Some(SwimHandsChange::Stow));
    hands.clear();
    assert_eq!(hands.stowed_item(), None);
    assert_eq!(hands.update(false, None), None);

    // Can stow and restore a fresh item after respawn
    assert_eq!(
        hands.update(true, Some("knife")),
        Some(SwimHandsChange::Stow)
    );
    assert_eq!(
        hands.update(false, None),
        Some(SwimHandsChange::Restore("knife".to_owned()))
    );
}

#[test]
fn water_warning_cases() {
    let cfg = SwimmingConfig::default();
    // new: DECISIONS 2026-11-23
    let max_stamina = 100.0;

    // Not in deep water: never warns, even at low or zero stamina
    assert_eq!(water_warning(max_stamina, max_stamina, false, &cfg), WaterWarning::None);
    // new: DECISIONS 2026-11-23
    let low_stamina = 20.0;
    assert_eq!(water_warning(low_stamina, max_stamina, false, &cfg), WaterWarning::None);
    assert_eq!(water_warning(0.0, max_stamina, false, &cfg), WaterWarning::None);

    // Deep water at or above tired threshold: None
    assert_eq!(water_warning(max_stamina, max_stamina, true, &cfg), WaterWarning::None);
    let threshold_stamina = max_stamina * cfg.tired_stamina_fraction;
    assert_eq!(
        water_warning(threshold_stamina, max_stamina, true, &cfg),
        WaterWarning::None
    );

    // Deep water below tired threshold (and > 0): Tired
    assert_eq!(
        water_warning(low_stamina, max_stamina, true, &cfg),
        WaterWarning::Tired
    );

    // Deep water at 0 (or negative) stamina: Drowning
    assert_eq!(water_warning(0.0, max_stamina, true, &cfg), WaterWarning::Drowning);
    // new: DECISIONS 2026-11-23
    assert_eq!(water_warning(-5.0, max_stamina, true, &cfg), WaterWarning::Drowning);
}

#[test]
fn swimming_config_loads_from_repo() {
    let cfg = SwimmingConfig::from_repo().expect("swimming.json must load");
    // data: swimming.json shallowSpeedMultiplier
    assert_eq!(cfg.shallow_speed_multiplier, 0.6);
    // data: swimming.json deepSpeedMultiplier
    assert_eq!(cfg.deep_speed_multiplier, 0.45);
    // data: swimming.json chestHeight
    assert_eq!(cfg.chest_height, 1.2);
    // data: swimming.json dirtyWaterCooldownSec
    assert_eq!(cfg.dirty_water_cooldown_sec, 2.0);
    // data: swimming.json tiredStaminaFraction
    assert_eq!(cfg.tired_stamina_fraction, 0.3);
}
