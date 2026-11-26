//! Tests for `damage.rs` and `state.rs` damage integration.

use std::collections::HashSet;

use glam::Vec2;

use super::*;
use crate::metro::config::MetroConfig;
use crate::metro::highway::{Highway, HighwayHeightField};
use crate::metro::paint::MetroLook;
use crate::metro::place::MetroSite;
use crate::metro::towers::plan_towers;

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

fn highway_for(layout: &MetroLayout, cfg: &MetroConfig) -> Highway {
    let avenue = layout
        .streets
        .iter()
        .find(|s| s.avenue && layout.contains((s.a + s.b) * 0.5))
        .expect("layout has an avenue");
    let entrance = layout.site.to_world(avenue.a);
    let out = (entrance - layout.site.center).normalize();
    let points = (0..=10)
        .map(|i| entrance + out * 500.0 * (1.0 - i as f32 / 10.0))
        .collect();
    Highway { from_town: "blackwater".into(), points, width: cfg.highway.highway_width }
}

fn setup_world() -> (MetroConfig, DamageConfig, MetroWorld) {
    let mcfg = cfg();
    let dcfg = DamageConfig::default();
    let layout = MetroLayout::generate(site(&mcfg), &mcfg);
    let towers = plan_towers(&layout, &mcfg);
    let highway = highway_for(&layout, &mcfg);
    let highway_field = HighwayHeightField::build(&highway, layout.site.ground_y, &|_, _| 10.0, &mcfg);
    let look = MetroLook::load_from_file(paths::repo_path("data/metro.json")).expect("look loads");
    let towers_by_block = index_towers_by_block(&towers);
    let damage = damage_sites(&layout, &towers, &dcfg);
    let world = MetroWorld {
        config: mcfg.clone(),
        layout,
        towers,
        highway,
        highway_field,
        jam: Vec::new(),
        look,
        towers_by_block,
        damage,
    };
    (mcfg, dcfg, world)
}

fn all_props(world: &MetroWorld, dcfg: &DamageConfig, height: &dyn Fn(f32, f32) -> f32) -> Vec<PropInstance> {
    let mut chunks = HashSet::new();
    for b in &world.layout.blocks {
        let c = world.layout.site.to_world((b.rect.0 + b.rect.1) * 0.5);
        let cx = (c.x / CHUNK_SIZE_M).floor() as i32;
        let cz = (c.y / CHUNK_SIZE_M).floor() as i32;
        for dx in -1..=1 {
            for dz in -1..=1 {
                chunks.insert((cx + dx, cz + dz));
            }
        }
    }
    let mut out = Vec::new();
    for chunk in chunks {
        out.extend(damage_props_in_chunk_with_config(world, chunk, height, dcfg));
    }
    out
}

#[test]
fn same_site_gives_same_sites_and_different_seed_differs() {
    let (mcfg, dcfg, world) = setup_world();
    let again = damage_sites(&world.layout, &world.towers, &dcfg);
    assert_eq!(world.damage, again);

    let mut moved = site(&mcfg);
    moved.center.x += 64.0;
    let other_layout = MetroLayout::generate(moved, &mcfg);
    let other_towers = plan_towers(&other_layout, &mcfg);
    let other = damage_sites(&other_layout, &other_towers, &dcfg);
    assert_ne!(
        (world.damage.fallen.block, world.damage.leaning.block, world.damage.burnt_block),
        (other.fallen.block, other.leaning.block, other.burnt_block)
    );
}

#[test]
fn nothing_on_reserved_block_or_avenue() {
    let (_, dcfg, world) = setup_world();
    let d = &world.damage;
    for bkey in [d.fallen.block, d.leaning.block, d.burnt_block] {
        let b = world.layout.block_at(bkey.0, bkey.1).expect("site block present");
        assert!(b.reserved.is_none(), "damage block {bkey:?} is reserved");
    }
    assert_ne!(d.fallen.block, d.leaning.block);
    assert_ne!(d.fallen.block, d.burnt_block);
    assert_ne!(d.leaning.block, d.burnt_block);

    for p in all_props(&world, &dcfg, &|_, _| 10.0) {
        let xz = Vec2::new(p.position.x, p.position.z);
        if let Some(b) = world.layout.block_at_point(xz) {
            assert!(b.reserved.is_none(), "{} landed on reserved block {:?}", p.model, (b.ix, b.iz));
        }
        if let Some(s) = world.layout.street_at(xz) {
            assert!(!s.avenue, "{} landed on avenue", p.model);
        }
    }
}

#[test]
fn tower_is_replaced_matches_fallen_leaning_and_crushed_only() {
    let (_, dcfg, world) = setup_world();
    let half_ext = Vec2::new(
        FALLEN_MODEL_LENGTH_M.clamp(dcfg.fallen_length_m[0], dcfg.fallen_length_m[1]) * 0.5,
        dcfg.fallen_width_m * 0.5,
    );
    let mut replaced_count = 0usize;
    for (i, t) in world.towers.iter().enumerate() {
        if world.damage.tower_is_replaced(i) {
            replaced_count += 1;
        } else {
            assert!(
                !obb_overlaps(world.damage.fallen.pos, world.damage.fallen.yaw, half_ext, t.pos, t.yaw, t.half),
                "non-replaced tower {i} overlaps fallen section"
            );
            assert_ne!(t.pos, world.damage.leaning.pos, "leaning tower was not replaced");
        }
    }
    assert!(replaced_count >= 2, "expected at least fallen + leaning replaced, got {replaced_count}");
    assert!(world.towers.iter().enumerate().any(|(i, t)| t.block == world.damage.fallen.block && world.damage.tower_is_replaced(i)));
    assert!(world.towers.iter().enumerate().any(|(i, t)| t.pos == world.damage.leaning.pos && world.damage.tower_is_replaced(i)));
    for &i in world.towers_by_block.get(&world.damage.burnt_block).expect("burnt block has towers") {
        assert!(!world.damage.tower_is_replaced(i), "burnt tower should remain standing");
    }
}

#[test]
fn fallen_section_spans_across_street_into_opposite_block() {
    let (_, dcfg, world) = setup_world();
    let f = &world.damage.fallen;
    let street = world.layout.street_at(f.pos).expect("fallen centre sits on a street");
    assert!(!street.avenue, "fallen street must be a plain street");

    let across = x_axis(f.yaw);
    let s_dir = (world.layout.site.to_world(street.b) - world.layout.site.to_world(street.a)).normalize();
    assert!(across.dot(s_dir).abs() < 1e-3, "fallen section must lie across (perpendicular to) the street");

    let half_len = FALLEN_MODEL_LENGTH_M.clamp(dcfg.fallen_length_m[0], dcfg.fallen_length_m[1]) * 0.5;
    assert!(half_len * 2.0 > street.width && dcfg.fallen_width_m >= street.width);
    let end_a = world.layout.block_at_point(f.pos - across * half_len).expect("base end in block");
    let end_b = world.layout.block_at_point(f.pos + across * half_len).expect("top end in opposite block");
    assert_eq!((end_a.ix, end_a.iz), f.block);
    assert_ne!((end_b.ix, end_b.iz), f.block);
    assert!(end_b.reserved.is_none());
}

#[test]
fn leaning_direction_points_at_neighbour_tower() {
    let (_, dcfg, world) = setup_world();
    let l = &world.damage.leaning;
    let block = world.layout.block_at(l.block.0, l.block.1).expect("leaning block");
    assert_eq!(block.district, dcfg.leaning_district);
    let tower = world.towers.iter().find(|t| t.pos == l.pos).expect("leaning matches a tower");
    assert!(tower.size >= dcfg.leaning_min_size);

    let lean_dir = x_axis(l.yaw);
    let cos_30 = 30.0_f32.to_radians().cos();
    let hits_neighbour = NEIGHBOURS.iter().any(|&(dx, dz)| {
        world.towers_by_block.get(&(l.block.0 + dx, l.block.1 + dz)).is_some_and(|list| {
            list.iter().any(|&i| (world.towers[i].pos - l.pos).normalize().dot(lean_dir) >= cos_30)
        })
    });
    assert!(hits_neighbour, "leaning yaw does not point within 30 deg of a neighbour tower");
}

#[test]
fn burnt_block_towers_overlays_ash_and_cars() {
    let (_, dcfg, world) = setup_world();
    let bb = world.damage.burnt_block;
    let block = world.layout.block_at(bb.0, bb.1).expect("burnt block exists");
    assert_eq!(block.district, dcfg.burnt_district);
    assert!(world.damage.tower_is_burnt(bb));
    assert!(!world.damage.tower_is_burnt(world.damage.fallen.block));

    let burnt_towers = world.towers_by_block.get(&bb).expect("burnt towers");
    assert!(burnt_towers.len() >= dcfg.burnt_min_towers);

    let props = all_props(&world, &dcfg, &|_, _| 10.0);
    let overlays: Vec<_> = props.iter().filter(|p| p.model == "burnt_floor_overlay").collect();
    assert_eq!(overlays.len(), burnt_towers.len());
    for (ov, &ti) in overlays.iter().zip(burnt_towers.iter()) {
        let expected_scale = world.towers[ti].size as f32 / BURNT_OVERLAY_BASE_WIDTH_M;
        assert!((ov.scale - expected_scale).abs() < 1e-4);
    }

    let ashes: Vec<_> = props.iter().filter(|p| p.model == "ash_patch").collect();
    assert_eq!(ashes.len(), burnt_towers.len() * dcfg.ash_patches_per_tower);
    for a in ashes {
        let xz = Vec2::new(a.position.x, a.position.z);
        assert!(world.layout.street_at(xz).is_none(), "ash_patch on street");
        assert!(!world.in_tower_footprint(xz), "ash_patch inside tower footprint");
    }

    let cars: Vec<_> = props.iter().filter(|p| p.model == "car_burnt_pile").collect();
    assert!((dcfg.burnt_car_count[0]..=dcfg.burnt_car_count[1]).contains(&cars.len()));
    for c in cars {
        let xz = Vec2::new(c.position.x, c.position.z);
        let s = world.layout.street_at(xz).expect("car_burnt_pile on street");
        assert!(!s.avenue);
        let local = world.layout.site.to_local(xz);
        assert!(s.closest_point(local).distance(local) >= CAR_MIN_CENTER_DIST_M - 1e-3);
    }
}

#[test]
fn in_tower_footprint_skips_replaced_towers() {
    let (_, _, world) = setup_world();
    let fallen_idx = world.damage.replaced_towers[0];
    let fallen_pos = world.towers[fallen_idx].pos;
    assert!(!world.in_tower_footprint(fallen_pos), "fallen tower footprint must be ignored");
    assert!(!world.in_tower_footprint(world.damage.leaning.pos), "leaning tower footprint must be ignored");
    let standing = world
        .towers
        .iter()
        .enumerate()
        .find(|(i, _)| !world.damage.tower_is_replaced(*i))
        .expect("standing tower")
        .1;
    assert!(world.in_tower_footprint(standing.pos), "standing tower footprint must still block");
}

#[test]
fn damage_props_in_chunk_union_has_no_duplicates_and_uses_height() {
    let (_, dcfg, world) = setup_world();
    let slope = |x: f32, z: f32| 10.0 + 0.1 * x - 0.05 * z;
    let props = all_props(&world, &dcfg, &slope);
    let expected = plan_damage_props(&world, &dcfg);
    assert_eq!(props.len(), expected.len(), "union over chunks must match plan count");
    for exp in &expected {
        let hits = props
            .iter()
            .filter(|p| p.model == exp.model && (p.position.x - exp.pos.x).abs() < 1e-4 && (p.position.z - exp.pos.y).abs() < 1e-4)
            .count();
        assert_eq!(hits, 1, "{} must appear once across all chunks", exp.model);
    }
    for p in &props {
        assert!((p.position.y - slope(p.position.x, p.position.z)).abs() < 1e-4);
        if p.model != "burnt_floor_overlay" {
            assert_eq!(p.scale, 1.0);
        }
    }
}

#[test]
fn damage_config_parses_and_rejects_reversed_range() {
    let json = std::fs::read_to_string(paths::repo_path("data/metro.json")).expect("read metro.json");
    assert!(load_damage(&json).is_ok());

    let custom = r#"{ "damage": { "fallenLengthM": [60, 80], "fallenWidthM": 24, "burntCarCount": [3, 5] } }"#;
    let parsed = load_damage(custom).expect("valid damage json");
    assert_eq!(parsed.fallen_length_m, [60.0, 80.0]);

    let bad_len = r#"{ "damage": { "fallenLengthM": [80, 60] } }"#;
    assert!(load_damage(bad_len).is_err(), "min > max fallenLengthM must fail");

    let bad_cars = r#"{ "damage": { "burntCarCount": [5, 3] } }"#;
    assert!(load_damage(bad_cars).is_err(), "min > max burntCarCount must fail");
}
