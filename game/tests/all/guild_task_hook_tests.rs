//! Unit and integration tests for guild task gameplay hooks and tracker rows.

use game::guild::task_hooks::{
    GuildTaskEvent, GuildTaskHookConfig, GuildTaskHooks, HandIn, NamedPlace, TaskRow, tracker_rows,
};
use game::guild::{GuildConfig, GuildManager, GuildReward, GuildTask, GuildTaskKind};
use glam::Vec3;

const TEST_SEED: i32 = 42; // new: DECISIONS 2026-11-23
const HAND_IN_RADIUS: f32 = 6.0; // data: guild.json hand_in_radius
const PLACE_RADIUS: f32 = 12.0; // data: guild.json place_radius
const HOOK_CHECK_SECONDS: f32 = 0.5; // data: guild.json hook_check_seconds

fn default_hook_config() -> GuildTaskHookConfig {
    GuildTaskHookConfig {
        hand_in_radius: HAND_IN_RADIUS,
        place_radius: PLACE_RADIUS,
        hook_check_seconds: HOOK_CHECK_SECONDS,
    }
}

fn manager_with_active(tasks: Vec<GuildTask>) -> GuildManager {
    let mut manager = GuildManager::new(TEST_SEED, GuildConfig::default());
    let ids: Vec<String> = tasks.iter().map(|t| t.id.clone()).collect();
    manager.board_mut().set_offered(tasks);
    for id in ids {
        manager
            .accept_task(&id)
            .expect("offered test task must be accepted");
    }
    manager
}

fn fetch_task(id: &str, title: &str, item_id: &str, target: u32) -> GuildTask {
    GuildTask::new(
        id,
        title,
        GuildTaskKind::Fetch {
            item_id: item_id.to_string(),
        },
        target,
        GuildReward::default(),
    )
}

fn explore_task(id: &str, title: &str, place_name: &str) -> GuildTask {
    let target = 1; // data: guild.json tasks
    GuildTask::new(
        id,
        title,
        GuildTaskKind::Explore {
            place_name: place_name.to_string(),
        },
        target,
        GuildReward::default(),
    )
}

fn clear_task(id: &str, title: &str, poi_id: &str) -> GuildTask {
    let target = 1; // data: guild.json tasks
    GuildTask::new(
        id,
        title,
        GuildTaskKind::Clear {
            poi_id: poi_id.to_string(),
        },
        target,
        GuildReward::default(),
    )
}

#[test]
fn partial_hand_in_takes_available_bag_and_leaves_task_open() {
    let target = 5; // new: DECISIONS 2026-11-23
    let bag_wood = 2; // new: DECISIONS 2026-11-23
    let mut manager = manager_with_active(vec![fetch_task("fetch_wood", "Wood", "wood", target)]);
    let mut hooks = GuildTaskHooks::new(default_hook_config());
    let guild_pos = Vec3::ZERO;
    let player = Vec3::new(2.0, 0.0, 0.0); // new: DECISIONS 2026-11-23

    let plan = hooks.plan_hand_in(
        &manager,
        |id| if id == "wood" { bag_wood } else { 0 },
        guild_pos,
        player,
    );
    assert_eq!(
        plan,
        vec![HandIn {
            task_id: "fetch_wood".to_string(),
            item_id: "wood".to_string(),
            count: bag_wood,
        }]
    );

    let events = hooks.apply_hand_in(&mut manager, &plan);
    assert_eq!(
        events,
        vec![GuildTaskEvent::Progress {
            task_id: "fetch_wood".to_string(),
            progress: bag_wood,
            target,
        }]
    );
    assert_eq!(manager.board().active()[0].progress, bag_wood);
    assert!(!manager.board().active()[0].is_done());
}

#[test]
fn over_delivery_clamps_plan_to_remaining_need() {
    let target = 5; // new: DECISIONS 2026-11-23
    let bag_wood = 9; // new: DECISIONS 2026-11-23
    let mut manager = manager_with_active(vec![fetch_task("fetch_wood", "Wood", "wood", target)]);
    let mut hooks = GuildTaskHooks::new(default_hook_config());

    let plan = hooks.plan_hand_in(
        &manager,
        |id| if id == "wood" { bag_wood } else { 0 },
        Vec3::ZERO,
        Vec3::ZERO,
    );
    assert_eq!(
        plan,
        vec![HandIn {
            task_id: "fetch_wood".to_string(),
            item_id: "wood".to_string(),
            count: target,
        }]
    );

    let events = hooks.apply_hand_in(&mut manager, &plan);
    assert_eq!(
        events,
        vec![
            GuildTaskEvent::Progress {
                task_id: "fetch_wood".to_string(),
                progress: target,
                target,
            },
            GuildTaskEvent::Ready {
                task_id: "fetch_wood".to_string(),
            },
        ]
    );
    assert!(manager.board().active()[0].is_done());
}

#[test]
fn wrong_item_produces_empty_plan_and_no_progress() {
    let target = 5; // new: DECISIONS 2026-11-23
    let bag_rope = 10; // new: DECISIONS 2026-11-23
    let mut manager = manager_with_active(vec![fetch_task("fetch_wood", "Wood", "wood", target)]);
    let mut hooks = GuildTaskHooks::new(default_hook_config());

    let plan = hooks.plan_hand_in(
        &manager,
        |id| if id == "rope" { bag_rope } else { 0 },
        Vec3::ZERO,
        Vec3::ZERO,
    );
    assert!(plan.is_empty());

    let events = hooks.apply_hand_in(&mut manager, &plan);
    assert!(events.is_empty());
    assert_eq!(manager.board().active()[0].progress, 0);
}

#[test]
fn two_fetch_tasks_for_same_item_share_bag_without_double_counting() {
    let target_a = 4; // new: DECISIONS 2026-11-23
    let target_b = 5; // new: DECISIONS 2026-11-23
    let bag_wood = 6; // new: DECISIONS 2026-11-23
    let expected_b = 2; // new: DECISIONS 2026-11-23
    let mut manager = manager_with_active(vec![
        fetch_task("fetch_a", "Timber A", "wood", target_a),
        fetch_task("fetch_b", "Timber B", "wood", target_b),
    ]);
    let mut hooks = GuildTaskHooks::new(default_hook_config());

    let plan = hooks.plan_hand_in(
        &manager,
        |id| if id == "wood" { bag_wood } else { 0 },
        Vec3::ZERO,
        Vec3::ZERO,
    );
    assert_eq!(
        plan,
        vec![
            HandIn {
                task_id: "fetch_a".to_string(),
                item_id: "wood".to_string(),
                count: target_a,
            },
            HandIn {
                task_id: "fetch_b".to_string(),
                item_id: "wood".to_string(),
                count: expected_b,
            },
        ]
    );

    let events = hooks.apply_hand_in(&mut manager, &plan);
    assert_eq!(
        events,
        vec![
            GuildTaskEvent::Progress {
                task_id: "fetch_a".to_string(),
                progress: target_a,
                target: target_a,
            },
            GuildTaskEvent::Progress {
                task_id: "fetch_b".to_string(),
                progress: expected_b,
                target: target_b,
            },
            GuildTaskEvent::Ready {
                task_id: "fetch_a".to_string(),
            },
        ]
    );
    assert!(manager.board().active()[0].is_done());
    assert_eq!(manager.board().active()[1].progress, expected_b);
    assert!(!manager.board().active()[1].is_done());
}

#[test]
fn hand_in_respects_horizontal_radius() {
    let target = 3; // new: DECISIONS 2026-11-23
    let bag_wood = 3; // new: DECISIONS 2026-11-23
    let manager = manager_with_active(vec![fetch_task("fetch_wood", "Wood", "wood", target)]);
    let hooks = GuildTaskHooks::new(default_hook_config());
    let guild_pos = Vec3::new(10.0, 0.0, 10.0); // new: DECISIONS 2026-11-23

    let outside = Vec3::new(16.1, 0.0, 10.0); // new: DECISIONS 2026-11-23
    let empty_plan = hooks.plan_hand_in(
        &manager,
        |id| if id == "wood" { bag_wood } else { 0 },
        guild_pos,
        outside,
    );
    assert!(empty_plan.is_empty());

    // Large Y difference is ignored; horizontal distance is 5.0 <= 6.0.
    let inside = Vec3::new(13.0, 80.0, 14.0); // new: DECISIONS 2026-11-23
    let valid_plan = hooks.plan_hand_in(
        &manager,
        |id| if id == "wood" { bag_wood } else { 0 },
        guild_pos,
        inside,
    );
    assert_eq!(
        valid_plan,
        vec![HandIn {
            task_id: "fetch_wood".to_string(),
            item_id: "wood".to_string(),
            count: target,
        }]
    );
}

#[test]
fn explore_completes_once_within_place_radius() {
    let target = 1; // data: guild.json tasks
    let mut manager = manager_with_active(vec![explore_task(
        "explore_ashford",
        "Scout Ashford Core",
        "Ashford core",
    )]);
    let mut hooks = GuildTaskHooks::new(default_hook_config());
    let places = [NamedPlace {
        name: "Ashford core",
        position: Vec3::new(100.0, 0.0, 50.0), // new: DECISIONS 2026-11-23
    }];
    // Horizontal distance is 10.0 <= 12.0; vertical offset is ignored.
    let player = Vec3::new(106.0, 30.0, 58.0); // new: DECISIONS 2026-11-23

    let first = hooks.tick(&mut manager, &places, player, HOOK_CHECK_SECONDS);
    assert_eq!(
        first,
        vec![
            GuildTaskEvent::Progress {
                task_id: "explore_ashford".to_string(),
                progress: target,
                target,
            },
            GuildTaskEvent::Ready {
                task_id: "explore_ashford".to_string(),
            },
        ]
    );
    assert!(manager.board().active()[0].is_done());

    let second = hooks.tick(&mut manager, &places, player, HOOK_CHECK_SECONDS);
    assert!(second.is_empty());
}

#[test]
fn explore_out_of_radius_makes_no_progress() {
    let mut manager = manager_with_active(vec![explore_task(
        "explore_lake",
        "Survey The Lake",
        "the lake",
    )]);
    let mut hooks = GuildTaskHooks::new(default_hook_config());
    let places = [NamedPlace {
        name: "the lake",
        position: Vec3::new(20.0, 0.0, 20.0), // new: DECISIONS 2026-11-23
    }];
    let far_player = Vec3::new(33.0, 0.0, 20.0); // new: DECISIONS 2026-11-23

    let events = hooks.tick(&mut manager, &places, far_player, HOOK_CHECK_SECONDS);
    assert!(events.is_empty());
    assert_eq!(manager.board().active()[0].progress, 0);
}

#[test]
fn explore_name_matching_is_case_insensitive() {
    let target = 1; // data: guild.json tasks
    let mut manager = manager_with_active(vec![explore_task(
        "explore_ashford",
        "Scout Ashford Core",
        "Ashford core",
    )]);
    let mut hooks = GuildTaskHooks::new(default_hook_config());
    let places = [NamedPlace {
        name: "ASHFORD CORE",
        position: Vec3::ZERO,
    }];

    let events = hooks.tick(&mut manager, &places, Vec3::ZERO, HOOK_CHECK_SECONDS);
    assert_eq!(
        events,
        vec![
            GuildTaskEvent::Progress {
                task_id: "explore_ashford".to_string(),
                progress: target,
                target,
            },
            GuildTaskEvent::Ready {
                task_id: "explore_ashford".to_string(),
            },
        ]
    );
}

#[test]
fn tick_throttle_waits_for_interval_and_avoids_large_dt_burst() {
    let step_dt = 0.1_f32; // new: DECISIONS 2026-11-23
    let pre_steps = 4; // new: DECISIONS 2026-11-23
    let huge_dt = 5.0_f32; // new: DECISIONS 2026-11-23
    let target = 1; // data: guild.json tasks

    let mut manager = manager_with_active(vec![explore_task(
        "explore_watchtower",
        "Recon Watchtower",
        "watchtower",
    )]);
    let mut hooks = GuildTaskHooks::new(default_hook_config());
    let places = [NamedPlace {
        name: "watchtower",
        position: Vec3::ZERO,
    }];

    for _ in 0..pre_steps {
        let events = hooks.tick(&mut manager, &places, Vec3::ZERO, step_dt);
        assert!(events.is_empty());
    }
    let fifth = hooks.tick(&mut manager, &places, Vec3::ZERO, step_dt);
    assert_eq!(
        fifth,
        vec![
            GuildTaskEvent::Progress {
                task_id: "explore_watchtower".to_string(),
                progress: target,
                target,
            },
            GuildTaskEvent::Ready {
                task_id: "explore_watchtower".to_string(),
            },
        ]
    );

    // Large dt hitch while out of range must not leave buffered checks for the next frame.
    let mut manager2 = manager_with_active(vec![explore_task(
        "explore_lake",
        "Survey The Lake",
        "the lake",
    )]);
    let mut hooks2 = GuildTaskHooks::new(default_hook_config());
    let lake = [NamedPlace {
        name: "the lake",
        position: Vec3::ZERO,
    }];
    let far = Vec3::new(50.0, 0.0, 0.0); // new: DECISIONS 2026-11-23
    assert!(hooks2.tick(&mut manager2, &lake, far, huge_dt).is_empty());
    assert!(hooks2.tick(&mut manager2, &lake, Vec3::ZERO, step_dt).is_empty());
}

#[test]
fn abandoned_task_stops_tracking_and_emits_no_events() {
    let mut manager = manager_with_active(vec![explore_task(
        "explore_ashford",
        "Scout Ashford Core",
        "Ashford core",
    )]);
    let mut hooks = GuildTaskHooks::new(default_hook_config());
    let places = [NamedPlace {
        name: "Ashford core",
        position: Vec3::ZERO,
    }];

    assert!(manager.abandon_task("explore_ashford"));
    assert!(tracker_rows(&manager).is_empty());

    let events = hooks.tick(&mut manager, &places, Vec3::ZERO, HOOK_CHECK_SECONDS);
    assert!(events.is_empty());
}

#[test]
fn ready_event_fires_once_for_completed_and_cleared_tasks() {
    let mut manager = manager_with_active(vec![clear_task(
        "clear_house",
        "Clear Small House",
        "house_small",
    )]);
    let mut hooks = GuildTaskHooks::new(default_hook_config());

    let advanced = manager.on_poi_cleared("house_small");
    let expected_advanced = 1; // new: DECISIONS 2026-11-23
    assert_eq!(advanced, expected_advanced);

    let first = hooks.tick(&mut manager, &[], Vec3::ZERO, HOOK_CHECK_SECONDS);
    assert_eq!(
        first,
        vec![GuildTaskEvent::Ready {
            task_id: "clear_house".to_string(),
        }]
    );

    let second = hooks.tick(&mut manager, &[], Vec3::ZERO, HOOK_CHECK_SECONDS);
    assert!(second.is_empty());

    // After death/load clear(), the ready task is announced again once.
    hooks.clear();
    let after_clear = hooks.tick(&mut manager, &[], Vec3::ZERO, HOOK_CHECK_SECONDS);
    assert_eq!(
        after_clear,
        vec![GuildTaskEvent::Ready {
            task_id: "clear_house".to_string(),
        }]
    );
    assert!(
        hooks
            .tick(&mut manager, &[], Vec3::ZERO, HOOK_CHECK_SECONDS)
            .is_empty()
    );
}

#[test]
fn tracker_rows_caps_at_three_and_copies_fields() {
    let target_a = 5; // new: DECISIONS 2026-11-23
    let progress_a = 2; // new: DECISIONS 2026-11-23
    let target_b = 1; // data: guild.json tasks
    let target_c = 4; // new: DECISIONS 2026-11-23
    let target_d = 3; // new: DECISIONS 2026-11-23

    let mut manager = manager_with_active(vec![
        fetch_task("t1", "Timber Reserve", "wood", target_a),
        clear_task("t2", "Clear Small House", "house_small"),
        fetch_task("t3", "Rations Delivery", "canned_beans", target_c),
    ]);
    manager.on_item_delivered("wood", progress_a);
    manager.on_poi_cleared("house_small");
    manager
        .board_mut()
        .active
        .push(fetch_task("t4", "Extra Fourth Task", "bandage", target_d));

    let rows = tracker_rows(&manager);
    assert_eq!(
        rows,
        vec![
            TaskRow {
                title: "Timber Reserve".to_string(),
                progress: progress_a,
                target: target_a,
                ready: false,
            },
            TaskRow {
                title: "Clear Small House".to_string(),
                progress: target_b,
                target: target_b,
                ready: true,
            },
            TaskRow {
                title: "Rations Delivery".to_string(),
                progress: 0,
                target: target_c,
                ready: false,
            },
        ]
    );
}

#[test]
fn data_tasks_from_repo_roll_all_kinds_and_hooks_handle_each() {
    let initial_seed = 1; // new: DECISIONS 2026-11-23
    let max_seed = 40; // new: DECISIONS 2026-11-23
    let day = 1; // new: DECISIONS 2026-11-23
    let hour = 7.0; // new: DECISIONS 2026-11-23

    let mut sample_kill = None;
    let mut sample_fetch = None;
    let mut sample_explore = None;
    let mut sample_clear = None;

    for seed in initial_seed..=max_seed {
        let mut rolled =
            GuildManager::from_repo(seed).expect("load guild manager from data/guild.json");
        rolled.refresh_morning(day, hour, None);
        assert!(!rolled.board().offered().is_empty());

        for task in rolled.board().offered() {
            match &task.kind {
                GuildTaskKind::Kill { .. } if sample_kill.is_none() => {
                    sample_kill = Some(task.clone());
                }
                GuildTaskKind::Fetch { .. } if sample_fetch.is_none() => {
                    sample_fetch = Some(task.clone());
                }
                GuildTaskKind::Explore { .. } if sample_explore.is_none() => {
                    sample_explore = Some(task.clone());
                }
                GuildTaskKind::Clear { .. } if sample_clear.is_none() => {
                    sample_clear = Some(task.clone());
                }
                _ => {}
            }
        }

        if sample_kill.is_some()
            && sample_fetch.is_some()
            && sample_explore.is_some()
            && sample_clear.is_some()
        {
            break;
        }
    }

    let kill_task = sample_kill.expect("data/guild.json must roll a kill task");
    let fetch_task = sample_fetch.expect("data/guild.json must roll a fetch task");
    let explore_task = sample_explore.expect("data/guild.json must roll an explore task");
    let clear_task = sample_clear.expect("data/guild.json must roll a clear task");

    let cfg = default_hook_config();
    let origin = Vec3::ZERO;

    // Kill task finished via manager.on_kill -> hooks emit Ready.
    let mut kill_mgr = manager_with_active(vec![kill_task.clone()]);
    let mut hooks = GuildTaskHooks::new(cfg.clone());
    if let GuildTaskKind::Kill {
        zombie_type,
        in_city,
    } = &kill_task.kind
    {
        let kind_str = zombie_type.as_deref().unwrap_or("walker");
        for _ in 0..kill_task.target {
            kill_mgr.on_kill(kind_str, *in_city);
        }
    }
    assert_eq!(
        hooks.tick(&mut kill_mgr, &[], origin, HOOK_CHECK_SECONDS),
        vec![GuildTaskEvent::Ready {
            task_id: kill_task.id,
        }]
    );

    // Fetch task finished via plan_hand_in + apply_hand_in -> Progress + Ready.
    let mut fetch_mgr = manager_with_active(vec![fetch_task.clone()]);
    let mut hooks = GuildTaskHooks::new(cfg.clone());
    let fetch_item = match &fetch_task.kind {
        GuildTaskKind::Fetch { item_id } => item_id.clone(),
        _ => unreachable!(),
    };
    let plan = hooks.plan_hand_in(
        &fetch_mgr,
        |id| if id == fetch_item { fetch_task.target } else { 0 },
        origin,
        origin,
    );
    assert_eq!(
        plan,
        vec![HandIn {
            task_id: fetch_task.id.clone(),
            item_id: fetch_item,
            count: fetch_task.target,
        }]
    );
    assert_eq!(
        hooks.apply_hand_in(&mut fetch_mgr, &plan),
        vec![
            GuildTaskEvent::Progress {
                task_id: fetch_task.id.clone(),
                progress: fetch_task.target,
                target: fetch_task.target,
            },
            GuildTaskEvent::Ready {
                task_id: fetch_task.id,
            },
        ]
    );

    // Explore task finished via tick within place_radius -> Progress + Ready.
    let mut explore_mgr = manager_with_active(vec![explore_task.clone()]);
    let mut hooks = GuildTaskHooks::new(cfg.clone());
    let place_name = match &explore_task.kind {
        GuildTaskKind::Explore { place_name } => place_name.clone(),
        _ => unreachable!(),
    };
    let places = [NamedPlace {
        name: &place_name,
        position: origin,
    }];
    assert_eq!(
        hooks.tick(&mut explore_mgr, &places, origin, HOOK_CHECK_SECONDS),
        vec![
            GuildTaskEvent::Progress {
                task_id: explore_task.id.clone(),
                progress: explore_task.target,
                target: explore_task.target,
            },
            GuildTaskEvent::Ready {
                task_id: explore_task.id,
            },
        ]
    );

    // Clear task finished via manager.on_poi_cleared -> hooks emit Ready.
    let mut clear_mgr = manager_with_active(vec![clear_task.clone()]);
    let mut hooks = GuildTaskHooks::new(cfg);
    let poi_id = match &clear_task.kind {
        GuildTaskKind::Clear { poi_id } => poi_id.clone(),
        _ => unreachable!(),
    };
    clear_mgr.on_poi_cleared(&poi_id);
    assert_eq!(
        hooks.tick(&mut clear_mgr, &[], origin, HOOK_CHECK_SECONDS),
        vec![GuildTaskEvent::Ready {
            task_id: clear_task.id,
        }]
    );
}

#[test]
fn config_from_repo_loads_hook_check_seconds() {
    let cfg = GuildTaskHookConfig::from_repo().expect("load hook config from data/guild.json");
    assert!((cfg.hand_in_radius - HAND_IN_RADIUS).abs() < f32::EPSILON);
    assert!((cfg.place_radius - PLACE_RADIUS).abs() < f32::EPSILON);
    assert!((cfg.hook_check_seconds - HOOK_CHECK_SECONDS).abs() < f32::EPSILON);
}

#[test]
fn config_from_json_reads_keys_and_ignores_others() {
    let json = r#"{
        "reset_hour": 6.0,
        "max_active": 3,
        "hand_in_radius": 8.5,
        "place_radius": 15.0,
        "hook_check_seconds": 0.25,
        "ranks": []
    }"#;
    let cfg = GuildTaskHookConfig::from_json(json).expect("parse custom hook config");
    let expected_hand_in = 8.5_f32; // new: DECISIONS 2026-11-23
    let expected_place = 15.0_f32; // new: DECISIONS 2026-11-23
    let expected_interval = 0.25_f32; // new: DECISIONS 2026-11-23
    assert!((cfg.hand_in_radius - expected_hand_in).abs() < f32::EPSILON);
    assert!((cfg.place_radius - expected_place).abs() < f32::EPSILON);
    assert!((cfg.hook_check_seconds - expected_interval).abs() < f32::EPSILON);
}

#[test]
fn plan_hand_in_for_targets_requested_task_even_with_duplicate_item() {
    let target = 5; // new: DECISIONS 2026-11-24
    let bag_wood = 3; // new: DECISIONS 2026-11-24
    let mut manager = manager_with_active(vec![
        fetch_task("fetch_wood_1", "Wood 1", "wood", target),
        fetch_task("fetch_wood_2", "Wood 2", "wood", target),
    ]);
    let mut hooks = GuildTaskHooks::new(default_hook_config());
    let guild_pos = Vec3::ZERO;
    let player = Vec3::new(1.0, 0.0, 0.0); // new: DECISIONS 2026-11-24

    assert!(hooks.is_in_hand_in_range(guild_pos, player));
    assert!(!hooks.is_in_hand_in_range(guild_pos, Vec3::new(HAND_IN_RADIUS + 0.5, 0.0, 0.0)));

    let entry = hooks
        .plan_hand_in_for(
            &manager,
            "fetch_wood_2",
            |id| if id == "wood" { bag_wood } else { 0 },
            guild_pos,
            player,
        )
        .expect("second task should receive a hand-in plan");
    assert_eq!(
        entry,
        HandIn {
            task_id: "fetch_wood_2".to_string(),
            item_id: "wood".to_string(),
            count: bag_wood,
        }
    );

    let events = hooks.apply_hand_in(&mut manager, &[entry]);
    assert_eq!(
        events,
        vec![GuildTaskEvent::Progress {
            task_id: "fetch_wood_2".to_string(),
            progress: bag_wood,
            target,
        }]
    );
    assert_eq!(manager.board().active()[0].progress, 0);
    assert_eq!(manager.board().active()[1].progress, bag_wood);
}

#[test]
fn plan_hand_in_for_returns_none_when_out_of_range_or_empty() {
    let target = 4; // new: DECISIONS 2026-11-24
    let manager = manager_with_active(vec![
        fetch_task("fetch_wood", "Wood", "wood", target),
        explore_task("explore_lake", "Lake", "the lake"),
    ]);
    let hooks = GuildTaskHooks::new(default_hook_config());

    assert_eq!(
        hooks.plan_hand_in_for(
            &manager,
            "fetch_wood",
            |_| target,
            Vec3::ZERO,
            Vec3::new(HAND_IN_RADIUS + 1.0, 0.0, 0.0),
        ),
        None
    );
    assert_eq!(
        hooks.plan_hand_in_for(&manager, "fetch_wood", |_| 0, Vec3::ZERO, Vec3::ZERO),
        None
    );
    assert_eq!(
        hooks.plan_hand_in_for(&manager, "explore_lake", |_| target, Vec3::ZERO, Vec3::ZERO),
        None
    );
    assert_eq!(
        hooks.plan_hand_in_for(&manager, "missing_id", |_| target, Vec3::ZERO, Vec3::ZERO),
        None
    );
}
