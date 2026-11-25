//! Integration tests for R33-09 town gameplay: loot tables, zombie density,
//! trapped survivors in towns, guild explore tasks, and town/trailhead hints.

use std::collections::HashSet;

use game::guild::{
    GuildConfig, GuildManager, GuildReward, GuildTask, GuildTaskHookConfig, GuildTaskHooks,
    GuildTaskKind, RecruitDef,
};
use game::hints::{HintConfig, HintContext, HintSystem};
use game::items::ItemRegistry;
use game::loot::LootTableRegistry;
use game::rescue::RescueConfig;
use game::rescue_place::place_town_survivors;
use game::spawner::{ActiveZombie, SettlementSite, SpawnConfig, SpawnContext, Spawner};
use glam::{IVec2, Vec2, Vec3};
use physics::{PhysicsWorld, CHUNK_SAMPLES};
use world::towns::{TownLot, TownSite};
use world::BiomeId;

fn flat_world() -> PhysicsWorld {
    let mut world = PhysicsWorld::new();
    let heights = vec![0.0f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
    for cz in -2..=2 {
        for cx in -2..=2 {
            world.add_ground_chunk(IVec2::new(cx, cz), &heights);
        }
    }
    world
}

fn sample_towns() -> Vec<TownSite> {
    vec![
        TownSite {
            key: "dusthollow".to_string(),
            name: "Dusthollow".to_string(),
            biome: BiomeId::Desert,
            center: Vec2::new(3600.0, 0.0),
            radius: 140.0,
            street_dir: Vec2::X,
            lots: vec![
                TownLot {
                    building: "adobe_house".to_string(),
                    pos: Vec2::new(3580.0, 15.0),
                    yaw: 0.0,
                    footprint: Vec2::new(5.0, 4.5),
                    loot_table: "town_home".to_string(),
                },
                TownLot {
                    building: "desert_store".to_string(),
                    pos: Vec2::new(3610.0, 15.0),
                    yaw: 0.0,
                    footprint: Vec2::new(7.0, 5.5),
                    loot_table: "town_store".to_string(),
                },
                TownLot {
                    building: "motel".to_string(),
                    pos: Vec2::new(3640.0, -15.0),
                    yaw: std::f32::consts::PI,
                    footprint: Vec2::new(9.0, 5.0),
                    loot_table: "town_home".to_string(),
                },
            ],
        },
        TownSite {
            key: "frostmere".to_string(),
            name: "Frostmere".to_string(),
            biome: BiomeId::Snow,
            center: Vec2::new(-3800.0, 0.0),
            radius: 125.0,
            street_dir: Vec2::X,
            lots: vec![
                TownLot {
                    building: "log_cabin_snow".to_string(),
                    pos: Vec2::new(-3810.0, 14.0),
                    yaw: 0.0,
                    footprint: Vec2::new(5.5, 4.5),
                    loot_table: "town_home".to_string(),
                },
                TownLot {
                    building: "research_station".to_string(),
                    pos: Vec2::new(-3780.0, -14.0),
                    yaw: std::f32::consts::PI,
                    footprint: Vec2::new(8.5, 6.0),
                    loot_table: "town_station".to_string(),
                },
            ],
        },
    ]
}

#[test]
fn town_loot_tables_exist_and_use_valid_items() {
    let items = ItemRegistry::from_repo().expect("data/items.json must load");
    let loot = LootTableRegistry::from_repo().expect("data/loot_tables.json must load");
    loot.validate_items(&items)
        .expect("all loot table entries must exist in data/items.json");

    for key in ["town_home", "town_store", "town_station"] {
        let table = loot
            .get(key)
            .unwrap_or_else(|| panic!("loot table '{key}' must exist"));
        assert!(!table.entries.is_empty(), "'{key}' must have entries");
        for entry in &table.entries {
            assert!(
                items.contains(&entry.id),
                "loot table '{key}' references unknown item '{}'",
                entry.id
            );
        }
    }
}

#[test]
fn town_spawner_density_is_above_one_and_scales_cap() {
    let config = SpawnConfig::load_default().expect("data/spawning.json must load");
    assert!(
        config.town_density > 1.0,
        "townDensity ({}) must be > 1.0",
        config.town_density
    );

    let world = flat_world();
    let spawner = Spawner::new(config, 42);
    let alive: Vec<ActiveZombie> = Vec::new();
    let settlements: Vec<SettlementSite> = Vec::new();

    let ctx_wild = SpawnContext::new(Vec3::ZERO, &world, &alive, &settlements).with_biome("desert");
    let ctx_town = SpawnContext::new(Vec3::ZERO, &world, &alive, &settlements)
        .with_biome("desert")
        .with_in_town(true);

    let cap_wild = spawner.calculate_cap(&ctx_wild);
    let cap_town = spawner.calculate_cap(&ctx_town);
    assert!(
        cap_town > cap_wild,
        "town cap ({cap_town}) must exceed wild desert cap ({cap_wild})"
    );
}

#[test]
fn town_rescue_places_one_to_two_survivors_per_town() {
    let cfg = RescueConfig::from_repo().expect("data/rescue.json must load");
    assert_eq!(cfg.per_town, [1, 2]);
    let recruits: Vec<RecruitDef> = GuildConfig::from_repo()
        .expect("data/guild.json must load")
        .recruits;
    let towns = sample_towns();
    let mut used_names = HashSet::new();
    let spots = place_town_survivors(42, &towns, &cfg, &recruits, 0, &mut used_names);

    for town in &towns {
        let prefix = format!("town_{}_", town.key);
        let count = spots.iter().filter(|s| s.poi_id.starts_with(&prefix)).count();
        assert!(
            (1..=2).contains(&count),
            "town '{}' should have 1..=2 rescue spots, got {count}",
            town.name
        );
    }
}

#[test]
fn guild_explore_task_can_target_a_town() {
    let guild_cfg = GuildConfig::from_repo().expect("data/guild.json must load");
    for name in ["Dusthollow", "Frostmere", "Blackwater", "Ridgeback Station"] {
        assert!(
            guild_cfg
                .tasks
                .iter()
                .any(|t| t.kind == "explore" && t.place_name.as_deref() == Some(name)),
            "data/guild.json must include an explore task for '{name}'"
        );
    }

    let towns = sample_towns();
    let mut manager = GuildManager::new(42, GuildConfig::default());
    let task = GuildTask::new(
        "explore_dusthollow",
        "Explore Dusthollow",
        GuildTaskKind::Explore {
            place_name: "Dusthollow".to_string(),
        },
        1,
        GuildReward::default(),
    );
    manager.board_mut().set_offered(vec![task]);
    manager.accept_task("explore_dusthollow").unwrap();

    let mut hooks = GuildTaskHooks::new(GuildTaskHookConfig::default());
    let outside = Vec3::new(3600.0 + 200.0, 0.0, 0.0);
    hooks.tick_with_towns(&mut manager, &[], &towns, outside, 0.6);
    assert!(!manager.board().active()[0].is_done());

    let inside = Vec3::new(3600.0 + 100.0, 0.0, 0.0);
    hooks.tick_with_towns(&mut manager, &[], &towns, inside, 0.6);
    assert!(manager.board().active()[0].is_done());
}

#[test]
fn town_and_trailhead_hints_load_and_can_be_requested() {
    let mut config = HintConfig::load_default().expect("data/hints.json must load");
    assert!(config.hints.iter().any(|h| h.id == "trailhead"));
    assert!(config.hints.iter().any(|h| h.id == "town"));
    config.hints.retain(|h| h.id == "trailhead" || h.id == "town");
    let mut system = HintSystem::new(config);
    let ctx = HintContext::new();

    assert!(system.request("trailhead"));
    let h1 = system.tick(0.0, &ctx).expect("trailhead hint should show");
    assert_eq!(h1.id, "trailhead");

    system.tick(8.0, &ctx);
    system.tick(0.5, &ctx);
    assert!(system.request("town"));
    let h2 = system.tick(0.0, &ctx).expect("town hint should show");
    assert_eq!(h2.id, "town");
}
