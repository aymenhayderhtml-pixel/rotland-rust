//! Tests for `towers.rs`, `styles.rs` and `tower_parts.rs`.

use glam::Vec2;

use crate::metro::config::MetroConfig;
use crate::metro::layout::{MetroDistrict, MetroLayout, Reserved};
use crate::metro::place::MetroSite;
use crate::metro::styles::{
    TowerStyle, VALID_SIZES, default_styles, load_styles, style_for,
};
use crate::metro::tower_parts::{PartKind, tower_parts};
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
fn heights_and_floors_follow_the_district_and_style() {
    let (cfg, layout, towers) = layout_and_towers();
    let styles = default_styles();
    for t in &towers {
        let block = layout.block_at(t.block.0, t.block.1).expect("tower sits in a present block");
        let style = style_for(styles, t.style).expect("style exists");
        let fh = if style.floor_height > 0.0 { style.floor_height } else { cfg.towers.floor_height };
        let expected_h = style.base_height + t.floors as f32 * fh + style.crown_height;
        assert!((t.height - expected_h).abs() < EPS, "height {} != {expected_h}", t.height);
        if t.landmark {
            assert_eq!(t.kind, TowerKind::Landmark);
            assert_eq!(t.style, TowerStyle::Deco);
            let [lo, hi] = cfg.towers.landmark_floors;
            assert!((lo..=hi).contains(&t.floors), "landmark floors {} out of range", t.floors);
            continue;
        }
        assert!(style.districts.contains(&block.district), "style {:?} not in {:?}", t.style, block.district);
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
        assert!((150.0..=175.0).contains(&m.height), "landmark {} m", m.height);
        assert!(marks[i + 1..].iter().all(|o| o.block != m.block), "two landmarks in a block");
    }
}

#[test]
fn towers_stay_inside_their_block_and_off_the_streets() {
    let (_, layout, towers) = layout_and_towers();
    for t in &towers {
        assert!(VALID_SIZES.contains(&t.size), "invalid snapped size {}", t.size);
        assert!((t.half.x - t.size as f32 * 0.5).abs() < EPS);
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

#[test]
fn front_faces_nearest_street() {
    let (_, layout, towers) = layout_and_towers();
    for t in towers.iter().take(64) {
        let local_pos = layout.site.to_local(t.pos);
        let nearest_local = layout
            .streets
            .iter()
            .map(|s| s.closest_point(local_pos))
            .min_by(|a, b| local_pos.distance_squared(*a).total_cmp(&local_pos.distance_squared(*b)))
            .expect("streets exist");
        let nearest_world = layout.site.to_world(nearest_local);
        let front_world = Vec2::new(-t.yaw.sin(), -t.yaw.cos());
        let to_street = nearest_world - t.pos;
        let along = to_street.dot(front_world);
        let perp = (to_street - front_world * along).length();
        assert!(along > 0.0, "tower {:?} front faces away from street", t.block);
        assert!(perp < 1.0, "tower {:?} front misses nearest street by {perp} m", t.block);
    }
}

#[test]
fn parts_stack_without_gaps_and_match_height() {
    let (_, _, towers) = layout_and_towers();
    let styles = default_styles();
    let ground_y = 10.0;
    for t in towers.iter().take(24) {
        let style = style_for(styles, t.style).expect("style");
        let parts = tower_parts(t, style, ground_y, 42);
        let base = parts.iter().find(|p| p.kind == PartKind::Base).expect("has base");
        assert!((base.pos.y - ground_y).abs() < EPS);

        let floors: Vec<_> = parts
            .iter()
            .filter(|p| matches!(p.kind, PartKind::Floor | PartKind::Setback))
            .collect();
        assert_eq!(floors.len(), t.floors as usize);
        let mut expected_y = ground_y + style.base_height;
        for f in &floors {
            assert!((f.pos.y - expected_y).abs() < EPS, "floor gap: {} != {expected_y}", f.pos.y);
            expected_y += style.floor_height;
        }
        let crown = parts
            .iter()
            .find(|p| matches!(p.kind, PartKind::Crown | PartKind::LandmarkCrown))
            .expect("has crown");
        assert!((crown.pos.y - expected_y).abs() < EPS);
        assert!((expected_y + style.crown_height - (ground_y + t.height)).abs() < EPS);
    }
}

#[test]
fn parts_are_deterministic_and_vary_with_seed() {
    let (_, _, towers) = layout_and_towers();
    let styles = default_styles();
    let t = towers.iter().find(|t| t.floors >= 6).expect("tower with >= 6 floors");
    let style = style_for(styles, t.style).expect("style");
    let a = tower_parts(t, style, 5.0, 101);
    let b = tower_parts(t, style, 5.0, 101);
    assert_eq!(a, b);
    let c = tower_parts(t, style, 5.0, 9999);
    assert_ne!(a, c, "different seeds should vary _floor/_floor_b order");
}

#[test]
fn landmarks_have_smaller_setbacks_and_landmark_crown() {
    let (_, _, towers) = layout_and_towers();
    let styles = default_styles();
    let lm = towers.iter().find(|t| t.landmark).expect("has landmark");
    let style = style_for(styles, lm.style).expect("deco style");
    let parts = tower_parts(lm, style, 0.0, 7);
    let setbacks: Vec<_> = parts.iter().filter(|p| p.kind == PartKind::Setback).collect();
    assert!(!setbacks.is_empty(), "landmark must have setback parts");
    for sb in setbacks {
        let sb_size: u32 = sb
            .model
            .split('_')
            .nth(2)
            .and_then(|s| s.parse().ok())
            .expect("model has size");
        assert!(sb_size < lm.size, "setback size {sb_size} not smaller than base {}", lm.size);
    }
    assert!(parts.iter().any(|p| p.kind == PartKind::LandmarkCrown));
}

#[test]
fn styles_config_parses_and_rejects_invalid_size() {
    let json = std::fs::read_to_string(paths::repo_path("data/metro.json")).expect("read metro.json");
    let styles = load_styles(&json).expect("valid styles");
    assert_eq!(styles.len(), 5);
    assert_eq!(TowerStyle::parse("brick"), Some(TowerStyle::Brick));
    let bad = json.replace("[16, 24]", "[20, 24]");
    assert!(load_styles(&bad).is_err(), "size 20 must be rejected");
}
