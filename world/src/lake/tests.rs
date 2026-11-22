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
    assert_eq!(cfg.docks.deck_height, 0.4);
    assert_eq!(cfg.deck_height, 0.4);
    assert_eq!(cfg.angle_from_city, 180.0);
    assert_eq!(cfg.beach_blend, 16.0);
    assert_eq!(cfg.shore_samples, 128);
    assert_eq!(cfg.region_margin, 32.0);
    assert_eq!(cfg.sand, [214, 196, 156]);
    assert_eq!(cfg.wet_sand, [148, 128, 94]);
    assert_eq!(cfg.lake_bed, [58, 56, 38]);

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

#[test]
fn lake_attached_in_from_data_dir() {
    let data_dir = repo_path("data");
    let wgen = crate::generator::WorldGenerator::from_data_dir(42, &data_dir)
        .expect("WorldGenerator::from_data_dir");
    assert!(wgen.lake().is_some());
    let lake = wgen.lake().unwrap();
    assert!(lake.water_level > 0.0);
    assert_eq!(lake.docks.len(), 3);
    assert_eq!(lake.village_lots.len(), 8);
}

#[test]
fn lake_far_from_city_and_camp() {
    let data_dir = repo_path("data");
    for seed in [1, 7, 42, 100, 999] {
        let wgen = crate::generator::WorldGenerator::from_data_dir(seed, &data_dir).unwrap();
        let lake = wgen.lake().expect("lake present");
        let camp = wgen.settlements()[0].center;
        let camp_p = Vec2::new(camp.x, camp.z);

        assert!(!lake.contains_region(camp_p));
        assert!(lake.center.distance(camp_p) > 1000.0);

        if let Some(city) = wgen.city() {
            let city_center = city.origin + Vec2::splat(city.size * 0.5);
            assert!(!lake.contains_region(city_center));
            assert!(!city.contains(lake.center));
            assert!(lake.center.distance(city_center) > 2000.0);

            // Bounding boxes do not overlap
            let (lake_min, lake_max) = lake.water_bounds();
            let city_half = city.size * 0.5;
            let city_min = city_center - Vec2::splat(city_half);
            let city_max = city_center + Vec2::splat(city_half);
            let x_overlap = lake_min.x <= city_max.x && lake_max.x >= city_min.x;
            let y_overlap = lake_min.y <= city_max.y && lake_max.y >= city_min.y;
            assert!(!(x_overlap && y_overlap), "lake bounding box overlaps city box");
        }
    }
}

#[test]
fn lake_ground_under_deep_water_is_below_water_level() {
    let data_dir = repo_path("data");
    let wgen = crate::generator::WorldGenerator::from_data_dir(42, &data_dir).unwrap();
    let lake = wgen.lake().expect("lake present");

    let (h_center, _) = wgen.sample(lake.center.x, lake.center.y);
    assert!(h_center < lake.water_level, "lake center height {h_center} should be below water level {}", lake.water_level);
    let depth = lake.water_level - h_center;
    assert!(depth >= lake.config.max_depth - 0.5, "deep water depth {depth} should be near max_depth {}", lake.config.max_depth);
}

#[test]
fn lake_beach_paint_is_sand_colored() {
    let data_dir = repo_path("data");
    let wgen = crate::generator::WorldGenerator::from_data_dir(42, &data_dir).unwrap();
    let lake = wgen.lake().expect("lake present");

    let shore_pt = lake.shore[0];
    let u = (shore_pt - lake.center).normalize();
    let beach_pt = shore_pt + u * (lake.config.beach_width * 0.5);

    let painted = lake.paint(crate::generator::WorldGenerator::PLAINS_GRASS, beach_pt.x, beach_pt.y);
    let expected_sand = lake.sand_color();
    assert!((painted.r - expected_sand.r).abs() < 1e-3);
    assert!((painted.g - expected_sand.g).abs() < 1e-3);
    assert!((painted.b - expected_sand.b).abs() < 1e-3);

    // Warm beige: red channel > green channel > blue channel
    assert!(painted.r > painted.g);
    assert!(painted.g > painted.b);
}

#[test]
fn lake_chunk_in_water_has_no_trees() {
    let data_dir = repo_path("data");
    let wgen = crate::generator::WorldGenerator::from_data_dir(42, &data_dir).unwrap();
    let lake = wgen.lake().expect("lake present");

    let cx = (lake.center.x / crate::generator::WorldGenerator::CHUNK_SIZE).floor() as i32;
    let cy = (lake.center.y / crate::generator::WorldGenerator::CHUNK_SIZE).floor() as i32;
    let coord = glam::IVec2::new(cx, cy);

    let (_, props) = wgen.chunk_contents(coord);
    let tree_count = props.iter().filter(|p| p.is_tree() || p.model.starts_with("tree")).count();
    assert_eq!(tree_count, 0, "chunk in deep water should have 0 trees");
    assert!(props.is_empty(), "chunk in deep water should have no props");
}

#[test]
fn lake_dock_props_line_up_with_docks() {
    let data_dir = repo_path("data");
    let wgen = crate::generator::WorldGenerator::from_data_dir(42, &data_dir).unwrap();
    let lake = wgen.lake().expect("lake present");

    for (dock_idx, dock) in lake.docks.iter().enumerate() {
        let dir = dock.forward();
        let chunk_size = crate::generator::WorldGenerator::CHUNK_SIZE;
        let c_start = glam::IVec2::new((dock.start.x / chunk_size).floor() as i32, (dock.start.y / chunk_size).floor() as i32);
        let c_end = glam::IVec2::new((dock.end.x / chunk_size).floor() as i32, (dock.end.y / chunk_size).floor() as i32);

        let mut dock_props = Vec::new();
        for cx in c_start.x.min(c_end.x)..=c_start.x.max(c_end.x) {
            for cy in c_start.y.min(c_end.y)..=c_start.y.max(c_end.y) {
                let (_, props) = wgen.chunk_contents(glam::IVec2::new(cx, cy));
                for p in props {
                    if p.model == "dock_section" || p.model == "dock_end" {
                        let to_p = Vec2::new(p.position.x, p.position.z) - dock.start;
                        let along = to_p.dot(dir);
                        let perp_dist = (to_p - dir * along).length();
                        if along >= -1.0 && along <= dock.length() + 1.0 && perp_dist < 0.1 {
                            dock_props.push(p);
                        }
                    }
                }
            }
        }

        assert!(!dock_props.is_empty(), "dock {dock_idx} should have placed props");
        for dp in &dock_props {
            assert!((dp.yaw - dock.yaw).abs() < 1e-4, "dock prop yaw must match dock yaw");
            assert!(dp.position.y > lake.water_level, "dock prop deck y must be above water level");
            assert!((dp.position.y - dock.deck_y).abs() < 1e-4, "dock prop height must match dock deck_y");
        }
    }
}

#[test]
fn lake_same_seed_same_lake() {
    let data_dir = repo_path("data");
    let wgen1 = crate::generator::WorldGenerator::from_data_dir(42, &data_dir).unwrap();
    let wgen2 = crate::generator::WorldGenerator::from_data_dir(42, &data_dir).unwrap();
    let wgen3 = crate::generator::WorldGenerator::from_data_dir(43, &data_dir).unwrap();

    let lake1 = wgen1.lake().unwrap();
    let lake2 = wgen2.lake().unwrap();
    let lake3 = wgen3.lake().unwrap();

    assert_eq!(lake1, lake2);
    assert_ne!(lake1, lake3);
}

#[test]
fn lake_docks_deck_y_above_water_level() {
    let cfg = load_test_cfg();
    let camp = Vec2::ZERO;
    let city = Vec2::new(1500.0, 0.0);
    for seed in [1, 7, 42, 100, 999] {
        let layout = LakeLayout::generate(seed, camp, Some(city), &cfg, |_| 10.0);
        assert!(!layout.docks.is_empty());
        for (i, dock) in layout.docks.iter().enumerate() {
            assert!(
                dock.deck_y > layout.water_level,
                "seed {seed} dock {i} deck y {} must be above water level {}",
                dock.deck_y,
                layout.water_level
            );
            assert!(
                (dock.deck_y - (layout.water_level + cfg.docks.deck_height)).abs() < 1e-4,
                "dock {i} deck y should be water_level + deck_height"
            );
        }
    }
}

#[test]
fn lake_water_mesh_no_open_edges_inside_lake() {
    let cfg = load_test_cfg();
    let camp = Vec2::ZERO;
    let city = Vec2::new(1500.0, 0.0);
    let layout = LakeLayout::generate(42, camp, Some(city), &cfg, |_| 10.0);

    let mesh = layout.build_water_mesh();
    assert!(!mesh.indices.is_empty());
    assert!(!mesh.vertices.is_empty());

    // Count how many triangles share each undirected edge.
    let mut edge_counts = std::collections::HashMap::new();
    for tri in &mesh.indices {
        let edges = [
            (tri[0].min(tri[1]), tri[0].max(tri[1])),
            (tri[1].min(tri[2]), tri[1].max(tri[2])),
            (tri[2].min(tri[0]), tri[2].max(tri[0])),
        ];
        for e in edges {
            *edge_counts.entry(e).or_insert(0usize) += 1;
        }
    }

    // Every edge must be shared by 1 or 2 triangles (manifold).
    // An edge with count == 1 is an open boundary edge: its endpoints MUST be on the shore line!
    for ((u, v), count) in edge_counts {
        assert!(count <= 2, "mesh non-manifold: edge ({u}, {v}) shared by {count} triangles");
        if count == 1 {
            let p_u = Vec2::new(mesh.vertices[u as usize].x, mesh.vertices[u as usize].z);
            let p_v = Vec2::new(mesh.vertices[v as usize].x, mesh.vertices[v as usize].z);
            let d_u = layout.shore_distance(p_u);
            let d_v = layout.shore_distance(p_v);

            assert!(
                d_u.abs() < 0.05,
                "open boundary vertex {u} must be on shore line, shore_dist was {d_u}"
            );
            assert!(
                d_v.abs() < 0.05,
                "open boundary vertex {v} must be on shore line, shore_dist was {d_v}"
            );
        }
    }
}

#[test]
fn lake_water_mesh_no_vertices_on_dry_land() {
    let cfg = load_test_cfg();
    let camp = Vec2::ZERO;
    let city = Vec2::new(1500.0, 0.0);
    let layout = LakeLayout::generate(42, camp, Some(city), &cfg, |_| 10.0);

    let mesh = layout.build_water_mesh();
    for (i, v) in mesh.vertices.iter().enumerate() {
        let p = Vec2::new(v.x, v.z);
        let d = layout.shore_distance(p);
        let depth = layout.depth_at(p);

        // Water vertex cannot be in dry land (beach/inland)
        assert!(
            d <= 0.05,
            "vertex {i} at {p} floats over land: shore_dist={d} > 0"
        );

        if depth <= 0.0 {
            // depth_at == 0 is ONLY allowed right on the shoreline
            assert!(
                d.abs() <= 0.05,
                "vertex {i} at {p} has depth <= 0 ({depth}) but is not on shore line: shore_dist={d}"
            );
        } else {
            // positive depth must be inside the water
            assert!(d < 0.0, "positive depth vertex must be in water: d={d}");
        }
    }
}

#[test]
fn lake_water_mesh_triangle_count() {
    let cfg = load_test_cfg();
    let camp = Vec2::ZERO;
    let city = Vec2::new(1500.0, 0.0);
    let layout = LakeLayout::generate(42, camp, Some(city), &cfg, |_| 10.0);

    let mesh = layout.build_water_mesh();
    let tris = mesh.triangle_count();
    assert!(
        tris < 20_000,
        "lake water mesh triangle count {tris} must be under 20k"
    );
    assert!(
        tris > 5_000,
        "lake water mesh triangle count {tris} must properly cover the lake"
    );
}
