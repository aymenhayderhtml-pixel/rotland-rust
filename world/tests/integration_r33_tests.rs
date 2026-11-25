use std::collections::HashSet;

use glam::{IVec2, Vec2};
use world::{
    CityConfigData, LakeConfigData, PoiConfig, ScatterConfig, SettlementConfig, WorldGenerator,
    city::CityGroundData, towns::BOARDWALK_MODEL,
};

fn make_full_generator(seed: i32) -> WorldGenerator {
    WorldGenerator::from_data_dir(seed, paths::repo_path("data"))
        .expect("WorldGenerator::from_data_dir should succeed")
}

fn make_core_only_generator(seed: i32) -> WorldGenerator {
    let dir = paths::repo_path("data");
    let settlements = SettlementConfig::load_from_file(dir.join("settlements.json")).unwrap();
    let scatter = ScatterConfig::load_from_file(dir.join("scatter.json")).unwrap();
    let poi = PoiConfig::load_from_file(dir.join("poi.json")).unwrap();
    let mut world_gen = WorldGenerator::with_configs(seed, settlements, scatter, poi);
    let city = CityConfigData::load_from_file(dir.join("city.json")).unwrap();
    let city_ground = CityGroundData::load_from_file(dir.join("city_ground.json")).unwrap();
    world_gen.attach_city_with_ground(&city, Some(&city_ground));
    let lake = LakeConfigData::load_from_file(dir.join("lake.json")).unwrap();
    world_gen.attach_lake(&lake);
    world_gen
}

fn chunk_of(p: Vec2) -> IVec2 {
    IVec2::new(
        (p.x / WorldGenerator::CHUNK_SIZE).floor() as i32,
        (p.y / WorldGenerator::CHUNK_SIZE).floor() as i32,
    )
}

#[test]
fn core_heights_unchanged_with_r33() {
    let with_r33 = make_full_generator(42);
    let without_r33 = make_core_only_generator(42);
    let camp = with_r33.settlements()[0].center;

    for i in 0..200 {
        let angle = (i as f32) * 0.618_034 * std::f32::consts::TAU;
        let radius = 20.0 + ((i as f32) / 200.0) * 1980.0;
        let x = camp.x + angle.cos() * radius;
        let z = camp.z + angle.sin() * radius;
        assert_eq!(
            with_r33.height(x, z),
            without_r33.height(x, z),
            "core height changed at ({x}, {z})"
        );
    }
}

#[test]
fn town_centres_are_flat_within_40m() {
    let world_gen = make_full_generator(42);
    assert!(!world_gen.towns().is_empty(), "expected placed towns");

    for town in world_gen.towns() {
        let mut min_h = f32::MAX;
        let mut max_h = f32::MIN;
        for dx in [-40.0_f32, -20.0, 0.0, 20.0, 40.0] {
            for dz in [-40.0_f32, -20.0, 0.0, 20.0, 40.0] {
                if dx * dx + dz * dz <= 40.0 * 40.0 {
                    let h = world_gen.height(town.center.x + dx, town.center.y + dz);
                    min_h = min_h.min(h);
                    max_h = max_h.max(h);
                }
            }
        }
        let relief = max_h - min_h;
        assert!(
            relief < 1.0,
            "town {} centre relief {relief} m >= 1.0 m",
            town.name
        );
    }
}

#[test]
fn river_line_is_lower_than_bank() {
    let world_gen = make_full_generator(42);
    let rivers = world_gen.rivers().rivers();
    assert!(!rivers.is_empty(), "expected rivers");

    let mut checked = 0usize;
    for river in rivers {
        if river.points.len() < 3 {
            continue;
        }
        let mid = river.points.len() / 2;
        let p = river.points[mid];
        let dir = (river.points[mid + 1] - river.points[mid - 1]).normalize_or_zero();
        let perp = Vec2::new(-dir.y, dir.x);
        let bank_offset = river.widths[mid] * 0.5 + 8.0;
        let h_river = world_gen.height(p.x, p.y);
        let h_bank_a = world_gen.height(p.x + perp.x * bank_offset, p.y + perp.y * bank_offset);
        let h_bank_b = world_gen.height(p.x - perp.x * bank_offset, p.y - perp.y * bank_offset);
        let bank_h = h_bank_a.max(h_bank_b);
        assert!(
            h_river < bank_h,
            "expected river height {h_river} < bank height {bank_h} at {p:?}"
        );
        checked += 1;
    }
    assert!(checked > 0, "expected at least one river checked");
}

#[test]
fn bridge_prop_exists_at_each_crossing() {
    let world_gen = make_full_generator(42);
    let crossings = world_gen.rivers().crossings(world_gen.roads());
    assert!(!crossings.is_empty(), "expected road-river crossings");

    for (pos, _, _) in crossings {
        let coord = chunk_of(pos);
        let (_, props) = world_gen.chunk_contents(coord);
        let found = props.iter().any(|prop| {
            (prop.model == "bridge_wood" || prop.model == "bridge_concrete")
                && Vec2::new(prop.position.x, prop.position.z).distance(pos) < 1.0
        });
        assert!(found, "missing bridge prop at crossing {pos:?} in chunk {coord:?}");
    }
}

#[test]
fn no_tree_within_3m_of_road_points() {
    let world_gen = make_full_generator(42);
    let mut checked_chunks = HashSet::new();
    let mut road_points = Vec::new();

    for road in world_gen.roads().roads() {
        for &pt in road.points.iter().step_by(4) {
            road_points.push(pt);
            checked_chunks.insert(chunk_of(pt));
        }
    }
    assert!(!road_points.is_empty(), "expected road points");

    for coord in checked_chunks {
        let (_, props) = world_gen.chunk_contents(coord);
        for prop in props.iter().filter(|p| p.is_tree()) {
            let pp = Vec2::new(prop.position.x, prop.position.z);
            for &rp in &road_points {
                assert!(
                    pp.distance(rp) >= 3.0,
                    "tree {} at {pp:?} is within 3 m of road point {rp:?}",
                    prop.model
                );
            }
        }
    }
}

#[test]
fn indoors_true_at_lot_centre_false_in_street() {
    let world_gen = make_full_generator(42);
    let town = world_gen.towns().first().expect("expected at least one town");
    let lot = town
        .lots
        .iter()
        .find(|l| l.building != BOARDWALK_MODEL)
        .expect("expected a building lot");

    assert!(
        world_gen.indoors(lot.pos.x, lot.pos.y),
        "expected indoors=true at lot centre {:?}",
        lot.pos
    );
    assert!(
        !world_gen.indoors(town.center.x, town.center.y),
        "expected indoors=false in street centre {:?}",
        town.center
    );
}
