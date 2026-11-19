use game::difficulty::{
    tally_inventory, DeathRule, DifficultyConfigData, DifficultySettings, RunDifficulty,
};
use game::items::{Inventory, ItemRegistry};

#[test]
fn difficulty_config_loads_and_validates() {
    let config = DifficultyConfigData::from_repo().expect("difficulty.json loads");
    assert!(config.validate().is_ok());

    assert_eq!(config.min_multiplier, 0.5);
    assert_eq!(config.max_multiplier, 2.0);
    assert_eq!(config.step, 0.25);
    assert_eq!(config.default_preset, "normal");

    assert_eq!(config.presets.len(), 3);
    assert_eq!(config.death_rules.len(), 3);

    let relaxed = config.preset("relaxed").expect("relaxed exists");
    assert_eq!(relaxed.name, "Relaxed");
    assert_eq!(relaxed.zombies, 0.5);
    assert_eq!(relaxed.loot, 1.5);
    assert_eq!(relaxed.damage, 0.5);
    assert_eq!(relaxed.death, "keep_all");

    let normal = config.preset("normal").expect("normal exists");
    assert_eq!(normal.name, "Normal");
    assert_eq!(normal.zombies, 1.0);
    assert_eq!(normal.loot, 1.0);
    assert_eq!(normal.damage, 1.0);
    assert_eq!(normal.death, "lose_all");

    let hard = config.preset("hard").expect("hard exists");
    assert_eq!(hard.name, "Hard");
    assert_eq!(hard.zombies, 1.5);
    assert_eq!(hard.loot, 0.75);
    assert_eq!(hard.damage, 1.5);
    assert_eq!(hard.death, "lose_all");

    assert_eq!(config.death_rule_name(DeathRule::LoseAll), "Lose everything");
    assert_eq!(config.death_rule_name(DeathRule::KeepHalf), "Keep half");
    assert_eq!(config.death_rule_name(DeathRule::KeepAll), "Keep all");
}

fn create_sample_inventory() -> Inventory {
    let items = ItemRegistry::from_repo().expect("items registry");
    let mut inv = Inventory::with_capacity(10, 50.0);

    // canned_beans: 3 items, value 7 each -> 21
    inv.add_with_registry("canned_beans", 3, &items);
    // bandage: 2 items, value 10 each -> 20
    inv.add_with_registry("bandage", 2, &items);
    // water_bottle: 1 item, value 6 each -> 6
    inv.add_with_registry("water_bottle", 1, &items);
    // pistol_ammo: 30 items, value 1 each -> 30
    inv.add_with_registry("pistol_ammo", 30, &items);

    inv
}

#[test]
fn death_rule_lose_all_clears_inventory() {
    let mut inv = create_sample_inventory();

    let (initial_count, initial_value) = tally_inventory(&inv);
    // 3 + 2 + 1 + 30 = 36 items
    assert_eq!(initial_count, 36);
    // 21 + 20 + 6 + 30 = 77 dollars
    assert_eq!(initial_value, 77);

    let res = DeathRule::LoseAll.apply_to_inventory_with_seed(&mut inv, 12345);
    assert_eq!(res.lost_count, 36);
    assert_eq!(res.lost_value, 77);

    // All slots must be empty
    assert!(inv.slots().iter().all(|s| s.is_none()));
}

#[test]
fn death_rule_keep_all_preserves_inventory() {
    let mut inv = create_sample_inventory();

    let res = DeathRule::KeepAll.apply_to_inventory_with_seed(&mut inv, 12345);
    assert_eq!(res.lost_count, 0);
    assert_eq!(res.lost_value, 0);

    let (count, value) = tally_inventory(&inv);
    assert_eq!(count, 36);
    assert_eq!(value, 77);
    assert_eq!(inv.slots().iter().filter(|s| s.is_some()).count(), 4);
}

#[test]
fn death_rule_keep_half_removes_half_stacks() {
    let mut inv = create_sample_inventory();
    assert_eq!(inv.slots().iter().filter(|s| s.is_some()).count(), 4);

    let res = DeathRule::KeepHalf.apply_to_inventory_with_seed(&mut inv, 42);
    // 4 stacks / 2 = 2 kept, 2 lost
    let remaining_stacks = inv.slots().iter().filter(|s| s.is_some()).count();
    assert_eq!(remaining_stacks, 2);

    let (remaining_count, remaining_value) = tally_inventory(&inv);
    assert_eq!(res.lost_count + remaining_count, 36);
    assert_eq!(res.lost_value + remaining_value, 77);
}

#[test]
fn run_difficulty_scales_bite_damage_to_player() {
    let config = DifficultyConfigData::from_repo().expect("config");

    let normal_settings = DifficultySettings::default_preset(&config);
    let run_normal = RunDifficulty::new(normal_settings);
    assert_eq!(run_normal.scale_incoming("bite", true, 20.0), 20.0);
    assert_eq!(run_normal.scale_incoming("bite", false, 20.0), 20.0);
    assert_eq!(run_normal.scale_incoming("fall", true, 20.0), 20.0);

    let relaxed_preset = config.preset("relaxed").unwrap();
    let relaxed_settings = DifficultySettings::from_preset(relaxed_preset, &config);
    let run_relaxed = RunDifficulty::new(relaxed_settings);
    // 20.0 * 0.5 = 10.0
    assert_eq!(run_relaxed.scale_incoming("bite", true, 20.0), 10.0);
    assert_eq!(run_relaxed.scale_incoming("bite", false, 20.0), 20.0);
    assert_eq!(run_relaxed.scale_incoming("fall", true, 20.0), 20.0);

    let hard_preset = config.preset("hard").unwrap();
    let hard_settings = DifficultySettings::from_preset(hard_preset, &config);
    let run_hard = RunDifficulty::new(hard_settings);
    // 20.0 * 1.5 = 30.0
    assert_eq!(run_hard.scale_incoming("bite", true, 20.0), 30.0);
    assert_eq!(run_hard.scale_incoming("bullet", true, 20.0), 20.0);
}

#[test]
fn difficulty_settings_labels_and_save_roundtrip() {
    let config = DifficultyConfigData::from_repo().expect("config");

    let normal = DifficultySettings::default_preset(&config);
    assert_eq!(normal.label(&config), "Normal");

    let relaxed = DifficultySettings::from_preset(config.preset("relaxed").unwrap(), &config);
    assert_eq!(relaxed.label(&config), "Relaxed");

    let hard = DifficultySettings::from_preset(config.preset("hard").unwrap(), &config);
    assert_eq!(hard.label(&config), "Hard");

    let mut custom = normal;
    custom.zombie_amount = 1.75;
    assert_eq!(custom.label(&config), "Custom");

    // Save and load round trip
    let saved = hard.to_save();
    assert_eq!(
        saved,
        serde_json::json!({
            "zombies": 1.5,
            "loot": 0.75,
            "damage": 1.5,
            "death": "lose_all"
        })
    );

    let loaded = DifficultySettings::from_save(&saved, &config);
    assert!(loaded.same_as(&hard));

    // Clamping on load
    let out_of_bounds = serde_json::json!({
        "zombies": 0.1,
        "loot": 5.0,
        "damage": 1.0,
        "death": "keep_all"
    });
    let clamped = DifficultySettings::from_save(&out_of_bounds, &config);
    assert_eq!(clamped.zombie_amount, 0.5); // clamped to minMultiplier
    assert_eq!(clamped.loot_amount, 2.0); // clamped to maxMultiplier
    assert_eq!(clamped.death, DeathRule::KeepAll);
}
