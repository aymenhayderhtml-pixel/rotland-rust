use std::f32::consts::FRAC_PI_2;

use game::bounties::{BountyConfigData, BountyDef, BountyManager, CLEAR_POI};
use game::poi_zombies::{
    PoiSpawn, PoiZombieSite, PoiZombies, POI_ZOMBIES_SAVE_ID, POI_ZOMBIES_SAVE_KEY,
};
use glam::Vec3;

#[test]
fn test_no_spawn_far_away() {
    let mut manager = PoiZombies::new(12345);
    let site = PoiZombieSite::new(
        "cabin_far",
        "cabin",
        Vec3::new(100.0, 0.0, 100.0),
        0.0,
        vec![Vec3::new(1.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)],
    );

    // Player at origin is ~141.42 m away, well past the 35 m approach threshold
    let player = Vec3::new(0.0, 0.0, 0.0);
    let spawns = manager.update(player, &[site], 0);

    assert!(spawns.is_empty(), "expected 0 spawns when far away, got {}", spawns.len());
    assert!(!manager.is_visited("cabin_far"), "building should remain unvisited");
    assert!(!manager.is_cleared("cabin_far"), "building should not be cleared");
}

#[test]
fn test_spawn_count_markers_near() {
    let mut manager = PoiZombies::new(12345);

    // 1. Known building "watchtower" from data/poi.json has zombies [1, 1], so exactly 1 zombie spawns
    let site_watchtower = PoiZombieSite::new(
        "watchtower_near",
        "watchtower",
        Vec3::new(10.0, 5.0, 20.0),
        0.0,
        vec![Vec3::new(2.4, 0.0, 0.0), Vec3::new(-2.4, 0.5, 0.0)],
    );

    let player = Vec3::new(10.0, 5.0, 25.0); // 5 m away (<= 35 m)
    let spawns = manager.update(player, &[site_watchtower], 0);

    assert_eq!(spawns.len(), 1, "watchtower should spawn exactly 1 zombie");
    assert_eq!(spawns[0].poi_id, "watchtower_near");
    let expected_pos = Vec3::new(10.0 + 2.4, 5.0, 20.0);
    assert!(
        spawns[0].position.distance_to(expected_pos) < 1e-4,
        "spawn position should match first marker offset"
    );
    assert!(manager.is_visited("watchtower_near"));
    assert!(!manager.is_cleared("watchtower_near"));

    // 2. Marker rotation by yaw: 90 degrees (FRAC_PI_2) turns local +X (2.0, 0.0, 0.0) into local -Z (0.0, 0.0, -2.0)
    let mut manager_rot = PoiZombies::new(12345);
    let site_rot = PoiZombieSite::new(
        "watchtower_rot",
        "watchtower",
        Vec3::new(10.0, 0.0, 10.0),
        FRAC_PI_2,
        vec![Vec3::new(2.0, 0.0, 0.0)],
    );
    let spawns_rot = manager_rot.update(Vec3::new(10.0, 0.0, 12.0), &[site_rot], 0);
    assert_eq!(spawns_rot.len(), 1);
    let expected_rot_pos = Vec3::new(10.0, 0.0, 8.0);
    assert!(
        spawns_rot[0].position.distance_to(expected_rot_pos) < 1e-4,
        "rotated marker position should match yaw transformation"
    );

    // 3. Custom lot with 2 spawn markers: spawn count equals marker count
    let mut manager_custom = PoiZombies::new(12345);
    let site_custom = PoiZombieSite::new(
        "custom_lot",
        "custom_building",
        Vec3::new(0.0, 0.0, 0.0),
        0.0,
        vec![Vec3::new(1.5, 0.0, 0.0), Vec3::new(-1.5, 0.0, 0.0)],
    );
    let spawns_custom = manager_custom.update(Vec3::new(0.0, 0.0, 5.0), &[site_custom], 0);
    assert_eq!(
        spawns_custom.len(),
        2,
        "custom lot should spawn count matching markers near"
    );
    assert_eq!(spawns_custom[0].position, Vec3::new(1.5, 0.0, 0.0));
    assert_eq!(spawns_custom[1].position, Vec3::new(-1.5, 0.0, 0.0));
}

#[test]
fn test_cap_respected() {
    let mut manager = PoiZombies::new(12345);
    manager.max_active_zombies = 16;

    let site_a = PoiZombieSite::new(
        "site_a",
        "watchtower",
        Vec3::new(10.0, 0.0, 10.0),
        0.0,
        vec![Vec3::new(1.0, 0.0, 0.0)],
    );
    let site_b = PoiZombieSite::new(
        "site_b",
        "watchtower",
        Vec3::new(12.0, 0.0, 10.0),
        0.0,
        vec![Vec3::new(1.0, 0.0, 0.0)],
    );

    let player = Vec3::new(11.0, 0.0, 10.0);

    // When alive_zombies is at cap (16), 16 + 1 > 16, so both buildings wait and remain unvisited
    let spawns_capped = manager.update(player, &[site_a.clone(), site_b.clone()], 16);
    assert!(spawns_capped.is_empty(), "spawns must be suppressed when at cap");
    assert!(!manager.is_visited("site_a"), "site_a must wait unvisited");
    assert!(!manager.is_visited("site_b"), "site_b must wait unvisited");

    // When alive_zombies is 15: site_a can wake (15 + 1 = 16 <= 16), then current_active is 16,
    // so site_b must wait (16 + 1 = 17 > 16)
    let spawns_partial = manager.update(player, &[site_a.clone(), site_b.clone()], 15);
    assert_eq!(spawns_partial.len(), 1, "only site_a should wake");
    assert_eq!(spawns_partial[0].poi_id, "site_a");
    assert!(manager.is_visited("site_a"));
    assert!(!manager.is_visited("site_b"), "site_b must wait because cap was reached");

    // Later when alive drops to 14, site_b wakes too
    let spawns_remaining = manager.update(player, &[site_a, site_b], 14);
    assert_eq!(spawns_remaining.len(), 1);
    assert_eq!(spawns_remaining[0].poi_id, "site_b");
    assert!(manager.is_visited("site_b"));
}

#[test]
fn test_cleared_after_all_die_and_never_respawns() {
    let mut manager = PoiZombies::new(12345);
    let site = PoiZombieSite::new(
        "cabin_clear",
        "custom_pair",
        Vec3::new(0.0, 0.0, 0.0),
        0.0,
        vec![Vec3::new(1.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)],
    );

    let player = Vec3::new(0.0, 0.0, 5.0);
    let spawns = manager.update(player, &[site.clone()], 0);
    assert_eq!(spawns.len(), 2);
    assert!(!manager.is_cleared("cabin_clear"));

    // Register spawned zombies
    manager.on_zombie_spawned("cabin_clear", 101);
    manager.on_zombie_spawned("cabin_clear", 102);
    assert_eq!(manager.remaining_zombies("cabin_clear"), 2);
    assert!(!manager.is_cleared("cabin_clear"));

    // First zombie dies: building is not cleared yet
    let clear_first = manager.on_zombie_died(101);
    assert_eq!(clear_first, None);
    assert_eq!(manager.remaining_zombies("cabin_clear"), 1);
    assert!(!manager.is_cleared("cabin_clear"));

    // Second zombie dies: building is now cleared, returns Some("cabin_clear")
    let clear_second = manager.on_zombie_died(102);
    assert_eq!(clear_second, Some("cabin_clear".to_string()));
    assert_eq!(manager.remaining_zombies("cabin_clear"), 0);
    assert!(manager.is_cleared("cabin_clear"));

    // Player leaves 200 m away, then walks back near: building never respawns
    let player_far = Vec3::new(200.0, 0.0, 0.0);
    let _ = manager.update(player_far, &[site.clone()], 0);

    let spawns_again = manager.update(player, &[site], 0);
    assert!(spawns_again.is_empty(), "cleared building must never wake again");
    assert!(manager.is_cleared("cabin_clear"));
}

#[test]
fn test_save_round_trip() {
    let mut manager = PoiZombies::new(42);

    manager.mark_woken_for_test("cabin_visited", 0);
    manager.mark_woken_for_test("barn_visited", 0);
    assert!(manager.is_cleared("cabin_visited"));
    assert!(manager.is_cleared("barn_visited"));

    let saved = manager.to_save();
    assert_eq!(POI_ZOMBIES_SAVE_ID, "worldstate");
    assert_eq!(POI_ZOMBIES_SAVE_KEY, "visited_pois");

    let list = saved
        .get(POI_ZOMBIES_SAVE_KEY)
        .and_then(|v| v.as_array())
        .expect("visited_pois array in save");
    assert_eq!(list.len(), 2);

    let json_text = serde_json::to_string(&saved).expect("serialize to json");
    let loaded_val: serde_json::Value = serde_json::from_str(&json_text).expect("parse json");

    let mut manager2 = PoiZombies::new(9999);
    assert!(!manager2.is_cleared("cabin_visited"));
    manager2.from_save(&loaded_val);

    assert!(manager2.is_visited("cabin_visited"));
    assert!(manager2.is_cleared("cabin_visited"));
    assert!(manager2.is_visited("barn_visited"));
    assert!(manager2.is_cleared("barn_visited"));
    assert!(!manager2.is_visited("unvisited_lot"));
    assert!(!manager2.is_cleared("unvisited_lot"));
}

#[test]
fn test_sleep_despawn_when_player_leaves() {
    let mut manager = PoiZombies::new(12345);
    manager.despawn_dist = 120.0;

    let site = PoiZombieSite::new(
        "shed_despawn",
        "custom_one",
        Vec3::new(10.0, 0.0, 10.0),
        0.0,
        vec![Vec3::new(0.0, 0.0, 0.0)],
    );

    let spawns = manager.update(Vec3::new(10.0, 0.0, 15.0), &[site.clone()], 0);
    assert_eq!(spawns.len(), 1);
    manager.on_zombie_spawned("shed_despawn", 301);

    // Player walks 150 m away: zombie position (10, 0, 10) vs player (160, 0, 10) -> distance = 150 m > 120 m
    let player_far = Vec3::new(160.0, 0.0, 10.0);
    let to_despawn = manager.check_despawns(player_far, &[(301, Vec3::new(10.0, 0.0, 10.0))]);
    assert_eq!(to_despawn, vec![301]);

    // Despawning releases the zombie and clears the building
    let cleared = manager.on_zombie_despawned(301);
    assert_eq!(cleared, Some("shed_despawn".to_string()));
    assert!(manager.is_cleared("shed_despawn"));

    // Player walks back: shed was visited and cleared, so no new spawns
    let spawns_back = manager.update(Vec3::new(10.0, 0.0, 15.0), &[site], 0);
    assert!(spawns_back.is_empty());
}

#[test]
fn test_bounty_receives_clear_event() {
    let mut bounty_mgr = BountyManager::new(
        12345,
        BountyConfigData {
            max_active: 2,
            ..Default::default()
        },
    );

    let mut clear_bounty = BountyDef::new(
        "bounty_clear_cabin",
        CLEAR_POI,
        "Clear the cabin",
        1,
        100,
        50,
        "combat",
    );
    clear_bounty.poi_id = Some("cabin_target".to_string());
    bounty_mgr.add_active_for_test(clear_bounty);
    assert_eq!(bounty_mgr.active()[0].progress, 0);

    let mut poi_mgr = PoiZombies::new(12345);
    poi_mgr.on_zombie_spawned("cabin_target", 501);

    // Kill the zombie -> triggers clear event
    if let Some(cleared_poi) = poi_mgr.on_zombie_died(501) {
        let changed = bounty_mgr.on_poi_cleared(&cleared_poi);
        assert_eq!(changed, 1);
    } else {
        panic!("expected building to be cleared");
    }

    assert_eq!(bounty_mgr.active()[0].progress, 1);
    assert!(bounty_mgr.active()[0].is_done());
}
