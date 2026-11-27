//! R42: loot tables and POI entries for the four big New Halden places.
//!
//! Parses `data/poi.json` with `serde_json` directly, so this test does not depend on the optional
//! `zombieModels` field (added by R42-07) being merged into `world::PoiBuildingDef` yet.

use game::items::{ItemCategory, ItemRegistry};
use game::loot::{LootRoller, LootTableRegistry};
use game::weapons::WeaponRegistry;

/// Crates rolled per table in the sampling tests.
const CRATES: usize = 2000;
/// World seed used for the crate rolls (the loot roller seeds per crate id from it).
const WORLD_SEED: i32 = 42;

const PLACE_TABLES: [&str; 3] = ["metro_hospital", "metro_police", "metro_fire"];

fn loot_tables() -> LootTableRegistry {
    LootTableRegistry::from_repo().expect("data/loot_tables.json must load")
}

fn items() -> ItemRegistry {
    ItemRegistry::from_repo().expect("data/items.json must load")
}

fn weapons() -> WeaponRegistry {
    let path = paths::repo_path("data/weapons.json");
    WeaponRegistry::load(&path).expect("data/weapons.json must load")
}

fn poi_json() -> serde_json::Value {
    let path = paths::repo_path("data/poi.json");
    let text = std::fs::read_to_string(path).expect("data/poi.json must read");
    serde_json::from_str(&text).expect("data/poi.json must be valid JSON")
}

fn poi_building(id: &str) -> serde_json::Value {
    let json = poi_json();
    let buildings = json["buildings"]
        .as_array()
        .expect("data/poi.json buildings must be an array");
    buildings
        .iter()
        .find(|b| b["id"].as_str() == Some(id))
        .unwrap_or_else(|| panic!("data/poi.json must have a '{id}' building"))
        .clone()
}

/// Every item id one table can drop.
fn table_ids(tables: &LootTableRegistry, name: &str) -> Vec<String> {
    tables
        .get(name)
        .unwrap_or_else(|| panic!("loot table '{name}' must exist"))
        .entries
        .iter()
        .map(|e| e.id.clone())
        .collect()
}

fn json_arr<T>(
    value: &serde_json::Value,
    field: &str,
    id: &str,
    to: impl Fn(&serde_json::Value) -> T,
) -> Vec<T> {
    value[field]
        .as_array()
        .unwrap_or_else(|| panic!("'{id}' must have a '{field}' array"))
        .iter()
        .map(to)
        .collect()
}

fn lot_of(entry: &serde_json::Value, id: &str) -> [f32; 2] {
    let lot = json_arr(entry, "lot", id, |v| v.as_f64().expect("lot numbers") as f32);
    assert_eq!(lot.len(), 2, "'{id}' lot must be [width, depth]");
    [lot[0], lot[1]]
}

#[test]
fn the_three_place_tables_exist_with_their_rolls() {
    let tables = loot_tables();

    for (name, rolls) in [
        ("metro_hospital", [3, 5]),
        ("metro_police", [3, 5]),
        ("metro_fire", [3, 4]),
    ] {
        let table = tables
            .get(name)
            .unwrap_or_else(|| panic!("loot table '{name}' must exist"));
        assert_eq!(table.rolls, rolls, "loot table '{name}' rolls");
        assert!(!table.entries.is_empty(), "loot table '{name}' needs entries");
    }
}

#[test]
fn every_place_table_item_exists_in_items_json() {
    let (tables, items) = (loot_tables(), items());

    for name in PLACE_TABLES {
        for id in table_ids(&tables, name) {
            assert!(
                items.contains(&id),
                "loot table '{name}' uses '{id}', which is not in data/items.json"
            );
        }
    }

    // The whole file must stay valid too (weights, counts, cash ranges, item ids).
    tables.validate_items(&items).expect("every loot item must exist");
}

#[test]
fn weapons_drop_with_their_own_ammo_in_the_same_table() {
    let (tables, items, weapons) = (loot_tables(), items(), weapons());

    for name in ["metro_police", "metro_fire"] {
        let ids = table_ids(&tables, name);
        for id in &ids {
            let Some(def) = items.get(id) else { continue };
            let Some(weapon) = weapons.get(&def.weapon_id) else { continue };
            if weapon.ammo_item.is_empty() {
                continue; // melee weapons (fire_axe, axe, crowbar) take no ammo
            }
            assert!(
                ids.contains(&weapon.ammo_item),
                "loot table '{name}' drops '{id}' but not its ammo '{}'",
                weapon.ammo_item
            );
        }
    }

    // The guns the police table is built around, with the real ammo ids from data/weapons.json.
    let police = table_ids(&tables, "metro_police");
    let drops = |id: &str| police.iter().any(|e| e.as_str() == id);
    for (gun, ammo) in [
        ("pistol", "pistol_ammo"),
        ("revolver", "revolver_ammo"),
        ("shotgun", "shells_12g"),
        ("smg", "pistol_ammo"), // data: weapons.json smg.ammoItem
        ("army_rifle", "ammo_556"),
    ] {
        assert!(drops(gun), "metro_police must drop '{gun}'");
        assert!(drops(ammo), "metro_police must drop '{ammo}' for '{gun}'");
    }
}

/// Average number of items of one category that one crate of `table_name` gives.
fn category_items_per_crate(
    tables: &LootTableRegistry,
    items: &ItemRegistry,
    table_name: &str,
    category: ItemCategory,
) -> f32 {
    let table = tables
        .get(table_name)
        .unwrap_or_else(|| panic!("loot table '{table_name}' must exist"));
    let mut total = 0u32;
    for i in 0..CRATES {
        for stack in LootRoller::roll(table, WORLD_SEED, &format!("crate_{i}"), 1.0) {
            if items.get(&stack.item_id).is_some_and(|def| def.category == category) {
                total += stack.count;
            }
        }
    }
    total as f32 / CRATES as f32
}

#[test]
fn hospital_gives_more_medical_than_the_edge() {
    let (tables, items) = (loot_tables(), items());

    let hospital =
        category_items_per_crate(&tables, &items, "metro_hospital", ItemCategory::Medical);
    let edge = category_items_per_crate(&tables, &items, "metro_edge", ItemCategory::Medical);

    assert!(
        hospital > edge,
        "metro_hospital medical per crate {hospital:.3} must beat metro_edge {edge:.3}"
    );
    assert!(
        hospital >= 2.0, // "top loot": several medical items in a typical crate
        "metro_hospital should be rich in medicine, got {hospital:.3} per crate"
    );
}

#[test]
fn police_gives_a_weapon_or_ammo_in_most_crates() {
    let (tables, items) = (loot_tables(), items());
    let table = tables.get("metro_police").expect("metro_police table");

    let mut with_gun_or_ammo = 0usize;
    for i in 0..CRATES {
        let rolled = LootRoller::roll(table, WORLD_SEED, &format!("crate_{i}"), 1.0);
        let hit = rolled.iter().any(|stack| {
            items.get(&stack.item_id).is_some_and(|def| {
                def.category == ItemCategory::Weapon || def.category == ItemCategory::Ammo
            })
        });
        if hit {
            with_gun_or_ammo += 1;
        }
    }

    let share = with_gun_or_ammo as f32 / CRATES as f32;
    assert!(
        share >= 0.60, // tasks/R42-08: police crates hold a weapon or ammo in at least 60%
        "metro_police gave a weapon or ammo in {share:.3} of {CRATES} crates, want >= 0.60"
    );
}

/// One expected `data/poi.json` place entry (the tasks/R42-08 table).
struct PlacePoi {
    id: &'static str,
    lot: [f32; 2],
    loot_table: &'static str,
    zombies: [i32; 2],
    zombie_models: &'static [&'static str],
}

const PLACE_POIS: [PlacePoi; 4] = [
    PlacePoi { id: "metro_hospital", lot: [40.0, 32.0], loot_table: "metro_hospital", zombies: [10, 14],
               zombie_models: &["zombie_medic", "zombie_03", "zombie_01"] },
    PlacePoi { id: "metro_police", lot: [38.0, 30.0], loot_table: "metro_police", zombies: [8, 12],
               zombie_models: &["zombie_police", "zombie_police", "zombie_02"] },
    PlacePoi { id: "metro_fire_station", lot: [36.0, 28.0], loot_table: "metro_fire", zombies: [6, 10],
               zombie_models: &["zombie_firefighter", "zombie_01"] },
    PlacePoi { id: "metro_army_line", lot: [60.0, 30.0], loot_table: "metro_army", zombies: [12, 16],
               zombie_models: &["zombie_soldier", "zombie_soldier", "zombie_soldier", "zombie_02"] },
];

#[test]
fn the_four_place_poi_entries_parse_with_their_zombie_looks() {
    let tables = loot_tables();
    for expected in PLACE_POIS {
        let PlacePoi { id, lot, loot_table, zombies, zombie_models } = expected;
        let entry = poi_building(id);

        assert_eq!(lot_of(&entry, id), lot, "'{id}' lot");
        assert_eq!(entry["lootTable"].as_str(), Some(loot_table), "'{id}' lootTable");
        assert!(tables.contains(loot_table), "'{id}' loot table must exist");

        let got_zombies = json_arr(&entry, "zombies", id, |v| v.as_i64().expect("zombie counts"));
        assert_eq!(got_zombies, zombies.map(i64::from), "'{id}' zombies");

        assert!(
            entry["biome"].as_object().is_some_and(|m| m.is_empty()),
            "'{id}' biome must be empty (placed by the city only)"
        );
        assert_eq!(entry["chancePerChunk"].as_f64(), Some(0.0), "'{id}' chancePerChunk");
        assert!(
            entry["minFlatness"].as_f64().is_some()
                && entry["minDistanceFromSpawn"].as_f64().is_some(),
            "'{id}' must keep the minFlatness / minDistanceFromSpawn fields"
        );

        let got_models = json_arr(&entry, "zombieModels", id, |v| {
            v.as_str().expect("zombie model ids").to_string()
        });
        let want_models: Vec<String> = zombie_models.iter().map(|m| m.to_string()).collect();
        assert_eq!(got_models, want_models, "'{id}' zombieModels");
    }

    // The places are the dangerous spots of the city: more zombies than the town police/hospital.
    for (metro, town) in [("metro_police", "police"), ("metro_hospital", "hospital")] {
        let metro_max = poi_building(metro)["zombies"][1].as_i64().expect("metro zombies");
        let town_max = poi_building(town)["zombies"][1].as_i64().expect("town zombies");
        assert!(metro_max > town_max, "{metro} ({metro_max}) must out-danger {town} ({town_max})");
    }
}

#[test]
fn the_place_poi_entries_still_load_through_the_world_config() {
    // The extra `zombieModels` field must not break the real POI loader.
    let cfg = world::PoiConfig::load_from_file(paths::repo_path("data/poi.json"))
        .expect("data/poi.json must load through world::PoiConfig");
    for expected in PLACE_POIS {
        let def = cfg
            .get(expected.id)
            .unwrap_or_else(|| panic!("world::PoiConfig must know '{}'", expected.id));
        assert_eq!(def.chance_per_chunk, 0.0, "'{}' must never scatter", expected.id);
        assert_eq!(def.zombies, expected.zombies, "'{}' zombies", expected.id);
    }
}
