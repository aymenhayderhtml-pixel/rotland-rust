//! Integration tests for New Halden runtime street herds (`StreetHerds`).

use glam::{Vec2, Vec3};
use game::street_herds::StreetHerds;
use world::metro::herds::{HerdRoute, HerdsConfig, route_pos};
use world::metro::MetroDistrict;

fn sample_route(id: u32, size: u32) -> HerdRoute {
    HerdRoute {
        id,
        points: vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(0.0, 100.0),
        ],
        size,
        speed: 0.55,
        zone: MetroDistrict::Core,
    }
}

#[test]
fn player_far_gives_no_spawn_walking_in_spawns_and_walking_out_despawns() {
    let cfg = HerdsConfig::default();
    let route = sample_route(1, 16);
    let mut herds = StreetHerds::new(vec![route.clone()], cfg.clone());
    let ground = |_x: f32, _z: f32| 10.0;

    // Far outside activeRadius (170 m) -> no spawn.
    let far = Vec3::new(500.0, 10.0, 500.0);
    let orders = herds.update(0.0, far, &|_| 0, &ground);
    assert!(orders.spawn.is_empty());
    assert!(orders.despawn_routes.is_empty());

    // Walk inside activeRadius -> spawns `size` zombies within spreadM of the route.
    let near = Vec3::new(50.0, 10.0, 0.0);
    let orders = herds.update(0.0, near, &|_| 0, &ground);
    assert_eq!(orders.spawn.len(), 1);
    let (route_id, positions) = &orders.spawn[0];
    assert_eq!(*route_id, 1);
    assert_eq!(positions.len(), 16);

    let (leader_xz, _) = route_pos(&route, 0.0);
    for p in positions {
        assert!((p.y - 10.0).abs() < 1e-4);
        let d = Vec2::new(p.x, p.z).distance(leader_xz);
        assert!(d <= cfg.spread_m, "spawn pos {p:?} is {d} m from route (> {})", cfg.spread_m);
    }

    // Walk beyond despawnRadius (210 m) while zombies are alive and not chasing -> despawn.
    let orders = herds.update(1.0, far, &|_| 16, &ground);
    assert!(orders.spawn.is_empty());
    assert_eq!(orders.despawn_routes, vec![1]);
}

#[test]
fn killed_herd_stays_gone_same_day_and_respawns_next_day() {
    let cfg = HerdsConfig::default();
    let mut herds = StreetHerds::new(vec![sample_route(3, 12)], cfg);
    let near = Vec3::new(10.0, 0.0, 0.0);
    let ground = |_: f32, _: f32| 0.0;

    // Spawn on day 1.
    let o1 = herds.update(10.0, near, &|_| 0, &ground);
    assert_eq!(o1.spawn.len(), 1);

    // All killed on day 1 -> marked killed, no respawn later on day 1.
    let o2 = herds.update(20.0, near, &|_| 0, &ground);
    assert!(o2.spawn.is_empty());
    assert!(o2.despawn_routes.is_empty());

    let o3 = herds.update(1000.0, near, &|_| 0, &ground);
    assert!(o3.spawn.is_empty());

    // Next in-game day (1440 s+) -> respawns when player is near.
    let o4 = herds.update(1450.0, near, &|_| 0, &ground);
    assert_eq!(o4.spawn.len(), 1);
}

#[test]
fn save_state_round_trips_killed_days() {
    let cfg = HerdsConfig::default();
    let routes = vec![sample_route(1, 12), sample_route(2, 15)];
    let mut herds = StreetHerds::new(routes.clone(), cfg.clone());
    let near = Vec3::new(0.0, 0.0, 0.0);
    let ground = |_: f32, _: f32| 0.0;

    // Spawn both, kill route 1 on day 1.
    herds.update(10.0, near, &|_| 0, &ground);
    herds.update(20.0, near, &|r| if r == 1 { 0 } else { 15 }, &ground);

    let saved = herds.to_save_state();
    assert_eq!(saved.killed_on_day, vec![(1, 1)]);

    let json = serde_json::to_string(&saved).expect("serialize");
    let decoded = serde_json::from_str(&json).expect("deserialize");

    let mut restored = StreetHerds::new(routes, cfg);
    restored.from_save_state(&decoded);

    // On day 1, route 1 stays dead while route 2 spawns.
    let orders = restored.update(30.0, near, &|_| 0, &ground);
    let spawned_ids: Vec<u32> = orders.spawn.iter().map(|(id, _)| *id).collect();
    assert_eq!(spawned_ids, vec![2]);
}
