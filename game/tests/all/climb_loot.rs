//! R43: loot tables and POI entries for climbable tower stop-off floors, stairwells, and radio.

use game::items::{ItemCategory, ItemRegistry};
use game::loot::{LootRoller, LootTableRegistry};
use game::weapons::WeaponRegistry;

/// Crates rolled per table in the sampling tests.
const CRATES: usize = 2000;
/// World seed used for the crate rolls.
const WORLD_SEED: i32 = 42;

const CLIMB_TABLES: [&str; 4] = [
    "metro_tower_office",
    "metro_tower_flat",
    "metro_tower_camp",
    "metro_radio",
];

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

fn table_ids(tables: &LootTableRegistry, name: &str) -> Vec<String> {
    tables
        .get(name)
        .unwrap_or_else(|| panic!("loot table '{name}' must exist"))
        .entries
        .iter()
        .map(|e| e.id.clone())
        .collect()
}

#[test]
fn climb_loot_tables_exist_with_valid_items_and_ammo() {
    let (tables, items, weapons) = (loot_tables(), items(), weapons());

    for name in CLIMB_TABLES {
        let ids = table_ids(&tables, name);
        assert!(!ids.is_empty(), "loot table '{name}' must not be empty");
        for id in &ids {
            assert!(
                items.contains(id),
                "loot table '{name}' uses '{id}', which is not in data/items.json"
            );
            let Some(def) = items.get(id) else { continue };
            let Some(weapon) = weapons.get(&def.weapon_id) else { continue };
            if weapon.ammo_item.is_empty() {
                continue;
            }
            assert!(
                ids.contains(&weapon.ammo_item),
                "loot table '{name}' drops '{id}' without its ammo '{}'",
                weapon.ammo_item
            );
        }
    }

    tables.validate_items(&items).expect("every loot item must exist");
}

fn weapon_crate_share(tables: &LootTableRegistry, items: &ItemRegistry, table_name: &str) -> f32 {
    let table = tables
        .get(table_name)
        .unwrap_or_else(|| panic!("loot table '{table_name}' must exist"));
    let mut hits = 0usize;
    for i in 0..CRATES {
        let rolled = LootRoller::roll(table, WORLD_SEED, &format!("climb_{i}"), 1.0);
        if rolled
            .iter()
            .any(|s| items.get(&s.item_id).is_some_and(|d| d.category == ItemCategory::Weapon))
        {
            hits += 1;
        }
    }
    hits as f32 / CRATES as f32
}

#[test]
fn metro_radio_and_camp_weapon_rates_match_expectations() {
    let (tables, items) = (loot_tables(), items());

    let radio_share = weapon_crate_share(&tables, &items, "metro_radio");
    assert!(
        radio_share >= 0.70,
        "metro_radio gave a weapon in {radio_share:.3} of {CRATES} crates, want >= 0.70"
    );

    let camp_share = weapon_crate_share(&tables, &items, "metro_tower_camp");
    let flat_share = weapon_crate_share(&tables, &items, "metro_tower_flat");
    assert!(
        camp_share > flat_share,
        "metro_tower_camp weapon rate ({camp_share:.3}) must beat metro_tower_flat ({flat_share:.3})"
    );
}

struct ClimbPoi {
    id: &'static str,
    lot: [f32; 2],
    loot_table: &'static str,
    zombies: [i32; 2],
    zombie_models: &'static [&'static str],
    wake_height_m: f32,
}

const CLIMB_POIS: [ClimbPoi; 7] = [
    ClimbPoi {
        id: "tw_interior_office_24",
        lot: [24.0, 24.0],
        loot_table: "metro_tower_office",
        zombies: [3, 5],
        zombie_models: &["zombie_01", "zombie_02"],
        wake_height_m: 3.0,
    },
    ClimbPoi {
        id: "tw_interior_office_32",
        lot: [32.0, 32.0],
        loot_table: "metro_tower_office",
        zombies: [4, 6],
        zombie_models: &["zombie_01", "zombie_02"],
        wake_height_m: 3.0,
    },
    ClimbPoi {
        id: "tw_interior_flat_24",
        lot: [24.0, 24.0],
        loot_table: "metro_tower_flat",
        zombies: [3, 5],
        zombie_models: &["zombie_01", "zombie_03"],
        wake_height_m: 3.0,
    },
    ClimbPoi {
        id: "tw_interior_flat_32",
        lot: [32.0, 32.0],
        loot_table: "metro_tower_flat",
        zombies: [4, 6],
        zombie_models: &["zombie_01", "zombie_03"],
        wake_height_m: 3.0,
    },
    ClimbPoi {
        id: "tw_interior_camp_24",
        lot: [24.0, 24.0],
        loot_table: "metro_tower_camp",
        zombies: [4, 6],
        zombie_models: &["zombie_02", "zombie_03", "zombie_soldier"],
        wake_height_m: 3.0,
    },
    ClimbPoi {
        id: "tw_interior_camp_32",
        lot: [32.0, 32.0],
        loot_table: "metro_tower_camp",
        zombies: [5, 7],
        zombie_models: &["zombie_02", "zombie_03", "zombie_soldier"],
        wake_height_m: 3.0,
    },
    ClimbPoi {
        id: "tw_stair_flight_z",
        lot: [5.0, 7.0],
        loot_table: "metro_edge",
        zombies: [1, 2],
        zombie_models: &["zombie_01", "zombie_02"],
        wake_height_m: 3.0,
    },
];

#[test]
fn climb_poi_entries_parse_with_wake_height_and_old_entries_default_to_zero() {
    let cfg = world::PoiConfig::load_from_file(paths::repo_path("data/poi.json"))
        .expect("data/poi.json must load through world::PoiConfig");
    let tables = loot_tables();

    for expected in CLIMB_POIS {
        let def = cfg
            .get(expected.id)
            .unwrap_or_else(|| panic!("world::PoiConfig must know '{}'", expected.id));
        assert_eq!(def.lot, expected.lot, "'{}' lot", expected.id);
        assert_eq!(def.loot_table, expected.loot_table, "'{}' lootTable", expected.id);
        assert!(tables.contains(&def.loot_table), "'{}' lootTable must exist", expected.id);
        assert_eq!(def.zombies, expected.zombies, "'{}' zombies", expected.id);
        assert_eq!(
            def.zombie_models,
            expected.zombie_models.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            "'{}' zombieModels",
            expected.id
        );
        assert_eq!(def.wake_height_m, expected.wake_height_m, "'{}' wakeHeightM", expected.id);
        assert_eq!(def.chance_per_chunk, 0.0, "'{}' chancePerChunk", expected.id);
        assert!(def.biome.is_empty(), "'{}' biome must be empty", expected.id);
    }

    for old_id in ["cabin", "metro_hospital", "metro_police"] {
        let def = cfg
            .get(old_id)
            .unwrap_or_else(|| panic!("world::PoiConfig must know '{old_id}'"));
        assert_eq!(def.wake_height_m, 0.0, "old entry '{old_id}' must default wakeHeightM to 0.0");
    }
}
