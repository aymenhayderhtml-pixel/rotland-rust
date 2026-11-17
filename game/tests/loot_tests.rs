use glam::{IVec2, Vec3};

use game::items::{Inventory, ItemRegistry};
use game::loot::*;
use world::BiomeId;

#[test]
fn golden_crate_loot_matches_godot_dump() {
    let path = paths::repo_path("golden/world.json");
    let text = std::fs::read_to_string(&path).expect("read golden/world.json");
    let golden: serde_json::Value = serde_json::from_str(&text).expect("valid golden json");

    let tables = LootTableRegistry::from_repo().expect("LootTableRegistry loads");

    let worlds = golden["worlds"].as_array().expect("worlds array");
    assert!(!worlds.is_empty(), "golden file must have worlds");

    let mut crates_compared = 0;
    let mut items_compared = 0;

    for world_val in worlds {
        let seed = world_val["seed"].as_i64().expect("seed") as i32;
        let chunks = world_val["chunks"].as_array().expect("chunks array");

        for chunk_val in chunks {
            let crates = chunk_val["crates"].as_array().expect("crates array");
            for crate_val in crates {
                let Some(loot_val) = crate_val.get("loot").and_then(|v| v.as_array()) else {
                    continue;
                };

                let crate_id = crate_val["crateId"].as_str().expect("crateId");
                let biome_num = crate_val["biome"].as_i64().expect("biome") as i32;
                let biome = match biome_num {
                    0 => BiomeId::Plains,
                    1 => BiomeId::Forest,
                    other => panic!("unexpected biome id: {other}"),
                };

                let table = tables.for_biome(biome);
                let rolled = LootRoller::roll(table, seed, crate_id, 1.0);

                assert_eq!(
                    rolled.len(),
                    loot_val.len(),
                    "seed {seed} crate {crate_id}: item count mismatch"
                );

                for (i, (r, g)) in rolled.iter().zip(loot_val).enumerate() {
                    let g_id = g["id"].as_str().expect("item id");
                    let g_count = g["count"].as_u64().expect("count") as u32;

                    assert_eq!(
                        r.item_id, g_id,
                        "seed {seed} crate {crate_id} item {i}: id mismatch"
                    );
                    assert_eq!(
                        r.count, g_count,
                        "seed {seed} crate {crate_id} item {i}: count mismatch"
                    );
                    items_compared += 1;
                }

                crates_compared += 1;
            }
        }
    }

    println!(
        "compared {crates_compared} crates, {items_compared} items against Godot golden dump"
    );
    assert!(
        crates_compared > 0,
        "golden test must compare crates (got {crates_compared})"
    );
    assert!(
        items_compared > 0,
        "golden test must compare items (got {items_compared})"
    );
}

#[test]
fn loot_tables_load_and_validate_against_items() {
    let tables = LootTableRegistry::from_repo().expect("LootTableRegistry loads");
    let items = ItemRegistry::from_repo().expect("ItemRegistry loads");

    tables.validate_items(&items).expect("all loot items valid");

    assert!(tables.contains("forest"));
    assert!(tables.contains("plains"));
    assert!(tables.contains("generic"));

    let forest = tables.for_biome(BiomeId::Forest);
    assert_eq!(forest.rolls, [1, 3]);
    assert!(forest.entries.len() > 10);
    assert!(forest.cash.is_some());

    let plains = tables.for_biome(BiomeId::Plains);
    assert_eq!(plains.rolls, [1, 2]);
}

#[test]
fn crate_seed_is_deterministic_and_matches_dotnet() {
    assert_eq!(crate_seed(7, "0_0_0"), -405301394);
    assert_eq!(crate_seed(7, "0_0_1"), 1588156498);
    assert_eq!(crate_seed(42, "crate_0"), -1527661607);
    assert_eq!(crate_seed(42, "crate_test_42"), -923467943);
}

#[test]
fn seed_42_deterministic_roll_matches_traced_numbers() {
    let tables = LootTableRegistry::from_repo().expect("tables load");
    let forest = tables.get("forest").expect("forest table");

    let rolled = LootRoller::roll(forest, 42, "crate_test_42", 1.0);
    let expected = [
        ("canned_beans", 2),
        ("wood", 2),
        ("cloth", 3),
    ];

    assert_eq!(rolled.len(), expected.len());
    for (r, (exp_id, exp_count)) in rolled.iter().zip(expected.iter()) {
        assert_eq!(&r.item_id, exp_id);
        assert_eq!(r.count, *exp_count);
    }

    let cash = LootRoller::roll_cash(forest, 42, "crate_test_42");
    assert_eq!(cash, 0);
}

#[test]
fn difficulty_loot_multiplier_and_scaling() {
    let diff = DifficultyConfigData::from_repo().expect("difficulty.json loads");
    assert_eq!(diff.default_preset, "normal");
    assert_eq!(diff.loot_multiplier("normal"), 1.0);
    assert_eq!(diff.loot_multiplier("relaxed"), 1.5);
    assert_eq!(diff.loot_multiplier("hard"), 0.75);

    // ScaleRolls tests (away from zero rounding)
    assert_eq!(scale_rolls(1, 1.0), 1);
    assert_eq!(scale_rolls(3, 0.5), 2); // 1.5 -> 2
    assert_eq!(scale_rolls(1, 0.5), 1); // 0.5 -> 1 (min 1)
    assert_eq!(scale_rolls(2, 1.5), 3); // 3.0 -> 3
    assert_eq!(scale_rolls(3, 1.5), 5); // 4.5 -> 5
    assert_eq!(scale_rolls(2, 0.75), 2); // 1.5 -> 2
    assert_eq!(scale_rolls(0, 1.5), 0);
}

#[test]
fn loot_state_tracks_crates() {
    let mut state = LootState::new();
    assert!(!state.is_looted("0_0_0"));

    state.mark_looted("0_0_0");
    assert!(state.is_looted("0_0_0"));
    assert!(state.is_chunk_looted(IVec2::new(0, 0), "0_0_0"));
    assert!(!state.is_looted("0_0_1"));

    state.mark_chunk_looted(IVec2::new(1, -2), "poi_crate_1");
    assert!(state.is_looted("poi_crate_1"));
    assert!(state.is_chunk_looted(IVec2::new(1, -2), "poi_crate_1"));
    assert_eq!(state.len(), 2);
}

#[test]
fn world_items_add_remove_nearest() {
    let mut world_items = WorldItems::new();
    assert!(world_items.is_empty());

    world_items.add("canned_beans", 2, Vec3::new(1.0, 0.0, 0.0));
    world_items.add("water_bottle", 1, Vec3::new(3.0, 0.0, 0.0));
    world_items.add("bandage", 3, Vec3::new(0.0, 0.0, 2.0));
    assert_eq!(world_items.len(), 3);

    // Within radius 1.5 of origin: finds beans at distance 1.0
    let nearest = world_items.nearest_within(Vec3::ZERO, 1.5);
    assert!(nearest.is_some());
    let (idx, item) = nearest.unwrap();
    assert_eq!(idx, 0);
    assert_eq!(item.item_id, "canned_beans");
    assert_eq!(item.count, 2);

    // Within radius 0.5: none
    assert!(world_items.nearest_within(Vec3::ZERO, 0.5).is_none());

    // Remove beans: now nearest within 2.5 of origin is bandage (distance 2.0 < bottle at 3.0)
    let removed = world_items.remove(0);
    assert_eq!(removed.unwrap().item_id, "canned_beans");
    assert_eq!(world_items.len(), 2);

    let next_nearest = world_items.nearest_within(Vec3::ZERO, 2.5);
    assert!(next_nearest.is_some());
    let (_, item2) = next_nearest.unwrap();
    assert_eq!(item2.item_id, "bandage");
}

#[test]
fn inventory_add_and_world_item_interaction() {
    let items = ItemRegistry::from_repo().expect("items registry");
    let mut inv = Inventory::with_capacity(2, 40.0);

    // Add 5 beans
    let left = inv.add_with_registry("canned_beans", 5, &items);
    assert_eq!(left, 0);
    assert_eq!(inv.count("canned_beans"), 5);

    // Restrict inventory weight to simulate full backpack
    inv.set_max_weight(inv.total_weight());
    let left2 = inv.add_with_registry("canned_beans", 3, &items);
    assert_eq!(left2, 3); // nothing fits
}
