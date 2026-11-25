use std::f32::consts::TAU;

use glam::IVec2;
use serde_json::Value;
use world::biome::registry::BiomeRegistry;
use world::biome::BiomeId;
use world::loot::{loot_rolls_bonus_for_tier, roll_crate_item_count};
use world::scatter::{PropInstance, ScatterPlacer};
use world::WorldGenerator;

fn test_world(seed: i32) -> (WorldGenerator, BiomeRegistry) {
    let data_dir = paths::repo_root().join("data");
    let world_gen = WorldGenerator::from_data_dir(seed, &data_dir).expect("load WorldGenerator");
    let biomes = BiomeRegistry::load().expect("load BiomeRegistry");
    (world_gen, biomes)
}

fn chunk_at(x: f32, z: f32) -> IVec2 {
    IVec2::new(
        (x / WorldGenerator::CHUNK_SIZE).floor() as i32,
        (z / WorldGenerator::CHUNK_SIZE).floor() as i32,
    )
}

/// Scans a ring at `radius` metres around the camp (or origin) for a pure chunk of `target` biome
/// whose props match `predicate`.
fn find_outer_biome_chunk(
    world_gen: &WorldGenerator,
    biomes: &BiomeRegistry,
    target: BiomeId,
    radius: f32,
    predicate: impl Fn(&[PropInstance]) -> bool,
) -> Option<(IVec2, Vec<PropInstance>)> {
    let center = world_gen
        .settlements()
        .first()
        .map_or((0.0f32, 0.0f32), |s| (s.center.x, s.center.z));

    let steps = 144;
    for i in 0..steps {
        let angle = (i as f32 / steps as f32) * TAU;
        let x = center.0 + angle.cos() * radius;
        let z = center.1 + angle.sin() * radius;
        let coord = chunk_at(x, z);
        let origin_x = coord.x as f32 * WorldGenerator::CHUNK_SIZE;
        let origin_z = coord.y as f32 * WorldGenerator::CHUNK_SIZE;
        let corners = [
            (origin_x, origin_z),
            (origin_x + WorldGenerator::CHUNK_SIZE, origin_z),
            (origin_x, origin_z + WorldGenerator::CHUNK_SIZE),
            (
                origin_x + WorldGenerator::CHUNK_SIZE,
                origin_z + WorldGenerator::CHUNK_SIZE,
            ),
        ];
        let pure_outer = corners.iter().all(|&(cx, cz)| {
            let s = world_gen.biome_sample(cx, cz);
            s.outer > 0.0 && s.primary == target
        });
        if !pure_outer {
            continue;
        }
        let props =
            ScatterPlacer::place_with_registry(world_gen, world_gen.scatter_config(), biomes, coord);
        if predicate(&props) {
            return Some((coord, props));
        }
    }
    None
}

#[test]
fn desert_chunk_at_4000m_has_cacti_and_no_pine_tall() {
    let (world_gen, biomes) = test_world(42);
    let (coord, props) = find_outer_biome_chunk(
        &world_gen,
        &biomes,
        BiomeId::Desert,
        4000.0,
        |p| p.iter().any(|prop| prop.model.starts_with("cactus_")),
    )
    .expect("expected a desert chunk at 4000 m with cacti");

    assert!(
        props.iter().any(|p| p.model.starts_with("cactus_")),
        "desert chunk {coord} should contain cacti"
    );
    assert!(
        !props.iter().any(|p| p.model == "pine_tall"),
        "desert chunk {coord} must not contain pine_tall"
    );
    for prop in props.iter().filter(|p| p.model.starts_with("cactus_")) {
        assert!(!prop.is_tree(), "cactus {} must not be treated as a felled tree", prop.model);
    }
}

#[test]
fn snow_chunk_has_pine_snow() {
    let (world_gen, biomes) = test_world(42);
    let (coord, props) = find_outer_biome_chunk(
        &world_gen,
        &biomes,
        BiomeId::Snow,
        4000.0,
        |p| p.iter().any(|prop| prop.model == "pine_snow"),
    )
    .expect("expected a snow chunk at 4000 m with pine_snow");

    let pine = props
        .iter()
        .find(|p| p.model == "pine_snow")
        .expect("pine_snow must exist");
    assert!(pine.is_tree(), "pine_snow in chunk {coord} must be a choppable tree");
}

#[test]
fn core_chunk_props_match_legacy_path() {
    let (world_gen, biomes) = test_world(42);
    let camp_coord = world_gen
        .settlements()
        .first()
        .map_or(IVec2::ZERO, |s| chunk_at(s.center.x, s.center.z));

    let offsets = [
        IVec2::new(0, 0),
        IVec2::new(1, 0),
        IVec2::new(-1, 1),
        IVec2::new(2, 2),
    ];
    let mut checked = 0usize;
    for off in offsets {
        let coord = camp_coord + off;
        if !ScatterPlacer::is_core_chunk(&world_gen, coord) {
            continue;
        }
        let legacy = ScatterPlacer::place(&world_gen, world_gen.scatter_config(), coord);
        let updated = ScatterPlacer::place_with_registry(
            &world_gen,
            world_gen.scatter_config(),
            &biomes,
            coord,
        );
        assert_eq!(
            legacy, updated,
            "core chunk {coord} props diverged from legacy path"
        );
        checked += 1;
    }
    assert!(checked > 0, "must compare at least one core chunk");
}

#[test]
fn chunk_crates_from_matches_chunk_crates() {
    let (world_gen, _) = test_world(42);
    let coords = [
        IVec2::new(0, 0),
        IVec2::new(2, 1),
        IVec2::new(-3, 4),
        IVec2::new(5, -2),
        IVec2::new(60, 0),
        IVec2::new(0, 60),
    ];
    for coord in coords {
        let (pois, props) = world_gen.chunk_contents(coord);
        let direct = world_gen.chunk_crates(coord);
        let reused = world_gen.chunk_crates_from(coord, &pois, &props);
        assert_eq!(
            direct, reused,
            "chunk_crates_from mismatch on chunk {coord}"
        );
    }
}

#[test]
fn tier_3_crates_roll_more_items_than_tier_1_with_same_seed() {
    let (world_gen, _) = test_world(42);
    let layout_text = std::fs::read_to_string(paths::repo_root().join("data/world_layout.json"))
        .expect("read data/world_layout.json");
    let layout_json: Value = serde_json::from_str(&layout_text).expect("parse world_layout.json");
    let tier_bonuses: Vec<i32> = layout_json["tiers"]
        .as_array()
        .expect("tiers array in world_layout.json")
        .iter()
        .map(|t| {
            t["loot_rolls_bonus"]
                .as_i64()
                .expect("loot_rolls_bonus integer") as i32
        })
        .collect();

    let center = world_gen
        .settlements()
        .first()
        .map_or((0.0f32, 0.0f32), |s| (s.center.x, s.center.z));

    let mut tier1_spot = None;
    let mut tier3_spot = None;
    for step in 10..200 {
        let dist = step as f32 * WorldGenerator::CHUNK_SIZE;
        let coord = chunk_at(center.0 + dist, center.1);
        for spot in world_gen.chunk_crates(coord) {
            if spot.tier == 1 && tier1_spot.is_none() {
                tier1_spot = Some(spot);
            } else if spot.tier == 3 && tier3_spot.is_none() {
                tier3_spot = Some(spot);
            }
        }
        if tier1_spot.is_some() && tier3_spot.is_some() {
            break;
        }
    }

    let spot1 = tier1_spot.expect("expected a tier-1 crate along +X");
    let spot3 = tier3_spot.expect("expected a tier-3 crate along +X");

    let bonus1 = loot_rolls_bonus_for_tier(spot1.tier, &tier_bonuses);
    let bonus3 = loot_rolls_bonus_for_tier(spot3.tier, &tier_bonuses);
    assert!(
        bonus3 > bonus1,
        "tier 3 bonus ({bonus3}) must exceed tier 1 bonus ({bonus1})"
    );

    let base_rolls = [1, 3];
    for seed in [7, 42, 1337, 9999] {
        let count1 = roll_crate_item_count(base_rolls, bonus1, seed);
        let count3 = roll_crate_item_count(base_rolls, bonus3, seed);
        assert!(
            count3 > count1,
            "seed {seed}: tier-3 crate ({count3}) must roll more items than tier-1 ({count1})"
        );
    }
}
