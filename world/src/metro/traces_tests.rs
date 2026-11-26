//! Unit tests for `traces.rs` and `traces_camps.rs`.

use std::collections::{HashMap, HashSet};

use glam::Vec2;

use super::*;
use crate::metro::config::MetroConfig;
use crate::metro::damage::{DamageSites, Site};
use crate::metro::highway::{Highway, HighwayHeightField};
use crate::metro::layout::MetroDistrict;
use crate::metro::paint::MetroLook;
use crate::metro::place::MetroSite;
use crate::metro::state::index_towers_by_block;
use crate::metro::styles::{TowerStyle, default_styles, style_for};
use crate::metro::towers::TowerSpot;
use camps::POSTER_MODEL;
use crate::metro::tower_parts::{PartKind, tower_parts};
use crate::metro::towers::plan_towers;

fn highway_for(layout: &MetroLayout, cfg: &MetroConfig) -> Highway {
    let avenue = layout
        .streets
        .iter()
        .find(|s| s.avenue && layout.contains(layout.site.to_world((s.a + s.b) * 0.5)))
        .expect("the layout has an avenue");
    let entrance = layout.site.to_world(avenue.a);
    let out = (entrance - layout.site.center).normalize();
    let steps = 30;
    let points = (0..=steps)
        .map(|i| entrance + out * 3000.0 * (1.0 - i as f32 / steps as f32))
        .collect();
    Highway { from_town: "blackwater".into(), points, width: cfg.highway.highway_width }
}

fn test_world(seed_x: f32, damage: DamageSites) -> MetroWorld {
    let config = MetroConfig::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json");
    let look = MetroLook::load_from_file(paths::repo_path("data/metro.json")).expect("look");
    let site = MetroSite {
        key: config.key.clone(),
        name: config.name.clone(),
        center: Vec2::new(seed_x, -2000.0),
        half_size: config.size_blocks as f32 * config.block_size * 0.5,
        yaw: 0.6,
        ground_y: 10.0,
    };
    let layout = MetroLayout::generate(site, &config);
    let towers = plan_towers(&layout, &config);
    let highway = highway_for(&layout, &config);
    let highway_field = HighwayHeightField::build(&highway, layout.site.ground_y, &|_, _| 10.0, &config);
    let jam = crate::metro::plan_jam(&highway, &layout, &config);
    let towers_by_block = index_towers_by_block(&towers);
    MetroWorld { config, layout, towers, highway, highway_field, jam, look, towers_by_block, damage }
}

fn all_chunks(metro: &MetroWorld) -> Vec<(i32, i32)> {
    let pad = metro.config.block_size * 2.0;
    let r = metro.layout.site.half_size * std::f32::consts::SQRT_2 + pad;
    let min_cx = ((metro.layout.site.center.x - r) / CHUNK_SIZE_M).floor() as i32;
    let max_cx = ((metro.layout.site.center.x + r) / CHUNK_SIZE_M).ceil() as i32;
    let min_cz = ((metro.layout.site.center.y - r) / CHUNK_SIZE_M).floor() as i32;
    let max_cz = ((metro.layout.site.center.y + r) / CHUNK_SIZE_M).ceil() as i32;
    (min_cz..=max_cz).flat_map(|cz| (min_cx..=max_cx).map(move |cx| (cx, cz))).collect()
}

fn collect_all(metro: &MetroWorld, height: &dyn Fn(f32, f32) -> f32) -> Vec<PropInstance> {
    all_chunks(metro)
        .into_iter()
        .flat_map(|c| traces_props_in_chunk(metro, c, height))
        .collect()
}

fn tower_local_offset(t: &TowerSpot, p: Vec2) -> Vec2 {
    let d = p - t.pos;
    let (s, c) = t.yaw.sin_cos();
    Vec2::new(d.x * c - d.y * s, d.x * s + d.y * c)
}

#[test]
fn car_walls_block_one_street_leave_gap_and_avoid_entrance_and_reserved() {
    let metro = test_world(1000.0, DamageSites::default());
    let cfg = default_traces();
    let entrance_aves = entrance_avenue_indices(&metro);
    let props = collect_all(&metro, &|_, _| 10.0);
    let walls: Vec<&PropInstance> = props.iter().filter(|p| p.model == CAR_WALL_MODEL).collect();
    assert!((cfg.car_walls.count[0]..=cfg.car_walls.count[1]).contains(&walls.len()));

    for w in &walls {
        let local = metro.layout.site.to_local(Vec2::new(w.position.x, w.position.z));
        let hits: Vec<(usize, &MetroStreet)> = metro
            .layout
            .streets
            .iter()
            .enumerate()
            .filter(|(_, s)| s.contains_point(local))
            .collect();
        assert_eq!(hits.len(), 1, "car wall should be on exactly one street");
        let (s_idx, s) = hits[0];
        assert!(!s.avenue && !entrance_aves.contains(&s_idx));
        let dist_centre = s.closest_point(local).distance(local);
        let wall_half = cfg.wall_width_m * 0.5;
        let open_end = s.width * 0.5 - (dist_centre + wall_half);
        let other_open = s.width * 0.5 - (wall_half - dist_centre).abs();
        assert!(open_end.max(other_open) >= cfg.gap_m - 1e-3, "gap too small: {open_end} / {other_open}");
        if let Some(b) = metro.layout.block_at_point(Vec2::new(w.position.x, w.position.z)) {
            assert!(b.reserved.is_none(), "car wall next to reserved block");
        }
    }

    let first_wall = Vec2::new(walls[0].position.x, walls[0].position.z);
    let damaged = test_world(
        1000.0,
        DamageSites {
            fallen: Site { block: (0, 0), pos: first_wall, yaw: 0.0 },
            ..DamageSites::default()
        },
    );
    let damaged_walls: Vec<Vec2> = collect_all(&damaged, &|_, _| 10.0)
        .into_iter()
        .filter(|p| p.model == CAR_WALL_MODEL)
        .map(|p| Vec2::new(p.position.x, p.position.z))
        .collect();
    assert!(damaged_walls.iter().all(|p| p.distance(first_wall) > 1.0), "damage street skipped");
}

#[test]
fn graffiti_and_posters_sit_two_cm_off_facade_and_beside_door() {
    let metro = test_world(1000.0, DamageSites::default());
    let cfg = default_traces();
    let ground = 10.0_f32;
    let props = collect_all(&metro, &|_, _| ground);
    let graffiti: Vec<&PropInstance> = props.iter().filter(|p| p.model.starts_with("graffiti_")).collect();
    let posters: Vec<&PropInstance> = props.iter().filter(|p| p.model == POSTER_MODEL).collect();
    assert!(!graffiti.is_empty() && !posters.is_empty());

    let find_tower = |pos: Vec2| {
        metro.towers.iter().min_by(|a, b| a.pos.distance_squared(pos).total_cmp(&b.pos.distance_squared(pos)))
    };

    for g in &graffiti {
        let p = Vec2::new(g.position.x, g.position.z);
        let t = find_tower(p).expect("nearest tower");
        let local = tower_local_offset(t, p);
        let off = local.x.abs().max(local.y.abs()) - t.half.x;
        assert!((0.015..=0.025).contains(&off), "graffiti offset {off} out of range");
        let h = g.position.y - ground;
        assert!((cfg.graffiti_height_m[0]..=cfg.graffiti_height_m[1]).contains(&h), "graffiti y {h}");
    }

    for poster in &posters {
        let p = Vec2::new(poster.position.x, poster.position.z);
        let t = find_tower(p).expect("nearest tower");
        let local = tower_local_offset(t, p);
        let front_off = (-local.y) - t.half.y;
        assert!((0.015..=0.025).contains(&front_off), "poster not 2 cm off front face: {front_off}");
        let x_share = local.x.abs() / t.size as f32;
        assert!((0.3..=0.4).contains(&x_share), "poster |x| share {x_share} not beside door");
        let h = poster.position.y - ground;
        assert!((cfg.poster_height_m[0]..=cfg.poster_height_m[1]).contains(&h), "poster y {h}");
    }
}

#[test]
fn camps_sit_on_brick_lowrise_roof_tops_and_fit_footprint() {
    let metro = test_world(1000.0, DamageSites::default());
    let cfg = default_traces();
    let styles = default_styles();
    let sites = camp_sites(&metro);
    assert!(!sites.is_empty(), "expected at least one rooftop camp");

    let props = collect_all(&metro, &|_, _| metro.layout.site.ground_y);
    let camp_props: Vec<&PropInstance> = props.iter().filter(|p| p.model == "rooftop_camp").collect();
    assert_eq!(camp_props.len(), sites.len());

    for site in &sites {
        let t = &metro.towers[site.tower];
        assert_eq!(t.style, TowerStyle::Brick);
        assert!((4..=8).contains(&t.floors), "camp on {} floors", t.floors);
        assert!(t.size as f32 >= cfg.camp_size_m + 2.0 * cfg.camp_margin_m);
        let style = style_for(styles, t.style).expect("brick style");
        let parts = tower_parts(t, style, metro.layout.site.ground_y, 0);
        let last_floor = parts.iter().rfind(|p| p.kind == PartKind::Floor).expect("floor part");
        let roof_top = last_floor.pos.y + style.floor_height + style.roof_deck_height;
        assert!((site.pos.y - roof_top).abs() <= 0.05, "camp y {} vs roof {roof_top}", site.pos.y);
    }
}

#[test]
fn sidewalk_bodies_stay_on_sidewalks_and_off_asphalt_and_reserved() {
    let metro = test_world(1000.0, DamageSites::default());
    let props = collect_all(&metro, &|_, _| 10.0);
    let sidewalk_bodies: Vec<&PropInstance> = props
        .iter()
        .filter(|p| p.model.starts_with("body_") && !metro.in_tower_footprint(Vec2::new(p.position.x, p.position.z)))
        .collect();
    assert!(!sidewalk_bodies.is_empty());

    for b in sidewalk_bodies {
        let w = Vec2::new(b.position.x, b.position.z);
        let local = metro.layout.site.to_local(w);
        let s = metro.layout.street_at(w).expect("body must lie on a street strip");
        let dist = s.closest_point(local).distance(local);
        assert!(dist > s.width * 0.5 - metro.look.sidewalk_width && dist < s.width * 0.5, "body on asphalt: {dist}");
        if let Some(block) = metro.layout.block_at_point(w) {
            assert!(block.reserved.is_none(), "body on reserved block {:?}", block.reserved);
        }
    }
}

#[test]
fn district_counts_order_edge_midtown_core() {
    let metro = test_world(1000.0, DamageSites::default());
    let props = collect_all(&metro, &|_, _| 10.0);
    let mut walls = HashMap::from([(MetroDistrict::Edge, 0usize), (MetroDistrict::Midtown, 0), (MetroDistrict::Core, 0)]);
    let mut graffiti = walls.clone();
    for p in &props {
        let Some(d) = metro.layout.district_at(Vec2::new(p.position.x, p.position.z)) else { continue };
        if p.model == CAR_WALL_MODEL {
            *walls.entry(d).or_default() += 1;
        } else if p.model.starts_with("graffiti_") {
            *graffiti.entry(d).or_default() += 1;
        }
    }
    assert!(walls[&MetroDistrict::Edge] >= walls[&MetroDistrict::Midtown]);
    assert!(walls[&MetroDistrict::Midtown] >= walls[&MetroDistrict::Core]);
    assert!(graffiti[&MetroDistrict::Edge] >= graffiti[&MetroDistrict::Midtown]);
    assert!(graffiti[&MetroDistrict::Midtown] >= graffiti[&MetroDistrict::Core]);
}

#[test]
fn replaced_towers_get_no_traces_and_burnt_blocks_get_no_camp_or_dressing() {
    let base = test_world(1000.0, DamageSites::default());
    let sites = camp_sites(&base);
    let camp_idx = sites[0].tower;
    let camp_tower = &base.towers[camp_idx];
    let other_brick_idx = base
        .towers
        .iter()
        .enumerate()
        .find(|(i, t)| *i != camp_idx && t.style == TowerStyle::Brick && (4..=8).contains(&t.floors))
        .map(|(i, _)| i)
        .expect("second brick tower");
    let burnt_block = base.towers[other_brick_idx].block;

    let damage = DamageSites {
        fallen: Site::default(),
        leaning: Site::default(),
        burnt_block,
        replaced_towers: vec![camp_idx],
    };
    let metro = test_world(1000.0, damage);
    assert!(camp_sites(&metro).iter().all(|c| c.tower != camp_idx && metro.towers[c.tower].block != burnt_block));

    let props = collect_all(&metro, &|_, _| 10.0);
    for p in &props {
        let pos2 = Vec2::new(p.position.x, p.position.z);
        let in_replaced = tower_local_offset(camp_tower, pos2).abs().max_element() < camp_tower.half.x + 0.5;
        if in_replaced && !p.model.starts_with("body_") {
            panic!("replaced tower still received prop {}", p.model);
        }
        if metro.layout.block_at_point(pos2).is_some_and(|b| b.ix == burnt_block.0 && b.iz == burnt_block.1)
            && (p.model == "rooftop_camp"
                || p.model.starts_with("lobby_")
                || (p.model == "body_sheet_b" && metro.in_tower_footprint(pos2)))
        {
            panic!("burnt block received {} ", p.model);
        }
    }

    let dressed = lobby_dressing(camp_tower, default_traces(), 7);
    assert!(dressed.iter().all(|d| d.z < 0.3 && d.x.abs() < 0.5));
}

#[test]
fn deterministic_and_chunk_union_has_no_duplicates_or_misses() {
    let metro = test_world(1000.0, DamageSites::default());
    let slope = |x: f32, z: f32| 5.0 + 0.01 * x - 0.02 * z;
    let a = collect_all(&metro, &slope);
    let b = collect_all(&metro, &slope);
    assert_eq!(a, b);
    let mut keys = HashSet::new();
    for p in &a {
        let k = (p.model.clone(), p.position.x.to_bits(), p.position.y.to_bits(), p.position.z.to_bits());
        assert!(keys.insert(k), "duplicate prop across chunks: {}", p.model);
    }
    let other = test_world(1600.0, DamageSites::default());
    assert_ne!(a, collect_all(&other, &slope));
}

#[test]
fn config_parses_default_and_rejects_chance_above_one() {
    let mut cfg = TraceConfig::default();
    assert!(cfg.validate().is_ok());
    let json = serde_json::json!({ "traces": { "graffitiLobbyChance": 1.2 } }).to_string();
    assert!(load_traces(&json).is_err());
    cfg.poster_chance = 1.1;
    assert!(cfg.validate().is_err());
}
