//! Unit tests for pure HUD mini-map projection and layout.

use std::f32::consts::FRAC_PI_2;
use std::path::Path;

use game::{
    MapBuilding, MapMarkerKind, MapState, MinimapConfig, MinimapCorner, collect_minimap_markers,
    collect_minimap_zombies, minimap_chunk_window, minimap_circle, minimap_player_axes,
    world_to_minimap_offset,
};
use glam::{IVec2, Vec2};

#[test]
fn config_loads_data_and_defaults_on_missing_file() {
    let cfg = MinimapConfig::load_default();
    assert_eq!(cfg, MinimapConfig::default());
    assert_eq!(
        MinimapConfig::load(Path::new("/missing/minimap.json")),
        MinimapConfig::default()
    );
}

#[test]
fn circle_position_and_scale() {
    let cfg = MinimapConfig::default();
    let (centre, radius) = minimap_circle(Vec2::new(1920.0, 1080.0), 1.0, &cfg);
    assert_eq!(radius, 86.0);
    assert_eq!(centre, Vec2::new(102.0, 102.0));
    let mut cfg = cfg;
    cfg.corner = MinimapCorner::TopRight;
    let (centre, radius) = minimap_circle(Vec2::new(1920.0, 1080.0), 0.5, &cfg);
    assert_eq!(radius, 43.0);
    assert_eq!(centre, Vec2::new(1869.0, 51.0));
}

#[test]
fn world_projection_is_north_up() {
    let p = Vec2::new(100.0, 200.0);
    let north = world_to_minimap_offset(Vec2::new(100.0, 140.0), p, 60.0, 90.0);
    let east = world_to_minimap_offset(Vec2::new(130.0, 200.0), p, 60.0, 90.0);
    assert!((north - Vec2::new(0.0, -90.0)).length() < 1e-4);
    assert!((east - Vec2::new(45.0, 0.0)).length() < 1e-4);
}

#[test]
fn player_axes_match_yaw_convention() {
    let (f, r) = minimap_player_axes(0.0);
    assert!((f - Vec2::new(0.0, -1.0)).length() < 1e-5);
    assert!((r - Vec2::X).length() < 1e-5);
    let (east, _) = minimap_player_axes(-FRAC_PI_2);
    let (west, _) = minimap_player_axes(FRAC_PI_2);
    assert!((east - Vec2::X).length() < 1e-5);
    assert!((west + Vec2::X).length() < 1e-5);
}

#[test]
fn filters_markers_and_pins_camp_and_waypoint() {
    let mut map = MapState::new();
    map.note_building(MapBuilding::new(
        "near",
        "cabin",
        Vec2::new(20.0, 0.0),
        false,
    ));
    map.note_building(MapBuilding::new("far", "barn", Vec2::new(120.0, 0.0), true));
    map.set_waypoint(Vec2::new(0.0, -180.0));
    let markers = collect_minimap_markers(
        &map,
        [Vec2::new(120.0, 0.0)],
        &[],
        Vec2::ZERO,
        60.0,
        86.0,
        78.0,
    );
    let buildings: Vec<_> = markers
        .iter()
        .filter(|m| m.kind == MapMarkerKind::Building)
        .collect();
    assert_eq!(buildings.len(), 1);
    assert_eq!(buildings[0].building_type, "cabin");
    let camp = markers
        .iter()
        .find(|m| m.kind == MapMarkerKind::Camp)
        .unwrap();
    assert!(camp.pinned);
    assert!((camp.offset_px.x - 78.0).abs() < 1e-3);
    let wp = markers
        .iter()
        .find(|m| m.kind == MapMarkerKind::Waypoint)
        .unwrap();
    assert!(wp.pinned);
    assert!((wp.offset_px.y + 78.0).abs() < 1e-3);
}

#[test]
fn filters_zombies_to_circle() {
    let dots = collect_minimap_zombies(
        [
            Vec2::new(10.0, 0.0),
            Vec2::new(0.0, -30.0),
            Vec2::splat(50.0),
        ],
        Vec2::ZERO,
        60.0,
        86.0,
        78.0,
    );
    assert_eq!(dots.len(), 2);
}

#[test]
fn finds_chunk_window() {
    assert_eq!(
        minimap_chunk_window(Vec2::new(-10.0, 70.0), 60.0),
        (IVec2::new(-1, 1), 1)
    );
    assert_eq!(minimap_chunk_window(Vec2::ZERO, 100.0).1, 2);
}
