//! Unit tests for R43-01 (`climb.rs` and `climb_hulls.rs`).

use std::collections::HashSet;

use glam::{Vec2, Vec3};

use crate::metro::climb::{
    ClimbKind, FloorCrateEntry, StopKind, climb_hull_boxes, climb_in_chunk, climb_in_chunk_with,
    climb_towers, module_size_at, roof_deck_y,
};
use crate::metro::config::MetroConfig;
use crate::metro::damage::{damage_sites, default_damage};
use crate::metro::highway::{Highway, HighwayHeightField};
use crate::metro::jam::plan_jam;
use crate::metro::layout::{MetroLayout, Reserved};
use crate::metro::paint::MetroLook;
use crate::metro::place::MetroSite;
use crate::metro::state::{MetroWorld, index_towers_by_block};
use crate::metro::styles::{default_styles, style_for};
use crate::metro::towers::plan_towers;

fn test_metro() -> MetroWorld {
    let cfg = MetroConfig::load_from_file(paths::repo_path("data/metro.json"))
        .expect("data/metro.json must load");
    let site = MetroSite {
        key: cfg.key.clone(),
        name: cfg.name.clone(),
        center: Vec2::new(6000.0, 0.0),
        yaw: 0.3,
        half_size: (cfg.size_blocks as f32 * cfg.block_size) * 0.5,
        ground_y: 0.0,
    };
    let layout = MetroLayout::generate(site.clone(), &cfg);
    let towers = plan_towers(&layout, &cfg);
    let entrance = site.to_world(Vec2::new(0.0, site.half_size));
    let out = (entrance - site.center).normalize();
    let highway = Highway {
        from_town: "blackwater".into(),
        points: vec![entrance + out * 500.0, entrance],
        width: cfg.highway.highway_width,
    };
    let highway_field = HighwayHeightField::build(&highway, 0.0, &|_x, _z| 0.0, &cfg);
    let jam = plan_jam(&highway, &layout, &cfg);
    let towers_by_block = index_towers_by_block(&towers);
    let damage = damage_sites(&layout, &towers, default_damage());
    let look = MetroLook::load_from_file(paths::repo_path("data/metro.json")).expect("look");
    MetroWorld { config: cfg, layout, towers, highway, highway_field, jam, look, towers_by_block, damage }
}

fn chunk_of(pos: Vec2) -> (i32, i32) {
    ((pos.x / 64.0).floor() as i32, (pos.y / 64.0).floor() as i32)
}

fn boxes_overlap(a: [Vec3; 2], b: [Vec3; 2]) -> bool {
    a[0].x < b[1].x - 1e-4 && a[1].x > b[0].x + 1e-4
        && a[0].y < b[1].y - 1e-4 && a[1].y > b[0].y + 1e-4
        && a[0].z < b[1].z - 1e-4 && a[1].z > b[0].z + 1e-4
}

#[test]
fn picking_counts_spacing_safety_and_determinism() {
    let metro = test_metro();
    let a = climb_towers(&metro);
    let b = climb_towers(&metro);
    assert_eq!(a, b);

    let mid = a.iter().filter(|p| p.kind == ClimbKind::Midtown).count();
    let core = a.iter().filter(|p| p.kind == ClimbKind::Core).count();
    let lm = a.iter().filter(|p| p.kind == ClimbKind::Landmark).count();
    assert_eq!((mid, core, lm), (2, 2, 1));

    let safe_block = metro.layout.blocks.iter()
        .find(|b| b.reserved == Some(Reserved::SafeHouseTower))
        .map(|b| (b.ix, b.iz));

    let non_lm: Vec<_> = a.iter().filter(|p| p.kind != ClimbKind::Landmark).collect();
    for (i, p) in non_lm.iter().enumerate() {
        let t = &metro.towers[p.tower];
        assert!(t.size >= metro.config.climb.min_size);
        assert!(!metro.damage.tower_is_replaced(p.tower));
        assert!(!metro.damage.tower_is_burnt(t.block));
        assert_ne!(Some(t.block), safe_block);
        for q in &non_lm[i + 1..] {
            let bt = &metro.towers[q.tower].block;
            let dist = (t.block.0 - bt.0).abs().max((t.block.1 - bt.1).abs());
            assert!(dist >= metro.config.climb.min_spacing_blocks, "dist={dist}");
        }
    }
}

#[test]
fn stop_floors_rules_and_landmark_camp() {
    let metro = test_metro();
    for p in climb_towers(&metro) {
        let t = &metro.towers[p.tower];
        let expected_stops = match p.kind {
            ClimbKind::Midtown => metro.config.climb.stop_floors.midtown,
            ClimbKind::Core => metro.config.climb.stop_floors.core,
            ClimbKind::Landmark => metro.config.climb.stop_floors.landmark,
        };
        assert_eq!(p.stop_floors.len(), expected_stops + 1);
        assert_eq!(p.stop_kinds.len(), p.stop_floors.len());
        assert_eq!(p.stop_floors.last().copied(), Some(t.floors - 1));
        assert!(!p.stop_floors.contains(&0));
        for w in p.stop_floors.windows(2) {
            assert!(w[1] >= w[0] + 2, "adjacent stop floors: {:?}", p.stop_floors);
        }
        if p.kind == ClimbKind::Landmark {
            assert_eq!(p.stop_kinds.last().copied(), Some(StopKind::Camp));
        }
    }
}

#[test]
fn modules_and_ladders_per_climb_tower() {
    let metro = test_metro();
    let styles = default_styles();
    let fixture_crates = [FloorCrateEntry {
        x: 2.0,
        y: 0.0,
        z: -4.0,
        yaw: 0.0,
        model: "crate_loot".into(),
        table: None,
    }];
    let chunks: HashSet<(i32, i32)> = climb_towers(&metro).iter()
        .map(|p| chunk_of(metro.towers[p.tower].pos))
        .collect();

    for chunk in chunks {
        let out = climb_in_chunk_with(&metro, chunk, &|_, _| 10.0, &|_| &fixture_crates);
        for plan in climb_towers(&metro) {
            let t = &metro.towers[plan.tower];
            if chunk_of(t.pos) != chunk {
                continue;
            }
            let style = style_for(styles, t.style).unwrap();
            let fh = style.floor_height;
            let expected_modules = 2 + t.floors as usize;
            let stair_props: Vec<_> = out.props.iter()
                .filter(|pr| pr.model.starts_with("tw_stair_") && (pr.position.x - t.pos.x).abs() < 20.0 && (pr.position.z - t.pos.y).abs() < 20.0)
                .collect();
            let stair_pois: Vec<_> = out.pois.iter()
                .filter(|poi| poi.id.starts_with(&format!("metro_stair_{}_", plan.tower)))
                .collect();
            assert_eq!(stair_props.len() + stair_pois.len(), expected_modules);

            let prop_ys: HashSet<i32> = stair_props.iter().map(|p| (p.position.y * 100.0).round() as i32).collect();
            for poi in &stair_pois {
                let yk = (poi.position.y * 100.0).round() as i32;
                assert!(!prop_ys.contains(&yk), "module is both prop and POI");
            }

            let top_y = 10.0 + style.base_height + (t.floors - 1) as f32 * fh;
            assert!(stair_props.iter().any(|p| p.model == "tw_stair_top" && (p.position.y - top_y).abs() < 0.05));
            for &sf in &plan.stop_floors {
                if sf + 1 < t.floors {
                    let sy = 10.0 + style.base_height + sf as f32 * fh;
                    assert!(stair_props.iter().any(|p| p.model == "tw_stair_open" && (p.position.y - sy).abs() < 0.05));
                }
            }

            let hatch = out.ladders.iter().find(|l| l.id == format!("metro_hatch_{}", plan.tower)).unwrap();
            assert!((hatch.foot.y - top_y).abs() < 1e-3);
            assert!((hatch.deck.y - roof_deck_y(t, style, 10.0)).abs() < 1e-3);

            if plan.kind == ClimbKind::Landmark {
                let mast = out.ladders.iter().find(|l| l.id == format!("metro_mast_{}", plan.tower)).unwrap();
                assert!((mast.foot.y - roof_deck_y(t, style, 10.0)).abs() < 1e-3);
                let crown_y = 10.0 + style.base_height + t.floors as f32 * fh;
                let plat_top = crown_y + metro.config.climb.mast.platform_y + 0.22;
                assert!((mast.deck.y - plat_top).abs() < 1e-3);
                assert!(out.boxes.iter().any(|b| (b.centre.y - (crown_y + metro.config.climb.mast.platform_y + 0.11)).abs() < 0.05));
                assert!(out.crates.iter().any(|c| c.crate_id == "metro_radio_crate"));
            }
        }
    }
    let empty_chunk = climb_in_chunk(&metro, (999, 999), &|_, _| 0.0);
    assert!(empty_chunk.props.is_empty() && empty_chunk.pois.is_empty());
}

#[test]
fn hull_boxes_shaft_hatch_stop_floors_and_fit() {
    let metro = test_metro();
    let styles = default_styles();
    for plan in climb_towers(&metro) {
        let t = &metro.towers[plan.tower];
        let style = style_for(styles, t.style).unwrap();
        let boxes = climb_hull_boxes(t, style, &plan);
        let shaft_z = metro.config.climb.shaft_centre_z_share * t.size as f32;
        let deck_y = roof_deck_y(t, style, 0.0);
        let shaft_col = [
            Vec3::new(-2.2, style.base_height, shaft_z - 3.2),
            Vec3::new(2.2, deck_y, shaft_z + 3.2),
        ];
        let top_ceil = style.base_height + t.floors as f32 * style.floor_height;
        let hatch_hole = [
            Vec3::new(3.4 - 0.7, top_ceil, shaft_z - 0.7),
            Vec3::new(3.4 + 0.7, deck_y, shaft_z + 0.7),
        ];

        let max_y = boxes.iter().map(|b| b[1].y).fold(f32::NEG_INFINITY, f32::max);
        assert!((max_y - deck_y).abs() < 1e-3, "max_y={max_y}, deck_y={deck_y}");

        for &b in &boxes {
            assert!(!boxes_overlap(b, shaft_col), "box {b:?} overlaps shaft");
            assert!(!boxes_overlap(b, hatch_hole), "box {b:?} overlaps hatch");
        }

        for i in 0..t.floors {
            let sz = module_size_at(t, i, style, &metro.config.towers.setbacks) as f32 * 0.5;
            assert!(2.2 < sz && shaft_z + 3.2 < sz && shaft_z - 3.2 > -sz, "shaft must fit inside module");
            let fy = style.base_height + i as f32 * style.floor_height;
            let on_floor: Vec<_> = boxes.iter().filter(|b| b[0].y >= fy - 1e-3 && b[1].y <= fy + style.floor_height + 1e-3).collect();
            if plan.stop_floors.contains(&i) {
                assert!(on_floor.iter().any(|b| (b[1].y - b[0].y - 0.3).abs() < 1e-3), "missing slab");
                assert!(on_floor.iter().any(|b| (b[1].y - b[0].y - (style.floor_height - 0.3)).abs() < 1e-3), "missing wall");
            } else {
                assert_eq!(on_floor.len(), 4, "non-stop floor should have 4 solid boxes around shaft");
            }
        }
    }
}
