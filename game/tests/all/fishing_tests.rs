use game::fishing::{
    can_fish_at, FishingConfig, FishingEvent, FishingInput, FishingMachine, FishingPhase,
    FishingSession,
};
use game::items::ItemRegistry;
use glam::Vec3;
use paths::repo_path;

fn idle_input() -> FishingInput {
    FishingInput {
        has_rod: true,
        can_fish_here: true,
        click: false,
        hurt: false,
        // new: DECISIONS 2026-11-23
        random_01: 0.0,
    }
}

#[test]
fn fishing_timing_window_and_state_machine() {
    let cfg = FishingConfig {
        // data: fishing.json maxWaterDistance
        max_water_distance: 3.0,
        // data: fishing.json castWaitMin
        cast_wait_min: 2.0,
        // new: DECISIONS 2026-11-23
        cast_wait_max: 4.0,
        // data: fishing.json biteDuration
        bite_duration: 2.0,
        // data: fishing.json sweetSpotMin
        sweet_spot_min: 0.35,
        // data: fishing.json sweetSpotMax
        sweet_spot_max: 0.65,
        // data: fishing.json resultSeconds
        result_seconds: 2.0,
    };

    // 1. Initial state is Idle
    let mut machine = FishingMachine::new(&cfg);
    assert_eq!(machine.phase, FishingPhase::Idle);

    // 2. Start cast transitions to Waiting
    // data: fishing.json castWaitMin
    assert!(machine.start_cast(2.0));
    assert_eq!(machine.phase, FishingPhase::Waiting);

    // Premature pull during waiting resets to Idle
    let mut premature = machine.clone();
    assert!(!premature.pull_rod());
    assert_eq!(premature.phase, FishingPhase::Idle);

    // 3. Waiting progresses and transitions to Bite when timer elapses
    // new: DECISIONS 2026-11-23
    let ev = machine.update(1.0);
    assert_eq!(ev, None);
    assert_eq!(machine.phase, FishingPhase::Waiting);

    // new: DECISIONS 2026-11-23
    let ev2 = machine.update(1.1);
    assert_eq!(ev2, Some(FishingPhase::Bite));
    assert_eq!(machine.phase, FishingPhase::Bite);

    // 4. Bite phase: pull in sweet spot catches fish
    let mut catch_machine = machine.clone();
    // Progress is bite_timer / bite_duration. Advance 1.0s -> progress = 0.5 (between 0.35 and 0.65)
    // new: DECISIONS 2026-11-23
    catch_machine.update(1.0);
    assert!(catch_machine.is_in_sweet_spot());
    assert!(catch_machine.pull_rod());
    assert_eq!(catch_machine.phase, FishingPhase::Caught);

    // 5. Bite phase: pull too early escapes
    let mut early_machine = machine.clone();
    // new: DECISIONS 2026-11-23
    early_machine.update(0.2); // progress = 0.1 (< 0.35)
    assert!(!early_machine.is_in_sweet_spot());
    assert!(!early_machine.pull_rod());
    assert_eq!(early_machine.phase, FishingPhase::Escaped);

    // 6. Bite phase: timeout escapes
    let mut timeout_machine = machine.clone();
    // new: DECISIONS 2026-11-23
    let ev_timeout = timeout_machine.update(2.1);
    assert_eq!(ev_timeout, Some(FishingPhase::Escaped));
    assert_eq!(timeout_machine.phase, FishingPhase::Escaped);
}

#[test]
fn session_full_catch_and_caught_only_once() {
    let cfg = FishingConfig::default();
    let mut session = FishingSession::new(cfg);

    // Cast with random_01 = 0.0 -> wait duration = cast_wait_min (2.0s)
    let cast_in = FishingInput {
        click: true,
        ..idle_input()
    };
    // new: DECISIONS 2026-11-23
    let dt_step = 0.1;
    assert_eq!(session.tick(&cast_in, dt_step), Some(FishingEvent::Cast));
    assert_eq!(session.machine().phase, FishingPhase::Waiting);

    // Wait for bite
    let wait_in = idle_input();
    // data: fishing.json castWaitMin
    assert_eq!(session.tick(&wait_in, 2.0), Some(FishingEvent::Bite));
    assert_eq!(session.machine().phase, FishingPhase::Bite);

    // Advance needle to center of sweet spot (1.0s / 2.0s = 0.5)
    // new: DECISIONS 2026-11-23
    assert_eq!(session.tick(&wait_in, 1.0), None);
    assert!(session.machine().is_in_sweet_spot());

    // Click inside sweet spot -> Caught emitted once
    let mut caught_count = 0;
    if session.tick(&cast_in, dt_step) == Some(FishingEvent::Caught) {
        caught_count += 1;
    }
    assert_eq!(caught_count, 1);
    assert_eq!(session.machine().phase, FishingPhase::Caught);

    // Clicking or ticking again during resultSeconds must NOT emit Caught again
    // new: DECISIONS 2026-11-23
    for _ in 0..10 {
        let ev = session.tick(&cast_in, dt_step);
        assert_ne!(ev, Some(FishingEvent::Caught));
        assert_eq!(ev, None);
    }
    assert_eq!(session.machine().phase, FishingPhase::Caught);

    // Finish resultSeconds -> returns to Idle so player can cast again
    // data: fishing.json resultSeconds
    assert_eq!(session.tick(&wait_in, cfg.result_seconds), None);
    assert_eq!(session.machine().phase, FishingPhase::Idle);

    // Can cast again cleanly
    assert_eq!(session.tick(&cast_in, dt_step), Some(FishingEvent::Cast));
    assert_eq!(session.machine().phase, FishingPhase::Waiting);
}

#[test]
fn session_miss_outside_green_zone() {
    let cfg = FishingConfig::default();
    let click_in = FishingInput {
        click: true,
        ..idle_input()
    };
    let wait_in = idle_input();
    // new: DECISIONS 2026-11-23
    let dt_step = 0.1;

    // 1. Click too early (before sweet_spot_min)
    let mut early = FishingSession::new(cfg);
    assert_eq!(early.tick(&click_in, dt_step), Some(FishingEvent::Cast));
    // data: fishing.json castWaitMin
    assert_eq!(early.tick(&wait_in, 2.0), Some(FishingEvent::Bite));
    // new: DECISIONS 2026-11-23
    assert_eq!(early.tick(&wait_in, 0.2), None); // progress = 0.1 < 0.35
    assert_eq!(early.tick(&click_in, dt_step), Some(FishingEvent::Escaped));
    assert_eq!(early.machine().phase, FishingPhase::Escaped);

    // 2. Click too late (after sweet_spot_max)
    let mut late = FishingSession::new(cfg);
    assert_eq!(late.tick(&click_in, dt_step), Some(FishingEvent::Cast));
    // data: fishing.json castWaitMin
    assert_eq!(late.tick(&wait_in, 2.0), Some(FishingEvent::Bite));
    // new: DECISIONS 2026-11-23
    assert_eq!(late.tick(&wait_in, 1.6), None); // progress = 0.8 > 0.65
    assert_eq!(late.tick(&click_in, dt_step), Some(FishingEvent::Escaped));
    assert_eq!(late.machine().phase, FishingPhase::Escaped);
}

#[test]
fn session_too_late_timeout_escapes() {
    let cfg = FishingConfig::default();
    let mut session = FishingSession::new(cfg);
    let click_in = FishingInput {
        click: true,
        ..idle_input()
    };
    let wait_in = idle_input();
    // new: DECISIONS 2026-11-23
    let dt_step = 0.1;

    assert_eq!(session.tick(&click_in, dt_step), Some(FishingEvent::Cast));
    // data: fishing.json castWaitMin
    assert_eq!(session.tick(&wait_in, 2.0), Some(FishingEvent::Bite));
    // data: fishing.json biteDuration
    assert_eq!(
        session.tick(&wait_in, cfg.bite_duration),
        Some(FishingEvent::Escaped)
    );
    assert_eq!(session.machine().phase, FishingPhase::Escaped);
}

#[test]
fn session_click_during_wait_reels_in_without_penalty() {
    let cfg = FishingConfig::default();
    let mut session = FishingSession::new(cfg);
    let click_in = FishingInput {
        click: true,
        ..idle_input()
    };
    // new: DECISIONS 2026-11-23
    let dt_step = 0.1;

    assert_eq!(session.tick(&click_in, dt_step), Some(FishingEvent::Cast));
    assert_eq!(session.machine().phase, FishingPhase::Waiting);

    // Click during wait cancels cleanly to Idle (not Escaped)
    assert_eq!(
        session.tick(&click_in, dt_step),
        Some(FishingEvent::Cancelled)
    );
    assert_eq!(session.machine().phase, FishingPhase::Idle);
}

#[test]
fn session_cancel_reasons() {
    let cfg = FishingConfig::default();
    let click_in = FishingInput {
        click: true,
        ..idle_input()
    };
    let wait_in = idle_input();
    // new: DECISIONS 2026-11-23
    let dt_step = 0.1;

    let reasons = [
        // Walking away from water
        FishingInput {
            can_fish_here: false,
            ..idle_input()
        },
        // Entering deep water (cannot fish here and rod stowed)
        FishingInput {
            has_rod: false,
            can_fish_here: false,
            ..idle_input()
        },
        // Getting hurt
        FishingInput {
            hurt: true,
            ..idle_input()
        },
        // Putting the rod away
        FishingInput {
            has_rod: false,
            ..idle_input()
        },
    ];

    for cancel_in in reasons {
        // Cancel during Waiting
        let mut waiting = FishingSession::new(cfg);
        assert_eq!(waiting.tick(&click_in, dt_step), Some(FishingEvent::Cast));
        assert_eq!(
            waiting.tick(&cancel_in, dt_step),
            Some(FishingEvent::Cancelled)
        );
        assert_eq!(waiting.machine().phase, FishingPhase::Idle);

        // Cancel during Bite
        let mut biting = FishingSession::new(cfg);
        assert_eq!(biting.tick(&click_in, dt_step), Some(FishingEvent::Cast));
        // data: fishing.json castWaitMin
        assert_eq!(biting.tick(&wait_in, 2.0), Some(FishingEvent::Bite));
        assert_eq!(
            biting.tick(&cancel_in, dt_step),
            Some(FishingEvent::Cancelled)
        );
        assert_eq!(biting.machine().phase, FishingPhase::Idle);
    }
}

#[test]
fn session_wants_click_in_each_phase() {
    let cfg = FishingConfig::default();
    let mut session = FishingSession::new(cfg);
    // new: DECISIONS 2026-11-23
    let dt_step = 0.1;

    // Idle (initial / away from water / no rod): does not claim click
    assert_eq!(session.machine().phase, FishingPhase::Idle);
    assert!(!session.wants_click());
    let no_rod = FishingInput {
        has_rod: false,
        ..idle_input()
    };
    session.tick(&no_rod, dt_step);
    assert!(!session.wants_click());

    // Idle while holding rod and facing water: claims click to cast
    let wait_in = idle_input();
    session.tick(&wait_in, dt_step);
    assert_eq!(session.machine().phase, FishingPhase::Idle);
    assert!(session.wants_click());

    // Waiting: claims click
    let click_in = FishingInput {
        click: true,
        ..idle_input()
    };
    assert_eq!(session.tick(&click_in, dt_step), Some(FishingEvent::Cast));
    assert_eq!(session.machine().phase, FishingPhase::Waiting);
    assert!(session.wants_click());

    // Bite: claims click
    // data: fishing.json castWaitMin
    assert_eq!(session.tick(&wait_in, 2.0), Some(FishingEvent::Bite));
    assert_eq!(session.machine().phase, FishingPhase::Bite);
    assert!(session.wants_click());

    // Caught: claims click while holding rod at water, releases once rod put away
    let mut caught_session = session.clone();
    // new: DECISIONS 2026-11-23
    caught_session.tick(&wait_in, 1.0);
    assert_eq!(
        caught_session.tick(&click_in, dt_step),
        Some(FishingEvent::Caught)
    );
    assert_eq!(caught_session.machine().phase, FishingPhase::Caught);
    assert!(caught_session.wants_click());
    caught_session.tick(&no_rod, dt_step);
    assert_eq!(caught_session.machine().phase, FishingPhase::Idle);
    assert!(!caught_session.wants_click());

    // Escaped: claims click while holding rod at water, releases once rod put away
    let mut escaped_session = session;
    assert_eq!(
        escaped_session.tick(&click_in, dt_step),
        Some(FishingEvent::Escaped)
    );
    assert_eq!(escaped_session.machine().phase, FishingPhase::Escaped);
    assert!(escaped_session.wants_click());
    escaped_session.tick(&no_rod, dt_step);
    assert_eq!(escaped_session.machine().phase, FishingPhase::Idle);
    assert!(!escaped_session.wants_click());
}

#[test]
fn session_wait_duration_scales_between_min_and_max() {
    let cfg = FishingConfig::default();
    let mut session = FishingSession::new(cfg);
    // new: DECISIONS 2026-11-23
    let dt_step = 0.1;
    let cast_max = FishingInput {
        click: true,
        // new: DECISIONS 2026-11-23
        random_01: 1.0,
        ..idle_input()
    };
    assert_eq!(session.tick(&cast_max, dt_step), Some(FishingEvent::Cast));
    assert_eq!(session.machine().wait_duration, cfg.cast_wait_max);
}

#[test]
fn can_fish_at_water() {
    let lake_cfg = world::LakeConfigData::load_from_file(repo_path("data/lake.json"))
        .unwrap_or_default();
    // new: DECISIONS 2026-11-23
    let seed = 42;
    let lake = world::LakeLayout::generate(seed, glam::Vec2::ZERO, None, &lake_cfg, |_| 0.0);
    // data: fishing.json maxWaterDistance
    let max_dist = 3.0;

    // Shore point facing toward lake center: can fish
    if let Some(&shore_pt) = lake.shore.first() {
        let to_center = (lake.center - shore_pt).normalize();
        let yaw = (-to_center.x).atan2(-to_center.y);
        let player_pos = Vec3::new(shore_pt.x, lake.water_level, shore_pt.y);
        assert!(can_fish_at(player_pos, yaw, &lake, max_dist));

        // Facing away from lake: cannot fish
        let opposite_yaw = yaw + std::f32::consts::PI;
        assert!(!can_fish_at(player_pos, opposite_yaw, &lake, max_dist));
    }

    // Swimming in deep water (lake center, not on dock): cannot fish
    let center_pos = Vec3::new(lake.center.x, lake.water_level, lake.center.y);
    assert!(!can_fish_at(center_pos, 0.0, &lake, max_dist));

    // Far away on land: cannot fish
    // new: DECISIONS 2026-11-23
    let far_land = Vec3::new(10000.0, 0.0, 10000.0);
    assert!(!can_fish_at(far_land, 0.0, &lake, max_dist));
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
    // data: fishing.json maxWaterDistance
    assert_eq!(cfg.max_water_distance, 3.0);
    // data: fishing.json castWaitMin
    assert_eq!(cfg.cast_wait_min, 2.0);
    // data: fishing.json castWaitMax
    assert_eq!(cfg.cast_wait_max, 5.0);
    // data: fishing.json biteDuration
    assert_eq!(cfg.bite_duration, 2.0);
    // data: fishing.json sweetSpotMin
    assert_eq!(cfg.sweet_spot_min, 0.35);
    // data: fishing.json sweetSpotMax
    assert_eq!(cfg.sweet_spot_max, 0.65);
    // data: fishing.json resultSeconds
    assert_eq!(cfg.result_seconds, 2.0);
}
