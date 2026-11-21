use game::spawner::{
    ActiveZombie, SettlementSite, SpawnConfig, SpawnContext, Spawner,
};
use glam::{IVec2, Vec3};
use physics::{PhysicsWorld, CHUNK_SAMPLES};

/// Flat ground over a 7x7 chunk area centered on (0, 0), so spawns up to 150+ m land on ground.
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
fn spawning_json_loads_correctly_and_completely() {
    let text = std::fs::read_to_string(paths::repo_path("data/spawning.json")).unwrap();
    let raw: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(raw["maxActive"].as_u64().unwrap(), 20);

    let config = SpawnConfig::load_default().expect("data/spawning.json must load");
    assert_eq!(config.max_active, 20);
    assert_eq!(config.min_spawn_dist, 40.0);
    assert_eq!(config.max_spawn_dist, 90.0);
    assert_eq!(config.despawn_dist, 120.0);
    assert_eq!(config.spawn_interval, 1.5);
    assert_eq!(config.start_delay, 10.0);
    assert_eq!(config.out_of_view_angle_deg, 75.0);
    assert_eq!(config.night_max_active_multiplier, 1.6);
    assert_eq!(config.camp_density.radius, 150.0);
    assert_eq!(config.camp_density.factor, 0.5);
    assert_eq!(config.density_for("forest"), 1.0);
    assert_eq!(config.density_for("plains"), 0.7);
    assert_eq!(config.density_for("unknown"), 1.0);

    let type_ids = config.all_type_ids();
    assert!(type_ids.contains(&"walker".to_string()));
    assert!(type_ids.contains(&"runner".to_string()));
    assert_eq!(type_ids.len(), 2);
}

#[test]
fn missing_key_in_json_is_an_error_not_a_silent_default() {
    let incomplete = r#"{"maxActive": 20, "minSpawnDist": 40.0}"#;
    assert!(SpawnConfig::from_json(incomplete).is_err());
}

#[test]
fn spawns_fill_up_to_the_cap_over_time_and_never_exceed_it() {
    let world = flat_world();
    let mut config = fast_test_config();
    config.max_active = 5;

    let mut spawner = Spawner::new(config, 101);
    let mut alive: Vec<ActiveZombie> = Vec::new();
    let settlements: Vec<SettlementSite> = Vec::new();
    let mut next_id = 1u64;
    let mut total_spawned = 0;

    for _ in 0..60 {
        let ctx = SpawnContext::new(Vec3::ZERO, &world, &alive, &settlements);
        let orders = spawner.update(0.1, &ctx);
        for spawn in orders.spawns {
            alive.push(ActiveZombie::new(next_id, spawn.position));
            next_id += 1;
            total_spawned += 1;
        }
        assert!(
            alive.len() <= 5,
            "active count {} must never exceed cap 5",
            alive.len()
        );
    }

    assert_eq!(alive.len(), 5, "spawner must fill up to the cap of 5");
    assert_eq!(
        total_spawned, 5,
        "spawner must not spawn once cap is reached"
    );
}

#[test]
fn cap_is_higher_at_night_and_scaled_by_difficulty() {
    let world = flat_world();
    let config = fast_test_config(); // max_active = 20, night_multiplier = 1.6
    let spawner = Spawner::new(config, 202);
    let alive: Vec<ActiveZombie> = Vec::new();
    let settlements: Vec<SettlementSite> = Vec::new();

    let test_cases = [
        // (is_night, zombie_amount, expected_cap)
        (false, 1.0, 20),
        (true, 1.0, 32),   // 20 * 1.6 = 32
        (false, 0.5, 10),  // 20 * 0.5 = 10
        (true, 0.5, 16),   // 10 * 1.6 = 16
        (false, 1.5, 30),  // 20 * 1.5 = 30
        (true, 1.5, 48),   // 30 * 1.6 = 48
        (false, 2.0, 40),  // 20 * 2.0 = 40
        (true, 2.0, 60),   // 40 * 1.6 = 64, capped by spawning.json hardCap 60
    ];

    let mut checked_cases = 0;
    for (is_night, zombie_amount, expected_cap) in test_cases {
        let ctx = SpawnContext::new(Vec3::ZERO, &world, &alive, &settlements)
            .with_night(is_night)
            .with_zombie_amount(zombie_amount);
        let cap = spawner.calculate_cap(&ctx);
        assert_eq!(
            cap, expected_cap,
            "night={is_night}, amount={zombie_amount}: expected cap {expected_cap}, got {cap}"
        );
        checked_cases += 1;
    }

    assert_eq!(checked_cases, 8, "must verify all 8 difficulty cap cases");
}

#[test]
fn every_spawn_is_between_min_and_max_distance_and_out_of_view() {
    let world = flat_world();
    let mut config = fast_test_config();
    config.max_active = 200;

    let mut spawner = Spawner::new(config.clone(), 303);
    let player = Vec3::new(10.0, 0.0, 10.0);
    let settlements: Vec<SettlementSite> = Vec::new();
    let alive: Vec<ActiveZombie> = Vec::new();

    let mut checked_spawns = 0;
    for _ in 0..50 {
        let ctx = SpawnContext::new(player, &world, &alive, &settlements)
            .with_forward(Vec3::NEG_Z);
        let orders = spawner.update(0.1, &ctx);
        for spawn in orders.spawns {
            let dx = spawn.position.x - player.x;
            let dz = spawn.position.z - player.z;
            let dist = (dx * dx + dz * dz).sqrt();

            assert!(
                dist >= config.min_spawn_dist - 0.01 && dist <= config.max_spawn_dist + 0.01,
                "spawn distance {dist} outside [{}, {}]",
                config.min_spawn_dist,
                config.max_spawn_dist
            );

            // Angle from camera/player forward (0, 0, -1)
            let to_spawn = Vec3::new(dx, 0.0, dz).normalize();
            let angle_deg = Vec3::NEG_Z.dot(to_spawn).clamp(-1.0, 1.0).acos().to_degrees();
            assert!(
                angle_deg >= config.out_of_view_angle_deg - 0.01,
                "spawn angle {angle_deg} inside view cone (< {})",
                config.out_of_view_angle_deg
            );

            checked_spawns += 1;
        }
    }

    assert!(
        checked_spawns >= 40,
        "must check at least 40 spawns, got {checked_spawns}"
    );
}

#[test]
fn no_spawn_lands_inside_camp_safe_zone_even_when_player_stands_in_camp() {
    let world = flat_world();
    let mut config = fast_test_config();
    config.max_active = 200;

    let camp = SettlementSite::new("camp_1", Vec3::ZERO, 0.0);
    let settlements = vec![camp];
    let mut spawner = Spawner::new(config, 404);
    let player = Vec3::ZERO; // Player stands directly at the camp center
    let alive: Vec<ActiveZombie> = Vec::new();

    // Camp safe zone is 17.5 m half + 40 m padding = 57.5 m box
    let limit = 17.5 + 40.0;
    let mut checked_spawns = 0;

    for _ in 0..60 {
        let ctx = SpawnContext::new(player, &world, &alive, &settlements);
        let orders = spawner.update(0.1, &ctx);
        for spawn in orders.spawns {
            let dx = spawn.position.x.abs();
            let dz = spawn.position.z.abs();
            let inside_camp = dx <= limit && dz <= limit;
            assert!(
                !inside_camp,
                "spawn at ({}, {}) landed inside camp safe zone box (+/- {limit})",
                spawn.position.x, spawn.position.z
            );
            checked_spawns += 1;
        }
    }

    assert!(
        checked_spawns >= 20,
        "must verify at least 20 spawns outside camp safe zone, got {checked_spawns}"
    );
}

#[test]
fn zombies_farther_than_despawn_distance_are_ordered_despawned() {
    let world = flat_world();
    let config = fast_test_config(); // despawn_dist = 120.0, chase factor = 1.5 -> 180.0
    let mut spawner = Spawner::new(config, 505);
    let player = Vec3::ZERO;
    let settlements: Vec<SettlementSite> = Vec::new();

    let alive = vec![
        ActiveZombie::new(1, Vec3::new(50.0, 0.0, 0.0)),         // 50 m: keep
        ActiveZombie::new(2, Vec3::new(119.0, 0.0, 0.0)),        // 119 m: keep
        ActiveZombie::new(3, Vec3::new(121.0, 0.0, 0.0)),        // 121 m: despawn
        ActiveZombie::chasing(4, Vec3::new(150.0, 0.0, 0.0)),    // 150 m chasing (< 180 m): keep
        ActiveZombie::chasing(5, Vec3::new(181.0, 0.0, 0.0)),    // 181 m chasing (> 180 m): despawn
        ActiveZombie::new(6, Vec3::new(0.0, 0.0, 300.0)),        // 300 m: despawn
    ];

    let ctx = SpawnContext::new(player, &world, &alive, &settlements);
    let orders = spawner.update(0.1, &ctx);

    assert_eq!(orders.despawns.len(), 3, "expected exactly 3 despawns");
    assert!(orders.despawns.contains(&3), "zombie 3 (121 m) must be despawned");
    assert!(orders.despawns.contains(&5), "zombie 5 (181 m chasing) must be despawned");
    assert!(orders.despawns.contains(&6), "zombie 6 (300 m) must be despawned");
    assert!(!orders.despawns.contains(&1), "zombie 1 (50 m) must be kept");
    assert!(!orders.despawns.contains(&2), "zombie 2 (119 m) must be kept");
    assert!(!orders.despawns.contains(&4), "zombie 4 (150 m chasing) must be kept");
}

#[test]
fn night_mix_produces_runners() {
    let world = flat_world();
    let mut config = fast_test_config();
    config.max_active = 500;

    let mut spawner = Spawner::new(config, 606);
    let player = Vec3::ZERO;
    let settlements: Vec<SettlementSite> = Vec::new();
    let alive: Vec<ActiveZombie> = Vec::new();

    let mut total_spawns = 0;
    let mut runners = 0;
    let mut walkers = 0;

    while total_spawns < 200 {
        let ctx = SpawnContext::new(player, &world, &alive, &settlements).with_night(true);
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

    assert!(runners > 0, "night spawns must produce runners");
    assert!(walkers > 0, "night spawns must produce walkers");

    let runner_share = runners as f64 / total_spawns as f64;
    assert!(
        (0.28..=0.52).contains(&runner_share),
        "runner share was {runner_share:.2} over {total_spawns} spawns, expected ~0.40"
    );
    assert_eq!(total_spawns, 200, "checked exactly 200 night spawns");
}

#[test]
fn same_seed_gives_the_same_spawns() {
    let world = flat_world();
    let config = fast_test_config();

    let mut spawner_a = Spawner::new(config.clone(), 777);
    let mut spawner_b = Spawner::new(config.clone(), 777);
    let mut spawner_c = Spawner::new(config, 888);

    let player = Vec3::new(5.0, 0.0, 5.0);
    let settlements: Vec<SettlementSite> = Vec::new();
    let alive: Vec<ActiveZombie> = Vec::new();

    let mut spawns_a = Vec::new();
    let mut spawns_b = Vec::new();
    let mut spawns_c = Vec::new();

    for _ in 0..30 {
        let ctx = SpawnContext::new(player, &world, &alive, &settlements);
        spawns_a.extend(spawner_a.update(0.1, &ctx).spawns);
        spawns_b.extend(spawner_b.update(0.1, &ctx).spawns);
        spawns_c.extend(spawner_c.update(0.1, &ctx).spawns);
    }

    assert_eq!(
        spawns_a.len(),
        spawns_b.len(),
        "identical seeds must produce identical number of spawns"
    );
    assert!(spawns_a.len() >= 25, "must produce at least 25 spawns");

    let mut matched_count = 0;
    for (a, b) in spawns_a.iter().zip(spawns_b.iter()) {
        assert_eq!(a.type_id, b.type_id);
        assert_eq!(a.position, b.position);
        matched_count += 1;
    }
    assert_eq!(matched_count, spawns_a.len(), "all spawns matched between identical seeds");

    // Different seed should not produce identical positions
    let differs = spawns_a.iter().zip(spawns_c.iter()).any(|(a, c)| a.position != c.position);
    assert!(differs, "different seeds must produce different spawn positions");
}

#[test]
fn daytime_camp_proximity_halves_the_spawn_cap() {
    let world = flat_world();
    let config = fast_test_config();
    let spawner = Spawner::new(config, 909);
    let settlements = vec![SettlementSite::new("camp_1", Vec3::ZERO, 0.0)];
    let alive: Vec<ActiveZombie> = Vec::new();

    // Player within 150 m radius of camp during day: cap halved (20 * 0.5 = 10)
    let ctx_near_day = SpawnContext::new(Vec3::new(50.0, 0.0, 50.0), &world, &alive, &settlements);
    assert_eq!(spawner.calculate_cap(&ctx_near_day), 10);

    // Player outside 150 m radius of camp during day: normal cap (20)
    let ctx_far_day = SpawnContext::new(Vec3::new(200.0, 0.0, 200.0), &world, &alive, &settlements);
    assert_eq!(spawner.calculate_cap(&ctx_far_day), 20);

    // Player near camp at night: camp factor ignored, full night cap (32)
    let ctx_near_night = ctx_near_day.with_night(true);
    assert_eq!(spawner.calculate_cap(&ctx_near_night), 32);
}
