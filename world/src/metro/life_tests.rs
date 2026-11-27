//! Unit tests for `life.rs` and `life_wires.rs`.

use std::collections::HashMap;

use glam::Vec2;

use super::*;
use crate::metro::config::MetroConfig;
use crate::metro::highway::Highway;
use crate::metro::highway::height::HighwayHeightField;
use crate::metro::layout::{MetroDistrict, MetroLayout};
use crate::metro::life::default_life;
use crate::metro::paint::MetroLook;
use crate::metro::place::MetroSite;
use crate::metro::state::index_towers_by_block;
use crate::metro::towers::plan_towers;

const CHUNK_SIZE_M: f32 = 64.0; // new: DECISIONS 2026-11-27 (64 m chunk)

const EPS: f32 = 1e-3; // new: DECISIONS 2026-11-27
const SPAN_TOL_M: f32 = 0.05; // new: DECISIONS 2026-11-27 (5 cm tolerance per R41-03)

fn build_metro(key: &str, center: Vec2, yaw: f32) -> MetroWorld {
    let config = MetroConfig::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json");
    let look = MetroLook::load_from_file(paths::repo_path("data/metro.json")).expect("look");
    let half_size = config.size_blocks as f32 * config.block_size * 0.5;
    let site = MetroSite {
        key: key.into(),
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

#[test]
fn deterministic_per_seed_and_varies_across_seeds() {
    let m1 = build_metro("seed_a", Vec2::new(3000.0, 3000.0), 0.15);
    let m2 = build_metro("seed_a", Vec2::new(3000.0, 3000.0), 0.15);
    let m3 = build_metro("seed_b", Vec2::new(3000.0, 3000.0), 0.15);

    let c = (47, 47);
    assert_eq!(life_props_in_chunk(&m1, c, &flat_height), life_props_in_chunk(&m2, c, &flat_height));
    assert_eq!(flappers_in_chunk(&m1, c, &flat_height), flappers_in_chunk(&m2, c, &flat_height));
    assert_eq!(crow_perches_in_chunk(&m1, c, &flat_height), crow_perches_in_chunk(&m2, c, &flat_height));

    let all1: Vec<_> = all_chunks(&m1).into_iter().flat_map(|ch| life_props_in_chunk(&m1, ch, &flat_height)).collect();
    let all3: Vec<_> = all_chunks(&m3).into_iter().flat_map(|ch| life_props_in_chunk(&m3, ch, &flat_height)).collect();
    assert_ne!(all1, all3);
}

#[test]
fn poles_stand_on_sidewalks_never_on_road_centre_or_tower() {
    let metro = build_metro("new_halden", Vec2::new(3000.0, 3000.0), 0.0);
    let props: Vec<_> = all_chunks(&metro)
        .into_iter()
        .flat_map(|c| life_props_in_chunk(&metro, c, &flat_height))
        .collect();
    let poles: Vec<_> = props.iter().filter(|p| p.model.starts_with("power_pole")).collect();
    assert!(!poles.is_empty(), "expected power poles along avenues");

    for p in poles {
        let wp = Vec2::new(p.position.x, p.position.z);
        assert!(!metro.in_tower_footprint(wp), "pole inside tower footprint at {wp:?}");
        if p.model != "power_pole_fallen" {
            let local = metro.layout.site.to_local(wp);
            let st = metro.layout.street_at(wp).expect("pole should be on street sidewalk band");
            let d = st.closest_point(local).distance(local);
            assert!(d > 1.0, "standing pole too close to road centreline: d={d}");
            assert!(
                d >= st.width * 0.5 - metro.look.sidewalk_width - EPS,
                "standing pole not on sidewalk: d={d}"
            );
        }
    }
}

#[test]
fn wire_spans_connect_standing_poles_and_match_x_scale() {
    let metro = build_metro("new_halden", Vec2::new(3000.0, 3000.0), 0.25);
    let props: Vec<_> = all_chunks(&metro)
        .into_iter()
        .flat_map(|c| life_props_in_chunk(&metro, c, &flat_height))
        .collect();
    let standing: Vec<Vec2> = props
        .iter()
        .filter(|p| p.model == "power_pole" || p.model == "power_pole_leaning")
        .map(|p| Vec2::new(p.position.x, p.position.z))
        .collect();
    let spans: Vec<_> = props.iter().filter(|p| p.model == "power_wire_span").collect();
    assert!(!spans.is_empty(), "expected wire spans");

    for sp in spans {
        let a = Vec2::new(sp.position.x, sp.position.z);
        let dir = Vec2::new(sp.yaw.cos(), -sp.yaw.sin());
        let len = sp.scale * 20.0;
        let b = a + dir * len;
        let near_a = standing.iter().any(|&p| p.distance(a) < SPAN_TOL_M);
        let near_b = standing.iter().any(|&p| p.distance(b) < SPAN_TOL_M);
        assert!(near_a && near_b, "span endpoints {a:?} -> {b:?} must sit at standing poles");
        assert!((a.distance(b) - len).abs() < SPAN_TOL_M);
    }
}

#[test]
fn every_billboard_has_one_matching_face() {
    let metro = build_metro("new_halden", Vec2::new(3000.0, 3000.0), 0.0);
    let props: Vec<_> = all_chunks(&metro)
        .into_iter()
        .flat_map(|c| life_props_in_chunk(&metro, c, &flat_height))
        .collect();
    let frames: Vec<_> = props
        .iter()
        .filter(|p| p.model.starts_with("billboard_roof_") || p.model == "billboard_street")
        .collect();
    let faces: Vec<_> = props.iter().filter(|p| p.model.starts_with("billboard_face_")).collect();
    assert!(!frames.is_empty(), "expected billboards");
    assert_eq!(frames.len(), faces.len(), "each billboard frame must have one face");

    for frame in frames {
        let matches = faces
            .iter()
            .filter(|f| f.position.distance(frame.position) < EPS && (f.yaw - frame.yaw).abs() < EPS)
            .count();
        assert_eq!(matches, 1, "billboard at {:?} must have exactly one matching face", frame.position);
    }
}

#[test]
fn flags_sit_at_lobby_fronts_at_four_to_six_metres() {
    let metro = build_metro("new_halden", Vec2::new(3000.0, 3000.0), 0.0);
    let props: Vec<_> = all_chunks(&metro)
        .into_iter()
        .flat_map(|c| life_props_in_chunk(&metro, c, &flat_height))
        .collect();
    let flags: Vec<_> = props.iter().filter(|p| p.model == "flag_pole_wall").collect();
    assert!(!flags.is_empty(), "expected wall flags");
    for f in flags {
        let y_above = f.position.y - 20.0;
        assert!((4.0..=6.0).contains(&y_above), "flag height {y_above} outside 4..=6 m");
    }
}

#[test]
fn perch_counts_in_range_and_core_fewer_per_block_than_edge() {
    let metro = build_metro("new_halden", Vec2::new(3000.0, 3000.0), 0.0);
    let cfg = default_life();
    let perches: Vec<_> = all_chunks(&metro)
        .into_iter()
        .flat_map(|c| crow_perches_in_chunk(&metro, c, &flat_height))
        .collect();
    assert!(!perches.is_empty(), "expected crow perches");

    let mut by_district: HashMap<MetroDistrict, usize> = HashMap::new();
    for p in &perches {
        assert!(
            (cfg.crows_per_perch[0]..=cfg.crows_per_perch[1]).contains(&p.count),
            "perch count {} out of range",
            p.count
        );
        if let Some(b) = metro.layout.block_at_point(Vec2::new(p.pos.x, p.pos.z)) {
            *by_district.entry(b.district).or_default() += 1;
        }
    }
    let edge_blocks = metro.layout.blocks.iter().filter(|b| b.district == MetroDistrict::Edge && b.reserved.is_none()).count();
    let core_blocks = metro.layout.blocks.iter().filter(|b| b.district == MetroDistrict::Core && b.reserved.is_none()).count();
    let edge_rate = *by_district.get(&MetroDistrict::Edge).unwrap_or(&0) as f32 / edge_blocks.max(1) as f32;
    let core_rate = *by_district.get(&MetroDistrict::Core).unwrap_or(&0) as f32 / core_blocks.max(1) as f32;
    assert!(core_rate < edge_rate, "core perch rate ({core_rate}) must be < edge ({edge_rate})");
}

#[test]
fn empty_outside_new_halden() {
    let metro = build_metro("new_halden", Vec2::new(3000.0, 3000.0), 0.0);
    let far_chunk = (-500, -500);
    assert!(life_props_in_chunk(&metro, far_chunk, &flat_height).is_empty());
    assert!(flappers_in_chunk(&metro, far_chunk, &flat_height).is_empty());
    assert!(crow_perches_in_chunk(&metro, far_chunk, &flat_height).is_empty());
}
