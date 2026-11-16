//! Tests for map math, compass, marker collection/saving, and explored tiles.
//! Port of Godot map and compass test specifications.

use game::bounties::BountyDef;
use game::map::{
    bearing_to_view_deg, collect_markers, direction_name, letter_for, marker_x,
    relative_angle_deg, screen_to_world, step_zoom, world_to_screen, MapBuilding, MapExplore,
    MapMarkerKind, MapState, MAP_DEFAULT_ZOOM, HALF_VIEW_DEG, SCALES,
};
use glam::{Vec2, Vec3};

#[test]
fn map_tests_pixel_math_round_trip() {
    let centres = [
        Vec2::ZERO,
        Vec2::new(150.0, -250.0),
        Vec2::new(-500.0, 300.0),
    ];
    let view_sizes = [
        Vec2::new(800.0, 600.0),
        Vec2::new(1920.0, 1080.0),
    ];
    let test_points = [
        Vec2::ZERO,
        Vec2::new(320.0, -180.0),
        Vec2::new(-40.0, 1000.0),
        Vec2::new(150.0, -250.0),
    ];

    let mut checks = 0;
    for &centre in &centres {
        for &view_size in &view_sizes {
            for &scale in &SCALES {
                for &world in &test_points {
                    let screen = world_to_screen(world, centre, scale, view_size);
                    let back = screen_to_world(screen, centre, scale, view_size);
                    assert!(
                        (back - world).length() < 1e-4,
                        "Round-trip failed: expected {world:?}, got {back:?} at scale {scale}"
                    );
                    checks += 1;
                }
            }
        }
    }
    assert_eq!(checks, 3 * 2 * 3 * 4); // 72 round trip comparisons

    // Verify step_zoom bounds
    assert_eq!(step_zoom(MAP_DEFAULT_ZOOM, 1), 2);
    assert_eq!(step_zoom(2, 1), 2);
    assert_eq!(step_zoom(2, 10), 2);
    assert_eq!(step_zoom(MAP_DEFAULT_ZOOM, -1), 0);
    assert_eq!(step_zoom(0, -1), 0);
    assert_eq!(step_zoom(0, -10), 0);
}

#[test]
fn map_tests_compass_bearing_cardinal_and_ordinal() {
    let player = Vec3::ZERO;
    let yaw_north = 0.0f32; // Looking North (-Z)

    // 1. Facing North:
    // Target North is at (0, 0, -100)
    let target_n = Vec3::new(0.0, 0.0, -100.0);
    let angle_n = relative_angle_deg(player, target_n, yaw_north);
    assert!((angle_n - 0.0).abs() < 1e-5);
    assert_eq!(direction_name(0), "N");
    assert!((bearing_to_view_deg(0.0, yaw_north) - 0.0).abs() < 1e-5);
    assert!((marker_x(0.0, 600.0) - 300.0).abs() < 1e-5);

    // Target East is at (100, 0, 0)
    let target_e = Vec3::new(100.0, 0.0, 0.0);
    let angle_e = relative_angle_deg(player, target_e, yaw_north);
    assert!((angle_e - 90.0).abs() < 1e-5);
    assert_eq!(direction_name(90), "E");
    assert!((bearing_to_view_deg(90.0, yaw_north) - 90.0).abs() < 1e-5);
    assert!((marker_x(90.0, 600.0) - 600.0).abs() < 1e-5);

    // Target South is at (0, 0, 100)
    let target_s = Vec3::new(0.0, 0.0, 100.0);
    let angle_s = relative_angle_deg(player, target_s, yaw_north);
    assert!((angle_s.abs() - 180.0).abs() < 1e-5);
    assert_eq!(direction_name(180), "S");
    assert!((bearing_to_view_deg(180.0, yaw_north).abs() - 180.0).abs() < 1e-5);

    // Target West is at (-100, 0, 0)
    let target_w = Vec3::new(-100.0, 0.0, 0.0);
    let angle_w = relative_angle_deg(player, target_w, yaw_north);
    assert!((angle_w - (-90.0)).abs() < 1e-5);
    assert_eq!(direction_name(270), "W");
    assert!((bearing_to_view_deg(270.0, yaw_north) - (-90.0)).abs() < 1e-5);
    assert!((marker_x(-90.0, 600.0) - 0.0).abs() < 1e-5);

    // Ordinal directions
    assert_eq!(direction_name(45), "NE");
    assert_eq!(direction_name(135), "SE");
    assert_eq!(direction_name(225), "SW");
    assert_eq!(direction_name(315), "NW");
    assert_eq!(direction_name(360), "N");

    // 2. Facing East: forward is (-sin yaw, -cos yaw), so east is yaw = -PI/2 (+PI/2 faces west)
    let yaw_east = -std::f32::consts::FRAC_PI_2;
    // North is now 90 degrees to the left (-90 deg relative)
    let angle_n_when_facing_e = relative_angle_deg(player, target_n, yaw_east);
    assert!((angle_n_when_facing_e - (-90.0)).abs() < 1e-4);
    assert!((bearing_to_view_deg(0.0, yaw_east) - (-90.0)).abs() < 1e-4);

    // East is directly ahead (0 deg relative)
    let angle_e_when_facing_e = relative_angle_deg(player, target_e, yaw_east);
    assert!((angle_e_when_facing_e - 0.0).abs() < 1e-4);
    assert!((bearing_to_view_deg(90.0, yaw_east) - 0.0).abs() < 1e-4);

    // MarkerX clamping beyond half view (90 deg)
    assert_eq!(marker_x(120.0, 600.0), 600.0);
    assert_eq!(marker_x(-150.0, 600.0), 0.0);
    assert_eq!(HALF_VIEW_DEG, 90.0);
}

#[test]
fn map_tests_marker_add_remove_save() {
    let mut state = MapState::new();
    assert!(state.found.is_empty());
    assert_eq!(state.waypoint, None);
    assert_eq!(state.death_spot, None);

    // Add waypoint and death spot
    state.set_waypoint(Vec2::new(120.0, -340.0));
    state.death_spot = Some(Vec2::new(-45.0, 78.0));

    // Note buildings
    state.note_building(MapBuilding::new("poi_cabin_1", "cabin", Vec2::new(64.0, 128.0), false));
    state.note_building(MapBuilding::new("poi_gas_1", "gas_station", Vec2::new(200.0, -100.0), false));
    assert!(state.has_building("poi_cabin_1"));
    assert!(state.has_building("poi_gas_1"));
    assert!(!state.has_building("poi_unknown"));
    assert_eq!(state.found.len(), 2);

    // Updating existing building updates looted flag without duplicating
    state.note_building(MapBuilding::new("poi_gas_1", "gas_station", Vec2::new(200.0, -100.0), true));
    assert_eq!(state.found.len(), 2);
    assert!(state.found[1].looted);

    // Collect markers with a camp and bounties
    let camps = [Vec2::new(0.0, 0.0)];
    let bounty_active = BountyDef {
        id: "b1".to_string(),
        bounty_type: "clear_poi".to_string(),
        target: 1,
        progress: 0,
        target_x: 200.0,
        target_z: -100.0,
        ..Default::default()
    };

    let bounty_done = BountyDef {
        id: "b2".to_string(),
        bounty_type: "clear_poi".to_string(),
        target: 1,
        progress: 1,
        ..Default::default()
    };

    let bounty_other = BountyDef { id: "b3".to_string(), bounty_type: "kill_zombies".to_string(), ..Default::default() };

    let bounties = [bounty_active, bounty_done, bounty_other];
    let markers = collect_markers(&state, camps, &bounties);

    // Expected markers: Camp (1), Building (2), Bounty (1, active clear_poi only), Death (1), Waypoint (1) = 6
    assert_eq!(markers.len(), 6);
    assert_eq!(markers[0].kind, MapMarkerKind::Camp);
    assert_eq!(markers[1].kind, MapMarkerKind::Building);
    assert_eq!(markers[1].building_type, "cabin");
    assert_eq!(letter_for(&markers[1].building_type), "C");
    assert!(!markers[1].dim);

    assert_eq!(markers[2].kind, MapMarkerKind::Building);
    assert_eq!(markers[2].building_type, "gas_station");
    assert_eq!(letter_for(&markers[2].building_type), "G");
    assert!(markers[2].dim);

    assert_eq!(markers[3].kind, MapMarkerKind::Bounty);
    assert_eq!(markers[3].position, Vec2::new(200.0, -100.0));

    assert_eq!(markers[4].kind, MapMarkerKind::Death);
    assert_eq!(markers[4].position, Vec2::new(-45.0, 78.0));

    assert_eq!(markers[5].kind, MapMarkerKind::Waypoint);
    assert_eq!(markers[5].position, Vec2::new(120.0, -340.0));

    // Save and load round-trip
    let saved_val = state.to_save();
    let mut restored = MapState::new();
    restored.from_save(&saved_val);

    assert_eq!(restored.waypoint, Some(Vec2::new(120.0, -340.0)));
    assert_eq!(restored.death_spot, Some(Vec2::new(-45.0, 78.0)));
    assert_eq!(restored.found.len(), 2);
    assert_eq!(restored.found[0].id, "poi_cabin_1");
    assert_eq!(restored.found[1].id, "poi_gas_1");
    assert!(restored.found[1].looted);

    // Clear waypoint
    restored.clear_waypoint();
    assert_eq!(restored.waypoint, None);
    let markers_after_clear = collect_markers(&restored, camps, &bounties);
    assert_eq!(markers_after_clear.len(), 5);
}

#[test]
fn map_tests_explored_tiles_reveal_and_save() {
    let mut explore = MapExplore::new();
    assert_eq!(explore.chunk_count(), 0);

    // Reveal at world (0, 0) with radius 40m
    let added = explore.reveal(Vec2::ZERO, 40.0);
    assert!(added > 0);
    assert!(explore.chunk_count() > 0);

    // Cell at origin (center is (4, 4), dist_sq = 32 <= 1600)
    let cell_origin = MapExplore::cell_of(Vec2::ZERO);
    assert!(explore.is_revealed(cell_origin));

    // Cell at 24m (center is (28, 4), dist_sq = 784 + 16 = 800 <= 1600)
    let cell_24m = MapExplore::cell_of(Vec2::new(24.0, 0.0));
    assert!(explore.is_revealed(cell_24m));

    // Cell at 60m (center is (60, 4), dist_sq = 3600 + 16 > 1600)
    let cell_60m = MapExplore::cell_of(Vec2::new(60.0, 0.0));
    assert!(!explore.is_revealed(cell_60m));

    // Revealing the same spot again returns 0 newly added cells
    let added_again = explore.reveal(Vec2::ZERO, 40.0);
    assert_eq!(added_again, 0);

    // Save and load round-trip
    let save_json = explore.to_save();
    let mut restored = MapExplore::new();
    restored.from_save(&save_json);

    assert_eq!(restored.chunk_count(), explore.chunk_count());
    assert!(restored.is_revealed(cell_origin));
    assert!(restored.is_revealed(cell_24m));
    assert!(!restored.is_revealed(cell_60m));

    // Check dirty chunk tracking
    let chunk0 = MapExplore::chunk_of_cell(cell_origin);
    assert!(restored.is_dirty(chunk0));
    restored.clear_dirty(chunk0);
    assert!(!restored.is_dirty(chunk0));
}
