use game::fishing::{can_fish_at, FishingConfig, FishingMachine, FishingPhase};
use game::items::ItemRegistry;
use glam::Vec3;
use paths::repo_path;

#[test]
fn fishing_timing_window_and_state_machine() {
    let cfg = FishingConfig {
        max_water_distance: 3.0,
        cast_wait_min: 2.0,
        cast_wait_max: 4.0,
        bite_duration: 2.0,
        sweet_spot_min: 0.35,
        sweet_spot_max: 0.65,
    };

    // 1. Initial state is Idle
    let mut machine = FishingMachine::new(&cfg);
    assert_eq!(machine.phase, FishingPhase::Idle);

    // 2. Start cast transitions to Waiting
    assert!(machine.start_cast(2.0));
    assert_eq!(machine.phase, FishingPhase::Waiting);

    // Premature pull during waiting resets to Idle
    let mut premature = machine.clone();
    assert!(!premature.pull_rod());
    assert_eq!(premature.phase, FishingPhase::Idle);

    // 3. Waiting progresses and transitions to Bite when timer elapses
    let ev = machine.update(1.0);
    assert_eq!(ev, None);
    assert_eq!(machine.phase, FishingPhase::Waiting);

    let ev2 = machine.update(1.1);
    assert_eq!(ev2, Some(FishingPhase::Bite));
    assert_eq!(machine.phase, FishingPhase::Bite);

    // 4. Bite phase: pull in sweet spot catches fish
    let mut catch_machine = machine.clone();
    // Progress is bite_timer / bite_duration. Advance 1.0s -> progress = 0.5 (between 0.35 and 0.65)
    catch_machine.update(1.0);
    assert!(catch_machine.is_in_sweet_spot());
    assert!(catch_machine.pull_rod());
    assert_eq!(catch_machine.phase, FishingPhase::Caught);

    // 5. Bite phase: pull too early escapes
    let mut early_machine = machine.clone();
    early_machine.update(0.2); // progress = 0.1 (< 0.35)
    assert!(!early_machine.is_in_sweet_spot());
    assert!(!early_machine.pull_rod());
    assert_eq!(early_machine.phase, FishingPhase::Escaped);

    // 6. Bite phase: timeout escapes
    let mut timeout_machine = machine.clone();
    let ev_timeout = timeout_machine.update(2.1);
    assert_eq!(ev_timeout, Some(FishingPhase::Escaped));
    assert_eq!(timeout_machine.phase, FishingPhase::Escaped);
}

#[test]
fn can_fish_at_water() {
    let lake_cfg = world::LakeConfigData::load_from_file(repo_path("data/lake.json"))
        .unwrap_or_default();
    let lake = world::LakeLayout::generate(42, glam::Vec2::ZERO, None, &lake_cfg, |_| 0.0);

    // Shore point facing toward lake center: can fish
    if let Some(&shore_pt) = lake.shore.first() {
        let to_center = (lake.center - shore_pt).normalize();
        let yaw = (-to_center.x).atan2(-to_center.y);
        let player_pos = Vec3::new(shore_pt.x, lake.water_level, shore_pt.y);
        assert!(can_fish_at(player_pos, yaw, &lake, 3.0));

        // Facing away from lake: cannot fish
        let opposite_yaw = yaw + std::f32::consts::PI;
        assert!(!can_fish_at(player_pos, opposite_yaw, &lake, 3.0));
    }

    // Far away on land: cannot fish
    let far_land = Vec3::new(10000.0, 0.0, 10000.0);
    assert!(!can_fish_at(far_land, 0.0, &lake, 3.0));
}

#[test]
fn fishing_items_exist_in_data() {
    let registry = ItemRegistry::load(&repo_path("data/items.json")).expect("items.json loads");

    let rod = registry.get("fishing_rod").expect("fishing_rod exists");
    assert_eq!(rod.category, game::items::ItemCategory::Tool);

    let raw = registry.get("fish_raw").expect("fish_raw exists");
    assert_eq!(raw.category, game::items::ItemCategory::Food);

    let cooked = registry.get("fish_cooked").expect("fish_cooked exists");
    assert_eq!(cooked.category, game::items::ItemCategory::Food);

    assert!(registry.get("dirty_water").is_some());
    assert!(registry.get("water_bottle").is_some());
}

#[test]
fn recipes_reference_real_items() {
    let registry = ItemRegistry::load(&repo_path("data/items.json")).expect("items.json loads");
    let recipes = game::crafting::RecipeRegistry::load(&repo_path("data/recipes.json"))
        .expect("recipes.json loads");

    // Check all recipes in recipes.json reference valid items
    for recipe in recipes.all() {
        for input in &recipe.inputs {
            assert!(
                registry.get(&input.id).is_some(),
                "recipe {} input {} does not exist in items.json",
                recipe.id,
                input.id
            );
        }
        for output in &recipe.outputs {
            assert!(
                registry.get(&output.id).is_some(),
                "recipe {} output {} does not exist in items.json",
                recipe.id,
                output.id
            );
        }
    }

    // Check fish_cooked recipe specifically
    let fish_recipe = recipes.get("fish_cooked").expect("fish_cooked recipe exists");
    assert_eq!(fish_recipe.station, "campfire");
    assert_eq!(fish_recipe.inputs[0].id, "fish_raw");
    assert_eq!(fish_recipe.outputs[0].id, "fish_cooked");

    // Check water_bottle recipe specifically
    let water_recipe = recipes.get("water_bottle").expect("water_bottle recipe exists");
    assert_eq!(water_recipe.station, "campfire");
    assert_eq!(water_recipe.inputs[0].id, "dirty_water");
    assert_eq!(water_recipe.outputs[0].id, "water_bottle");
}

#[test]
fn loot_tables_include_fishing_rod() {
    let text = std::fs::read_to_string(repo_path("data/loot_tables.json")).expect("loot_tables.json");
    let tables: serde_json::Value = serde_json::from_str(&text).expect("valid json");

    let house_entries = tables["house"]["entries"].as_array().expect("house entries");
    assert!(
        house_entries.iter().any(|e| e["id"] == "fishing_rod"),
        "house loot table must contain fishing_rod"
    );

    let cabin_entries = tables["cabin"]["entries"].as_array().expect("cabin entries");
    assert!(
        cabin_entries.iter().any(|e| e["id"] == "fishing_rod"),
        "cabin loot table must contain fishing_rod"
    );
}

#[test]
fn fishing_config_loads_from_repo() {
    let cfg = FishingConfig::from_repo().expect("fishing.json must load");
    assert_eq!(cfg.max_water_distance, 3.0);
    assert_eq!(cfg.cast_wait_min, 2.0);
    assert_eq!(cfg.cast_wait_max, 5.0);
    assert_eq!(cfg.bite_duration, 2.0);
    assert_eq!(cfg.sweet_spot_min, 0.35);
    assert_eq!(cfg.sweet_spot_max, 0.65);
}
