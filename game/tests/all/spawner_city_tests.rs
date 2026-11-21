//! Tests for city zombie spawning rules: density, cap, night multiplier, and core night runners.

use game::spawner::{
    ActiveZombie, CitySpawnConfig, SettlementSite, SpawnConfig, SpawnContext, SpawnTypeEntry,
    Spawner, SpawnerRng,
};
use glam::{IVec2, Vec3};
use physics::{PhysicsWorld, CHUNK_SAMPLES};

fn flat_world() -> PhysicsWorld {
    let mut world = PhysicsWorld::new();
    let heights = vec![0.0f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
    for cz in -3..=3 {
        for cx in -3..=3 {
            world.add_ground_chunk(IVec2::new(cx, cz), &heights);
        }
    }
    world
}

fn fast_test_config() -> SpawnConfig {
    let mut cfg = SpawnConfig::load_default().expect("data/spawning.json must load");
    cfg.start_delay = 0.0;
    cfg.spawn_interval = 0.01;
    cfg
}

#[test]
fn spawner_city_config_loaded_from_json() {
    let config = SpawnConfig::load_default().expect("data/spawning.json must load");
    assert!(config.city.is_some(), "spawning.json must define city config");

    let city: &CitySpawnConfig = config.city.as_ref().unwrap();
    assert_eq!(city.density, 2.0);
    assert_eq!(city.max_active, 32);
    assert_eq!(city.core_night_types.len(), 2);
    assert_eq!(city.core_night_types[0].id, "walker");
    assert_eq!(city.core_night_types[0].weight, 0.4);
    assert_eq!(city.core_night_types[1].id, "runner");
    assert_eq!(city.core_night_types[1].weight, 0.6);
}

#[test]
fn spawner_city_cap_day_capped_at_32_and_night_multiplier() {
    let world = flat_world();
    let config = SpawnConfig::load_default().expect("data/spawning.json must load");
    let spawner = Spawner::new(config, 101);
    let alive: Vec<ActiveZombie> = Vec::new();
    let settlements: Vec<SettlementSite> = Vec::new();

    // In city during daytime:
    // scaled_max_active = 20, density = 2.0 -> day_cap = 40.0, capped at city.maxActive (32).
    let ctx_city_day = SpawnContext::new(Vec3::ZERO, &world, &alive, &settlements)
        .with_biome("plains")
        .with_in_city(true);
    let cap_day = spawner.calculate_cap(&ctx_city_day);
    assert_eq!(cap_day, 32, "city day cap must be capped at 32");

    // Also verify with alias builder `with_city`:
    let ctx_city_alias = SpawnContext::new(Vec3::ZERO, &world, &alive, &settlements)
        .with_biome("plains")
        .with_city(true);
    assert_eq!(spawner.calculate_cap(&ctx_city_alias), 32);

    // In city at night:
    // day_cap = 32, night_multiplier = 1.6 -> 32 * 1.6 = 51.2 -> 51.
    let ctx_city_night = SpawnContext::new(Vec3::ZERO, &world, &alive, &settlements)
        .with_biome("plains")
        .with_in_city(true)
        .with_night(true);
    let cap_night = spawner.calculate_cap(&ctx_city_night);
    assert_eq!(cap_night, 51, "city night cap must apply night multiplier on top of capped 32");

    // Outside city in plains:
    // day: 20 * 0.7 = 14.
    // night: 14 * 1.6 = 22.4 -> 22.
    let ctx_plains_day = SpawnContext::new(Vec3::ZERO, &world, &alive, &settlements)
        .with_biome("plains")
        .with_in_city(false);
    assert_eq!(spawner.calculate_cap(&ctx_plains_day), 14);

    let ctx_plains_night = SpawnContext::new(Vec3::ZERO, &world, &alive, &settlements)
        .with_biome("plains")
        .with_in_city(false)
        .with_night(true);
    assert_eq!(spawner.calculate_cap(&ctx_plains_night), 22);
}

#[test]
fn spawner_city_cap_scaling_below_max_active() {
    let world = flat_world();
    let config = SpawnConfig::load_default().expect("data/spawning.json must load");
    let spawner = Spawner::new(config, 202);
    let alive: Vec<ActiveZombie> = Vec::new();
    let settlements: Vec<SettlementSite> = Vec::new();

    // With zombie_amount = 0.5:
    // scaled_max_active = 10, density = 2.0 -> day_cap = 20.0 (below 32, so 20).
    // night: 20 * 1.6 = 32.
    let ctx_half_day = SpawnContext::new(Vec3::ZERO, &world, &alive, &settlements)
        .with_zombie_amount(0.5)
        .with_in_city(true);
    assert_eq!(spawner.calculate_cap(&ctx_half_day), 20);

    let ctx_half_night = SpawnContext::new(Vec3::ZERO, &world, &alive, &settlements)
        .with_zombie_amount(0.5)
        .with_in_city(true)
        .with_night(true);
    assert_eq!(spawner.calculate_cap(&ctx_half_night), 32);
}

#[test]
fn spawner_city_no_city_config_gives_today_numbers() {
    let world = flat_world();
    let mut config = SpawnConfig::load_default().expect("data/spawning.json must load");
    config.city = None; // simulate world / save without city config

    let spawner = Spawner::new(config, 303);
    let alive: Vec<ActiveZombie> = Vec::new();
    let settlements: Vec<SettlementSite> = Vec::new();

    // Even if in_city or in_city_core is true, None city config gives standard biome numbers.
    let ctx_day = SpawnContext::new(Vec3::ZERO, &world, &alive, &settlements)
        .with_biome("plains")
        .with_in_city(true)
        .with_in_city_core(true);
    assert_eq!(
        spawner.calculate_cap(&ctx_day),
        14,
        "without city config, plains day cap must remain 14"
    );

    let ctx_night = SpawnContext::new(Vec3::ZERO, &world, &alive, &settlements)
        .with_biome("plains")
        .with_in_city(true)
        .with_in_city_core(true)
        .with_night(true);
    assert_eq!(
        spawner.calculate_cap(&ctx_night),
        22,
        "without city config, plains night cap must remain 22"
    );
}

#[test]
fn spawner_city_core_night_picks_runners_about_60_percent() {
    let world = flat_world();
    let mut config = fast_test_config();
    config.max_active = 500;

    let mut spawner = Spawner::new(config.clone(), 404);
    let player = Vec3::ZERO;
    let settlements: Vec<SettlementSite> = Vec::new();
    let alive: Vec<ActiveZombie> = Vec::new();

    let mut total_spawns = 0;
    let mut runners = 0;
    let mut walkers = 0;

    while total_spawns < 300 {
        let ctx = SpawnContext::new(player, &world, &alive, &settlements)
            .with_night(true)
            .with_in_city(true)
            .with_in_city_core(true);
        let orders = spawner.update(0.1, &ctx);
        for spawn in orders.spawns {
            total_spawns += 1;
            if spawn.type_id == "runner" {
                runners += 1;
            } else if spawn.type_id == "walker" {
                walkers += 1;
            }
        }
    }

    assert_eq!(total_spawns, 300);
    assert!(walkers > 0, "core night spawns must produce walkers");
    assert_eq!(runners + walkers, total_spawns);
    let runner_share = runners as f64 / total_spawns as f64;
    assert!(
        (0.50..=0.70).contains(&runner_share),
        "core night runner share was {runner_share:.3} over {total_spawns} spawns, expected ~0.60"
    );

    // Also verify direct method pick_type_id_core over 1000 rolls:
    let mut rng = SpawnerRng::new(505);
    let mut direct_runners = 0;
    for _ in 0..1000 {
        let tid = config.pick_type_id_core(&mut rng, true, true);
        if tid == "runner" {
            direct_runners += 1;
        }
    }
    let direct_share = direct_runners as f64 / 1000.0;
    assert!(
        (0.55..=0.65).contains(&direct_share),
        "direct pick_type_id_core runner share was {direct_share:.3}, expected ~0.60"
    );
}

#[test]
fn spawner_city_day_and_outside_core_night_distribution() {
    let config = fast_test_config();
    let mut rng = SpawnerRng::new(606);

    // In city core during day: uses standard day table (walker: 1.0, runner: 0.1 -> ~9.1% runner).
    let mut day_runners = 0;
    for _ in 0..1000 {
        let tid = config.pick_type_id_core(&mut rng, false, true);
        if tid == "runner" {
            day_runners += 1;
        }
    }
    let day_share = day_runners as f64 / 1000.0;
    assert!(
        day_share < 0.20,
        "day core runner share was {day_share:.3}, expected ~0.09"
    );

    // In city outside core at night: uses standard night table (walker: 0.6, runner: 0.4 -> 40% runner).
    let mut night_outside_runners = 0;
    for _ in 0..1000 {
        let tid = config.pick_type_id_core(&mut rng, true, false);
        if tid == "runner" {
            night_outside_runners += 1;
        }
    }
    let night_outside_share = night_outside_runners as f64 / 1000.0;
    assert!(
        (0.34..=0.46).contains(&night_outside_share),
        "night outside core runner share was {night_outside_share:.3}, expected ~0.40"
    );
}

#[test]
fn spawner_city_builder_methods_and_pick_slice() {
    let world = flat_world();
    let alive: Vec<ActiveZombie> = Vec::new();
    let settlements: Vec<SettlementSite> = Vec::new();

    let ctx_default = SpawnContext::new(Vec3::ZERO, &world, &alive, &settlements);
    assert!(!ctx_default.in_city);
    assert!(!ctx_default.in_city_core);

    let ctx1 = ctx_default.with_in_city(true).with_in_city_core(true);
    assert!(ctx1.in_city);
    assert!(ctx1.in_city_core);

    let ctx2 = ctx1.with_city(false).with_city_core(false);
    assert!(!ctx2.in_city);
    assert!(!ctx2.in_city_core);

    // Test pick_from_slice directly with custom table
    let custom = vec![
        SpawnTypeEntry {
            id: "special".to_string(),
            weight: 1.0,
        },
    ];
    let mut rng = SpawnerRng::new(707);
    assert_eq!(SpawnConfig::pick_from_slice(&mut rng, &custom), "special");
}
