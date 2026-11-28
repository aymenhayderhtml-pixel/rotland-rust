//! Climb-floor rescue survivors (R44): POI id test, config default, and deterministic spots.

use std::collections::HashSet;

use game::guild::{GuildConfig, RecruitDef};
use game::rescue::RescueConfig;
use game::rescue_floor::{floor_rescue_spots, is_climb_floor_poi, FloorRescueSeed};
use glam::Vec3;

fn seeds(n: usize) -> Vec<FloorRescueSeed> {
    (0..n)
        .map(|i| FloorRescueSeed {
            poi_id: format!("metro_floor_{}_{}", i + 1, 2),
            building_id: "metro_floor".to_string(),
            position: Vec3::new(i as f32 * 10.0, 5.0, -3.0),
            yaw: 0.5 * i as f32,
        })
        .collect()
}

fn repo_recruits() -> Vec<RecruitDef> {
    GuildConfig::from_repo().expect("data/guild.json").recruits
}

fn cfg_with(floor_survivors: usize) -> RescueConfig {
    RescueConfig {
        floor_survivors,
        ..RescueConfig::default()
    }
}

#[test]
fn climb_floor_poi_ids_are_detected() {
    assert!(is_climb_floor_poi("metro_floor_3_2"));
    assert!(!is_climb_floor_poi("poi_12"));
    assert!(!is_climb_floor_poi("metro_hospital"));
}

#[test]
fn rescue_config_floor_survivors_loads_from_repo() {
    let cfg = RescueConfig::from_repo().expect("data/rescue.json");
    assert_eq!(cfg.floor_survivors, 3);
}

#[test]
fn rescue_config_without_floor_key_defaults_to_zero() {
    let json = r#"{ "chancePerPoi": 0.15, "maxSurvivors": 6, "perTown": [1, 2] }"#;
    let cfg = RescueConfig::from_json(json).expect("old config parses");
    assert_eq!(cfg.floor_survivors, 0);
}

#[test]
fn three_seeds_make_three_named_spots() {
    let recruits = repo_recruits();
    let spots = floor_rescue_spots(&seeds(3), &cfg_with(3), &recruits);
    assert_eq!(spots.len(), 3);
    for (i, spot) in spots.iter().enumerate() {
        assert_eq!(spot.id, format!("rescue_floor_{}", i + 1));
        assert_eq!(spot.poi_id, format!("metro_floor_{}_2", i + 1));
    }
    let names: HashSet<&str> = spots.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names.len(), 3, "names must be unique");
}

#[test]
fn zero_floor_survivors_gives_no_spots() {
    let spots = floor_rescue_spots(&seeds(3), &cfg_with(0), &repo_recruits());
    assert!(spots.is_empty());
}

#[test]
fn two_seeds_with_three_allowed_give_two_spots() {
    let spots = floor_rescue_spots(&seeds(2), &cfg_with(3), &repo_recruits());
    assert_eq!(spots.len(), 2);
}

#[test]
fn same_input_gives_same_output() {
    let recruits = repo_recruits();
    let a = floor_rescue_spots(&seeds(3), &cfg_with(3), &recruits);
    let b = floor_rescue_spots(&seeds(3), &cfg_with(3), &recruits);
    assert_eq!(a, b);
}

#[test]
fn empty_recruits_use_fallback_template() {
    let spots = floor_rescue_spots(&seeds(1), &cfg_with(3), &[]);
    assert_eq!(spots.len(), 1);
    assert!(!spots[0].name.is_empty());
}
