//! Unit tests for city map pure helpers (exploration checks, spans, camp road discovery, labels, bounds).

use game::map::{
    city_center, district_center, explored_segment_spans, is_city_center_revealed,
    is_district_revealed, is_road_discovered, is_segment_any_explored, is_segment_explored,
    map_visible_world_bounds, segment_in_bounds, screen_to_world, world_to_screen,
    MapExplore, CITY_BLOCKED_COLOR, CITY_ROAD_COLOR, CITY_STREET_COLOR, CITY_STREET_MIN_PX,
};
use glam::Vec2;
use world::{CityBlock, CityLayout, District, StreetSegment};

#[test]
fn map_city_constants_and_colors() {
    assert_eq!(CITY_STREET_COLOR, [69, 69, 74, 255]);
    assert_eq!(CITY_ROAD_COLOR, [90, 83, 72, 255]);
    assert_eq!(CITY_BLOCKED_COLOR, [217, 53, 43, 255]);
    assert!((CITY_STREET_MIN_PX - 2.0).abs() < 1e-6);
}

#[test]
fn map_city_segment_explored_checks() {
    let mut explore = MapExplore::new();
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(64.0, 0.0);

    // Completely unrevealed
    assert!(!is_segment_explored(a, b, &explore));
    assert!(!is_segment_any_explored(a, b, &explore));
    assert!(explored_segment_spans(a, b, &explore).is_empty());

    // Reveal only one end (around x = 0)
    explore.reveal(a, 16.0);
    assert!(!is_segment_explored(a, b, &explore));
    assert!(is_segment_any_explored(a, b, &explore));

    let spans = explored_segment_spans(a, b, &explore);
    assert!(!spans.is_empty());
    assert_eq!(spans[0].0, a);
    assert!(spans[0].1.x < b.x);

    // Reveal the rest of the segment
    explore.reveal(b, 60.0);
    assert!(is_segment_explored(a, b, &explore));
    assert!(is_segment_any_explored(a, b, &explore));

    let full_spans = explored_segment_spans(a, b, &explore);
    assert_eq!(full_spans.len(), 1);
    assert!((full_spans[0].0.x - a.x).abs() < 1e-4);
    assert!((full_spans[0].1.x - b.x).abs() < 1e-4);
}

#[test]
fn map_city_road_discovered_once_any_part_visited() {
    let mut explore = MapExplore::new();
    let road = vec![Vec2::new(100.0, 100.0), Vec2::new(500.0, 500.0)];

    // Empty road
    assert!(!is_road_discovered(&[], &explore));

    // Road is unvisited
    assert!(!is_road_discovered(&road, &explore));

    // Player visits near road midpoint
    let midpoint = (road[0] + road[1]) * 0.5;
    explore.reveal(midpoint, 20.0);

    // Now road is discovered
    assert!(is_road_discovered(&road, &explore));

    // Single-point road
    let single = vec![Vec2::new(100.0, 100.0)];
    let mut empty_explore = MapExplore::new();
    assert!(!is_road_discovered(&single, &empty_explore));
    empty_explore.reveal(Vec2::new(100.0, 100.0), 10.0);
    assert!(is_road_discovered(&single, &empty_explore));
}

#[test]
fn map_city_center_and_district_reveals() {
    let mut explore = MapExplore::new();
    let origin = Vec2::new(200.0, 200.0);
    let size = 640.0;

    let city = CityLayout {
        origin,
        size,
        blocks: vec![
            CityBlock {
                coord: glam::IVec2::new(3, 3),
                district: District::Core,
                present: true,
            },
            CityBlock {
                coord: glam::IVec2::new(3, 4),
                district: District::Core,
                present: true,
            },
            CityBlock {
                coord: glam::IVec2::new(5, 5),
                district: District::Industrial,
                present: true,
            },
        ],
        streets: vec![StreetSegment {
            a: origin,
            b: origin + Vec2::new(64.0, 0.0),
            width: 10.0,
            blocked: false,
            avenue: false,
        }],
        lots: Vec::new(),
        entry: origin,
        road: vec![Vec2::ZERO, origin],
    };

    let c = city_center(&city);
    assert_eq!(c, origin + Vec2::splat(320.0));

    // Unrevealed initially
    assert!(!is_city_center_revealed(&city, &explore));
    assert!(!is_district_revealed(&city, District::Core, &explore));
    assert!(!is_district_revealed(&city, District::Industrial, &explore));

    // District centers
    let core_c = district_center(&city, District::Core).expect("Core center");
    let ind_c = district_center(&city, District::Industrial).expect("Industrial center");
    assert!(core_c.x > 0.0 && core_c.y > 0.0);
    assert!(ind_c.x > 0.0 && ind_c.y > 0.0);
    assert_eq!(district_center(&city, District::Park), None);

    // Reveal industrial block center
    explore.reveal(ind_c, 20.0);
    assert!(is_district_revealed(&city, District::Industrial, &explore));
    assert!(!is_district_revealed(&city, District::Core, &explore));
    assert!(!is_city_center_revealed(&city, &explore));

    // Reveal Core block center
    explore.reveal(core_c, 20.0);
    assert!(is_district_revealed(&city, District::Core, &explore));
    assert!(is_city_center_revealed(&city, &explore));
}

#[test]
fn map_city_bounds_and_culling() {
    let centre = Vec2::new(500.0, 500.0);
    let scale = 0.5;
    let view_size = Vec2::new(800.0, 600.0);
    let margin = 20.0;

    let (min, max) = map_visible_world_bounds(centre, scale, view_size, margin);
    // view_size * 0.5 / 0.5 = (800, 600), plus margin 20 = (820, 620)
    assert!((min.x - (500.0 - 820.0)).abs() < 1e-4);
    assert!((max.x - (500.0 + 820.0)).abs() < 1e-4);

    // Segment inside bounds
    let a = Vec2::new(500.0, 500.0);
    let b = Vec2::new(550.0, 550.0);
    assert!(segment_in_bounds(a, b, min, max, 10.0));

    // Segment outside bounds to the far left
    let far_a = Vec2::new(-2000.0, 500.0);
    let far_b = Vec2::new(-1900.0, 500.0);
    assert!(!segment_in_bounds(far_a, far_b, min, max, 10.0));

    // Segment crossing bounds
    let cross_a = Vec2::new(-2000.0, 500.0);
    let cross_b = Vec2::new(500.0, 500.0);
    assert!(segment_in_bounds(cross_a, cross_b, min, max, 10.0));
}

#[test]
fn map_city_world_to_map_transform_reuse() {
    let centre = Vec2::new(300.0, -150.0);
    let scale = 1.0;
    let view_size = Vec2::new(1024.0, 768.0);

    let city_pt = Vec2::new(350.0, -100.0);
    let screen = world_to_screen(city_pt, centre, scale, view_size);
    let world_back = screen_to_world(screen, centre, scale, view_size);

    assert!((world_back - city_pt).length() < 1e-4);
}
