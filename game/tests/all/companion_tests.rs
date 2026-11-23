//! Unit and integration tests for companion AI, orders, roles, recovery, and config loading.

use glam::Vec3;

use game::guild::companion::{
    CompanionAnim, CompanionConfig, CompanionDraw, CompanionEvent, CompanionSquad, CompanionTarget,
    CompanionWorld,
};
use game::guild::{GuildRecruit, RecruitOrder, RecruitRole, RecruitState};

fn load_cfg() -> CompanionConfig {
    CompanionConfig::from_repo().expect("data/companions.json should load")
}

fn make_recruit(id: &str, name: &str, role: RecruitRole, order: RecruitOrder) -> GuildRecruit {
    let mut r = GuildRecruit::new(id, name, role, 1, 15, 100, 30); // new: DECISIONS 2026-11-23
    r.set_order(order);
    r
}

fn free_move(from: Vec3, step: Vec3) -> Vec3 {
    from + step
}

fn draw_for(squad: &CompanionSquad, id: &str) -> CompanionDraw {
    squad
        .draws()
        .find(|d| d.id == id)
        .expect("companion should exist")
}

#[test]
fn companions_json_loads_with_expected_keys() {
    let cfg = load_cfg();
    assert!((cfg.walk_speed - 2.4).abs() < 1e-4); // data: companions.json walkSpeed
    assert!((cfg.run_speed - 4.6).abs() < 1e-4); // data: companions.json runSpeed
    assert!((cfg.teleport_distance - 45.0).abs() < 1e-4); // data: companions.json teleportDistance
    assert!((cfg.guard_leash - 20.0).abs() < 1e-4); // data: companions.json guardLeash
    assert!((cfg.damage[&RecruitRole::Fighter] - 22.0).abs() < 1e-4); // data: companions.json damage.fighter
    assert!((cfg.damage_taken_multiplier[&RecruitRole::Guard] - 0.7).abs() < 1e-4); // data: companions.json damageTakenMultiplier.guard
}

#[test]
fn follow_reaches_spot_and_stops() {
    let cfg = load_cfg();
    let mut squad = CompanionSquad::new(cfg.clone());
    let hired = [make_recruit("m1", "Mark", RecruitRole::Fighter, RecruitOrder::Follow)];
    squad.sync(&hired, Vec3::ZERO);

    let world = CompanionWorld {
        player: Vec3::new(0.0, 0.0, -5.0), // new: DECISIONS 2026-11-23
        player_forward: Vec3::new(0.0, 0.0, -1.0),
        camp: Vec3::ZERO,
        zombies: &[],
    };
    let mut events = Vec::new();
    let dt = 0.05; // new: DECISIONS 2026-11-23
    for _ in 0..120 {
        squad.tick(&world, &mut free_move, dt, &mut events);
    }

    let d = draw_for(&squad, "m1");
    let expected_spot = Vec3::new(cfg.follow_distance, 0.0, -5.0 + cfg.follow_distance * 0.3); // beside the player
    assert!(
        d.position.distance(expected_spot) < 0.35, // new: DECISIONS 2026-11-23
        "expected companion near {expected_spot:?}, got {:?}",
        d.position
    );
    assert_eq!(d.anim, CompanionAnim::Idle);
}

#[test]
fn two_followers_get_different_spots() {
    let cfg = load_cfg();
    let mut squad = CompanionSquad::new(cfg.clone());
    let hired = [
        make_recruit("m1", "Mark", RecruitRole::Fighter, RecruitOrder::Follow),
        make_recruit("m2", "Elena", RecruitRole::Guard, RecruitOrder::Follow),
    ];
    squad.sync(&hired, Vec3::ZERO);

    let world = CompanionWorld {
        player: Vec3::new(0.0, 0.0, -4.0), // new: DECISIONS 2026-11-23
        player_forward: Vec3::new(0.0, 0.0, -1.0),
        camp: Vec3::ZERO,
        zombies: &[],
    };
    let mut events = Vec::new();
    let dt = 0.05; // new: DECISIONS 2026-11-23
    for _ in 0..120 {
        squad.tick(&world, &mut free_move, dt, &mut events);
    }

    let d1 = draw_for(&squad, "m1");
    let d2 = draw_for(&squad, "m2");
    let sep = d1.position.distance(d2.position);
    assert!(
        sep >= cfg.follow_spread * 0.75, // new: DECISIONS 2026-11-23
        "two followers should not stack; separation was {sep}"
    );
    assert_eq!(d1.anim, CompanionAnim::Idle);
    assert_eq!(d2.anim, CompanionAnim::Idle);
}

#[test]
fn follower_runs_when_far_and_teleports_when_very_far_out_of_view() {
    let cfg = load_cfg();
    let mut squad = CompanionSquad::new(cfg.clone());
    let hired = [make_recruit("m1", "Mark", RecruitRole::Fighter, RecruitOrder::Follow)];
    squad.sync(&hired, Vec3::ZERO);

    let mut events = Vec::new();
    // Far (> runDistance = 7) but below teleportDistance: must run.
    let mid_world = CompanionWorld {
        player: Vec3::new(20.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
        player_forward: Vec3::new(1.0, 0.0, 0.0),
        camp: Vec3::ZERO,
        zombies: &[],
    };
    let before = draw_for(&squad, "m1").position;
    let dt = 0.1; // new: DECISIONS 2026-11-23
    squad.tick(&mid_world, &mut free_move, dt, &mut events);
    let after = draw_for(&squad, "m1");
    assert_eq!(after.anim, CompanionAnim::Run);
    let moved = after.position.distance(before);
    assert!(
        moved > cfg.walk_speed * dt * 1.2, // new: DECISIONS 2026-11-23
        "expected running speed, moved {moved}"
    );

    // Very far (> teleportDistance = 45) and behind player's view direction: must teleport near player.
    let far_player = Vec3::new(100.0, 0.0, 0.0); // new: DECISIONS 2026-11-23
    let far_world = CompanionWorld {
        player: far_player,
        player_forward: Vec3::new(1.0, 0.0, 0.0),
        camp: Vec3::ZERO,
        zombies: &[],
    };
    squad.tick(&far_world, &mut free_move, dt, &mut events);
    let teleported = draw_for(&squad, "m1");
    assert!(
        teleported.position.distance(far_player) < cfg.run_distance,
        "companion should teleport near player, got {:?}",
        teleported.position
    );
}

#[test]
fn guard_stays_within_leash() {
    let cfg = load_cfg();
    let mut squad = CompanionSquad::new(cfg.clone());
    let hired = [make_recruit("g1", "Elena", RecruitRole::Guard, RecruitOrder::Guard)];
    let camp = Vec3::ZERO;
    squad.sync(&hired, camp);

    let mut events = Vec::new();
    let dt = 0.05; // new: DECISIONS 2026-11-23

    // Zombie starts inside guardRadius (14m) so guard engages.
    let near_z = [CompanionTarget {
        id: 1,
        position: Vec3::new(12.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
    }];
    let w1 = CompanionWorld {
        player: Vec3::new(-10.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
        player_forward: Vec3::new(0.0, 0.0, -1.0),
        camp,
        zombies: &near_z,
    };
    for _ in 0..40 {
        squad.tick(&w1, &mut free_move, dt, &mut events);
    }

    // Now kite the zombie beyond guardLeash (20m): guard must never leave guardLeash.
    let far_z = [CompanionTarget {
        id: 1,
        position: Vec3::new(28.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
    }];
    let w2 = CompanionWorld {
        player: Vec3::new(-10.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
        player_forward: Vec3::new(0.0, 0.0, -1.0),
        camp,
        zombies: &far_z,
    };
    for _ in 0..120 {
        squad.tick(&w2, &mut free_move, dt, &mut events);
        let d = draw_for(&squad, "g1");
        let dist_from_camp = d.position.distance(camp);
        assert!(
            dist_from_camp <= cfg.guard_leash + 1e-3,
            "guard exceeded leash: {dist_from_camp} > {}",
            cfg.guard_leash
        );
    }
}

#[test]
fn stay_does_not_wander_and_only_defends_in_attack_range() {
    let cfg = load_cfg();
    let mut squad = CompanionSquad::new(cfg.clone());
    let hired = [make_recruit("s1", "Mark", RecruitRole::Fighter, RecruitOrder::Stay)];
    squad.sync(&hired, Vec3::ZERO);

    let start_pos = draw_for(&squad, "s1").position;
    let far_zombie = [CompanionTarget {
        id: 1,
        position: start_pos + Vec3::new(5.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
    }];
    let world = CompanionWorld {
        player: Vec3::new(20.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
        player_forward: Vec3::new(0.0, 0.0, -1.0),
        camp: Vec3::ZERO,
        zombies: &far_zombie,
    };
    let mut events = Vec::new();
    let dt = 0.05; // new: DECISIONS 2026-11-23
    for _ in 0..100 {
        squad.tick(&world, &mut free_move, dt, &mut events);
    }

    let d = draw_for(&squad, "s1");
    assert!(d.position.distance(start_pos) < 1e-3);
    assert_eq!(d.anim, CompanionAnim::Idle);
    assert!(events.is_empty());
}

#[test]
fn fighter_attacks_after_arriving_in_range_and_respects_cooldown() {
    let cfg = load_cfg();
    let mut squad = CompanionSquad::new(cfg.clone());
    let hired = [make_recruit("f1", "Mark", RecruitRole::Fighter, RecruitOrder::Follow)];
    squad.sync(&hired, Vec3::ZERO);

    let start_pos = draw_for(&squad, "f1").position;
    // Place zombie 5m away (outside attackRange 1.7, inside fightRange 11).
    let zombies = [CompanionTarget {
        id: 7,
        position: start_pos + Vec3::new(5.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
    }];
    let world = CompanionWorld {
        player: Vec3::ZERO,
        player_forward: Vec3::new(1.0, 0.0, 0.0),
        camp: Vec3::ZERO,
        zombies: &zombies,
    };

    let mut events = Vec::new();
    let mut hit_times = Vec::new();
    let dt = 0.05; // new: DECISIONS 2026-11-23
    for step in 1..=80 {
        events.clear();
        squad.tick(&world, &mut free_move, dt, &mut events);
        for ev in &events {
            if let CompanionEvent::Hit {
                companion,
                zombie_id,
                damage,
            } = ev
            {
                assert_eq!(companion, "f1");
                assert_eq!(*zombie_id, 7);
                assert!((*damage - cfg.damage[&RecruitRole::Fighter]).abs() < 1e-4);
                let dist = draw_for(&squad, "f1").position.distance(zombies[0].position);
                assert!(dist <= cfg.attack_range + 0.05);
                hit_times.push(step as f32 * dt);
            }
        }
    }

    assert!(hit_times.len() >= 2, "fighter should land multiple hits");
    assert!(
        hit_times[0] > 0.5, // new: DECISIONS 2026-11-23
        "first hit must wait until fighter reaches attackRange, happened at {}",
        hit_times[0]
    );
    let interval = hit_times[1] - hit_times[0];
    assert!(
        interval >= cfg.attack_cooldown - dt,
        "hits must respect attackCooldown ({}), got {interval}",
        cfg.attack_cooldown
    );
}

#[test]
fn scavenger_backs_away_toward_player_when_zombie_approaches() {
    let cfg = load_cfg();
    let mut squad = CompanionSquad::new(cfg.clone());
    let hired = [make_recruit("sc1", "Sarah", RecruitRole::Scavenger, RecruitOrder::Follow)];
    squad.sync(&hired, Vec3::new(5.0, 0.0, 0.0)); // new: DECISIONS 2026-11-23

    let start_pos = draw_for(&squad, "sc1").position;
    let player = Vec3::ZERO;
    let zombies = [CompanionTarget {
        id: 3,
        position: start_pos + Vec3::new(2.5, 0.0, 0.0), // new: DECISIONS 2026-11-23
    }];
    let world = CompanionWorld {
        player,
        player_forward: Vec3::new(1.0, 0.0, 0.0),
        camp: Vec3::ZERO,
        zombies: &zombies,
    };

    let mut events = Vec::new();
    let dt = 0.05; // new: DECISIONS 2026-11-23
    for _ in 0..25 {
        squad.tick(&world, &mut free_move, dt, &mut events);
    }

    let end_pos = draw_for(&squad, "sc1").position;
    assert!(
        end_pos.distance(zombies[0].position) > start_pos.distance(zombies[0].position),
        "scavenger should back away from zombie"
    );
    assert!(
        end_pos.distance(player) < start_pos.distance(player),
        "scavenger should retreat toward player"
    );
    assert!(events.is_empty(), "non-cornered scavenger should not attack");
}

#[test]
fn downed_stops_acting_excluded_from_targets_recovers_on_morning_and_saved_health_never_zero() {
    let cfg = load_cfg();
    let mut squad = CompanionSquad::new(cfg.clone());
    let hired = [make_recruit("m1", "Mark", RecruitRole::Fighter, RecruitOrder::Follow)];
    let camp = Vec3::new(2.0, 0.0, 2.0); // new: DECISIONS 2026-11-23
    squad.sync(&hired, camp);

    let mut events = Vec::new();
    let mut live_targets = Vec::new();
    squad.targets(&mut live_targets);
    assert_eq!(live_targets.len(), 1);

    // Lethal damage downs the teammate instead of killing it.
    squad.damage("m1", 999.0, &mut events); // new: DECISIONS 2026-11-23
    assert_eq!(
        events,
        vec![CompanionEvent::Downed {
            companion: "m1".to_string()
        }]
    );
    assert_eq!(draw_for(&squad, "m1").anim, CompanionAnim::Downed);
    assert!(squad.saved_health("m1").unwrap() >= 1);

    // Downed teammate is excluded from zombie targets and does not move or fight.
    squad.targets(&mut live_targets);
    assert!(live_targets.is_empty());

    let downed_pos = draw_for(&squad, "m1").position;
    let world = CompanionWorld {
        player: Vec3::new(15.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
        player_forward: Vec3::new(1.0, 0.0, 0.0),
        camp,
        zombies: &[CompanionTarget {
            id: 1,
            position: downed_pos + Vec3::new(0.5, 0.0, 0.0), // new: DECISIONS 2026-11-23
        }],
    };
    events.clear();
    squad.tick(&world, &mut free_move, 0.1, &mut events); // new: DECISIONS 2026-11-23
    assert!(events.is_empty());
    assert_eq!(draw_for(&squad, "m1").position, downed_pos);

    // Next morning recovers at camp with recoverHealthFraction health.
    squad.on_morning(camp, &mut events);
    assert_eq!(
        events,
        vec![CompanionEvent::Recovered {
            companion: "m1".to_string()
        }]
    );
    let recovered = draw_for(&squad, "m1");
    assert_eq!(recovered.anim, CompanionAnim::Idle);
    assert!((recovered.health_fraction - cfg.recover_health_fraction).abs() < 1e-4);
    assert_eq!(squad.saved_health("m1"), Some(50)); // data: companions.json recoverHealthFraction
    squad.targets(&mut live_targets);
    assert_eq!(live_targets.len(), 1);
}

#[test]
fn sync_adds_removes_and_updates_orders() {
    let cfg = load_cfg();
    let mut squad = CompanionSquad::new(cfg);
    let mut r1 = make_recruit("r1", "Mark", RecruitRole::Fighter, RecruitOrder::Stay);
    let r2 = make_recruit("r2", "Sarah", RecruitRole::Scavenger, RecruitOrder::Follow);

    squad.sync(&[r1.clone(), r2], Vec3::ZERO);
    assert_eq!(squad.draws().count(), 2);

    // Dismiss r2 and switch r1 from Stay to Follow.
    r1.state = RecruitState::FollowingPlayer;
    squad.sync(&[r1], Vec3::ZERO);
    assert_eq!(squad.draws().count(), 1);
    assert_eq!(draw_for(&squad, "r1").id, "r1");

    // Verify r1 now follows the player instead of staying at camp.
    let world = CompanionWorld {
        player: Vec3::new(12.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
        player_forward: Vec3::new(1.0, 0.0, 0.0),
        camp: Vec3::ZERO,
        zombies: &[],
    };
    let before = draw_for(&squad, "r1").position;
    let mut events = Vec::new();
    squad.tick(&world, &mut free_move, 0.1, &mut events); // new: DECISIONS 2026-11-23
    assert!(draw_for(&squad, "r1").position.x > before.x);
}

#[test]
fn vanished_target_is_handled_safely() {
    let cfg = load_cfg();
    let mut squad = CompanionSquad::new(cfg);
    let hired = [make_recruit("f1", "Mark", RecruitRole::Fighter, RecruitOrder::Follow)];
    squad.sync(&hired, Vec3::ZERO);

    let zombies = [CompanionTarget {
        id: 42,
        position: Vec3::new(4.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
    }];
    let w_with = CompanionWorld {
        player: Vec3::ZERO,
        player_forward: Vec3::new(1.0, 0.0, 0.0),
        camp: Vec3::ZERO,
        zombies: &zombies,
    };
    let mut events = Vec::new();
    squad.tick(&w_with, &mut free_move, 0.05, &mut events); // new: DECISIONS 2026-11-23

    // Zombie disappears before companion arrives.
    let w_empty = CompanionWorld {
        player: Vec3::ZERO,
        player_forward: Vec3::new(1.0, 0.0, 0.0),
        camp: Vec3::ZERO,
        zombies: &[],
    };
    events.clear();
    for _ in 0..20 {
        squad.tick(&w_empty, &mut free_move, 0.05, &mut events); // new: DECISIONS 2026-11-23
    }
    assert!(events.is_empty());
    let d = draw_for(&squad, "f1");
    assert!(d.position.is_finite());
    assert!(d.yaw.is_finite());
}

#[test]
fn identical_positions_and_dt_range_produce_no_nan() {
    let cfg = load_cfg();
    let mut squad = CompanionSquad::new(cfg);
    let hired = [
        make_recruit("m1", "Mark", RecruitRole::Fighter, RecruitOrder::Follow),
        make_recruit("m2", "Sarah", RecruitRole::Scavenger, RecruitOrder::Follow),
        make_recruit("m3", "Elena", RecruitRole::Guard, RecruitOrder::Guard),
    ];
    squad.sync(&hired, Vec3::ZERO);

    // Force every companion onto Vec3::ZERO first to create exact overlap with player and zombie.
    let zombie = [CompanionTarget {
        id: 1,
        position: Vec3::ZERO,
    }];
    let world = CompanionWorld {
        player: Vec3::ZERO,
        player_forward: Vec3::ZERO,
        camp: Vec3::ZERO,
        zombies: &zombie,
    };
    let mut events = Vec::new();
    squad.tick(&world, &mut |_, _| Vec3::ZERO, 0.016, &mut events); // new: DECISIONS 2026-11-23

    for dt in [0.001_f32, 0.016, 0.05, 0.1] {
        // new: DECISIONS 2026-11-23
        for _ in 0..10 {
            squad.tick(&world, &mut free_move, dt, &mut events);
            for d in squad.draws() {
                assert!(d.position.is_finite(), "non-finite position at dt={dt}");
                assert!(d.yaw.is_finite(), "non-finite yaw at dt={dt}");
                assert!(
                    d.health_fraction.is_finite(),
                    "non-finite health_fraction at dt={dt}"
                );
            }
        }
    }

    // Verify set_health and passive healing stay finite and bounded.
    squad.set_health("m1", 50); // new: DECISIONS 2026-11-23
    assert_eq!(squad.saved_health("m1"), Some(50));
}
