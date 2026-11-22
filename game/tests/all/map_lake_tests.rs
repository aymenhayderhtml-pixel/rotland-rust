//! Unit tests for lake map geometry, simplification, bounds, and discovery checks.

use game::map::{
    is_lake_discovered, simplify_shore, LakeMapData, LAKE_DOCK_COLOR, LAKE_DOCK_WIDTH,
    LAKE_FISHING_SPOT_COLOR, LAKE_FISHING_SPOT_RADIUS, LAKE_LABEL_COLOR, LAKE_LABEL_SIZE,
    LAKE_SHORE_COLOR, LAKE_SHORE_WIDTH, LAKE_VILLAGE_LOT_COLOR, LAKE_VILLAGE_LOT_OUTLINE,
    LAKE_WATER_COLOR, MAX_SHORE_POINTS, MapExplore,
};
use glam::Vec2;
use world::{LakeConfigData, LakeLayout};

fn test_lake_layout() -> LakeLayout {
    let cfg = LakeConfigData::default();
    let camp = Vec2::new(0.0, 0.0);
    let city_center = Some(Vec2::new(500.0, 500.0));
    LakeLayout::generate(1234, camp, city_center, &cfg, |_| 10.0)
}

#[test]
fn map_lake_constants_and_colors() {
    assert_eq!(MAX_SHORE_POINTS, 64);
    assert_eq!(LAKE_WATER_COLOR, [66, 135, 245, 255]);
    assert_eq!(LAKE_SHORE_COLOR, [27, 79, 138, 255]);
    assert!((LAKE_SHORE_WIDTH - 2.0).abs() < 1e-6);
    assert_eq!(LAKE_DOCK_COLOR, [122, 82, 48, 255]);
    assert!((LAKE_DOCK_WIDTH - 2.5).abs() < 1e-6);
    assert_eq!(LAKE_VILLAGE_LOT_COLOR, [120, 122, 125, 255]);
    assert_eq!(LAKE_VILLAGE_LOT_OUTLINE, [74, 76, 78, 255]);
    assert_eq!(LAKE_FISHING_SPOT_COLOR, [240, 180, 41, 255]);
    assert!((LAKE_FISHING_SPOT_RADIUS - 3.5).abs() < 1e-6);
    assert_eq!(LAKE_LABEL_COLOR, [245, 240, 225, 255]);
    assert!((LAKE_LABEL_SIZE - 16.0).abs() < 1e-6);
}

#[test]
fn map_lake_data_polygon_closed_simplified_and_bounded() {
    let layout = test_lake_layout();
    assert!(layout.shore.len() > MAX_SHORE_POINTS);

    let lake_data = LakeMapData::from_layout(&layout);

    // 1. Polygon is simplified to at most MAX_SHORE_POINTS
    assert!(lake_data.shore.len() <= MAX_SHORE_POINTS);
    assert_eq!(lake_data.shore.len(), MAX_SHORE_POINTS);

    // 2. Polygon is closed (has at least 3 vertices)
    assert!(lake_data.is_polygon_closed());

    // 3. Shore points are strictly inside lake water bounds
    let (min_bound, max_bound) = lake_data.water_bounds;
    for &pt in &lake_data.shore {
        assert!(
            pt.x >= min_bound.x && pt.x <= max_bound.x,
            "shore pt.x {pt} outside bounds [{min_bound}, {max_bound}]"
        );
        assert!(
            pt.y >= min_bound.y && pt.y <= max_bound.y,
            "shore pt.y {pt} outside bounds [{min_bound}, {max_bound}]"
        );
    }

    // 4. Center and label pos are within bounds
    assert!(lake_data.center.x >= min_bound.x && lake_data.center.x <= max_bound.x);
    assert!(lake_data.center.y >= min_bound.y && lake_data.center.y <= max_bound.y);
    assert_eq!(lake_data.label_pos, lake_data.center);

    // 5. Docks exist and have points inside bounds
    assert!(!lake_data.docks.is_empty());
    for dock in &lake_data.docks {
        assert!(dock.start.x >= min_bound.x - 10.0 && dock.start.x <= max_bound.x + 10.0);
        assert!(dock.end.x >= min_bound.x && dock.end.x <= max_bound.x);
    }

    // 6. Fishing spots exist and have points within bounds
    assert!(!lake_data.fishing_spots.is_empty());
    for spot in &lake_data.fishing_spots {
        assert!(spot.position.x >= min_bound.x - 20.0 && spot.position.x <= max_bound.x + 20.0);
        assert!(spot.position.y >= min_bound.y - 20.0 && spot.position.y <= max_bound.y + 20.0);
    }

    // 7. Village lots exist and have 4 corners each
    assert!(!lake_data.village_lots.is_empty());
    for lot in &lake_data.village_lots {
        assert_eq!(lot.corners.len(), 4);
    }
}

#[test]
fn map_lake_simplify_shore_logic() {
    let points: Vec<Vec2> = (0..100).map(|i| Vec2::new(i as f32, (i * 2) as f32)).collect();

    // Simplify to 64 points
    let sim64 = simplify_shore(&points, 64);
    assert_eq!(sim64.len(), 64);
    assert_eq!(sim64[0], points[0]);

    // Already smaller than max points
    let sim120 = simplify_shore(&points, 120);
    assert_eq!(sim120.len(), 100);
    assert_eq!(sim120, points);

    // Empty or zero max
    let empty = simplify_shore(&[], 64);
    assert!(empty.is_empty());
    let zero = simplify_shore(&points, 0);
    assert_eq!(zero.len(), 100);
}

#[test]
fn map_lake_discovery_checks() {
    let layout = test_lake_layout();
    let lake_data = LakeMapData::from_layout(&layout);
    let mut explore = MapExplore::new();

    // 1. Initial state: unexplored
    assert!(!is_lake_discovered(&lake_data, &explore));

    // 2. Discover via center
    let mut exp_center = explore.clone();
    exp_center.reveal(lake_data.center, 16.0);
    assert!(is_lake_discovered(&lake_data, &exp_center));

    // 3. Discover via dock
    let mut exp_dock = explore.clone();
    exp_dock.reveal(lake_data.docks[0].start, 10.0);
    assert!(is_lake_discovered(&lake_data, &exp_dock));

    // 4. Discover via village lot
    let mut exp_lot = explore.clone();
    exp_lot.reveal(lake_data.village_lots[0].center, 10.0);
    assert!(is_lake_discovered(&lake_data, &exp_lot));

    // 5. Discover via fishing spot
    let mut exp_spot = explore.clone();
    exp_spot.reveal(lake_data.fishing_spots[0].position, 10.0);
    assert!(is_lake_discovered(&lake_data, &exp_spot));

    // 6. Discover via shore point
    let mut exp_shore = explore.clone();
    exp_shore.reveal(lake_data.shore[0], 10.0);
    assert!(is_lake_discovered(&lake_data, &exp_shore));

    // 7. Unrelated exploration (at camp far away) does not reveal lake
    explore.reveal(Vec2::new(0.0, 0.0), 30.0);
    assert!(!is_lake_discovered(&lake_data, &explore));
}
