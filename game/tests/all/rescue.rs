//! Unit and integration tests for rescue survivor placement and rescue rules.

use std::collections::HashSet;

use game::guild::{
    GuildConfig, GuildRecruit, GuildRecruits, RecruitDef, RecruitHireError, RecruitRole,
    RecruitState,
};
use game::rescue::{RescueBlocker, RescueConfig, RescueLoadError, RescueState};
use game::rescue_place::{
    collect_candidate_pois, is_rescue_building, parse_recruit_role, place_survivors,
    place_survivors_in_world, RescueSpot, RESCUE_BUILDINGS,
};
use glam::Vec3;
use world::{PoiBuildingDef, PoiConfigData, PoiInstance, WorldGenerator};

fn sample_poi_config() -> PoiConfigData {
    let mut cfg = PoiConfigData::default();
    for &id in RESCUE_BUILDINGS {
        cfg.buildings.push(PoiBuildingDef {
            id: id.to_string(),
            lot: [10.0, 8.0],
            biome: Default::default(),
            chance_per_chunk: 0.5,
            min_flatness: 0.9,
            min_distance_from_spawn: 60.0,
            loot_table: String::new(),
            zombies: [1, 3],
            zombie_models: Vec::new(),
            wake_height_m: 0.0,
        });
    }
    cfg
}

fn sample_recruits() -> Vec<RecruitDef> {
    GuildConfig::from_repo()
        .expect("data/guild.json should load")
        .recruits
}

fn many_house_pois(count: usize, start_dist: f32) -> Vec<PoiInstance> {
    (0..count)
        .map(|i| {
            let b = RESCUE_BUILDINGS[i % RESCUE_BUILDINGS.len()];
            let x = start_dist + (i as f32) * 80.0;
            PoiInstance::new(format!("poi_{i}_0"), b, Vec3::new(x, 2.0, 0.0), 0.0)
        })
        .collect()
}

#[test]
fn rescue_json_loads_with_expected_values() {
    let cfg = RescueConfig::from_repo().expect("data/rescue.json should load");
    assert!((cfg.chance_per_poi - 0.15).abs() < 1e-6);
    assert_eq!(cfg.max_survivors, 6);
    assert!((cfg.min_dist_from_camp - 300.0).abs() < 1e-6);
    assert!((cfg.clear_radius - 12.0).abs() < 1e-6);
    assert!((cfg.talk_range - 2.5).abs() < 1e-6);
    assert_eq!(cfg.min_zombies_near, 3);

    let json = serde_json::to_string(&cfg).unwrap();
    let round_trip = RescueConfig::from_json(&json).unwrap();
    assert_eq!(cfg, round_trip);

    let missing = RescueConfig::from_file(std::path::Path::new("nonexistent_rescue.json"));
    assert!(matches!(missing, Err(RescueLoadError::NotFound(_))));
}

#[test]
fn placement_is_deterministic_per_seed_and_caps_at_six() {
    let cfg = RescueConfig::from_repo().unwrap();
    let poi_cfg = sample_poi_config();
    let recruits = sample_recruits();
    let pois = many_house_pois(120, 350.0);
    let camps = [Vec3::ZERO];

    let spots_a = place_survivors(42, &pois, &poi_cfg, &camps, &cfg, &recruits);
    let mut reversed = pois.clone();
    reversed.reverse();
    let spots_b = place_survivors(42, &reversed, &poi_cfg, &camps, &cfg, &recruits);

    assert_eq!(spots_a, spots_b, "same seed and POIs must yield identical spots");
    assert_eq!(spots_a.len(), 6, "maxSurvivors cap of 6 should be respected");

    let ids: HashSet<&str> = spots_a.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids.len(), spots_a.len(), "rescue ids must be unique");
    assert!(spots_a.iter().all(|s| s.id.starts_with("rescue_")));

    let valid_names: HashSet<&str> = recruits.iter().map(|r| r.name.as_str()).collect();
    for s in &spots_a {
        assert!(valid_names.contains(s.name.as_str()));
        assert!(is_rescue_building(&s.building_id));
    }

    assert_eq!(parse_recruit_role("fighter"), RecruitRole::Fighter);
    assert_eq!(parse_recruit_role("Scavenger"), RecruitRole::Scavenger);
    assert_eq!(parse_recruit_role("GUARD"), RecruitRole::Guard);

    let spots_diff = place_survivors(999, &pois, &poi_cfg, &camps, &cfg, &recruits);
    assert_ne!(spots_a, spots_diff, "different seed should produce different placement");
}

#[test]
fn placement_excludes_pois_within_camp_radius_and_non_house_types() {
    let cfg = RescueConfig {
        chance_per_poi: 1.0,
        ..RescueConfig::from_repo().unwrap()
    };
    let poi_cfg = sample_poi_config();
    let recruits = sample_recruits();
    let camp = Vec3::new(100.0, 0.0, 50.0);

    let pois = vec![
        // Inside 300 m of camp -> excluded
        PoiInstance::new("poi_near", "house_small", Vec3::new(100.0, 0.0, 349.0), 0.0),
        // Non-house/shop building -> excluded
        PoiInstance::new("poi_barn", "barn", Vec3::new(500.0, 0.0, 50.0), 0.0),
        PoiInstance::new("poi_shed", "shed", Vec3::new(600.0, 0.0, 50.0), 0.0),
        // Valid house >= 300 m away -> included
        PoiInstance::new("poi_far", "shop", Vec3::new(405.0, 0.0, 50.0), 0.0),
    ];

    let spots = place_survivors(7, &pois, &poi_cfg, &[camp], &cfg, &recruits);
    assert_eq!(spots.len(), 1);
    assert_eq!(spots[0].poi_id, "poi_far");
    assert_eq!(spots[0].id, "rescue_1");
}

#[test]
fn world_generator_placement_is_deterministic_and_far_from_camp() {
    let cfg = RescueConfig::from_repo().unwrap();
    let recruits = sample_recruits();
    let wgen = WorldGenerator::from_data_dir(42, paths::repo_path("data"))
        .expect("WorldGenerator::from_data_dir should succeed");

    let candidate_pois = collect_candidate_pois(&wgen);
    assert!(!candidate_pois.is_empty());

    let spots_a = place_survivors_in_world(&wgen, &cfg, &recruits);
    let spots_b = place_survivors_in_world(&wgen, &cfg, &recruits);
    assert_eq!(spots_a, spots_b);
    assert!(spots_a.len() <= cfg.max_survivors);
    assert!(!spots_a.is_empty());

    let camp = wgen.settlements()[0].center;
    for spot in &spots_a {
        let dx = spot.position.x - camp.x;
        let dz = spot.position.z - camp.z;
        let dist = (dx * dx + dz * dz).sqrt();
        assert!(
            dist >= cfg.min_dist_from_camp - 10.0,
            "spot {} too close to camp: {dist} m",
            spot.id
        );
    }
}

fn make_test_spot(id: &str, name: &str, pos: Vec3) -> RescueSpot {
    RescueSpot {
        id: id.to_string(),
        poi_id: format!("poi_{id}"),
        building_id: "house_small".to_string(),
        name: name.to_string(),
        role: RecruitRole::Scavenger,
        level: 1,
        daily_wage: 12,
        max_health: 80,
        position: pos,
        yaw: 0.0,
    }
}

#[test]
fn clear_vs_not_clear_blocks_and_allows_rescue() {
    let cfg = RescueConfig::from_repo().unwrap();
    let spot = make_test_spot("rescue_1", "Sarah Jenkins", Vec3::new(400.0, 0.0, 0.0));
    let mut state = RescueState::new(cfg, vec![spot.clone()]);
    let mut recruits = GuildRecruits::default();
    let player = Vec3::new(401.5, 0.0, 0.0); // 1.5 m away <= 2.5 m

    // Living zombie within 12 m blocks rescue and hides prompt
    let zombie_near = [Vec3::new(411.0, 0.0, 0.0)]; // 11 m <= 12 m
    assert!(!state.is_area_clear(&spot, &zombie_near));
    assert!(state.prompt_for(player, &zombie_near, 0, 1).is_none());
    assert_eq!(
        state.can_rescue("rescue_1", player, &zombie_near, 0, 1),
        Err(RescueBlocker::ZombiesNearby)
    );
    assert_eq!(
        state.try_rescue_nearest(player, &zombie_near, &mut recruits, 1),
        Err(RescueBlocker::ZombiesNearby)
    );

    // Living zombie beyond 12 m (or dead -> not in living_zombies) allows rescue
    let zombie_far = [Vec3::new(412.5, 0.0, 0.0)]; // 12.5 m > 12 m
    assert!(state.is_area_clear(&spot, &zombie_far));
    assert!(state.has_survivor_at_poi("poi_rescue_1"));
    assert_eq!(state.nearest_in_range(player).map(|s| s.id.as_str()), Some("rescue_1"));
    assert_eq!(spot.to_recruit().hire_cost, 0);
    let (_, prompt) = state
        .prompt_for(player, &zombie_far, 0, 1)
        .expect("prompt should show when clear and in range");
    assert_eq!(prompt, "[E] Rescue Sarah Jenkins");

    // Out of talk range (> 2.5 m) blocks rescue
    let player_far = Vec3::new(403.0, 0.0, 0.0); // 3.0 m > 2.5 m
    assert_eq!(
        state.can_rescue("rescue_1", player_far, &zombie_far, 0, 1),
        Err(RescueBlocker::OutOfRange)
    );

    let joined = state
        .try_rescue_nearest(player, &zombie_far, &mut recruits, 1)
        .expect("rescue should succeed when clear");
    assert_eq!(joined.id, "rescue_1");
    assert_eq!(joined.name, "Sarah Jenkins");
    assert_eq!(joined.hire_cost, 0);
    assert_eq!(joined.state, RecruitState::FollowingPlayer);
    assert_eq!(recruits.hired().len(), 1);
    assert_eq!(
        RescueState::joined_message(&joined.name),
        "Sarah Jenkins joined your team"
    );
}

#[test]
fn team_full_blocks_rescue_and_shows_team_full_prompt() {
    let cfg = RescueConfig::from_repo().unwrap();
    let spot = make_test_spot("rescue_1", "Mark Vance", Vec3::new(500.0, 0.0, 0.0));
    let mut state = RescueState::new(cfg, vec![spot]);
    let mut recruits = GuildRecruits::default();
    let existing = GuildRecruit::new("rec_1", "Existing", RecruitRole::Fighter, 1, 15, 100, 30);
    recruits.add_rescued(existing.clone(), 1).unwrap();
    assert_eq!(
        recruits.add_rescued(existing, 2),
        Err(RecruitHireError::AlreadyHired)
    );

    let player = Vec3::new(501.0, 0.0, 0.0);
    let (_, prompt) = state
        .prompt_for(player, &[], recruits.hired().len(), 1)
        .expect("prompt should show Team full");
    assert_eq!(prompt, "Team full");
    assert_eq!(
        state.try_rescue("rescue_1", player, &[], &mut recruits, 1),
        Err(RescueBlocker::TeamFull)
    );
    assert_eq!(state.active_spots().len(), 1, "spot remains when team is full");
}

#[test]
fn rescued_survivor_never_returns_on_restore() {
    let cfg = RescueConfig::from_repo().unwrap();
    let spot1 = make_test_spot("rescue_1", "Mark Vance", Vec3::new(500.0, 0.0, 0.0));
    let spot2 = make_test_spot("rescue_2", "Elena Rostova", Vec3::new(700.0, 0.0, 0.0));
    let mut state = RescueState::new(cfg.clone(), vec![spot1.clone(), spot2.clone()]);
    let mut recruits = GuildRecruits::default();

    let player = Vec3::new(501.0, 0.0, 0.0);
    state
        .try_rescue("rescue_1", player, &[], &mut recruits, 2)
        .unwrap();

    assert!(state.is_rescued("rescue_1"));
    assert!(!state.is_rescued("rescue_2"));
    assert_eq!(state.active_spots().len(), 1);
    assert_eq!(
        state.can_rescue("rescue_1", player, &[], 1, 2),
        Err(RescueBlocker::AlreadyRescued)
    );

    let saved_ids = state.rescued_ids_sorted();
    assert_eq!(saved_ids, vec!["rescue_1".to_string()]);

    // Simulate reload: re-place spots from seed, then restore rescued IDs
    let mut reloaded = RescueState::new(cfg, vec![spot1, spot2]);
    reloaded.restore_rescued(&saved_ids);
    assert!(reloaded.is_rescued("rescue_1"));
    assert_eq!(reloaded.active_spots().len(), 1);
    assert_eq!(reloaded.active_spots()[0].id, "rescue_2");
    assert!(reloaded.prompt_for(player, &[], 1, 2).is_none());

    let old_save: game::rescue::RescueSave = serde_json::from_str("{}").unwrap();
    assert!(old_save.rescued_ids.is_empty());
    let with_ids: game::rescue::RescueSave =
        serde_json::from_str(r#"{"reputation":10,"rescued_ids":["rescue_1"]}"#).unwrap();
    assert_eq!(with_ids.rescued_ids, vec!["rescue_1".to_string()]);
}
