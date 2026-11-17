//! Tests for POI building discovery on the map and save round-trips.
//! Port of Godot scripts/World/WorldMap.cs NoteNearbyBuildings and scripts/UI/MapMarkers.cs.

use game::map::{
    collect_markers, letter_for, MapBuilding, MapMarkerKind, MapState, PoiSiteInfo,
};
use glam::Vec2;

#[test]
fn map_buildings_noted_once_when_within_reveal_range() {
    let mut state = MapState::new();
    assert!(state.found.is_empty());

    let poi_id = "poi_cabin_42";
    let b_type = "cabin";
    let building_xz = Vec2::new(100.0, 0.0);

    // 1. Player is at (0, 0): distance is 100 m > FOUND_RADIUS (60 m)
    let player_far = Vec2::new(0.0, 0.0);
    let noted_far = state.note_nearby_building(player_far, poi_id, b_type, building_xz, false);
    assert!(!noted_far, "Building beyond 60 m must not be discovered");
    assert!(!state.has_building(poi_id));
    assert_eq!(state.found.len(), 0);

    // 2. Player moves to (50, 0): distance is 50 m <= FOUND_RADIUS (60 m)
    let player_near = Vec2::new(50.0, 0.0);
    let noted_near = state.note_nearby_building(player_near, poi_id, b_type, building_xz, false);
    assert!(noted_near, "Building within 60 m must be discovered");
    assert!(state.has_building(poi_id));
    assert_eq!(state.found.len(), 1);

    let building = &state.found[0];
    assert_eq!(building.id, poi_id);
    assert_eq!(building.building_type, b_type);
    assert_eq!(building.position, building_xz);
    assert!(!building.looted);

    // 3. Second call at the same or nearer position: noted once (no duplicates)
    let noted_again = state.note_nearby_building(player_near, poi_id, b_type, building_xz, false);
    assert!(noted_again);
    assert_eq!(state.found.len(), 1, "Building must be noted only once");
}

#[test]
fn map_buildings_looted_status_update_and_markers() {
    let mut state = MapState::new();
    let poi_id = "poi_gas_7";
    let b_type = "gas_station";
    let building_xz = Vec2::new(30.0, 40.0); // distance from (0,0) is 50 m <= 60 m

    // Discover building unlooted
    state.note_nearby_building(Vec2::ZERO, poi_id, b_type, building_xz, false);
    assert_eq!(state.found.len(), 1);
    assert!(!state.found[0].looted);

    // Collect markers: dim is false when unlooted
    let markers_unlooted = collect_markers(&state, [], &[]);
    assert_eq!(markers_unlooted.len(), 1);
    assert_eq!(markers_unlooted[0].kind, MapMarkerKind::Building);
    assert_eq!(markers_unlooted[0].building_type, b_type);
    assert_eq!(letter_for(&markers_unlooted[0].building_type), "G");
    assert!(!markers_unlooted[0].dim);

    // Now player moves away (e.g. to (200, 200)), but building crates get looted
    let player_away = Vec2::new(200.0, 200.0);
    // Already seen building updates looted status even when far
    let updated = state.note_nearby_building(player_away, poi_id, b_type, building_xz, true);
    assert!(updated);
    assert_eq!(state.found.len(), 1);
    assert!(state.found[0].looted);

    // Collect markers: dim is true when looted
    let markers_looted = collect_markers(&state, [], &[]);
    assert_eq!(markers_looted.len(), 1);
    assert!(markers_looted[0].dim);
}

#[test]
fn map_buildings_kept_after_save_round_trip() {
    let mut state = MapState::new();

    let b1 = MapBuilding::new("poi_cabin_1", "cabin", Vec2::new(10.0, 20.0), false);
    let b2 = MapBuilding::new("poi_barn_2", "barn", Vec2::new(-50.0, 80.0), true);

    state.note_building(b1.clone());
    state.note_building(b2.clone());
    assert_eq!(state.found.len(), 2);

    // Save to JSON
    let save_json = state.to_save();

    // Verify Godot save structure: "found" array of buildings
    let found_array = save_json
        .get("found")
        .and_then(|v| v.as_array())
        .expect("save data must contain 'found' array");
    assert_eq!(found_array.len(), 2);

    // Check keys in first building
    let first = &found_array[0];
    assert_eq!(first.get("id").and_then(|v| v.as_str()), Some("poi_cabin_1"));
    assert_eq!(first.get("type").and_then(|v| v.as_str()), Some("cabin"));
    assert_eq!(first.get("x").and_then(|v| v.as_f64()), Some(10.0));
    assert_eq!(first.get("z").and_then(|v| v.as_f64()), Some(20.0));
    assert_eq!(first.get("looted").and_then(|v| v.as_bool()), Some(false));

    // Restore into a fresh MapState
    let mut restored = MapState::new();
    restored.from_save(&save_json);

    assert_eq!(restored.found.len(), 2);
    assert!(restored.has_building("poi_cabin_1"));
    assert!(restored.has_building("poi_barn_2"));

    assert_eq!(restored.found[0], b1);
    assert_eq!(restored.found[1], b2);

    // Markers from restored state match expectations
    let markers = collect_markers(&restored, [], &[]);
    assert_eq!(markers.len(), 2);
    assert_eq!(markers[0].building_type, "cabin");
    assert_eq!(letter_for(&markers[0].building_type), "C");
    assert!(!markers[0].dim);

    assert_eq!(markers[1].building_type, "barn");
    assert_eq!(letter_for(&markers[1].building_type), "B");
    assert!(markers[1].dim);
}

#[test]
fn map_buildings_note_nearby_poi_sites_scanning() {
    let mut state = MapState::new();

    let site1 = PoiSiteInfo::new("shed_1", "shed", Vec2::new(10.0, 10.0), vec!["c1", "c2"]);
    let site2 = PoiSiteInfo::new("tower_2", "watchtower", Vec2::new(500.0, 500.0), vec!["c3"]);

    let sites = vec![site1, site2];

    // Player at origin (0, 0): shed_1 is at ~14.1 m (discovered), tower_2 is at ~707 m (not discovered)
    let newly_noted = state.note_nearby_poi_sites(Vec2::ZERO, &sites, |_| false);
    assert_eq!(newly_noted, 1);
    assert_eq!(state.found.len(), 1);
    assert!(state.has_building("shed_1"));
    assert!(!state.has_building("tower_2"));
    assert!(!state.found[0].looted);

    // Scan again with same state: 0 newly noted
    let newly_noted_2 = state.note_nearby_poi_sites(Vec2::ZERO, &sites, |_| false);
    assert_eq!(newly_noted_2, 0);

    // Crate c1 looted, c2 still unlooted -> still not fully looted
    state.note_nearby_poi_sites(Vec2::ZERO, &sites, |id| id == "c1");
    assert!(!state.found[0].looted);

    // Both c1 and c2 looted -> now marked looted
    state.note_nearby_poi_sites(Vec2::ZERO, &sites, |id| id == "c1" || id == "c2");
    assert!(state.found[0].looted);

    // Player moves near tower_2: newly noted becomes 1
    let newly_noted_3 = state.note_nearby_poi_sites(Vec2::new(500.0, 510.0), &sites, |_| false);
    assert_eq!(newly_noted_3, 1);
    assert_eq!(state.found.len(), 2);
    assert!(state.has_building("tower_2"));
}
