use game::items::ItemRegistry;
use game::player::Stamina;
use game::survival::{apply_item_effects, update_stamina_regen_multiplier, NeedStat, SurvivalData, WellFedRegen};

#[test]
fn survival_data_loads_from_real_file() {
    let data = SurvivalData::from_repo().expect("data/survival.json loads");

    // Trace from data/survival.json
    assert_eq!(data.hunger.max, 100.0);
    assert_eq!(data.hunger.start, 100.0);
    assert_eq!(data.hunger.drain_per_sec, 0.037);
    assert_eq!(data.hunger.sprint_drain_multiplier, 1.5);
    assert_eq!(data.hunger.empty_damage_per_sec, 1.0);
    assert_eq!(data.hunger.low_threshold, 25.0);

    assert_eq!(data.thirst.max, 100.0);
    assert_eq!(data.thirst.start, 100.0);
    assert_eq!(data.thirst.drain_per_sec, 0.056);
    assert_eq!(data.thirst.sprint_drain_multiplier, 1.5);
    assert_eq!(data.thirst.empty_damage_per_sec, 1.5);
    assert_eq!(data.thirst.low_threshold, 25.0);

    assert_eq!(data.stamina.sprint_cost_per_sec, 15.0);
    assert_eq!(data.stamina.min_to_start_sprint, 10.0);
    assert_eq!(data.stamina.jump_cost, 8.0);
    assert_eq!(data.stamina.low_needs_regen_multiplier, 0.5);

    assert_eq!(data.well_fed.threshold, 80.0);
    assert_eq!(data.well_fed.heal_per_sec, 0.5);
    assert_eq!(data.respawn_needs_value, 75.0);
}

#[test]
fn drain_after_n_game_hours_on_real_data() {
    let data = SurvivalData::from_repo().expect("survival data loads");
    let mut hunger = NeedStat::from_config("hunger", &data).expect("hunger exists");
    let mut thirst = NeedStat::from_config("thirst", &data).expect("thirst exists");

    // 1 in-game hour = 60 real seconds (from TimeOfDay.cs MinutesPerGameHour = 1)
    // 5 in-game hours = 300 seconds
    let five_hours_seconds = 5.0 * 60.0;
    let hunger_damage = hunger.update(five_hours_seconds, false, false);
    let thirst_damage = thirst.update(five_hours_seconds, false, false);

    assert_eq!(hunger_damage, 0.0);
    assert_eq!(thirst_damage, 0.0);

    // Expected: 100 - (300 * 0.037) = 100 - 11.1 = 88.9
    // Expected: 100 - (300 * 0.056) = 100 - 16.8 = 83.2
    assert!((hunger.current() - 88.9).abs() < 1e-4);
    assert!((thirst.current() - 83.2).abs() < 1e-4);

    // Another 5 hours (total 10 hours = 600 s)
    hunger.update(five_hours_seconds, false, false);
    thirst.update(five_hours_seconds, false, false);

    // Expected: 100 - (600 * 0.037) = 100 - 22.2 = 77.8
    // Expected: 100 - (600 * 0.056) = 100 - 33.6 = 66.4
    assert!((hunger.current() - 77.8).abs() < 1e-4);
    assert!((thirst.current() - 66.4).abs() < 1e-4);

    // Dead body does not get hungry or thirsty
    let hunger_before = hunger.current();
    hunger.update(60.0, false, true);
    assert_eq!(hunger.current(), hunger_before);

    // Sprinting drain (1.5x multiplier) for 1 hour (60s)
    let mut sprint_hunger = NeedStat::from_config("hunger", &data).unwrap();
    sprint_hunger.update(60.0, true, false);
    // 100 - (60 * 0.037 * 1.5) = 100 - 3.33 = 96.67
    assert!((sprint_hunger.current() - 96.67).abs() < 1e-4);
}

#[test]
fn starving_and_dehydration_damage_when_empty() {
    let data = SurvivalData::from_repo().expect("survival data loads");
    let mut hunger = NeedStat::from_config("hunger", &data).unwrap();
    let mut thirst = NeedStat::from_config("thirst", &data).unwrap();

    hunger.set(0.0);
    thirst.set(0.0);
    assert!(hunger.is_empty());
    assert!(thirst.is_empty());

    // In 2 seconds:
    // hunger emptyDamagePerSec = 1.0 -> 2.0 damage
    // thirst emptyDamagePerSec = 1.5 -> 3.0 damage (matches EmptyNeedDamagesHealth.cs)
    let hunger_dmg = hunger.update(2.0, false, false);
    let thirst_dmg = thirst.update(2.0, false, false);

    assert!((hunger_dmg - 2.0).abs() < 1e-4);
    assert!((thirst_dmg - 3.0).abs() < 1e-4);
}

#[test]
fn eating_and_drinking_restores_needs() {
    let data = SurvivalData::from_repo().expect("survival data loads");
    let items = ItemRegistry::from_repo().expect("items load");

    let beans_def = items.get("canned_beans").expect("canned_beans exists");
    let water_def = items.get("water_bottle").expect("water_bottle exists");

    let mut hunger = NeedStat::from_config("hunger", &data).unwrap();
    let mut thirst = NeedStat::from_config("thirst", &data).unwrap();

    // Eat canned beans (+35 hunger, matches EatRestoresHunger.cs)
    hunger.set(50.0);
    let hp_delta = apply_item_effects(&beans_def.effects, &mut hunger, &mut thirst, 1.0);
    assert_eq!(hp_delta, 0.0);
    assert!((hunger.current() - 85.0).abs() < 1e-4);

    // Top up to max (85 + 35 = 120 -> clamped to 100.0)
    apply_item_effects(&beans_def.effects, &mut hunger, &mut thirst, 1.0);
    assert_eq!(hunger.current(), 100.0);

    // Drink water bottle (+40 thirst)
    thirst.set(40.0);
    apply_item_effects(&water_def.effects, &mut hunger, &mut thirst, 1.0);
    assert!((thirst.current() - 80.0).abs() < 1e-4);

    // Cooking skill bonus: 50% bonus on positive food value
    hunger.set(10.0);
    apply_item_effects(&beans_def.effects, &mut hunger, &mut thirst, 1.5);
    // 10.0 + (35.0 * 1.5) = 10.0 + 52.5 = 62.5
    assert!((hunger.current() - 62.5).abs() < 1e-4);
}

#[test]
fn well_fed_health_regeneration() {
    let data = SurvivalData::from_repo().expect("survival data loads");
    let mut hunger = NeedStat::from_config("hunger", &data).unwrap();
    let mut thirst = NeedStat::from_config("thirst", &data).unwrap();

    hunger.set(80.0);
    thirst.set(80.0);

    // At threshold (80.0), well-fed heals 0.5 HP/s
    assert!(WellFedRegen::can_heal(&hunger, &thirst, 50.0, 100.0, false, data.well_fed.threshold));
    let heal = WellFedRegen::update(&hunger, &thirst, 50.0, 100.0, false, &data.well_fed, 2.0);
    // 0.5 * 2.0 = 1.0
    assert!((heal - 1.0).abs() < 1e-4);

    // Below threshold on hunger (79.9): no healing
    hunger.set(79.9);
    assert!(!WellFedRegen::can_heal(&hunger, &thirst, 50.0, 100.0, false, data.well_fed.threshold));
    assert_eq!(WellFedRegen::update(&hunger, &thirst, 50.0, 100.0, false, &data.well_fed, 2.0), 0.0);

    // Dead or full health: no healing
    hunger.set(100.0);
    assert_eq!(WellFedRegen::update(&hunger, &thirst, 50.0, 100.0, true, &data.well_fed, 2.0), 0.0);
    assert_eq!(WellFedRegen::update(&hunger, &thirst, 100.0, 100.0, false, &data.well_fed, 2.0), 0.0);
}

#[test]
fn low_needs_slows_stamina_regeneration() {
    let data = SurvivalData::from_repo().expect("survival data loads");
    let mut hunger = NeedStat::from_config("hunger", &data).unwrap();
    let mut thirst = NeedStat::from_config("thirst", &data).unwrap();
    let mut stamina = Stamina::default();

    hunger.set(100.0);
    thirst.set(100.0);
    update_stamina_regen_multiplier(&mut stamina, &hunger, &thirst, data.stamina.low_needs_regen_multiplier);
    assert_eq!(stamina.regen_multiplier, 1.0);

    // Hunger low (threshold is 25.0, so 25.0 is low)
    hunger.set(25.0);
    assert!(hunger.is_low());
    update_stamina_regen_multiplier(&mut stamina, &hunger, &thirst, data.stamina.low_needs_regen_multiplier);
    assert_eq!(stamina.regen_multiplier, 0.5);

    // Reset hunger, drop thirst to low
    hunger.set(50.0);
    thirst.set(15.0);
    assert!(thirst.is_low());
    update_stamina_regen_multiplier(&mut stamina, &hunger, &thirst, data.stamina.low_needs_regen_multiplier);
    assert_eq!(stamina.regen_multiplier, 0.5);
}

#[test]
fn need_save_load_roundtrip() {
    let data = SurvivalData::from_repo().expect("survival data loads");
    let mut hunger = NeedStat::from_config("hunger", &data).unwrap();
    let thirst = NeedStat::from_config("thirst", &data).unwrap();

    assert_eq!(hunger.save_id(), "need_hunger");
    assert_eq!(thirst.save_id(), "need_thirst");

    hunger.set(42.5);
    let saved_hunger = hunger.to_save();
    assert_eq!(saved_hunger, serde_json::json!({ "value": 42.5 }));

    hunger.set(10.0);
    hunger.from_save(&saved_hunger);
    assert!((hunger.current() - 42.5).abs() < 1e-4);

    // Clamping on load
    hunger.from_save(&serde_json::json!({ "value": 150.0 }));
    assert_eq!(hunger.current(), 100.0);
    hunger.from_save(&serde_json::json!({ "value": -20.0 }));
    assert_eq!(hunger.current(), 0.0);
}
