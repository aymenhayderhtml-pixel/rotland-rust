use std::hint::black_box;

use glam::Vec2;
use world::biome::BiomeId;
use world::biome::layout::{WorldLayout, WorldLayoutData};
use world::roads::{RoadKind, RoadNetwork, RoadsData};
use world::towns::TownSite;

fn sample_layout(seed: i32, origin: Vec2) -> WorldLayout {
    let layout_data = WorldLayoutData::load().expect("load data/world_layout.json");
    WorldLayout::new(&layout_data, seed, origin, Vec2::X)
}

fn sample_towns(layout: &WorldLayout, origin: Vec2) -> Vec<TownSite> {
    let specs = [
        ("dusthollow", "Dusthollow", BiomeId::Desert, 0.0_f32, 3400.0_f32),
        ("blackwater", "Blackwater", BiomeId::Swamp, 90.0_f32, 3500.0_f32),
        ("frostmere", "Frostmere", BiomeId::Snow, 180.0_f32, 3600.0_f32),
        (
            "ridgeback_station",
            "Ridgeback Station",
            BiomeId::Mountains,
            270.0_f32,
            3450.0_f32,
        ),
    ];

    specs
        .into_iter()
        .map(|(key, name, biome, deg, dist)| {
            let dir = layout.direction_at_deg(deg);
            TownSite {
                key: key.to_string(),
                name: name.to_string(),
                biome,
                center: origin + dir * dist,
                radius: 120.0,
                street_dir: dir,
                lots: Vec::new(),
            }
        })
        .collect()
}

#[test]
fn four_trailheads_sit_at_sector_angles_and_expected_radius() {
    let origin = Vec2::new(120.0, -80.0);
    let layout = sample_layout(42, origin);
    let roads_data = RoadsData::load().expect("load data/roads.json");
    let towns = sample_towns(&layout, origin);

    let net = RoadNetwork::build(&roads_data, &towns, &layout, 42, origin);
    let trailheads = net.trailheads();
    assert_eq!(trailheads.len(), 4);

    let expected_radius = layout.data().core_radius + roads_data.trailhead_margin;
    let expected_degs = [0.0_f32, 90.0_f32, 180.0_f32, 270.0_f32];

    for (th, deg) in trailheads.iter().zip(expected_degs) {
        let expected_pos = origin + layout.direction_at_deg(deg) * expected_radius;
        assert!(
            th.distance(expected_pos) < 1e-3,
            "trailhead at {deg} deg was {th:?}, expected {expected_pos:?}"
        );
    }
}

#[test]
fn roads_start_and_end_at_trailheads_and_towns_with_expected_names() {
    let origin = Vec2::ZERO;
    let layout = sample_layout(42, origin);
    let roads_data = RoadsData::load().expect("load data/roads.json");
    let towns = sample_towns(&layout, origin);

    let net = RoadNetwork::build(&roads_data, &towns, &layout, 42, origin);
    let roads = net.roads();
    assert_eq!(roads.len(), 8);

    let highways: Vec<_> = roads
        .iter()
        .filter(|r| r.kind == RoadKind::Highway)
        .collect();
    assert_eq!(highways.len(), 4);

    for (i, hw) in highways.iter().enumerate() {
        let th = net.trailheads()[i];
        let town = &towns[i];
        assert_eq!(hw.name, format!("Route to {}", town.name));
        assert_eq!(*hw.points.first().unwrap(), th);
        assert_eq!(*hw.points.last().unwrap(), town.center);
    }

    let trails: Vec<_> = roads
        .iter()
        .filter(|r| r.kind == RoadKind::Trail)
        .collect();
    assert_eq!(trails.len(), 4);

    let expected_pairs = [
        (&towns[0], &towns[1]),
        (&towns[1], &towns[2]),
        (&towns[2], &towns[3]),
        (&towns[3], &towns[0]),
    ];

    for (trail, (town_a, town_b)) in trails.iter().zip(expected_pairs) {
        assert_eq!(trail.name, format!("{} - {} Trail", town_a.name, town_b.name));
        assert_eq!(*trail.points.first().unwrap(), town_a.center);
        assert_eq!(*trail.points.last().unwrap(), town_b.center);
    }
}

#[test]
fn no_road_point_or_segment_enters_core_plus_blend_band() {
    let origin = Vec2::new(-50.0, 75.0);
    let layout = sample_layout(77, origin);
    let roads_data = RoadsData::load().expect("load data/roads.json");
    let towns = sample_towns(&layout, origin);
    let min_highway_radius = layout.data().core_radius;
    let min_trail_radius = layout.data().core_radius + layout.data().blend_band;

    let net = RoadNetwork::build(&roads_data, &towns, &layout, 77, origin);
    for road in net.roads() {
        assert!(road.points.len() >= 2);
        let min_radius = if road.kind == RoadKind::Highway {
            min_highway_radius
        } else {
            min_trail_radius
        };
        for pt in &road.points {
            let r = pt.distance(origin);
            assert!(
                r > min_radius,
                "road '{}' vertex {pt:?} at radius {r} entered min radius ({min_radius})",
                road.name
            );
        }
        for pair in road.points.windows(2) {
            let mid = pair[0].lerp(pair[1], 0.5);
            let r = mid.distance(origin);
            assert!(
                r > min_radius,
                "road '{}' segment midpoint {mid:?} at radius {r} entered min radius ({min_radius})",
                road.name
            );
        }
    }
}

#[test]
fn distance_to_road_returns_expected_distances_and_cutoff() {
    let origin = Vec2::ZERO;
    let layout = sample_layout(42, origin);
    let roads_data = RoadsData::load().expect("load data/roads.json");
    let towns = sample_towns(&layout, origin);

    let net = RoadNetwork::build(&roads_data, &towns, &layout, 42, origin);
    let highway = net
        .roads()
        .iter()
        .find(|r| r.kind == RoadKind::Highway)
        .expect("highway exists");

    let mid_idx = highway.points.len() / 2;
    let on_pt = highway.points[mid_idx];
    let (d_on, kind_on) = net
        .distance_to_road(on_pt)
        .expect("point on road must be detected");
    assert!(d_on < 1e-3, "expected ~0 m on road, got {d_on}");
    assert_eq!(kind_on, RoadKind::Highway);

    let a = highway.points[mid_idx];
    let b = highway.points[mid_idx + 1];
    let seg_mid = a.lerp(b, 0.5);
    let dir = (b - a).normalize();
    let perp = Vec2::new(-dir.y, dir.x);
    let offset = 12.0_f32;
    let beside_pt = seg_mid + perp * offset;

    let (d_beside, kind_beside) = net
        .distance_to_road(beside_pt)
        .expect("point 12 m beside road must be detected");
    assert!(
        (d_beside - offset).abs() < 0.25,
        "expected ~{offset} m beside road, got {d_beside}"
    );
    assert_eq!(kind_beside, RoadKind::Highway);

    assert!(
        net.distance_to_road(origin).is_none(),
        "origin inside core must be >30 m from any road"
    );

    let far_pt = seg_mid + perp * 100.0;
    assert!(
        net.distance_to_road(far_pt).is_none(),
        "point 100 m away must return None"
    );
}

#[test]
fn same_seed_is_deterministic_and_different_seed_changes_wiggles() {
    let origin = Vec2::ZERO;
    let layout_a = sample_layout(42, origin);
    let layout_b = sample_layout(999, origin);
    let roads_data = RoadsData::load().expect("load data/roads.json");
    let towns = sample_towns(&layout_a, origin);

    let net_a1 = RoadNetwork::build(&roads_data, &towns, &layout_a, 42, origin);
    let net_a2 = RoadNetwork::build(&roads_data, &towns, &layout_a, 42, origin);
    let net_b = RoadNetwork::build(&roads_data, &towns, &layout_b, 999, origin);

    assert_eq!(net_a1.trailheads(), net_a2.trailheads());
    assert_eq!(net_a1.roads(), net_a2.roads());

    assert_eq!(net_a1.trailheads(), net_b.trailheads());
    assert_ne!(net_a1.roads(), net_b.roads());

    for (ra, rb) in net_a1.roads().iter().zip(net_b.roads()) {
        assert_eq!(ra.points.first(), rb.points.first());
        assert_eq!(ra.points.last(), rb.points.last());
    }
}

#[test]
fn hundred_thousand_distance_queries_run_cleanly() {
    let origin = Vec2::ZERO;
    let layout = sample_layout(42, origin);
    let roads_data = RoadsData::load().expect("load data/roads.json");
    let towns = sample_towns(&layout, origin);
    let net = RoadNetwork::build(&roads_data, &towns, &layout, 42, origin);

    let mut hits = 0usize;
    for i in 0..100_000_u32 {
        let fx = (i % 500) as f32 * 18.0 - 4500.0;
        let fz = (i / 500) as f32 * 45.0 - 4500.0;
        if black_box(net.distance_to_road(black_box(Vec2::new(fx, fz)))).is_some() {
            hits += 1;
        }
    }
    assert!(hits > 0);
}
