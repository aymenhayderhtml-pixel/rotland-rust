//! R44-01: safe house config, claim / ladder state, travel block and chase count.

use game_world::safe_house::{SafeHouseConfig, SafeHouseState, travel_blocked};
use game_zombie::chasing::count_chasing_near;
use glam::Vec3;
use serde_json::json;

#[test]
fn safe_house_repo_config_loads() {
    let cfg = SafeHouseConfig::from_repo().expect("data/safe_house.json must load");
    assert_eq!(cfg.travel_hours, 3.0);
    assert_eq!(cfg.rest_hour, 7.0);
    assert_eq!(cfg.map_label, "Safe House");
    let p = &cfg.prompts;
    let all = [
        &p.claim,
        &p.bed,
        &p.chest,
        &p.ladder_up,
        &p.ladder_down,
        &p.travel_to_safe_house,
        &p.travel_to_camp,
        &p.travel_blocked,
        &p.not_claimed,
        &p.claimed_toast,
        &p.rested,
    ];
    assert_eq!(all.len(), 11);
    for text in all {
        assert!(!text.is_empty(), "every prompt must be non-empty");
    }
}

#[test]
fn safe_house_bad_json_is_error() {
    assert!(SafeHouseConfig::from_json("{").is_err());
}

#[test]
fn safe_house_default_is_unclaimed_ladder_up() {
    let s = SafeHouseState::default();
    assert!(!s.claimed);
    assert!(!s.ladder_down);
}

#[test]
fn safe_house_toggle_before_claim_does_nothing() {
    let mut s = SafeHouseState::default();
    assert!(!s.toggle_ladder());
    assert_eq!(s, SafeHouseState::default());
}

#[test]
fn safe_house_claim_lowers_ladder() {
    let mut s = SafeHouseState::default();
    s.claim();
    assert!(s.claimed);
    assert!(s.ladder_down);
}

#[test]
fn safe_house_toggle_flips_and_returns_new_value() {
    let mut s = SafeHouseState::default();
    s.claim();
    assert!(!s.toggle_ladder(), "first toggle pulls the ladder up");
    assert!(!s.ladder_down);
    assert!(s.toggle_ladder(), "second toggle lowers it again");
    assert!(s.ladder_down);
}

#[test]
fn safe_house_second_claim_keeps_ladder_up() {
    let mut s = SafeHouseState::default();
    s.claim();
    s.toggle_ladder();
    assert!(!s.ladder_down);
    s.claim();
    assert!(s.claimed);
    assert!(!s.ladder_down, "a second claim must not lower the ladder");
}

#[test]
fn safe_house_save_round_trip() {
    let cases = [
        SafeHouseState { claimed: true, ladder_down: true },
        SafeHouseState { claimed: true, ladder_down: false },
        SafeHouseState::default(),
    ];
    for s in cases {
        assert_eq!(SafeHouseState::from_json(&s.to_json()), s);
    }
}

#[test]
fn safe_house_missing_or_junk_save_is_default() {
    assert_eq!(SafeHouseState::from_json(&json!({})), SafeHouseState::default());
    assert_eq!(SafeHouseState::from_json(&json!("junk")), SafeHouseState::default());
}

#[test]
fn safe_house_travel_blocked_only_when_chasing() {
    assert!(!travel_blocked(0));
    assert!(travel_blocked(1));
    assert!(travel_blocked(5));
}

#[test]
fn chasing_inside_radius_counts() {
    let zombies = [(Vec3::new(3.0, 0.0, 4.0), true)];
    assert_eq!(count_chasing_near(zombies, Vec3::ZERO, 10.0), 1);
}

#[test]
fn chasing_outside_radius_does_not_count() {
    let zombies = [(Vec3::new(30.0, 0.0, 40.0), true)];
    assert_eq!(count_chasing_near(zombies, Vec3::ZERO, 10.0), 0);
}

#[test]
fn non_chasing_inside_radius_does_not_count() {
    let zombies = [(Vec3::new(1.0, 0.0, 1.0), false)];
    assert_eq!(count_chasing_near(zombies, Vec3::ZERO, 10.0), 0);
}

#[test]
fn chasing_exactly_on_radius_counts() {
    let zombies = [(Vec3::new(6.0, 0.0, 8.0), true)];
    assert_eq!(count_chasing_near(zombies, Vec3::ZERO, 10.0), 1);
}

#[test]
fn chasing_height_difference_is_ignored() {
    let zombies = [(Vec3::new(1.0, 50.0, 1.0), true)];
    assert_eq!(count_chasing_near(zombies, Vec3::ZERO, 10.0), 1);
}
