//! Tests for zombie hordes, loose formation steering, HordeSet operations, and special zombies loading.

use glam::Vec3;
use game::director::{DirectorEvent, WaveKind};
use game::horde::*;
use game::zombie::ZombieDefs;

#[test]
fn horde_leader_steers_with_flow_field_and_target_fallback() {
    let mut horde = Horde::new(1, vec![2, 3]);
    horde.set_leader_pos(Vec3::new(10.0, 0.0, 10.0));

    // When flow field provides direction, leader moves in that direction
    let flow_dir = |pos: Vec3| {
        if (pos - Vec3::new(10.0, 0.0, 10.0)).length_squared() < 1.0 {
            Some(Vec3::new(0.0, 0.0, -1.0))
        } else {
            None
        }
    };

    let vel = horde.steer(horde.leader_pos, flow_dir);
    assert!(vel.z < -1.5, "Leader should move forward along flow dir: {vel}");
    assert_eq!(vel.x, 0.0);

    // When flow field returns None, leader falls back to target_pos
    let no_flow = |_pos: Vec3| None;
    horde.set_target_pos(Some(Vec3::new(20.0, 0.0, 10.0)));
    let vel_target = horde.steer(horde.leader_pos, no_flow);
    assert!(vel_target.x > 1.5, "Leader should move towards target_pos (+X): {vel_target}");

    // Also test steer_member with leader id
    let vel_member = horde.steer_member(1, horde.leader_pos, no_flow);
    assert_eq!(vel_member, vel_target);
}

#[test]
fn horde_followers_stay_together_with_bounded_distance() {
    let mut horde = Horde::new(100, vec![101, 102, 103]);
    let mut leader_pos = Vec3::new(0.0, 0.0, 0.0);
    horde.set_leader_pos(leader_pos);

    // Initial follower positions scattered near leader
    let mut follower_positions = [
        Vec3::new(1.0, 0.0, 1.0),
        Vec3::new(-1.0, 0.0, 0.5),
        Vec3::new(0.5, 0.0, -1.2),
    ];

    let flow_dir = |_pos: Vec3| Some(Vec3::new(0.0, 0.0, 1.0)); // Moving along +Z
    let dt = 0.1;

    // Simulate 50 seconds of movement (500 ticks)
    for _ in 0..500 {
        horde.set_leader_pos(leader_pos);
        let leader_vel = horde.steer(leader_pos, flow_dir);
        leader_pos += leader_vel * dt;

        for (i, pos) in follower_positions.iter_mut().enumerate() {
            let follower_id = 101 + i as u64;
            let follower_vel = horde.steer_member(follower_id, *pos, flow_dir);
            *pos += follower_vel * dt;

            // Assert that distance to leader is always strictly bounded
            let dist_to_leader = (*pos - leader_pos).length();
            assert!(
                dist_to_leader <= horde.max_distance,
                "Follower {follower_id} drifted too far from leader: {dist_to_leader} > {}",
                horde.max_distance
            );
        }
    }

    // After 50 seconds, leader moved significantly
    assert!(leader_pos.z > 50.0, "Leader should have moved forward significantly");

    // All followers are still close to leader
    for (i, pos) in follower_positions.iter().enumerate() {
        let dist = (*pos - leader_pos).length();
        assert!(dist < 8.0, "Follower {i} ended at distance {dist} from leader");
    }
}

#[test]
fn horde_set_create_from_wave_and_counts() {
    let mut set = HordeSet::new();
    assert_eq!(set.count(), 0);
    assert_eq!(set.total_zombies(), 0);

    let event = DirectorEvent::WaveStart {
        size: 5,
        kind: WaveKind::City,
    };
    let ids = vec![10, 11, 12, 13, 14, 15];
    let spawn_pos = Vec3::new(50.0, 0.0, -30.0);

    let idx = set.create_from_wave(&event, &ids, spawn_pos).expect("create from wave");
    assert_eq!(idx, 0);
    assert_eq!(set.count(), 1);
    assert_eq!(set.total_zombies(), 5);

    let horde = &set.hordes()[0];
    assert_eq!(horde.leader, 10);
    assert_eq!(horde.members.len(), 4);
    assert_eq!(horde.member_count(), 5);
    assert!(horde.contains(10));
    assert!(horde.contains(14));
    assert!(!horde.contains(99));

    // Also test empty ids
    assert!(set.create_from_wave(&event, &[], spawn_pos).is_none());

    // Test create from CampAttack
    let attack = DirectorEvent::CampAttack { size: 3 };
    let idx2 = set.create_from_wave(&attack, &[20, 21, 22], Vec3::ZERO).expect("camp attack horde");
    assert_eq!(idx2, 1);
    assert_eq!(set.count(), 2);
    assert_eq!(set.total_zombies(), 8);
}

#[test]
fn horde_set_remove_dead_promotes_leader_and_disbands_empty() {
    let mut set = HordeSet::new();
    set.create_horde(1, vec![2, 3], Vec3::ZERO);
    assert_eq!(set.total_zombies(), 3);

    // Remove follower member
    set.remove_dead(3);
    assert_eq!(set.total_zombies(), 2);
    assert!(!set.hordes()[0].contains(3));

    // Remove leader -> promotes member 2 as new leader
    set.remove_dead(1);
    assert_eq!(set.total_zombies(), 1);
    assert_eq!(set.hordes()[0].leader, 2);
    assert!(set.hordes()[0].members.is_empty());

    // Remove last member (current leader) -> disbands horde
    set.remove_dead(2);
    assert_eq!(set.count(), 0);
    assert_eq!(set.total_zombies(), 0);

    // Test remove_dead_with
    set.create_horde(10, vec![11, 12, 13], Vec3::ZERO);
    set.remove_dead_with(|id| id == 12 || id == 13);
    assert_eq!(set.count(), 1);
    assert_eq!(set.hordes()[0].leader, 12);
    assert_eq!(set.hordes()[0].members, vec![13]);
}

#[test]
fn horde_set_merge_when_two_hordes_meet() {
    let mut set = HordeSet::new();
    let h1_idx = set.create_horde(1, vec![2, 3], Vec3::new(0.0, 0.0, 0.0));
    let _h2_idx = set.create_horde(10, vec![11, 12], Vec3::new(4.0, 0.0, 0.0)); // 4 m apart
    let _h3_idx = set.create_horde(20, vec![21], Vec3::new(50.0, 0.0, 0.0)); // 50 m apart

    assert_eq!(set.count(), 3);
    assert_eq!(set.total_zombies(), 8);

    // With merge_distance = 6.0 m, horde 1 and horde 2 meet and merge; horde 3 remains separate
    set.merge_meeting(6.0);
    assert_eq!(set.count(), 2, "Horde 1 and Horde 2 should have merged");
    assert_eq!(set.total_zombies(), 8, "Total zombie count preserved");

    let merged = &set.hordes()[h1_idx];
    assert_eq!(merged.leader, 1);
    assert!(merged.contains(10));
    assert!(merged.contains(11));
    assert!(merged.contains(12));
    assert_eq!(merged.member_count(), 6);

    let distant = &set.hordes()[1];
    assert_eq!(distant.leader, 20);
    assert_eq!(distant.member_count(), 2);

    // Test hordes_mut
    let mut_hordes = set.hordes_mut();
    mut_hordes[0].speed = 3.0;
    assert_eq!(set.hordes()[0].speed, 3.0);
}

#[test]
fn horde_save_state_roundtrip_and_default() {
    let empty: HordeSaveState = serde_json::from_str("{}").expect("empty json parses as default");
    assert_eq!(empty, HordeSaveState::default());

    let mut set = HordeSet::new();
    set.create_horde(1, vec![2, 3], Vec3::new(12.0, 0.0, -5.0));

    let save = set.to_save_state();
    let json = serde_json::to_string(&save).expect("serialize horde save");
    let loaded: HordeSaveState = serde_json::from_str(&json).expect("deserialize horde save");
    assert_eq!(save, loaded);

    let mut restored = HordeSet::new();
    restored.from_save_state(&loaded);
    assert_eq!(restored.count(), 1);
    assert_eq!(restored.hordes()[0].leader, 1);
    assert_eq!(restored.hordes()[0].leader_pos, Vec3::new(12.0, 0.0, -5.0));
}

#[test]
fn specials_load_from_zombies_json() {
    let defs = ZombieDefs::load_default().expect("data/zombies.json must load cleanly");

    // Brute
    let brute = defs.types.get("brute").expect("brute must exist in zombies.json");
    assert_eq!(brute.max_health, 400.0);
    assert_eq!(brute.wander_speed, 0.6);
    assert_eq!(brute.chase_speed, 1.8);
    assert_eq!(brute.attack_damage, 25.0);
    assert_eq!(brute.models, vec!["zombie_brute".to_string()]);

    // Screamer
    let screamer = defs.types.get("screamer").expect("screamer must exist in zombies.json");
    assert_eq!(screamer.max_health, 30.0);
    assert_eq!(screamer.sight_range, 24.0);
    assert_eq!(screamer.hearing_multiplier, 1.5);
    assert_eq!(screamer.models, vec!["zombie_screamer".to_string()]);

    // Crawler
    let crawler = defs.types.get("crawler").expect("crawler must exist in zombies.json");
    assert_eq!(crawler.max_health, 35.0);
    assert_eq!(crawler.chase_speed, 1.2);
    assert_eq!(crawler.models, vec!["zombie_crawler".to_string()]);
    assert_eq!(crawler.clips.get("crawl"), Some(&"zombie_crawler_crawl".to_string()));
    assert_eq!(crawler.clips.get("idle"), Some(&"zombie_crawler_idle".to_string()));
}

#[test]
fn director_multipliers_scale_spawner_cap_and_rate() {
    use physics::PhysicsWorld;
    use game::spawner::*;
    let cfg = SpawnConfig::load_default().expect("data/spawning.json must load");
    let mut spawner = Spawner::new(cfg, 42);

    let physics = PhysicsWorld::new();
    let ctx_normal = SpawnContext::new(Vec3::ZERO, &physics, &[], &[])
        .with_multipliers(1.0, 1.0, 0.0);
    let normal_cap = spawner.calculate_cap(&ctx_normal);

    // Multiplier 1.5 scales cap
    let ctx_boosted = SpawnContext::new(Vec3::ZERO, &physics, &[], &[])
        .with_multipliers(2.0, 1.5, 0.25);
    let boosted_cap = spawner.calculate_cap(&ctx_boosted);
    assert!(boosted_cap > normal_cap, "Boosted cap {boosted_cap} should exceed normal {normal_cap}");

    // Hard cap at 60
    let ctx_extreme = SpawnContext::new(Vec3::ZERO, &physics, &[], &[])
        .with_multipliers(1.0, 10.0, 0.0);
    let extreme_cap = spawner.calculate_cap(&ctx_extreme);
    assert_eq!(extreme_cap, 60, "Cap must never exceed hard cap of 60");

    // Spawn rate multiplier advances timer faster
    spawner.play_time = 100.0; // past start delay
    spawner.spawn_timer = 1.0;
    let _ = spawner.update(0.25, &ctx_boosted); // dt=0.25 with rate_mult=2.0 advances by 0.5
    assert!((spawner.spawn_timer - 0.5).abs() < 1e-4);
}

#[test]
fn screamer_and_crawler_special_behavior_tests() {
    use game::zombie::{ZombieBrain, ZombieState};
    let defs = ZombieDefs::load_default().expect("data/zombies.json must load");

    // Crawler brain initializes as crawler
    let crawler_def = defs.types.get("crawler").unwrap();
    let crawler_brain = ZombieBrain::new(crawler_def, &defs.behaviour, Vec3::ZERO, 100);
    assert!(crawler_brain.is_crawler(), "Crawler brain must initialize with crawler = true");

    // Screamer brain initializes as screamer and screams on first seeing player
    let screamer_def = defs.types.get("screamer").unwrap();
    let screamer_brain = ZombieBrain::new(screamer_def, &defs.behaviour, Vec3::ZERO, 200);
    assert!(screamer_brain.is_screamer(), "Screamer brain must initialize with is_screamer = true");
    assert!(!screamer_brain.has_screamed());

    // Other zombie alerted to chase
    let walker_def = defs.types.get("walker").unwrap();
    let mut bystander = ZombieBrain::new(walker_def, &defs.behaviour, Vec3::new(10.0, 0.0, 0.0), 300);
    assert_eq!(bystander.state(), ZombieState::Idle);

    bystander.alert_chase(Vec3::new(5.0, 0.0, 5.0));
    assert_eq!(bystander.state(), ZombieState::Chase, "Bystander must switch to Chase on alert");
    assert_eq!(bystander.last_seen(), Vec3::new(5.0, 0.0, 5.0));
}

#[test]
fn director_horde_events_emitted() {
    use game::director::*;
    let cfg = DirectorConfig::load_default().expect("data/director.json must load");
    let mut director = Director::new(cfg);

    // Fast forward into Building and then Wave
    let mut input = DirectorInput::new(41.0);
    let out1 = director.update(&input);
    assert!(out1.horde_events.is_empty());

    input.dt = 26.0;
    let out2 = director.update(&input);
    assert!(out2.horde_events.iter().any(|e| matches!(e, HordeEvent::HordeIncoming { .. })));
}

#[test]
fn horde_boids_helpers_test() {
    let v = flat_unit(Vec3::new(3.0, 10.0, 4.0));
    assert!((v.length() - 1.0).abs() < 1e-4);
    assert_eq!(v.y, 0.0);

    let zero = flat_unit(Vec3::ZERO);
    assert_eq!(zero, Vec3::ZERO);

    let leader_vel = steer_leader(Vec3::ZERO, None, 2.0, |_| None);
    assert_eq!(leader_vel, Vec3::ZERO);

    let follower_vel = steer_follower(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        1.5,
        12.0,
        2.0,
        1.0,
        1.5,
        1.2,
    );
    assert!(follower_vel.length() > 0.0);
}
