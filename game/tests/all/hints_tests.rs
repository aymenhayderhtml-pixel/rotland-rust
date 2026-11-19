//! Tests for hint config data loading, trigger evaluation, sequential queueing/gap cooldown, and save/load.
//! Port of Godot hint test specifications.

use game::hints::{
    check_trigger, parse_number, split, split_npc, HintConfig, HintContext, HintSystem,
    GAP_SECONDS,
};

#[test]
fn hints_tests_data_loads_and_validates() {
    let config = HintConfig::load_default().expect("data/hints.json must load and validate");
    assert!(config.enabled);
    assert!((config.show_seconds - 8.0).abs() < 1e-6);
    assert_eq!(config.hints.len(), 12);

    // Hand-verified hint list from data/hints.json:
    let expected_ids = [
        "start",
        "trader",
        "leave_camp",
        "zombie",
        "crate",
        "hungry",
        "dusk",
        "death",
        "heavy",
        "wood",
        "dog_breeder",
        "bounty",
    ];

    for (i, &expected_id) in expected_ids.iter().enumerate() {
        assert_eq!(config.hints[i].id, expected_id);
        assert!(!config.hints[i].text.is_empty());
        assert!(!config.hints[i].trigger.is_empty());
    }

    // Specific trigger checks
    assert_eq!(config.hints[0].trigger, "start");
    assert_eq!(config.hints[1].trigger, "after:10");
    assert_eq!(config.hints[1].done_when.as_deref(), Some("npc_talked"));
    assert_eq!(config.hints[9].trigger, "has_item:wood");
    assert_eq!(config.hints[10].trigger, "near_npc:breeder:6");
    assert_eq!(config.hints[11].trigger, "board_near:8");
    assert_eq!(config.hints[11].done_when.as_deref(), Some("board_used"));

    // Check vocabulary parser helpers
    assert_eq!(split("zombie_near:25"), ("zombie_near", "25"));
    assert_eq!(split("start"), ("start", ""));
    assert_eq!(parse_number("25"), 25.0);
    assert_eq!(parse_number("invalid"), 0.0);
    assert_eq!(split_npc("breeder:6"), ("breeder", 6.0));
    assert!(check_trigger("near_npc:breeder:6").is_ok());
    assert!(check_trigger("unknown_trigger").is_err());
}

#[test]
fn hints_tests_trigger_shows_hint_once() {
    let config = HintConfig::load_default().expect("data/hints.json must load");
    let mut system = HintSystem::new(config);
    let ctx = HintContext::new();

    // First frame tick: poll timer starts at 0, "start" trigger fires immediately
    let hint = system.tick(0.0, &ctx);
    assert!(hint.is_some());
    assert_eq!(system.current_id(), Some("start"));
    assert!(system.is_shown("start"));
    assert_eq!(system.times_shown("start"), 1);

    // Tip stays visible during its 8-second duration
    let mid_hint = system.tick(4.0, &ctx);
    assert!(mid_hint.is_some());
    assert_eq!(system.current_id(), Some("start"));

    // At 8 seconds, tip finishes and clears
    let end_hint = system.tick(4.0, &ctx);
    assert!(end_hint.is_none());
    assert_eq!(system.current_id(), None);

    // Advance past gap cooldown (0.5s)
    system.tick(GAP_SECONDS, &ctx);
    assert_eq!(system.current_id(), None);

    // Next polling intervals do NOT re-trigger "start" (the "trader" tip, trigger after:10, may show now)
    for _ in 0..10 {
        system.tick(0.25, &ctx);
        assert_ne!(system.current_id(), Some("start"));
    }
    assert_eq!(system.times_shown("start"), 1);

    // Manual request for already-shown tip is rejected
    assert!(!system.request("start"));
}

#[test]
fn hints_tests_cooldown_and_ordering() {
    let config = HintConfig::load_default().expect("data/hints.json must load");
    let mut system = HintSystem::new(config);
    let mut ctx = HintContext::new();
    // The "trader" tip (trigger after:10) would fire by itself during this test's 20+ seconds: mark it seen
    system.from_save(&serde_json::json!({ "shown": ["trader"] }));

    // Consume the initial "start" tip
    system.tick(0.0, &ctx);
    system.tick(8.0, &ctx);
    system.tick(GAP_SECONDS, &ctx);
    assert_eq!(system.current_id(), None);

    // Request two tips: "heavy" and "wood"
    assert!(system.request("heavy"));
    assert!(system.request("wood"));
    assert!(system.is_queued("heavy"));
    assert!(system.is_queued("wood"));

    // Tick 0: FIFO order shows "heavy" first
    system.tick(0.0, &ctx);
    assert_eq!(system.current_id(), Some("heavy"));
    assert!(!system.is_queued("heavy"));
    assert!(system.is_queued("wood"));

    // "heavy" finishes after 8 seconds
    system.tick(8.0, &ctx);
    assert_eq!(system.current_id(), None);

    // During the 0.5s gap cooldown, "wood" is held back
    system.tick(0.25, &ctx);
    assert_eq!(system.current_id(), None);
    assert!(system.is_queued("wood"));

    // After remaining gap (0.25s), "wood" appears
    system.tick(0.25, &ctx);
    assert_eq!(system.current_id(), Some("wood"));

    // Early termination via doneWhen event:
    // Consume "wood"
    system.tick(8.0, &ctx);
    system.tick(GAP_SECONDS, &ctx);

    // "crate" tip has doneWhen = "item_picked"
    assert!(system.request("crate"));
    system.tick(0.0, &ctx);
    assert_eq!(system.current_id(), Some("crate"));

    // Player picks up an item -> triggers "item_picked" event early (at 1.5s instead of 8.0s)
    system.on_event("item_picked");
    system.tick(1.5, &ctx);
    // Tip terminates early because doneWhen was seen
    assert_eq!(system.current_id(), None);

    // Dynamic environmental triggers: near_npc:breeder:6
    // 20m away: not triggered
    ctx.npc_distances.insert("breeder".to_string(), 20.0);
    system.tick(GAP_SECONDS, &ctx);
    system.tick(0.25, &ctx);
    assert!(!system.is_shown("dog_breeder"));

    // 3m away: triggers dog_breeder
    ctx.npc_distances.insert("breeder".to_string(), 3.0);
    system.tick(0.25, &ctx);
    assert_eq!(system.current_id(), Some("dog_breeder"));
}

#[test]
fn hints_tests_save_load_round_trip() {
    let config = HintConfig::load_default().expect("data/hints.json must load");
    let mut system = HintSystem::new(config.clone());
    let ctx = HintContext::new();

    // Trigger "start" and "heavy"
    system.tick(0.0, &ctx); // shows start
    assert!(system.request("heavy"));
    system.tick(8.0, &ctx); // finishes start
    system.tick(GAP_SECONDS, &ctx); // finishes gap
    system.tick(0.0, &ctx); // shows heavy

    assert!(system.is_shown("start"));
    assert!(system.is_shown("heavy"));
    assert!(!system.is_shown("wood"));

    let save_json = system.to_save();

    // Verify structure has "shown" array
    let shown_arr = save_json
        .get("shown")
        .and_then(|v| v.as_array())
        .expect("save must contain 'shown' array");
    assert!(shown_arr.iter().any(|v| v.as_str() == Some("start")));
    assert!(shown_arr.iter().any(|v| v.as_str() == Some("heavy")));

    // Load into clean HintSystem
    let mut restored = HintSystem::new(config);
    restored.from_save(&save_json);

    assert!(restored.is_shown("start"));
    assert!(restored.is_shown("heavy"));
    assert!(!restored.is_shown("wood"));

    // Tips loaded from save remain marked as shown and cannot be requested
    assert!(!restored.request("start"));
    assert!(!restored.request("heavy"));
    assert!(restored.request("wood"));
}
