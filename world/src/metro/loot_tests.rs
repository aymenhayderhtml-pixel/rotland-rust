//! Tests for New Halden district loot placement and `LootSpot` compatibility.

use std::collections::HashSet;

use glam::Vec2;

use crate::biome::BiomeId;
use crate::loot::LootSpot;
use crate::metro::config::MetroConfig;
use crate::metro::damage::{damage_sites, default_damage};
use crate::metro::highway::{Highway, HighwayHeightField};
use crate::metro::jam::plan_jam;
use crate::metro::layout::MetroLayout;
use crate::metro::loot::{MetroLootConfig, metro_loot_in_chunk, metro_loot_in_chunk_with_config};
use crate::metro::paint::MetroLook;
use crate::metro::place::MetroSite;
use crate::metro::state::{MetroWorld, index_towers_by_block};
use crate::metro::towers::plan_towers;

fn test_metro() -> MetroWorld {
    let cfg = MetroConfig::load_from_file(paths::repo_path("data/metro.json"))
        .expect("data/metro.json must load");
    let site = MetroSite {
        key: cfg.key.clone(),
        name: cfg.name.clone(),
        center: Vec2::new(6000.0, 0.0),
        yaw: 0.0,
        half_size: (cfg.size_blocks as f32 * cfg.block_size) * 0.5,
        ground_y: 0.0,
    };
    let layout = MetroLayout::generate(site.clone(), &cfg);
    let towers = plan_towers(&layout, &cfg);
    let entrance_world = site.to_world(Vec2::new(0.0, site.half_size));
    let out = (entrance_world - site.center).normalize();
    let highway = Highway {
        from_town: "blackwater".into(),
        points: vec![entrance_world + out * 500.0, entrance_world],
        width: cfg.highway.highway_width,
    };
    let highway_field = HighwayHeightField::build(&highway, 0.0, &|_x, _z| 0.0, &cfg);
    let jam = plan_jam(&highway, &layout, &cfg);
    let towers_by_block = index_towers_by_block(&towers);
    let damage = damage_sites(&layout, &towers, default_damage());
    let look = MetroLook::load_from_file(paths::repo_path("data/metro.json")).expect("look");
    MetroWorld {
        config: cfg,
        layout,
        towers,
        highway,
        highway_field,
        jam,
        look,
        towers_by_block,
        damage,
    }
}

fn all_city_chunks(metro: &MetroWorld) -> Vec<(i32, i32)> {
    let min_x = ((metro.layout.site.center.x - metro.layout.site.half_size) / 64.0).floor() as i32 - 1;
    let max_x = ((metro.layout.site.center.x + metro.layout.site.half_size) / 64.0).ceil() as i32 + 1;
    let min_z = ((metro.layout.site.center.y - metro.layout.site.half_size) / 64.0).floor() as i32 - 1;
    let max_z = ((metro.layout.site.center.y + metro.layout.site.half_size) / 64.0).ceil() as i32 + 1;
    let mut chunks = Vec::new();
    for cx in min_x..=max_x {
        for cz in min_z..=max_z {
            chunks.push((cx, cz));
        }
    }
    chunks
}

#[test]
fn same_seed_gives_same_spots() {
    let metro = test_metro();
    let chunks = all_city_chunks(&metro);
    let a: Vec<LootSpot> = chunks
        .iter()
        .flat_map(|&c| metro_loot_in_chunk(&metro, c, &|_, _| 0.0, 42))
        .collect();
    let b: Vec<LootSpot> = chunks
        .iter()
        .flat_map(|&c| metro_loot_in_chunk(&metro, c, &|_, _| 0.0, 42))
        .collect();
    assert_eq!(a, b);
    assert!(!a.is_empty(), "metro must place loot spots");
}

#[test]
fn army_caches_total_matches_config_and_all_spots_inside_city() {
    let metro = test_metro();
    let cfg = MetroLootConfig::default();
    let chunks = all_city_chunks(&metro);
    let spots: Vec<LootSpot> = chunks
        .iter()
        .flat_map(|&c| metro_loot_in_chunk_with_config(&metro, &cfg, c, &|_, _| 0.0, 7))
        .collect();

    let army_count = spots
        .iter()
        .filter(|s| s.table.as_deref() == Some("metro_army"))
        .count();
    assert_eq!(army_count, cfg.army_caches.midtown + cfg.army_caches.core);

    for spot in &spots {
        let xz = Vec2::new(spot.position.x, spot.position.z);
        assert!(
            metro.layout.site.contains_square(xz),
            "spot {} outside New Halden at {xz:?}",
            spot.crate_id
        );
        assert_eq!(spot.tier, 0);
    }
}

#[test]
fn all_spot_tables_exist_in_loot_tables_json_and_all_items_exist() {
    let metro = test_metro();
    let chunks = all_city_chunks(&metro);
    let spots: Vec<LootSpot> = chunks
        .iter()
        .flat_map(|&c| metro_loot_in_chunk(&metro, c, &|_, _| 0.0, 99))
        .collect();

    let tables_json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(paths::repo_path("data/loot_tables.json"))
            .expect("data/loot_tables.json must read"),
    )
    .expect("data/loot_tables.json must parse");
    let items_json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(paths::repo_path("data/items.json"))
            .expect("data/items.json must read"),
    )
    .expect("data/items.json must parse");

    let items_map = items_json.as_object().expect("items.json must be object");
    let tables_map = tables_json.as_object().expect("loot_tables.json must be object");

    for spot in &spots {
        let table = spot.table.as_deref().expect("metro spot must set table");
        assert!(tables_map.contains_key(table), "missing table {table}");
    }

    let metro_tables = [
        "metro_edge",
        "metro_midtown",
        "metro_core",
        "metro_army",
        "metro_medical",
        "metro_camp",
        "metro_car",
    ];
    let r41_items: HashSet<&str> = [
        "body_armor",
        "army_helmet",
        "army_rifle",
        "army_ration",
        "ammo_556",
        "night_vision",
        "battery",
        "antibiotics",
        "surgery_kit",
        "adrenaline_shot",
        "gun_parts",
        "car_parts",
    ]
    .into_iter()
    .collect();

    for tname in metro_tables {
        let entries = tables_map[tname]["entries"]
            .as_array()
            .expect("entries must be array");
        assert!(!entries.is_empty(), "{tname} must have entries");
        for entry in entries {
            let id = entry["id"].as_str().expect("entry id must be string");
            assert!(
                items_map.contains_key(id) || r41_items.contains(id),
                "unknown item '{id}' in {tname}"
            );
        }
    }
}

#[test]
fn old_saved_loot_spot_without_new_fields_deserializes() {
    let old_json = r#"{
        "crate_id": "0_0_0",
        "position": [10.0, 0.0, 20.0],
        "yaw": 1.5,
        "biome": "forest",
        "tier": 2
    }"#;
    let spot: LootSpot = serde_json::from_str(old_json).expect("old LootSpot must deserialize");
    assert_eq!(spot.biome, BiomeId::Forest);
    assert_eq!(spot.tier, 2);
    assert_eq!(spot.table, None);
    assert_eq!(spot.model, None);
}
