//! Integration tests for unified biome keys across spawning, animals, loot tables, and danger tiers.

use game::items::ItemRegistry;
use game::spawner::{ActiveZombie, SpawnContext, Spawner};
use game::spawner_config::SpawnConfig;
use game::{AnimalConfigData, LootTableRegistry};
use glam::Vec3;
use physics::PhysicsWorld;
use world::BiomeId;

#[test]
fn every_biome_key_has_spawning_animals_and_loot_table() {
    let spawn_cfg = SpawnConfig::load_default().expect("load data/spawning.json");
    let animal_cfg = AnimalConfigData::from_repo().expect("load data/animals.json");
    let loot_tables = LootTableRegistry::from_repo().expect("load data/loot_tables.json");
    let items = ItemRegistry::from_repo().expect("load data/items.json");
    loot_tables
        .validate_items(&items)
        .expect("all loot table items must exist in data/items.json");

    for biome in BiomeId::ALL {
        let key = biome.key();
        assert!(
            spawn_cfg.biome_density.contains_key(key),
            "data/spawning.json missing biomeDensity entry for '{key}'"
        );
        assert!(
            spawn_cfg.density_for_biome(biome) > 0.0,
            "data/spawning.json density for '{key}' must be > 0"
        );
        assert!(
            animal_cfg
                .spawn
                .biomes
                .get(key)
                .is_some_and(|species| !species.is_empty()),
            "data/animals.json missing spawn.biomes entry for '{key}'"
        );
        assert!(
            loot_tables.contains(key),
            "data/loot_tables.json missing loot table for '{key}'"
        );
        assert_eq!(
            loot_tables.for_biome(biome),
            loot_tables.get(key).unwrap(),
            "for_biome({biome:?}) must return the '{key}' table"
        );
    }
}

#[test]
fn for_biome_desert_returns_desert_table() {
    let loot_tables = LootTableRegistry::from_repo().expect("load data/loot_tables.json");
    let expected = loot_tables.get("desert").expect("desert loot table");
    assert_eq!(loot_tables.for_biome("desert"), expected);
    assert_eq!(loot_tables.for_biome(BiomeId::Desert), expected);
}

#[test]
fn for_biome_unknown_key_falls_back_to_plains() {
    let loot_tables = LootTableRegistry::from_repo().expect("load data/loot_tables.json");
    let plains = loot_tables.get("plains").expect("plains loot table");
    assert_eq!(loot_tables.for_biome("not_a_biome"), plains);
}

#[test]
fn higher_tier_mult_spawns_more_zombies_with_same_rng() {
    let cfg = SpawnConfig::load_default().expect("load data/spawning.json");
    let physics = PhysicsWorld::default();
    let ground = |p: Vec3| Some(Vec3::new(p.x, 0.0, p.z));

    let run = |tier_mult: f32| -> usize {
        let mut spawner = Spawner::new(cfg.clone(), 42);
        let mut alive: Vec<ActiveZombie> = Vec::new();
        let mut next_id = 1u64;
        let player = Vec3::new(500.0, 0.0, 500.0);

        // Advance past start_delay then run 40 spawn intervals.
        spawner.play_time = cfg.start_delay;
        for _ in 0..40 {
            let mut ctx = SpawnContext::new(player, &physics, &alive, &[])
                .with_biome_id(BiomeId::Plains)
                .with_tier_mult(tier_mult);
            ctx.ground_check = Some(&ground);
            let orders = spawner.update(cfg.spawn_interval, &ctx);
            for order in orders.spawns {
                alive.push(ActiveZombie::new(next_id, order.position));
                next_id += 1;
            }
        }
        alive.len()
    };

    let base_spawns = run(1.0);
    let tier_spawns = run(2.2);
    assert!(
        tier_spawns > base_spawns,
        "expected tier_mult 2.2 ({tier_spawns}) to spawn more zombies than 1.0 ({base_spawns})"
    );
}
