use glam::{IVec2, Vec2};
use paths::repo_path;

use crate::city::config::{CityConfig, District};
use crate::city::ground::CityGroundData;
use crate::generator::{GroundTriangle, WorldGenerator};

fn load_world_42() -> WorldGenerator {
    let data_dir = repo_path("data");
    WorldGenerator::from_data_dir(42, data_dir).expect("Failed to create WorldGenerator from data dir")
}

fn triangle_at(world_gen: &WorldGenerator, x: f32, z: f32) -> GroundTriangle {
    let coord = IVec2::new(
        (x / WorldGenerator::CHUNK_SIZE).floor() as i32,
        (z / WorldGenerator::CHUNK_SIZE).floor() as i32,
    );
    let chunk = world_gen.build_chunk_ground(coord);
    chunk
        .triangles
        .into_iter()
        .min_by(|a, b| {
            let ca = (a.positions[0] + a.positions[1] + a.positions[2]) / 3.0;
            let cb = (b.positions[0] + b.positions[1] + b.positions[2]) / 3.0;
            let da = (coord.x as f32 * 64.0 + ca.x - x).hypot(coord.y as f32 * 64.0 + ca.z - z);
            let db = (coord.x as f32 * 64.0 + cb.x - x).hypot(coord.y as f32 * 64.0 + cb.z - z);
            da.partial_cmp(&db).unwrap()
        })
        .unwrap()
}

#[test]
fn city_ground_config_load() {
    let path = repo_path("data/city_ground.json");
    let cfg = CityGroundData::load_from_file(&path).expect("Failed to load data/city_ground.json");
    assert_eq!(cfg.edge_blend, 48.0);
    assert_eq!(cfg.road_width, 8.0);
    assert_eq!(cfg.road_blend, 6.0);
    assert_eq!(cfg.road_smooth_distance, 8.0);
    assert_eq!(cfg.sidewalk_width, 1.5);
    assert_eq!(cfg.asphalt, [50, 50, 52]);
    assert_eq!(cfg.sidewalk, [140, 140, 140]);

    let json_str = serde_json::to_string(&cfg).unwrap();
    let cfg2 = CityGroundData::from_json(&json_str).unwrap();
    assert_eq!(cfg, cfg2);
}

#[test]
fn city_street_center_is_asphalt() {
    let world_gen = load_world_42();
    let city = world_gen.city().expect("world should have city");
    let street = city.streets.first().expect("city should have streets");
    let center = (street.a + street.b) * 0.5;

    let tri = triangle_at(&world_gen, center.x, center.y);
    let asphalt = world_gen.city_ground().expect("city ground").asphalt;

    // After brightness jitter (+-6%) and hue shift (+-8%), color stays very close to asphalt
    assert!(
        (tri.color.r - asphalt.r).abs() < 0.05,
        "expected asphalt r ~ {}, got {}",
        asphalt.r,
        tri.color.r
    );
    assert!(
        (tri.color.g - asphalt.g).abs() < 0.05,
        "expected asphalt g ~ {}, got {}",
        asphalt.g,
        tri.color.g
    );
    assert!(
        (tri.color.b - asphalt.b).abs() < 0.05,
        "expected asphalt b ~ {}, got {}",
        asphalt.b,
        tri.color.b
    );
}

#[test]
fn city_park_block_center_is_green() {
    let world_gen = load_world_42();
    let city = world_gen.city().expect("world should have city");
    let park_block = city
        .blocks
        .iter()
        .find(|b| b.present && b.district == District::Park)
        .expect("city should have at least one park block");

    let min_chunk_x = (city.origin.x / 64.0).round() as i32;
    let min_chunk_y = (city.origin.y / 64.0).round() as i32;
    let bx = park_block.coord.x - min_chunk_x;
    let by = park_block.coord.y - min_chunk_y;
    let center = city.origin + Vec2::new((bx as f32 + 0.5) * 64.0, (by as f32 + 0.5) * 64.0);

    let tri = triangle_at(&world_gen, center.x, center.y);
    assert!(
        tri.color.g > tri.color.r && tri.color.g > tri.color.b,
        "park center should be green: r={}, g={}, b={}",
        tri.color.r,
        tri.color.g,
        tri.color.b
    );
}

#[test]
fn city_height_range_within_max_slope() {
    let world_gen = load_world_42();
    let city = world_gen.city().expect("world should have city");
    let cfg_path = repo_path("data/city.json");
    let city_cfg = CityConfig::load_from_file(&cfg_path).unwrap();

    let mut min_h = f32::MAX;
    let mut max_h = f32::MIN;

    let steps = 20;
    for j in 0..=steps {
        for i in 0..=steps {
            let px = city.origin.x + (i as f32 / steps as f32) * city.size;
            let pz = city.origin.y + (j as f32 / steps as f32) * city.size;
            let h = world_gen.height(px, pz);
            min_h = min_h.min(h);
            max_h = max_h.max(h);
        }
    }

    let diff = max_h - min_h;
    assert!(
        diff <= city_cfg.max_slope + 0.5,
        "city height range {} exceeded max_slope {} + 0.5",
        diff,
        city_cfg.max_slope
    );
}

#[test]
fn city_outside_matches_no_city() {
    let gen_with = load_world_42();
    let city = gen_with.city().expect("world should have city");


    // Pick a point 300m outside the city box away from camp, road, and river
    let camp = Vec2::new(gen_with.settlements()[0].center.x, gen_with.settlements()[0].center.z);
    let city_center = city.origin + Vec2::splat(city.size * 0.5);
    let dir = (city_center - camp).normalize();
    let side = Vec2::new(-dir.y, dir.x);
    let p = city_center + side * (city.size * 0.5 + 300.0);

    // R31: the outer biome sectors point at Ashford only when a city is attached, so the
    // no-city world has a different biome layout out here. Compare with the same world's
    // unflattened height instead: the city must not change terrain 300 m outside its box.
    let h1 = gen_with.height(p.x, p.y);
    let raw = gen_with.raw_height(p.x, p.y);
    assert_eq!(h1, raw, "city changed height 300m outside city: {h1} vs raw {raw}");
}

#[test]
fn city_road_midpoint_is_asphalt() {
    let world_gen = load_world_42();
    let city = world_gen.city().expect("world should have city");
    assert!(city.road.len() >= 2, "city road should have at least 2 points");

    let midpoint = (city.road[0] + city.road[1]) * 0.5;
    let tri = triangle_at(&world_gen, midpoint.x, midpoint.y);
    let asphalt = world_gen.city_ground().expect("city ground").asphalt;

    assert!(
        (tri.color.r - asphalt.r).abs() < 0.05,
        "expected road midpoint r ~ {}, got {}",
        asphalt.r,
        tri.color.r
    );
    assert!(
        (tri.color.g - asphalt.g).abs() < 0.05,
        "expected road midpoint g ~ {}, got {}",
        asphalt.g,
        tri.color.g
    );
    assert!(
        (tri.color.b - asphalt.b).abs() < 0.05,
        "expected road midpoint b ~ {}, got {}",
        asphalt.b,
        tri.color.b
    );
}
