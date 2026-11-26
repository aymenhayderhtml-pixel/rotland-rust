//! Tests for `overgrowth.rs` and `overgrowth_roofs.rs`.

use std::collections::HashSet;

use glam::{IVec2, Vec2};

use super::*;
use crate::metro::config::MetroConfig;
use crate::metro::highway::Highway;
use crate::metro::highway::height::HighwayHeightField;
use crate::metro::layout::{MetroDistrict, MetroLayout, Reserved};
use crate::metro::paint::MetroLook;
use crate::metro::place::MetroSite;
use crate::metro::state::index_towers_by_block;
use crate::metro::styles::{default_styles, style_for};
use crate::metro::tower_parts::{PartKind, tower_parts};
use crate::metro::towers::plan_towers;

const EPS: f32 = 1e-3; // new: DECISIONS 2026-11-26 (test tolerance)

fn load_cfg() -> MetroConfig {
    MetroConfig::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json loads")
}

fn load_look() -> MetroLook {
    MetroLook::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json look loads")
}

fn build_metro(center: Vec2, yaw: f32) -> MetroWorld {
    let config = load_cfg();
    let look = load_look();
    let half_size = config.size_blocks as f32 * config.block_size * 0.5;
    let site = MetroSite {
        key: config.key.clone(),
        name: config.name.clone(),
        center,
        half_size,
        yaw,
        ground_y: 20.0,
    };
    let layout = MetroLayout::generate(site.clone(), &config);
    let towers = plan_towers(&layout, &config);
    let towers_by_block = index_towers_by_block(&towers);
    let entrance_local = Vec2::new(0.0, half_size);
    let entrance_world = site.to_world(entrance_local);
    let out = (entrance_world - site.center).normalize();
    let highway = Highway {
        from_town: "blackwater".into(),
        points: vec![entrance_world + out * 500.0, entrance_world],
        width: config.highway.highway_width,
    };
    let highway_field = HighwayHeightField::build(&highway, 20.0, &|_x, _z| 20.0, &config);
    let damage = crate::metro::damage::damage_sites(&layout, &towers, crate::metro::damage::default_damage());
    MetroWorld {
        config,
        layout,
        towers,
        highway,
        highway_field,
        jam: Vec::new(),
        look,
        towers_by_block,
        damage,
    }
}

fn flat_height(_x: f32, _z: f32) -> f32 {
    20.0
}

fn all_chunks(metro: &MetroWorld) -> Vec<(i32, i32)> {
    let pad = 2;
    let min_x = (metro.layout.site.center.x - metro.layout.site.half_size * 1.5) / CHUNK_SIZE_M;
    let max_x = (metro.layout.site.center.x + metro.layout.site.half_size * 1.5) / CHUNK_SIZE_M;
    let min_z = (metro.layout.site.center.y - metro.layout.site.half_size * 1.5) / CHUNK_SIZE_M;
    let max_z = (metro.layout.site.center.y + metro.layout.site.half_size * 1.5) / CHUNK_SIZE_M;
    let mut coords = Vec::new();
    for cz in (min_z.floor() as i32 - pad)..=(max_z.ceil() as i32 + pad) {
        for cx in (min_x.floor() as i32 - pad)..=(max_x.ceil() as i32 + pad) {
            coords.push((cx, cz));
        }
    }
    coords
}

fn collect_all(metro: &MetroWorld) -> Vec<PropInstance> {
    let cfg = OvergrowthConfig::default();
    let styles = default_styles();
    all_chunks(metro)
        .into_iter()
        .flat_map(|c| overgrowth_props_in_chunk_with_config(metro, c, &flat_height, &cfg, styles))
        .collect()
}

fn to_tower_local(t: &crate::metro::towers::TowerSpot, p: Vec2) -> Vec2 {
    let d = p - t.pos;
    let (s, c) = t.yaw.sin_cos();
    Vec2::new(d.x * c - d.y * s, d.x * s + d.y * c)
}

#[test]
fn chunk_size_matches_block_size_and_centre_hits_block() {
    let metro = build_metro(Vec2::ZERO, 0.0);
    assert_eq!(metro.config.block_size, CHUNK_SIZE_M);
    let b = metro.layout.block_at(10, 10).expect("present block");
    let center = (b.rect.0 + b.rect.1) * 0.5;
    let chunk = ((center.x / CHUNK_SIZE_M).floor() as i32, (center.y / CHUNK_SIZE_M).floor() as i32);
    let chunk_center = Vec2::new((chunk.0 as f32 + 0.5) * CHUNK_SIZE_M, (chunk.1 as f32 + 0.5) * CHUNK_SIZE_M);
    let hit = metro.layout.block_at_point(chunk_center).expect("block at chunk centre");
    assert_eq!((hit.ix, hit.iz), (b.ix, b.iz));
}

#[test]
fn counts_per_district_edge_gt_midtown_gt_core() {
    let metro = build_metro(Vec2::new(1000.0, -2000.0), 0.6);
    let props = collect_all(&metro);
    let mut weeds = [0usize; 3];
    let mut vines = [0usize; 3];
    for p in &props {
        let pos = Vec2::new(p.position.x, p.position.z);
        let Some(d) = metro.layout.district_at(pos) else { continue };
        let idx = match d {
            MetroDistrict::Edge => 0,
            MetroDistrict::Midtown => 1,
            MetroDistrict::Core => 2,
        };
        if p.model == "weeds_crack" {
            weeds[idx] += 1;
        } else if p.model.starts_with("vine_wall_") {
            vines[idx] += 1;
        }
    }
    assert!(weeds[0] > weeds[1] && weeds[1] > weeds[2], "weeds {weeds:?}");
    assert!(vines[0] > vines[1] && vines[1] > vines[2], "vines {vines:?}");
}

#[test]
fn nothing_on_entrance_avenue_or_reserved_blocks() {
    let metro = build_metro(Vec2::new(1000.0, -2000.0), 0.6);
    let props = collect_all(&metro);
    let entrance_indices = pick_entrance_avenues(&metro.layout, metro.highway.points.last().copied());
    assert_eq!(entrance_indices.len(), AVENUE_COUNT);
    for p in &props {
        let wp = Vec2::new(p.position.x, p.position.z);
        let local = metro.layout.site.to_local(wp);
        for &idx in &entrance_indices {
            assert!(
                !metro.layout.streets[idx].contains_point(local),
                "{} placed on entrance avenue at {wp:?}",
                p.model
            );
        }
        if let Some(b) = metro.layout.block_at_point(wp) {
            assert!(
                b.reserved.is_none(),
                "{} placed on reserved block {:?} ({:?})",
                p.model,
                (b.ix, b.iz),
                b.reserved
            );
            assert_ne!(b.reserved, Some(Reserved::Park));
        }
    }
}

#[test]
fn vines_are_flat_off_facade_at_ground_and_clear_of_the_door() {
    let metro = build_metro(Vec2::new(1000.0, -2000.0), 0.6);
    let props = collect_all(&metro);
    let vines: Vec<_> = props.iter().filter(|p| p.model.starts_with("vine_wall_")).collect();
    assert!(!vines.is_empty(), "expected vines");
    for v in vines {
        let wp = Vec2::new(v.position.x, v.position.z);
        let b = metro.layout.block_at_point(wp).expect("vine inside a block");
        let list = metro.towers_by_block.get(&(b.ix, b.iz)).expect("block has towers");
        let t = list
            .iter()
            .map(|&i| &metro.towers[i])
            .min_by(|a, b| a.pos.distance_squared(wp).total_cmp(&b.pos.distance_squared(wp)))
            .expect("nearest tower");
        let local = to_tower_local(t, wp);
        assert!(
            !(local.y < -t.half.y && local.x.abs() < 4.0 - EPS),
            "vine placed over the lobby door: local={local:?}"
        );
        let off = local.x.abs().max(local.y.abs()) - t.half.x - 0.15;
        assert!((0.02 - EPS..=0.05 + EPS).contains(&off), "vine back standoff {off} out of 2..5 cm");
        assert!((v.position.y - flat_height(v.position.x, v.position.z)).abs() < EPS);
    }
}

#[test]
fn roof_props_sit_at_last_floor_top_inside_roof_rect() {
    let metro = build_metro(Vec2::new(1000.0, -2000.0), 0.6);
    let props = collect_all(&metro);
    let styles = default_styles();
    let roofs: Vec<_> = props
        .iter()
        .filter(|p| p.model == "roof_bush" || p.model == "roof_tree")
        .collect();
    assert!(!roofs.is_empty(), "expected roof props");
    for r in roofs {
        let wp = Vec2::new(r.position.x, r.position.z);
        let b = metro.layout.block_at_point(wp).expect("roof prop in block");
        let list = metro.towers_by_block.get(&(b.ix, b.iz)).expect("block has towers");
        let t = list
            .iter()
            .map(|&i| &metro.towers[i])
            .min_by(|a, b| a.pos.distance_squared(wp).total_cmp(&b.pos.distance_squared(wp)))
            .expect("nearest tower");
        let local = to_tower_local(t, wp);
        assert!(
            local.x.abs() <= t.half.x - 1.5 + EPS && local.y.abs() <= t.half.y - 1.5 + EPS,
            "roof prop {local:?} outside inset roof rect (half={:?})",
            t.half
        );
        let style = style_for(styles, t.style).expect("style");
        let fh = if style.floor_height > 0.0 { style.floor_height } else { metro.config.towers.floor_height };
        let parts = tower_parts(t, style, flat_height(t.pos.x, t.pos.y), 0);
        let last_top = parts
            .iter()
            .rfind(|p| p.kind == PartKind::Floor)
            .map(|p| p.pos.y + fh + style.roof_deck_height)
            .expect("tower has floors");
        assert!(
            (r.position.y - last_top).abs() <= 0.05,
            "roof prop y {} != last floor top {last_top}",
            r.position.y
        );
    }
}

#[test]
fn lot_props_stay_out_of_tower_footprints_and_off_streets() {
    let metro = build_metro(Vec2::new(1000.0, -2000.0), 0.6);
    let props = collect_all(&metro);
    let lot_props: Vec<_> = props
        .iter()
        .filter(|p| matches!(p.model.as_str(), "shrub_wild" | "sapling_city" | "grass_tall_patch"))
        .collect();
    assert!(!lot_props.is_empty(), "expected lot props");
    for p in lot_props {
        let wp = Vec2::new(p.position.x, p.position.z);
        assert!(!metro.in_tower_footprint(wp), "{} inside tower footprint", p.model);
        assert!(metro.layout.street_at(wp).is_none(), "{} on street surface", p.model);
    }
}

#[test]
fn deterministic_and_union_over_all_chunks_has_no_duplicates_or_misses() {
    let metro = build_metro(Vec2::new(512.0, -768.0), 0.35);
    let _default_cfg = default_overgrowth();
    let chunk = IVec2::new(8, -12);
    let a = overgrowth_props_in_chunk(&metro, (chunk.x, chunk.y), &flat_height);
    let b = overgrowth_props_in_chunk(&metro, (chunk.x, chunk.y), &flat_height);
    assert_eq!(a, b);

    let cfg = OvergrowthConfig::default();
    let styles = default_styles();
    let mut seen = HashSet::new();
    let mut total = 0usize;
    for (cx, cz) in all_chunks(&metro) {
        let items = overgrowth_props_in_chunk_with_config(&metro, (cx, cz), &flat_height, &cfg, styles);
        for p in &items {
            assert!(in_chunk(p.position.x, p.position.z, (cx, cz)));
            let key = (
                p.model.clone(),
                p.position.x.to_bits(),
                p.position.y.to_bits(),
                p.position.z.to_bits(),
                p.yaw.to_bits(),
            );
            assert!(seen.insert(key), "duplicate prop across chunks: {}", p.model);
        }
        total += items.len();
    }
    assert!(total > 0 && seen.len() == total);
}

#[test]
fn load_overgrowth_parses_and_rejects_negative_chance() {
    let json = std::fs::read_to_string(paths::repo_path("data/metro.json")).expect("read metro.json");
    assert!(load_overgrowth(&json).is_ok());
    let custom = r#"{
        "overgrowth": {
            "edge": { "crackWeedsPerBlock": 9, "streetTreeChance": 0.8 },
            "vineSpacingM": 5.0
        }
    }"#;
    let parsed = load_overgrowth(custom).expect("valid custom overgrowth");
    let edge: &DistrictOvergrowth = &parsed.edge;
    assert_eq!(edge.crack_weeds_per_block, 9);
    assert!((edge.street_tree_chance - 0.8).abs() < EPS);
    assert!((parsed.vine_spacing_m - 5.0).abs() < EPS);

    let sample = r#"{ "overgrowth": { "edge": { "streetTreeChance": -0.1 } } }"#;
    assert!(load_overgrowth(sample).is_err(), "negative chance must be rejected");
    let bad_roof = r#"{ "overgrowth": { "roofTreeChance": -0.05 } }"#;
    assert!(load_overgrowth(bad_roof).is_err(), "negative roofTreeChance must be rejected");
}
