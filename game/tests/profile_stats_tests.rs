//! Integration tests for profile stats counters, movement tracking, and save/load compatibility.
//! Port of Godot `scripts/Player/PlayerStats.cs` specifications.

use game::profile_stats::{ProfileStats, STATS_LOAD_ORDER, STATS_MAX_STEP_METRES, STATS_SAVE_ID};
use glam::Vec2;

#[test]
fn default_stats_match_godot_initial_values() {
    let stats = ProfileStats::new();

    // Hand-traced defaults from PlayerStats.cs:
    // HighestDay = 1, DaysSurvived = 0, all counters at 0
    assert_eq!(stats.kills(), 0);
    assert!(stats.kills_by_type().is_empty());
    assert_eq!(stats.headshots(), 0);
    assert_eq!(stats.deaths(), 0);
    assert_eq!(stats.highest_day(), 1);
    assert_eq!(stats.days_survived(), 0);
    assert_eq!(stats.play_time_seconds(), 0.0);
    assert_eq!(stats.play_time(), 0.0);
    assert_eq!(stats.distance_metres(), 0.0);
    assert_eq!(stats.distance_walked(), 0.0);
    assert_eq!(stats.distance_driven(), 0.0);
    assert_eq!(STATS_MAX_STEP_METRES, 5.0);
    assert_eq!(STATS_SAVE_ID, "stats");
    assert_eq!(STATS_LOAD_ORDER, 0);
}

#[test]
fn record_kill_and_on_kill_counts_accurately() {
    let mut stats = ProfileStats::new();

    // 1 generic non-headshot kill
    stats.on_kill(false);
    assert_eq!(stats.kills(), 1);
    assert_eq!(stats.headshots(), 0);
    assert_eq!(stats.kills_by_type().get("unknown").copied(), Some(1));

    // 1 generic headshot kill
    stats.on_kill(true);
    assert_eq!(stats.kills(), 2);
    assert_eq!(stats.headshots(), 1);
    assert_eq!(stats.kills_by_type().get("unknown").copied(), Some(2));

    // 2 walker kills (1 body, 1 headshot)
    stats.record_kill("walker", false);
    stats.record_kill("walker", true);
    assert_eq!(stats.kills(), 4);
    assert_eq!(stats.headshots(), 2);
    assert_eq!(stats.kills_by_type().get("walker").copied(), Some(2));

    // 1 runner kill (non-headshot)
    stats.record_kill("runner", false);
    assert_eq!(stats.kills(), 5);
    assert_eq!(stats.headshots(), 2);
    assert_eq!(stats.kills_by_type().get("runner").copied(), Some(1));

    // Hand-traced total: 2 unknown + 2 walker + 1 runner = 5 kills, 2 headshots
    assert_eq!(stats.kills_by_type().len(), 3);
}

#[test]
fn record_death_and_day_counters() {
    let mut stats = ProfileStats::new();

    // Deaths: start at 0, increment by 1 each call
    stats.on_death();
    assert_eq!(stats.deaths(), 1);
    stats.record_death();
    assert_eq!(stats.deaths(), 2);

    // Days: starts at day 1 (0 days survived)
    assert_eq!(stats.highest_day(), 1);
    assert_eq!(stats.days_survived(), 0);

    // on_day advances day
    stats.on_day();
    assert_eq!(stats.highest_day(), 2);
    assert_eq!(stats.days_survived(), 1);

    stats.on_day();
    assert_eq!(stats.highest_day(), 3);
    assert_eq!(stats.days_survived(), 2);

    // record_day sets day to max(current, day)
    stats.record_day(7);
    assert_eq!(stats.highest_day(), 7);
    assert_eq!(stats.days_survived(), 6);

    // Lower day does not decrease highest day
    stats.record_day(4);
    assert_eq!(stats.highest_day(), 7);
    assert_eq!(stats.days_survived(), 6);
}

#[test]
fn distance_walked_and_driven_counters() {
    let mut stats = ProfileStats::new();

    // Walking distance
    stats.add_walked(15.5);
    assert_eq!(stats.distance_walked(), 15.5);
    assert_eq!(stats.distance_metres(), 15.5);

    // Negative distances ignored
    stats.add_walked(-5.0);
    assert_eq!(stats.distance_walked(), 15.5);

    // Driving distance
    stats.add_driven(84.5);
    assert_eq!(stats.distance_driven(), 84.5);
    // Total distance = 15.5 + 84.5 = 100.0
    assert_eq!(stats.distance_metres(), 100.0);

    // Negative driving ignored
    stats.add_driven(-10.0);
    assert_eq!(stats.distance_driven(), 84.5);

    // Play time
    stats.add_play_time(42.5);
    assert_eq!(stats.play_time_seconds(), 42.5);
    stats.add_play_time(-10.0);
    assert_eq!(stats.play_time_seconds(), 42.5);
}

#[test]
fn movement_tracking_filters_teleports() {
    let mut stats = ProfileStats::new();

    // Frame 1: set starting position at (0, 0)
    stats.update_position(Vec2::new(0.0, 0.0));
    assert_eq!(stats.distance_metres(), 0.0);

    // Frame 2: walk to (1.5, 2.0).
    // Hand-traced distance: sqrt(1.5^2 + 2.0^2) = sqrt(2.25 + 4.0) = sqrt(6.25) = 2.5 m.
    // 2.5 < 5.0 (STATS_MAX_STEP_METRES) -> accepted as walking.
    stats.update_position(Vec2::new(1.5, 2.0));
    assert!((stats.distance_metres() - 2.5).abs() < 1e-5);

    // Frame 3: sudden teleport to (100.0, 100.0).
    // Step is ~138 m >= 5.0 m -> rejected as teleport.
    stats.update_position(Vec2::new(100.0, 100.0));
    assert!((stats.distance_metres() - 2.5).abs() < 1e-5);

    // Frame 4: normal walk from (100.0, 100.0) to (102.0, 100.0) -> 2.0 m.
    stats.update_position(Vec2::new(102.0, 100.0));
    // Hand-traced total: 2.5 + 2.0 = 4.5 m.
    assert!((stats.distance_metres() - 4.5).abs() < 1e-5);

    // Reset position clears tracker
    stats.reset_position();
    assert!(stats.last_xz.is_none());
}

#[test]
fn save_load_round_trip_preserves_exact_godot_keys() {
    let mut stats = ProfileStats::new();
    stats.record_kill("walker", true);
    stats.record_kill("runner", false);
    stats.on_kill(false);
    stats.on_death();
    stats.record_day(5);
    stats.add_play_time(360.0);
    stats.add_walked(120.5);
    stats.add_driven(80.5);

    let save_json = stats.to_save();
    let obj = save_json.as_object().expect("save is a JSON object");

    // Hand-traced Godot save keys:
    assert!(obj.contains_key("kills"));
    assert!(obj.contains_key("headshots"));
    assert!(obj.contains_key("deaths"));
    assert!(obj.contains_key("highestDay"));
    assert!(obj.contains_key("playTime"));
    assert!(obj.contains_key("distance"));

    // Expected values:
    assert_eq!(obj["headshots"].as_u64(), Some(1));
    assert_eq!(obj["deaths"].as_u64(), Some(1));
    assert_eq!(obj["highestDay"].as_i64(), Some(5));
    assert_eq!(obj["playTime"].as_f64(), Some(360.0));
    // Total distance = 120.5 + 80.5 = 201.0
    assert_eq!(obj["distance"].as_f64(), Some(201.0));

    // Load back into a fresh instance
    let mut loaded = ProfileStats::new();
    loaded.from_save(&save_json);

    assert_eq!(loaded.kills(), 3);
    assert_eq!(loaded.headshots(), 1);
    assert_eq!(loaded.deaths(), 1);
    assert_eq!(loaded.highest_day(), 5);
    assert_eq!(loaded.days_survived(), 4);
    assert_eq!(loaded.play_time_seconds(), 360.0);
    assert_eq!(loaded.distance_metres(), 201.0);
}

#[test]
fn load_from_godot_json_data() {
    // Simulates an existing Godot save file payload
    let godot_json = serde_json::json!({
        "kills": {
            "walker": 10,
            "runner": 3,
            "fat": 1
        },
        "headshots": 7,
        "deaths": 4,
        "highestDay": 8,
        "playTime": 7200.5,
        "distance": 3500.25
    });

    let loaded = ProfileStats::from_save_value(&godot_json);

    // Hand-traced expected stats:
    // Total kills = 10 + 3 + 1 = 14
    assert_eq!(loaded.kills(), 14);
    assert_eq!(loaded.kills_by_type().get("walker").copied(), Some(10));
    assert_eq!(loaded.kills_by_type().get("runner").copied(), Some(3));
    assert_eq!(loaded.kills_by_type().get("fat").copied(), Some(1));
    assert_eq!(loaded.headshots(), 7);
    assert_eq!(loaded.deaths(), 4);
    assert_eq!(loaded.highest_day(), 8);
    // Days survived = 8 - 1 = 7
    assert_eq!(loaded.days_survived(), 7);
    assert_eq!(loaded.play_time_seconds(), 7200.5);
    assert!((loaded.distance_metres() - 3500.25).abs() < 1e-4);
}
