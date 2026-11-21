use glam::Vec2;
use paths::repo_path;

use crate::lake::config::{LakeConfig, LakeConfigData};
use crate::lake::layout::{FishingSpotKind, LakeLayout, LakeZone};

fn load_test_cfg() -> LakeConfig {
    let path = repo_path("data/lake.json");
    LakeConfig::load_from_file(&path).expect("Failed to load data/lake.json")
}

#[test]
fn lake_config_load() {
    let cfg = load_test_cfg();
    assert_eq!(cfg.distance_from_camp, 1500.0);
    assert_eq!(cfg.radius, [260.0, 200.0]);
    assert_eq!(cfg.shore_noise.amplitude, 12.0);
    assert_eq!(cfg.shore_noise.frequency, 0.02);
    assert_eq!(cfg.beach_width, 14.0);
    assert_eq!(cfg.shallow_width, 12.0);
    assert_eq!(cfg.max_depth, 8.0);
    assert_eq!(cfg.water_level_above_shore, 0.3);
    assert_eq!(cfg.village.count, 8);
    assert_eq!(cfg.village.sizes.len(), 8);
    assert_eq!(cfg.docks.count, 3);
    assert_eq!(cfg.docks.width, 4.0);
    assert_eq!(cfg.docks.length, 20.0);
    assert_eq!(cfg.angle_from_city, 180.0);
    assert_eq!(cfg.beach_blend, 16.0);
    assert_eq!(cfg.shore_samples, 128);
    assert_eq!(cfg.region_margin, 32.0);

    assert!(cfg.validate().is_ok());

    let json_str = serde_json::to_string(&cfg).unwrap();
    let cfg2 = LakeConfigData::from_json(&json_str).unwrap();
    assert_eq!(cfg, cfg2);
}

#[test]
fn lake_deterministic() {
    let cfg = load_test_cfg();
    let camp = Vec2::new(100.0, -50.0);
    let city = Vec2::new(1600.0, -50.0);

    let layout_a1 = LakeLayout::generate(12345, camp, Some(city), &cfg, |_| 15.0);
    let layout_a2 = LakeLayout::generate(12345, camp, Some(city), &cfg, |_| 15.0);
    let layout_b = LakeLayout::generate(54321, camp, Some(city), &cfg, |_| 15.0);

    assert_eq!(layout_a1, layout_a2);
    assert_ne!(layout_a1, layout_b);
}

#[test]
fn lake_opposite_city() {
    let cfg = load_test_cfg();
    let camp = Vec2::new(50.0, 100.0);

    for (cx, cy) in [(1550.0, 100.0), (50.0, 1600.0), (-1000.0, -1000.0)] {
        let city = Vec2::new(cx, cy);
        let layout = LakeLayout::generate(42, camp, Some(city), &cfg, |_| 10.0);

        let to_city = (city - camp).normalize();
        let to_lake = (layout.center - camp).normalize();
        let dot = to_city.dot(to_lake);
        assert!(
            (dot - (-1.0)).abs() < 1e-3,
            "Lake center should be opposite the city from camp, dot was {dot}"
        );
    }

    let layout_no_city = LakeLayout::generate(42, camp, None, &cfg, |_| 10.0);
    assert!(layout_no_city.center.distance(camp) > cfg.distance_from_camp);
}

#[test]
fn lake_distance_from_camp() {
    let cfg = load_test_cfg();
    let camp = Vec2::new(0.0, 0.0);
    let city = Vec2::new(1500.0, 0.0);

    for seed in [1, 7, 42, 1234, 9999] {
        let layout = LakeLayout::generate(seed, camp, Some(city), &cfg, |_| 10.0);
        let min_shore_dist = layout
            .shore
            .iter()
            .map(|&p| camp.distance(p))
            .fold(f32::INFINITY, f32::min);

        let diff = (min_shore_dist - cfg.distance_from_camp).abs();
        assert!(
            diff < 30.0,
            "seed {seed}: nearest shore distance {min_shore_dist} deviates by {diff} from 1500m"
        );
    }
}

#[test]
fn lake_depth_at() {
    let cfg = load_test_cfg();
    let camp = Vec2::ZERO;
    let city = Vec2::new(1500.0, 0.0);
    let layout = LakeLayout::generate(42, camp, Some(city), &cfg, |_| 10.0);

    for pt in &layout.shore {
        assert!(
            layout.depth_at(*pt) <= 1e-4,
            "Depth at shore must be 0, got {}",
            layout.depth_at(*pt)
        );
        assert!(
            layout.shore_distance(*pt).abs() <= 1e-3,
            "Shore point distance must be 0, got {}",
            layout.shore_distance(*pt)
        );
    }

    let center_depth = layout.depth_at(layout.center);
    assert!(
        (center_depth - cfg.max_depth).abs() < 1e-4,
        "Depth at center must be maxDepth {}, got {center_depth}",
        cfg.max_depth
    );

    let shore_pt = layout.shore[0];
    let to_center = layout.center - shore_pt;
    let steps = 20;
    let mut prev_depth = 0.0;
    for i in 1..=steps {
        let frac = i as f32 / steps as f32;
        let p = shore_pt + to_center * frac;
        let depth = layout.depth_at(p);
        assert!(
            depth >= prev_depth - 1e-4,
            "Depth must increase toward middle: step {i}: {depth} vs prev {prev_depth}"
        );
        prev_depth = depth;
    }
}

#[test]
fn lake_zones_order() {
    let cfg = load_test_cfg();
    let camp = Vec2::ZERO;
    let city = Vec2::new(1500.0, 0.0);
    let layout = LakeLayout::generate(42, camp, Some(city), &cfg, |_| 10.0);

    let shore_pt = layout.shore[0];
    let from_center = (shore_pt - layout.center).normalize();

    let mut observed_zones = Vec::new();
    let start_pos = shore_pt + from_center * 50.0;
    let end_pos = layout.center;

    let steps = 150;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let p = start_pos.lerp(end_pos, t);
        let zone = layout.zone_at(p);
        if observed_zones.last() != Some(&zone) {
            observed_zones.push(zone);
        }
    }

    assert_eq!(
        observed_zones,
        vec![
            LakeZone::Land,
            LakeZone::Beach,
            LakeZone::Shallow,
            LakeZone::Deep
        ],
        "Zones walking from land to middle must be Land -> Beach -> Shallow -> Deep"
    );
}

#[test]
fn lake_docks_start_on_land_end_in_water() {
    let cfg = load_test_cfg();
    let camp = Vec2::ZERO;
    let city = Vec2::new(1500.0, 0.0);
    let layout = LakeLayout::generate(42, camp, Some(city), &cfg, |_| 10.0);

    assert_eq!(layout.docks.len(), 3);
    for (i, dock) in layout.docks.iter().enumerate() {
        let start_dist = layout.shore_distance(dock.start);
        let end_dist = layout.shore_distance(dock.end);

        assert!(
            start_dist > 0.0,
            "dock {i} start must be on land (shore_dist > 0), got {start_dist}"
        );
        assert!(
            end_dist < 0.0,
            "dock {i} end must be over water (shore_dist < 0), got {end_dist}"
        );

        let length = dock.length();
        assert!(
            (length - cfg.docks.length).abs() < 1e-3,
            "dock {i} length must be {}, got {length}",
            cfg.docks.length
        );

        let fwd = dock.forward();
        let expected_end = dock.start + fwd * length;
        assert!(
            expected_end.distance(dock.end) < 1e-3,
            "dock {i} forward direction mismatch"
        );
    }
}

#[test]
fn lake_village_lots_valid() {
    let cfg = load_test_cfg();
    let camp = Vec2::ZERO;
    let city = Vec2::new(1500.0, 0.0);
    let layout = LakeLayout::generate(42, camp, Some(city), &cfg, |_| 10.0);

    assert_eq!(layout.village_lots.len(), 8);

    for (i, lot) in layout.village_lots.iter().enumerate() {
        let center_dist = layout.shore_distance(lot.center);
        assert!(
            center_dist > 0.0,
            "lot {i} center must be on land (shore_dist > 0), got {center_dist}"
        );

        for (c_idx, &corner) in lot.corners().iter().enumerate() {
            let corner_dist = layout.shore_distance(corner);
            assert!(
                corner_dist > 0.0,
                "lot {i} corner {c_idx} must be on land (shore_dist > 0), got {corner_dist}"
            );
        }

        let (min, max) = lot.aabb();
        assert!(min.x <= max.x && min.y <= max.y);

        let fwd = Vec2::new(-lot.yaw.sin(), -lot.yaw.cos());
        let to_lake_center = (layout.center - lot.center).normalize();
        let facing_dot = fwd.dot(to_lake_center);
        assert!(
            facing_dot > 0.5,
            "lot {i} should face toward the lake, dot was {facing_dot}"
        );
    }

    for i in 0..layout.village_lots.len() {
        for j in (i + 1)..layout.village_lots.len() {
            assert!(
                !layout.village_lots[i].overlaps(&layout.village_lots[j]),
                "village lot {i} overlaps with lot {j}"
            );
        }
    }
}

#[test]
fn lake_fishing_spots() {
    let cfg = load_test_cfg();
    let camp = Vec2::ZERO;
    let city = Vec2::new(1500.0, 0.0);
    let layout = LakeLayout::generate(42, camp, Some(city), &cfg, |_| 10.0);

    assert!(!layout.fishing_spots.is_empty());

    let mut dock_spots = 0;
    let mut beach_spots = 0;

    for spot in &layout.fishing_spots {
        match spot.kind {
            FishingSpotKind::Dock => {
                dock_spots += 1;
                let on_any_dock = layout
                    .docks
                    .iter()
                    .any(|d| d.end.distance(spot.position) < 1e-3);
                assert!(on_any_dock, "dock fishing spot must be on a dock end");
            }
            FishingSpotKind::Beach => {
                beach_spots += 1;
                let zone = layout.zone_at(spot.position);
                assert_eq!(
                    zone,
                    LakeZone::Beach,
                    "beach fishing spot must be in Beach zone"
                );
            }
        }
    }

    assert_eq!(dock_spots, 3);
    assert!(beach_spots > 0);
}

#[test]
fn lake_ground_height_carving() {
    let cfg = load_test_cfg();
    let camp = Vec2::ZERO;
    let city = Vec2::new(1500.0, 0.0);
    let raw_h = 25.0;
    let layout = LakeLayout::generate(42, camp, Some(city), &cfg, |_| raw_h);

    let center_h = layout.ground_height(layout.center, raw_h);
    let expected_center_h = layout.water_level - cfg.max_depth;
    assert!(
        (center_h - expected_center_h).abs() < 1e-4,
        "Center ground height should be water_level - maxDepth ({expected_center_h}), got {center_h}"
    );

    let beach_spot = layout
        .fishing_spots
        .iter()
        .find(|s| s.kind == FishingSpotKind::Beach)
        .unwrap();
    let beach_h = layout.ground_height(beach_spot.position, raw_h);
    assert!(
        beach_h >= layout.water_level - 1e-4
            && beach_h <= layout.water_level + cfg.water_level_above_shore + 1e-4,
        "Beach height should be just above water: {beach_h} vs water_level {}",
        layout.water_level
    );

    let far_points = [
        camp,
        Vec2::new(5000.0, 5000.0),
        Vec2::new(-4000.0, 2000.0),
        layout.center + Vec2::new(2000.0, 0.0),
    ];
    for far in far_points {
        let h = layout.ground_height(far, raw_h);
        assert_eq!(
            h, raw_h,
            "Far point {far} must have unchanged ground height"
        );
    }
}

#[test]
fn lake_contains_region() {
    let cfg = load_test_cfg();
    let camp = Vec2::ZERO;
    let city = Vec2::new(1500.0, 0.0);
    let layout = LakeLayout::generate(42, camp, Some(city), &cfg, |_| 10.0);

    assert!(layout.contains_region(layout.center));
    for pt in &layout.shore {
        assert!(layout.contains_region(*pt));
    }
    for lot in &layout.village_lots {
        assert!(layout.contains_region(lot.center));
    }
    for dock in &layout.docks {
        assert!(layout.contains_region(dock.start));
        assert!(layout.contains_region(dock.end));
    }

    assert!(!layout.contains_region(camp));
    assert!(!layout.contains_region(Vec2::new(5000.0, 5000.0)));
}
