//! Tests for `towers.rs` (kept in a sibling file to stay under the size rule).

use glam::Vec2;

use crate::metro::config::MetroConfig;
use crate::metro::layout::{MetroDistrict, MetroLayout, Reserved};
use crate::metro::place::MetroSite;
use crate::metro::towers::{TowerKind, TowerSpot, plan_towers};

/// Tolerance for float round trips through the site frame (metres).
const EPS: f32 = 1e-3; // new: DECISIONS 2026-11-26 (test tolerance)

fn cfg() -> MetroConfig {
    MetroConfig::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json loads")
}

fn site(cfg: &MetroConfig) -> MetroSite {
    MetroSite {
        key: cfg.key.clone(),
        name: cfg.name.clone(),
        center: Vec2::new(1000.0, -2000.0),
        half_size: cfg.size_blocks as f32 * cfg.block_size * 0.5,
        yaw: 0.6,
        ground_y: 10.0,
    }
}

fn layout_and_towers() -> (MetroConfig, MetroLayout, Vec<TowerSpot>) {
    let cfg = cfg();
    let layout = MetroLayout::generate(site(&cfg), &cfg);
    let towers = plan_towers(&layout, &cfg);
    (cfg, layout, towers)
}

/// Local-space min and max corners of a tower footprint (axis-aligned in the site frame).
fn local_rect(layout: &MetroLayout, t: &TowerSpot) -> (Vec2, Vec2) {
    let c = layout.site.to_local(t.pos);
    (c - t.half, c + t.half)
}

#[test]
fn same_site_gives_identical_towers() {
    let (cfg, layout, towers) = layout_and_towers();
    assert_eq!(towers, plan_towers(&layout, &cfg));
    let mut moved = site(&cfg);
    moved.center.x += 64.0;
    let other = MetroLayout::generate(moved, &cfg);
    assert_ne!(towers, plan_towers(&other, &cfg));
}

#[test]
fn ordinary_blocks_hold_one_to_max_lots() {
    let (cfg, layout, towers) = layout_and_towers();
    let max = cfg.towers.max_lots_per_block;
    for block in &layout.blocks {
        if block.reserved.is_some() {
            continue;
        }
        let key = (block.ix, block.iz);
        let n = towers.iter().filter(|t| t.block == key).count();
        assert!(n <= max, "block {key:?} has {n} towers, max is {max}");
    }
}

#[test]
fn reserved_blocks_have_no_towers_except_one_safe_house_tower() {
    let (_, layout, towers) = layout_and_towers();
    for block in &layout.blocks {
        let Some(kind) = block.reserved else { continue };
        let n = towers.iter().filter(|t| t.block == (block.ix, block.iz)).count();
        let want = usize::from(kind == Reserved::SafeHouseTower);
        assert_eq!(n, want, "reserved {kind:?} block ({}, {}) has {n} towers", block.ix, block.iz);
    }
}

#[test]
fn heights_and_floors_follow_the_district() {
    let (cfg, layout, towers) = layout_and_towers();
    for t in &towers {
        let block = layout.block_at(t.block.0, t.block.1).expect("tower sits in a present block");
        assert!((t.height - t.floors as f32 * cfg.towers.floor_height).abs() < EPS);
        if t.landmark {
            assert_eq!(t.kind, TowerKind::Landmark);
            let [lo, hi] = cfg.towers.landmark_floors;
            assert!((lo..=hi).contains(&t.floors), "landmark floors {} out of range", t.floors);
            continue;
        }
        assert!(t.floors < cfg.towers.landmark_floors[0], "non-landmark tower too tall");
        let (range, kind) = match block.district {
            MetroDistrict::Edge => (cfg.towers.edge.floors, TowerKind::Lowrise),
            MetroDistrict::Midtown => (cfg.towers.midtown.floors, TowerKind::Midrise),
            MetroDistrict::Core => (cfg.towers.core.floors, TowerKind::Highrise),
        };
        assert_eq!(t.kind, kind, "wrong kind in {:?}", block.district);
        assert!((range[0]..=range[1]).contains(&t.floors), "{:?} floors {}", block.district, t.floors);
    }
}

#[test]
fn landmarks_are_three_to_five_core_skyscrapers_near_150_metres() {
    let (cfg, layout, towers) = layout_and_towers();
    let marks: Vec<&TowerSpot> = towers.iter().filter(|t| t.landmark).collect();
    let [lo, hi] = cfg.towers.landmark_count;
    assert!((lo..=hi).contains(&marks.len()), "{} landmarks", marks.len());
    for (i, m) in marks.iter().enumerate() {
        let block = layout.block_at(m.block.0, m.block.1).expect("landmark block present");
        assert_eq!(block.district, MetroDistrict::Core);
        assert!(block.reserved.is_none(), "landmark on a reserved block");
        assert!((140.0..=165.0).contains(&m.height), "landmark {} m", m.height);
        assert!(marks[i + 1..].iter().all(|o| o.block != m.block), "two landmarks in a block");
    }
}

#[test]
fn towers_stay_inside_their_block_and_off_the_streets() {
    let (_, layout, towers) = layout_and_towers();
    for t in &towers {
        let block = layout.block_at(t.block.0, t.block.1).expect("block exists");
        let (min, max) = local_rect(&layout, t);
        assert!(min.x >= block.rect.0.x && max.x <= block.rect.1.x, "{:?} leaves its block in x", t.block);
        assert!(min.y >= block.rect.0.y && max.y <= block.rect.1.y, "{:?} leaves its block in y", t.block);
        for ix in 0..=4 {
            for iz in 0..=4 {
                let f = Vec2::new(ix as f32 / 4.0, iz as f32 / 4.0);
                let local = min + (max - min) * f;
                let world = layout.site.to_world(local);
                assert!(layout.street_at(world).is_none(), "tower {:?} overlaps a street", t.block);
            }
        }
    }
}

#[test]
fn towers_never_overlap_each_other() {
    let (_, layout, towers) = layout_and_towers();
    for (i, a) in towers.iter().enumerate() {
        let (amin, amax) = local_rect(&layout, a);
        for b in &towers[i + 1..] {
            let (bmin, bmax) = local_rect(&layout, b);
            let apart = amax.x <= bmin.x + EPS
                || bmax.x <= amin.x + EPS
                || amax.y <= bmin.y + EPS
                || bmax.y <= amin.y + EPS;
            assert!(apart, "towers in {:?} and {:?} overlap", a.block, b.block);
        }
    }
}

#[test]
fn the_city_is_dense() {
    let (_, _, towers) = layout_and_towers();
    assert!(towers.len() > 700, "only {} towers", towers.len());
}
