use glam::Vec2;
use std::collections::{HashMap, HashSet, VecDeque};
use crate::city::config::{CityConfig, District};
use crate::city::layout::CityLayout;
use paths::repo_path;

fn load_test_cfg() -> CityConfig {
    let path = repo_path("data/city.json");
    CityConfig::load_from_file(&path).expect("Failed to load data/city.json")
}

fn distance_to_city_box(camp: Vec2, origin: Vec2, size: f32) -> f32 {
    let min = origin;
    let max = origin + Vec2::splat(size);
    let dx = if camp.x < min.x {
        min.x - camp.x
    } else if camp.x > max.x {
        camp.x - max.x
    } else {
        0.0
    };
    let dy = if camp.y < min.y {
        min.y - camp.y
    } else if camp.y > max.y {
        camp.y - max.y
    } else {
        0.0
    };
    (dx * dx + dy * dy).sqrt()
}

#[test]
fn city_config_load() {
    let cfg = load_test_cfg();
    assert_eq!(cfg.blocks, 10);
    assert_eq!(cfg.block_size, 64.0);
    assert_eq!(cfg.distance_from_camp, 1500.0);
    assert_eq!(cfg.street_width, 10.0);
    assert_eq!(cfg.avenue_width, 14.0);
    assert_eq!(cfg.avenue_every, 3);
    assert_eq!(cfg.blocked_chance, 0.25);
    assert_eq!(cfg.corner_drop_chance, 0.5);
    assert_eq!(cfg.max_slope, 3.0);
    assert_eq!(cfg.direction_tries, 8);
    assert_eq!(cfg.core_blocks, 2);
    assert_eq!(cfg.outskirts_rings, 2);
    assert_eq!(cfg.industrial_strip, [2, 6]);
    assert_eq!(cfg.parks, [1, 2]);
    assert_eq!(cfg.lot_gap, 2.0);
    assert!(!cfg.buildings.is_empty());

    let json_str = serde_json::to_string(&cfg).unwrap();
    let cfg2 = CityConfig::from_json(&json_str).unwrap();
    assert_eq!(cfg, cfg2);
}

#[test]
fn city_deterministic_seed() {
    let cfg = load_test_cfg();
    let camp = Vec2::new(50.0, -100.0);

    let layout_a1 = CityLayout::generate(12345, camp, &cfg, |_| 0.0);
    let layout_a2 = CityLayout::generate(12345, camp, &cfg, |_| 0.0);
    let layout_b = CityLayout::generate(54321, camp, &cfg, |_| 0.0);

    assert_eq!(layout_a1, layout_a2);
    assert_ne!(layout_a1, layout_b);
}

#[test]
fn city_distance_from_camp() {
    let cfg = load_test_cfg();
    let camp = Vec2::new(120.0, -400.0);

    for seed in [1, 7, 42, 100, 2026, 9999] {
        let layout = CityLayout::generate(seed, camp, &cfg, |_| 0.0);
        let dist = distance_to_city_box(camp, layout.origin, layout.size);
        let diff = (dist - cfg.distance_from_camp).abs();
        assert!(
            diff <= 64.0 + 1e-3,
            "seed {seed}: dist {dist} deviates by {diff} from {} (> 64m)",
            cfg.distance_from_camp
        );
    }
}

#[test]
fn city_lots_no_overlap_and_inside_blocks() {
    let cfg = load_test_cfg();
    let camp = Vec2::ZERO;

    for seed in [3, 42, 77] {
        let layout = CityLayout::generate(seed, camp, &cfg, |_| 0.0);
        assert!(!layout.lots.is_empty(), "seed {seed} should place lots");

        for (i, lot) in layout.lots.iter().enumerate() {
            let (lot_min, lot_max) = lot.aabb();

            // Lot must fall within city bounds.
            assert!(lot_min.x >= layout.origin.x);
            assert!(lot_max.x <= layout.origin.x + layout.size);
            assert!(lot_min.y >= layout.origin.y);
            assert!(lot_max.y <= layout.origin.y + layout.size);

            // Find block containing lot.
            let rel_center = lot.center - layout.origin;
            let bx = (rel_center.x / cfg.block_size).floor() as i32;
            let by = (rel_center.y / cfg.block_size).floor() as i32;
            assert!(bx >= 0 && bx < cfg.blocks && by >= 0 && by < cfg.blocks);

            let block_min = layout.origin + Vec2::new(bx as f32 * cfg.block_size, by as f32 * cfg.block_size);
            let block_max = block_min + Vec2::splat(cfg.block_size);

            // Lot must be strictly inside its block.
            assert!(lot_min.x >= block_min.x);
            assert!(lot_max.x <= block_max.x);
            assert!(lot_min.y >= block_min.y);
            assert!(lot_max.y <= block_max.y);

            // Lot must be off the streets (at least lot_gap clearance from street centerline +/- half width).
            let w_south = if by % cfg.avenue_every == 0 { cfg.avenue_width } else { cfg.street_width };
            let w_north = if (by + 1) % cfg.avenue_every == 0 { cfg.avenue_width } else { cfg.street_width };
            let w_west = if bx % cfg.avenue_every == 0 { cfg.avenue_width } else { cfg.street_width };
            let w_east = if (bx + 1) % cfg.avenue_every == 0 { cfg.avenue_width } else { cfg.street_width };

            let usable_min = Vec2::new(block_min.x + w_west * 0.5 + cfg.lot_gap, block_min.y + w_south * 0.5 + cfg.lot_gap);
            let usable_max = Vec2::new(block_max.x - w_east * 0.5 - cfg.lot_gap, block_max.y - w_north * 0.5 - cfg.lot_gap);

            assert!(lot_min.x >= usable_min.x - 1e-4);
            assert!(lot_max.x <= usable_max.x + 1e-4);
            assert!(lot_min.y >= usable_min.y - 1e-4);
            assert!(lot_max.y <= usable_max.y + 1e-4);

            // No two lots may overlap.
            for other in &layout.lots[i + 1..] {
                let (o_min, o_max) = other.aabb();
                let sep_x = (lot_min.x - o_max.x).max(o_min.x - lot_max.x);
                let sep_y = (lot_min.y - o_max.y).max(o_min.y - lot_max.y);
                assert!(
                    sep_x >= cfg.lot_gap - 1e-3 || sep_y >= cfg.lot_gap - 1e-3,
                    "Lots overlap/too close: {:?} vs {:?}",
                    lot,
                    other
                );
            }
        }
    }
}

#[test]
fn city_connectivity_from_entry() {
    let cfg = load_test_cfg();
    let camp = Vec2::new(-200.0, 300.0);

    for seed in [11, 22, 33, 44, 55, 66] {
        let layout = CityLayout::generate(seed, camp, &cfg, |_| 0.0);

        // Build graph of unblocked street segments.
        let mut adj = HashMap::new();
        for (idx, seg) in layout.streets.iter().enumerate() {
            if seg.blocked {
                continue;
            }
            let u = (seg.a.x.round() as i32, seg.a.y.round() as i32);
            let v = (seg.b.x.round() as i32, seg.b.y.round() as i32);
            adj.entry(u).or_insert_with(Vec::new).push((v, idx));
            adj.entry(v).or_insert_with(Vec::new).push((u, idx));
        }

        // Find street segment containing entry.
        let entry_seg = layout
            .streets
            .iter()
            .find(|s| !s.blocked && (s.closest_point(layout.entry) - layout.entry).length() < 1e-3)
            .expect("entry must lie on an unblocked street segment");

        let u0 = (entry_seg.a.x.round() as i32, entry_seg.a.y.round() as i32);
        let v0 = (entry_seg.b.x.round() as i32, entry_seg.b.y.round() as i32);

        // BFS from entry.
        let mut visited: HashSet<(i32, i32)> = HashSet::new();
        let mut queue = VecDeque::new();
        visited.insert(u0);
        visited.insert(v0);
        queue.push_back(u0);
        queue.push_back(v0);

        while let Some(u) = queue.pop_front() {
            if let Some(neighbors) = adj.get(&u) {
                for &(v, _) in neighbors {
                    if visited.insert(v) {
                        queue.push_back(v);
                    }
                }
            }
        }

        // Every present block must be reached (at least one corner vertex visited).
        for block in &layout.blocks {
            if !block.present {
                continue;
            }
            let bx = block.coord.x - (layout.origin.x / cfg.block_size).round() as i32;
            let by = block.coord.y - (layout.origin.y / cfg.block_size).round() as i32;
            let c0 = (
                (layout.origin.x + bx as f32 * cfg.block_size).round() as i32,
                (layout.origin.y + by as f32 * cfg.block_size).round() as i32,
            );
            let c1 = (
                (layout.origin.x + (bx + 1) as f32 * cfg.block_size).round() as i32,
                (layout.origin.y + by as f32 * cfg.block_size).round() as i32,
            );
            let c2 = (
                (layout.origin.x + bx as f32 * cfg.block_size).round() as i32,
                (layout.origin.y + (by + 1) as f32 * cfg.block_size).round() as i32,
            );
            let c3 = (
                (layout.origin.x + (bx + 1) as f32 * cfg.block_size).round() as i32,
                (layout.origin.y + (by + 1) as f32 * cfg.block_size).round() as i32,
            );

            let reached = visited.contains(&c0)
                || visited.contains(&c1)
                || visited.contains(&c2)
                || visited.contains(&c3);
            assert!(reached, "seed {seed}: block {:?} is not reachable from entry", block.coord);
        }
    }
}

#[test]
fn city_blocked_street_ratio() {
    let cfg = load_test_cfg();
    let camp = Vec2::ZERO;

    let mut total_non_avenue = 0;
    let mut total_blocked = 0;

    for seed in 0..50 {
        let layout = CityLayout::generate(seed, camp, &cfg, |_| 0.0);
        for s in &layout.streets {
            if !s.avenue {
                total_non_avenue += 1;
                if s.blocked {
                    total_blocked += 1;
                }
            } else {
                assert!(!s.blocked, "Avenues must never be blocked");
            }
        }
    }

    assert!(total_non_avenue > 0);
    let ratio = total_blocked as f32 / total_non_avenue as f32;
    let diff = (ratio - cfg.blocked_chance).abs();
    assert!(
        diff < 0.08,
        "blocked street ratio {ratio} deviates from {} by {diff} (>= 0.08)",
        cfg.blocked_chance
    );
}

#[test]
fn city_avoids_forest() {
    let cfg = load_test_cfg();
    let camp = Vec2::ZERO;

    // Forest everywhere on the positive X half plane.
    let forest_fn = |p: Vec2| if p.x > 0.0 { 1.0 } else { 0.0 };

    for seed in [1, 42, 105, 555] {
        let layout = CityLayout::generate(seed, camp, &cfg, forest_fn);
        assert!(
            layout.origin.x + layout.size <= 0.0,
            "seed {seed}: city at x={}..{} should be strictly at x < 0",
            layout.origin.x,
            layout.origin.x + layout.size
        );
    }
}

#[test]
fn city_districts_core_and_industrial() {
    let cfg = load_test_cfg();
    let camp = Vec2::ZERO;

    for seed in [10, 20, 30, 40] {
        let layout = CityLayout::generate(seed, camp, &cfg, |_| 0.0);

        let core_blocks: Vec<&_> = layout
            .blocks
            .iter()
            .filter(|b| b.district == District::Core)
            .collect();
        assert_eq!(core_blocks.len(), 4, "Core must be exactly 2x2 = 4 blocks");

        let min_chunk_x = (layout.origin.x / cfg.block_size).round() as i32;
        let min_chunk_y = (layout.origin.y / cfg.block_size).round() as i32;

        for b in &core_blocks {
            let bx = b.coord.x - min_chunk_x;
            let by = b.coord.y - min_chunk_y;
            assert!(bx == 4 || bx == 5, "Core bx must be 4 or 5");
            assert!(by == 4 || by == 5, "Core by must be 4 or 5");
        }

        let industrial_blocks: Vec<&_> = layout
            .blocks
            .iter()
            .filter(|b| b.district == District::Industrial)
            .collect();
        assert_eq!(
            industrial_blocks.len(),
            (cfg.industrial_strip[0] * cfg.industrial_strip[1]) as usize,
            "Industrial strip must be exactly 2x6 = 12 blocks"
        );
    }
}

#[test]
fn city_queries() {
    let cfg = load_test_cfg();
    let camp = Vec2::ZERO;
    let layout = CityLayout::generate(42, camp, &cfg, |_| 0.0);

    for lot in &layout.lots {
        assert!(layout.contains(lot.center));
        assert_eq!(layout.district_at(lot.center), Some(lot.district));
    }

    for seg in &layout.streets {
        let mid = (seg.a + seg.b) * 0.5;
        assert!(layout.contains(mid));
        assert!(layout.street_at(mid).is_some());
    }

    assert!(!layout.contains(camp));
    assert!(layout.district_at(camp).is_none());
    assert!(layout.street_at(camp).is_none());
}
