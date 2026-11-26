//! Tests for the New Halden placeholder towers: budgets, winding, colours, chunks, colliders, skyline.

use std::collections::HashSet;

use engine::render::ColorVertex;
use glam::{IVec2, Vec2, Vec3};
use world::metro::{
    DamageSites, MetroConfig, MetroLayout, MetroLook, MetroSite, PartKind, TowerKind, TowerSpot, TowerStyle,
    default_styles, plan_towers, style_for,
};

use crate::props::{FADE, lod_fade, shadow_lod_for_rule};

use super::boxes::{Detail, spot_seed, tower_mesh};
use super::hulls::{tower_bases, tower_hulls};
use super::near::{tower_chunk, towers_in_chunk};
use super::parts::{chunk_tower_parts, tower_load_ring_range};
use super::skyline::{skyline_indices, skyline_key, skyline_vertices};

/// Triangles in a flat vertex list.
fn tri_count(verts: &[ColorVertex]) -> usize {
    verts.len() / 3
}

const GROUND_Y: f32 = 20.0;

fn cfg() -> MetroConfig {
    MetroConfig::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json loads")
}

fn look() -> MetroLook {
    MetroLook::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json look loads")
}

/// A city 6 km east of the origin (so its towers sit in other chunks than chunk 0).
fn city(cfg: &MetroConfig) -> Vec<TowerSpot> {
    let site = MetroSite {
        key: cfg.key.clone(),
        name: cfg.name.clone(),
        center: Vec2::new(6000.0, 0.0),
        half_size: cfg.size_blocks as f32 * cfg.block_size * 0.5,
        yaw: 0.0,
        ground_y: GROUND_Y,
    };
    let layout = MetroLayout::generate(site, cfg);
    plan_towers(&layout, cfg)
}

fn spot(floors: u32, half: f32, landmark: bool) -> TowerSpot {
    TowerSpot {
        pos: Vec2::new(6000.0, 0.0),
        half: Vec2::splat(half),
        yaw: 0.0,
        height: floors as f32 * 3.8,
        floors,
        kind: if landmark { TowerKind::Landmark } else { TowerKind::Highrise },
        landmark,
        block: (0, 0),
        style: if landmark { TowerStyle::Deco } else { TowerStyle::Stone },
        size: (half * 2.0) as u32,
    }
}

fn luma(c: [u8; 3]) -> f32 {
    0.299 * f32::from(c[0]) + 0.587 * f32::from(c[1]) + 0.114 * f32::from(c[2])
}

#[test]
fn near_tower_stays_inside_the_triangle_budget() {
    let look = look();
    let towers = city(&cfg());
    assert!(!towers.is_empty(), "the city plans towers");
    for t in &towers {
        assert!(tri_count(&tower_mesh(t, &look, Detail::Near)) <= 1500);
    }
    // Worst case: the tallest landmark on the biggest footprint, and a 30-floor Core tower.
    let worst_landmark = spot(42, 20.0, true);
    let worst_core = spot(30, 20.0, false);
    assert!(tri_count(&tower_mesh(&worst_landmark, &look, Detail::Near)) <= 1500);
    assert!(tri_count(&tower_mesh(&worst_core, &look, Detail::Near)) <= 1500);
}

#[test]
fn far_tower_stays_inside_tris_per_tower() {
    let cfg = cfg();
    let look = look();
    for t in &city(&cfg) {
        assert!(tri_count(&tower_mesh(t, &look, Detail::Far)) <= cfg.skyline.tris_per_tower);
    }
    assert!(tri_count(&tower_mesh(&spot(42, 20.0, true), &look, Detail::Far)) <= cfg.skyline.tris_per_tower);
}

#[test]
fn every_triangle_is_clockwise_seen_from_outside() {
    // The mesh pipeline culls back faces with FrontFace::Cw, so each triangle's normal (edge cross
    // product) must point into the tower's interior side of its own centroid direction.
    let look = look();
    let samples = [spot(12, 10.0, false), spot(42, 20.0, true), spot(30, 14.0, false)];
    for s in &samples {
        for detail in [Detail::Near, Detail::Far] {
            let verts = tower_mesh(s, &look, detail);
            let inside = Vec3::new(0.0, s.height * 0.5, 0.0);
            for tri in verts.chunks(3) {
                let [a, b, c] = [tri[0], tri[1], tri[2]].map(|v| Vec3::from(v.position));
                let cross = (b - a).cross(c - a);
                let centroid = (a + b + c) / 3.0;
                assert!(cross.dot(centroid - inside) < 0.0, "triangle faces inward");
            }
        }
    }
}

#[test]
fn base_band_is_darker_than_every_wall_colour() {
    let look = look();
    let t = spot(20, 12.0, false);
    let verts = tower_mesh(&t, &look, Detail::Near);
    let base_verts: Vec<&ColorVertex> = verts.iter().filter(|v| v.color[..3] == look.base_color).collect();
    assert!(!base_verts.is_empty(), "the base band is drawn");
    assert!(base_verts.iter().all(|v| v.position[1] <= look.base_height + 1e-3));
    for body in &look.wall_palette {
        assert!(luma(look.base_color) < luma(*body), "base must be darker than {body:?}");
    }
}

#[test]
fn ordinary_towers_have_a_rooftop_box_and_landmarks_a_step() {
    let look = look();
    let ordinary = spot(20, 12.0, false);
    let top_box = ordinary.height + look.roof_box_height;
    let verts = tower_mesh(&ordinary, &look, Detail::Near);
    assert!(verts.iter().any(|v| (v.position[1] - top_box).abs() < 1e-3), "rooftop box top at height + 4");

    let landmark = spot(40, 16.0, true);
    let step_top = landmark.height + look.step_height;
    let verts = tower_mesh(&landmark, &look, Detail::Near);
    assert!(verts.iter().any(|v| (v.position[1] - step_top).abs() < 1e-3), "landmark step on top");
}

#[test]
fn spot_seed_is_stable_and_distinct_for_different_spots() {
    let a = spot(10, 10.0, false);
    let mut b = a.clone();
    b.pos.x += 64.0;
    assert_eq!(spot_seed(&a), spot_seed(&a.clone()));
    assert_ne!(spot_seed(&a), spot_seed(&b));
}

#[test]
fn chunk_parts_hold_only_the_towers_of_their_chunk_and_are_deterministic() {
    let cfg = cfg();
    let styles = default_styles();
    let towers = city(&cfg);
    let coord = tower_chunk(towers[0].pos);
    let a = chunk_tower_parts(&towers, coord, GROUND_Y, styles, &cfg, &DamageSites::default());
    assert!(!a.is_empty(), "chunk with towers produces parts");
    let b = chunk_tower_parts(&towers, coord, GROUND_Y, styles, &cfg, &DamageSites::default());
    assert_eq!(a, b, "same input, same parts");
    assert!(chunk_tower_parts(&towers, IVec2::new(-500, -500), GROUND_Y, styles, &cfg, &DamageSites::default()).is_empty());

    // Single-tower slice: every non-interior part shares the tower's yaw.
    let single = vec![towers[0].clone()];
    let one_parts = chunk_tower_parts(&single, coord, GROUND_Y, styles, &cfg, &DamageSites::default());
    assert!(one_parts.iter().filter(|p| p.kind != PartKind::Interior).all(|p| (p.yaw - single[0].yaw).abs() < 1e-6));
}

#[test]
fn tower_lod_switch_uses_lod0_distance_and_fade_band() {
    let cfg = cfg();
    let ends = [cfg.towers.lod0_distance, tower_load_ring_range()];
    assert_eq!(&lod_fade(84.0, &ends, FADE)[..], &[(0, 1.0, false)]);
    assert_eq!(&lod_fade(96.0, &ends, FADE)[..], &[(1, 1.0, false)]);

    // Inside the FADE band around 90 m: 89 m favours LOD0, 91 m favours LOD1.
    let at_89 = lod_fade(89.0, &ends, FADE);
    assert_eq!(at_89.len(), 2);
    assert_eq!(at_89[0].0, 0);
    assert!(at_89[0].1 > 0.5);

    let at_91 = lod_fade(91.0, &ends, FADE);
    assert_eq!(at_91.len(), 2);
    assert_eq!(at_91[1].0, 1);
    assert!(at_91[1].1 > 0.5);
}

#[test]
fn tower_shadows_only_cast_from_lod0() {
    assert_eq!(shadow_lod_for_rule(0, 2, 50.0, true), Some(0));
    assert_eq!(shadow_lod_for_rule(1, 2, 100.0, true), None);
}

#[test]
fn lobby_interior_parts_stay_inside_the_base_footprint() {
    let cfg = cfg();
    let styles = default_styles();
    let mut t = spot(12, 12.0, false);
    t.yaw = 0.7;
    let coord = tower_chunk(t.pos);
    let parts = chunk_tower_parts(&[t.clone()], coord, GROUND_Y, styles, &cfg, &DamageSites::default());
    let interior: Vec<_> = parts.iter().filter(|p| p.kind == PartKind::Interior).collect();
    assert!(!interior.is_empty(), "tower has lobby interior props");
    let (s, c) = t.yaw.sin_cos();
    for p in interior {
        let d = Vec2::new(p.pos.x - t.pos.x, p.pos.z - t.pos.y);
        let local_x = d.x * c - d.y * s;
        let local_z = d.x * s + d.y * c;
        assert!(local_x.abs() <= t.half.x + 1e-3, "interior x out of bounds: {local_x}");
        assert!(local_z.abs() <= t.half.y + 1e-3, "interior z out of bounds: {local_z}");
    }
}

#[test]
fn hulls_are_one_box_per_tower_starting_at_base_height() {
    let styles = default_styles();
    let towers = city(&cfg());
    let coord = tower_chunk(towers[0].pos);
    let hulls = tower_hulls(&towers, coord, GROUND_Y, styles, &DamageSites::default());
    let bases = tower_bases(&towers, coord, GROUND_Y, &DamageSites::default());
    assert_eq!(hulls.len(), towers_in_chunk(&towers, coord).count());
    assert_eq!(bases.len(), hulls.len());
    for (i, (pos, yaw, pts)) in hulls.iter().enumerate() {
        let t = towers_in_chunk(&towers, coord).nth(i).expect("same order");
        let base_h = style_for(styles, t.style).expect("style exists").base_height;
        assert_eq!(bases[i].0, format!("tw_{}_{}_base", t.style.key(), t.size));
        assert_eq!(pts.len(), 8);
        assert_eq!(*pos, Vec3::new(t.pos.x, GROUND_Y, t.pos.y));
        assert_eq!(*yaw, t.yaw);
        let (min, max) = pts.iter().fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), p| {
            (
                [lo[0].min(p[0]), lo[1].min(p[1]), lo[2].min(p[2])],
                [hi[0].max(p[0]), hi[1].max(p[1]), hi[2].max(p[2])],
            )
        });
        assert!((max[0] - min[0] - 2.0 * t.half.x).abs() < 1e-3);
        assert!((max[2] - min[2] - 2.0 * t.half.y).abs() < 1e-3);
        assert!((min[1] - base_h).abs() < 1e-3);
        assert!((max[1] - t.height.max(base_h)).abs() < 1e-3);
    }
}

#[test]
fn a_tower_in_a_loaded_chunk_is_not_in_the_skyline() {
    let cfg = cfg();
    let towers = city(&cfg);
    let first = towers[0].pos;
    let loaded: HashSet<IVec2> = [tower_chunk(first)].into_iter().collect();
    let idx = skyline_indices(&towers, &loaded, first, &cfg);
    for (i, t) in towers.iter().enumerate() {
        assert_eq!(idx.contains(&i), tower_chunk(t.pos) != tower_chunk(first), "tower {i}");
    }
}

#[test]
fn a_tower_outside_the_loaded_chunks_is_in_the_skyline() {
    let cfg = cfg();
    let towers = city(&cfg);
    let idx = skyline_indices(&towers, &HashSet::new(), towers[0].pos, &cfg);
    assert!(idx.contains(&0));
    assert_eq!(idx.len(), towers.len(), "all towers are within 6 km of the first tower");
}

#[test]
fn the_skyline_is_empty_when_every_tower_is_loaded_and_far_towers_are_dropped() {
    let cfg = cfg();
    let look = look();
    let towers = city(&cfg);
    let all: HashSet<IVec2> = towers.iter().map(|t| tower_chunk(t.pos)).collect();
    let idx = skyline_indices(&towers, &all, towers[0].pos, &cfg);
    assert!(idx.is_empty());
    assert!(skyline_vertices(&towers, &idx, GROUND_Y, &look).is_empty());

    let far_player = Vec2::new(-20000.0, 0.0);
    assert!(skyline_indices(&towers, &HashSet::new(), far_player, &cfg).is_empty());
}

#[test]
fn the_skyline_key_is_stable_inside_a_cell_and_changes_with_the_set_or_cell() {
    let mut loaded: HashSet<IVec2> = [IVec2::new(1, 2), IVec2::new(3, 4)].into_iter().collect();
    let same: HashSet<IVec2> = [IVec2::new(3, 4), IVec2::new(1, 2)].into_iter().collect();
    let key = skyline_key(&loaded, Vec2::new(100.0, 100.0), 512.0);
    assert_eq!(key, skyline_key(&same, Vec2::new(100.0, 100.0), 512.0), "same set, same key");
    assert_eq!(key, skyline_key(&loaded, Vec2::new(110.0, 120.0), 512.0), "same cell, same key");
    assert_ne!(key, skyline_key(&loaded, Vec2::new(600.0, 100.0), 512.0), "new cell, new key");
    loaded.insert(IVec2::new(9, 9));
    assert_ne!(key, skyline_key(&loaded, Vec2::new(100.0, 100.0), 512.0), "new set, new key");
}
