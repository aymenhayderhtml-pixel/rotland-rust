//! Unit and integration tests for zombie base siege logic and data balance.

use glam::{IVec2, Vec3};
use serde_json::Value;

use game::building::siege::{SiegeAttacker, SiegeConfig, SiegeEvent, SiegeSystem};
use game::building::{BaseState, BuildConfig, PlacedPart};

fn load_build() -> BuildConfig {
    BuildConfig::load_default().expect("data/build_parts.json should load")
}

fn load_siege() -> SiegeConfig {
    SiegeConfig::from_repo().expect("data/siege.json should load")
}

fn place_part(
    base: &mut BaseState,
    build: &BuildConfig,
    part_name: &str,
    pos: Vec3,
    hp_override: Option<f32>,
) -> u32 {
    let default_hp = build
        .get(part_name)
        .map(|d| d.hp)
        .expect("part should exist in build_parts.json");
    base.place_part(PlacedPart {
        part: part_name.to_string(),
        position: pos,
        hp: hp_override.unwrap_or(default_hp),
        ..Default::default()
    })
}

#[test]
fn walker_damages_blocking_wall_on_hit_interval_not_every_frame() {
    let build = load_build();
    let cfg = load_siege();
    let mut base = BaseState::default();
    let wall_id = place_part(&mut base, &build, "wall", Vec3::new(1.0, 0.0, 0.0), None); // new: DECISIONS 2026-11-23
    let start_hp = base.get(wall_id).expect("wall exists").hp;

    let mut sys = SiegeSystem::new(cfg.clone());
    let player = Vec3::new(5.0, 0.0, 0.0); // new: DECISIONS 2026-11-23
    let attackers = [SiegeAttacker {
        id: 1,
        kind: "walker",
        position: Vec3::ZERO,
        chasing: true,
    }];

    // Sub-interval frames should not deal damage yet.
    let small_dt = 0.2; // new: DECISIONS 2026-11-23
    for _ in 0..5 {
        let frame = sys.tick(&mut base, &build, &attackers, player, small_dt);
        assert!(frame.events.is_empty());
        assert!(!frame.geometry_changed);
        assert!(!sys.under_attack());
    }
    assert!((base.get(wall_id).unwrap().hp - start_hp).abs() < 1e-4);

    // Reaching hitInterval (1.2s total) deals exactly one walker hit.
    let frame = sys.tick(&mut base, &build, &attackers, player, small_dt);
    let expected = cfg.damage_for("walker");
    assert_eq!(
        frame.events,
        vec![SiegeEvent::Hit {
            attacker: 1,
            part_id: wall_id,
            amount: expected,
        }]
    );
    assert!((base.get(wall_id).unwrap().hp - (start_hp - expected)).abs() < 1e-4);
    assert!(sys.under_attack());
    assert!((sys.alert_fraction() - 1.0).abs() < 1e-4);

    // Immediately after a hit, the next frame must not hit again.
    let next = sys.tick(&mut base, &build, &attackers, player, small_dt);
    assert!(next.events.is_empty());
}

#[test]
fn brute_hits_harder_than_walker() {
    let build = load_build();
    let cfg = load_siege();
    let dt = cfg.hit_interval;
    let player = Vec3::new(5.0, 0.0, 0.0); // new: DECISIONS 2026-11-23

    let mut walker_base = BaseState::default();
    let w_wall = place_part(
        &mut walker_base,
        &build,
        "wall",
        Vec3::new(1.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
        None,
    );
    let mut walker_sys = SiegeSystem::new(cfg.clone());
    let walker = [SiegeAttacker {
        id: 1,
        kind: "walker",
        position: Vec3::ZERO,
        chasing: true,
    }];
    let w_frame = walker_sys.tick(&mut walker_base, &build, &walker, player, dt);

    let mut brute_base = BaseState::default();
    let b_wall = place_part(
        &mut brute_base,
        &build,
        "wall",
        Vec3::new(1.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
        None,
    );
    let mut brute_sys = SiegeSystem::new(cfg.clone());
    let brute = [SiegeAttacker {
        id: 2,
        kind: "brute",
        position: Vec3::ZERO,
        chasing: true,
    }];
    let b_frame = brute_sys.tick(&mut brute_base, &build, &brute, player, dt);

    let w_dmg = match w_frame.events[0] {
        SiegeEvent::Hit { amount, .. } => amount,
        _ => panic!("expected Hit"),
    };
    let b_dmg = match b_frame.events[0] {
        SiegeEvent::Hit { amount, .. } => amount,
        _ => panic!("expected Hit"),
    };

    assert!(b_dmg > w_dmg);
    assert!((b_dmg - cfg.damage_for("brute")).abs() < 1e-4);
    assert!(brute_base.get(b_wall).unwrap().hp < walker_base.get(w_wall).unwrap().hp);
}

#[test]
fn closed_door_takes_multiplier_and_open_door_is_ignored() {
    let build = load_build();
    let cfg = load_siege();
    let mut base = BaseState::default();
    let door_id = place_part(
        &mut base,
        &build,
        "wall_door",
        Vec3::new(1.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
        None,
    );
    let start_hp = base.get(door_id).unwrap().hp;

    let mut sys = SiegeSystem::new(cfg.clone());
    let player = Vec3::new(5.0, 0.0, 0.0); // new: DECISIONS 2026-11-23
    let attackers = [SiegeAttacker {
        id: 1,
        kind: "walker",
        position: Vec3::ZERO,
        chasing: true,
    }];

    // Closed door takes walker damage * doorMultiplier.
    let frame = sys.tick(&mut base, &build, &attackers, player, cfg.hit_interval);
    let expected = cfg.damage_for("walker") * cfg.door_multiplier;
    assert_eq!(
        frame.events,
        vec![SiegeEvent::Hit {
            attacker: 1,
            part_id: door_id,
            amount: expected,
        }]
    );
    let hp_after_hit = base.get(door_id).unwrap().hp;
    assert!((hp_after_hit - (start_hp - expected)).abs() < 1e-4);

    // Opening the door makes it non-blocking, so it cannot be hit.
    assert!(base.toggle_door(door_id));
    let open_frame = sys.tick(&mut base, &build, &attackers, player, cfg.hit_interval);
    assert!(open_frame.events.is_empty());
    assert!((base.get(door_id).unwrap().hp - hp_after_hit).abs() < 1e-4);
}

#[test]
fn part_breaks_once_and_emits_broken_and_geometry_changed() {
    let build = load_build();
    let cfg = load_siege();
    let mut base = BaseState::default();
    let wall_pos = Vec3::new(1.0, 0.0, 0.0); // new: DECISIONS 2026-11-23
    let low_hp = 5.0; // new: DECISIONS 2026-11-23
    let wall_id = place_part(&mut base, &build, "wall", wall_pos, Some(low_hp));

    let mut sys = SiegeSystem::new(cfg.clone());
    let player = Vec3::new(5.0, 0.0, 0.0); // new: DECISIONS 2026-11-23
    let attackers = [SiegeAttacker {
        id: 1,
        kind: "brute",
        position: Vec3::ZERO,
        chasing: true,
    }];

    let frame = sys.tick(&mut base, &build, &attackers, player, cfg.hit_interval);
    assert!(frame.geometry_changed);
    assert!(base.get(wall_id).is_none());
    assert!(base.is_empty());

    let broken_events: Vec<_> = frame
        .events
        .iter()
        .filter(|e| matches!(e, SiegeEvent::Broken { .. }))
        .collect();
    assert_eq!(
        broken_events,
        vec![&SiegeEvent::Broken {
            part_id: wall_id,
            position: wall_pos,
        }]
    );

    // Subsequent tick on empty base does nothing and still counts alert down.
    let alert_before = sys.alert_fraction();
    let empty_frame = sys.tick(&mut base, &build, &attackers, player, 1.0); // new: DECISIONS 2026-11-23
    assert!(empty_frame.events.is_empty());
    assert!(!empty_frame.geometry_changed);
    assert!(sys.alert_fraction() < alert_before);
}

#[test]
fn breaking_support_wall_reports_collapsed_upper_wall() {
    let build = load_build();
    let cfg = load_siege();
    let mut base = BaseState::default();

    // Floor-0 wall at cell (0, 0), rot 0 -> position (0, 0, -1).
    let lower_id = base.place_part(PlacedPart {
        part: "wall".to_string(),
        cell: IVec2::ZERO,
        floor: 0,
        rot: 0,
        hp: 10.0, // new: DECISIONS 2026-11-23
        position: Vec3::new(0.0, 0.0, -1.0), // new: DECISIONS 2026-11-23
        ..Default::default()
    });
    // Floor-1 wall directly above, supported only by lower_id (no floors present).
    let upper_id = base.place_part(PlacedPart {
        part: "wall".to_string(),
        cell: IVec2::ZERO,
        floor: 1,
        rot: 0,
        hp: 150.0, // data: build_parts.json parts.wall.hp
        position: Vec3::new(0.0, 3.0, -1.0), // data: build_parts.json floorHeight
        ..Default::default()
    });

    let mut sys = SiegeSystem::new(cfg.clone());
    let player = Vec3::new(0.0, 0.0, -5.0); // new: DECISIONS 2026-11-23
    let attackers = [SiegeAttacker {
        id: 1,
        kind: "brute",
        position: Vec3::ZERO,
        chasing: true,
    }];

    let frame = sys.tick(&mut base, &build, &attackers, player, cfg.hit_interval);
    assert!(frame.geometry_changed);
    assert!(base.get(lower_id).is_none());
    assert!(base.get(upper_id).is_none());
    assert!(frame.events.contains(&SiegeEvent::Broken {
        part_id: lower_id,
        position: Vec3::new(0.0, 0.0, -1.0),
    }));
    assert!(frame.events.contains(&SiegeEvent::Collapsed {
        part_id: upper_id,
    }));
}

#[test]
fn zombie_beyond_active_range_does_nothing() {
    let build = load_build();
    let cfg = load_siege();
    let mut base = BaseState::default();
    let wall_id = place_part(
        &mut base,
        &build,
        "wall",
        Vec3::new(1.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
        None,
    );
    let start_hp = base.get(wall_id).unwrap().hp;

    let mut sys = SiegeSystem::new(cfg.clone());
    let player = Vec3::new(cfg.active_range + 5.0, 0.0, 0.0); // new: DECISIONS 2026-11-23
    let attackers = [SiegeAttacker {
        id: 1,
        kind: "brute",
        position: Vec3::ZERO,
        chasing: true,
    }];

    let frame = sys.tick(&mut base, &build, &attackers, player, cfg.hit_interval);
    assert!(frame.events.is_empty());
    assert!(!sys.under_attack());
    assert!((base.get(wall_id).unwrap().hp - start_hp).abs() < 1e-4);
}

#[test]
fn non_chasing_zombie_does_nothing() {
    let build = load_build();
    let cfg = load_siege();
    let mut base = BaseState::default();
    let wall_id = place_part(
        &mut base,
        &build,
        "wall",
        Vec3::new(1.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
        None,
    );
    let start_hp = base.get(wall_id).unwrap().hp;

    let mut sys = SiegeSystem::new(cfg.clone());
    let player = Vec3::new(5.0, 0.0, 0.0); // new: DECISIONS 2026-11-23
    let attackers = [SiegeAttacker {
        id: 1,
        kind: "brute",
        position: Vec3::ZERO,
        chasing: false,
    }];

    let frame = sys.tick(&mut base, &build, &attackers, player, cfg.hit_interval);
    assert!(frame.events.is_empty());
    assert!((base.get(wall_id).unwrap().hp - start_hp).abs() < 1e-4);
}

#[test]
fn wall_behind_zombie_not_toward_player_is_not_hit() {
    let build = load_build();
    let cfg = load_siege();
    let mut base = BaseState::default();
    let behind_id = place_part(
        &mut base,
        &build,
        "wall",
        Vec3::new(-1.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
        None,
    );
    let start_hp = base.get(behind_id).unwrap().hp;

    let mut sys = SiegeSystem::new(cfg.clone());
    let player = Vec3::new(5.0, 0.0, 0.0); // new: DECISIONS 2026-11-23
    let attackers = [SiegeAttacker {
        id: 1,
        kind: "brute",
        position: Vec3::ZERO,
        chasing: true,
    }];

    let frame = sys.tick(&mut base, &build, &attackers, player, cfg.hit_interval);
    assert!(frame.events.is_empty());
    assert!((base.get(behind_id).unwrap().hp - start_hp).abs() < 1e-4);
}

#[test]
fn max_attackers_caps_hits_and_prioritizes_nearest_to_player() {
    let build = load_build();
    let mut cfg = load_siege();
    cfg.max_attackers = 2; // new: DECISIONS 2026-11-23

    let mut base = BaseState::default();
    let _wall = place_part(
        &mut base,
        &build,
        "wall",
        Vec3::new(1.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
        None,
    );

    let mut sys = SiegeSystem::new(cfg.clone());
    let player = Vec3::new(5.0, 0.0, 0.0); // new: DECISIONS 2026-11-23
    // Attacker 10 is farthest from player, 20 and 30 are closer.
    let attackers = [
        SiegeAttacker {
            id: 10,
            kind: "walker",
            position: Vec3::new(-0.4, 0.0, 0.0), // new: DECISIONS 2026-11-23
            chasing: true,
        },
        SiegeAttacker {
            id: 20,
            kind: "walker",
            position: Vec3::new(0.2, 0.0, 0.0), // new: DECISIONS 2026-11-23
            chasing: true,
        },
        SiegeAttacker {
            id: 30,
            kind: "walker",
            position: Vec3::new(0.5, 0.0, 0.0), // new: DECISIONS 2026-11-23
            chasing: true,
        },
    ];

    let frame = sys.tick(&mut base, &build, &attackers, player, cfg.hit_interval);
    let hit_ids: Vec<u32> = frame
        .events
        .iter()
        .filter_map(|e| match e {
            SiegeEvent::Hit { attacker, .. } => Some(*attacker),
            _ => None,
        })
        .collect();
    assert_eq!(hit_ids, vec![30, 20]);
}

#[test]
fn two_attackers_on_breaking_part_do_not_double_report() {
    let build = load_build();
    let cfg = load_siege();
    let mut base = BaseState::default();
    let wall_id = place_part(
        &mut base,
        &build,
        "wall",
        Vec3::new(1.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
        Some(5.0),                // new: DECISIONS 2026-11-23
    );

    let mut sys = SiegeSystem::new(cfg.clone());
    let player = Vec3::new(5.0, 0.0, 0.0); // new: DECISIONS 2026-11-23
    let attackers = [
        SiegeAttacker {
            id: 1,
            kind: "brute",
            position: Vec3::new(0.2, 0.0, 0.0), // new: DECISIONS 2026-11-23
            chasing: true,
        },
        SiegeAttacker {
            id: 2,
            kind: "brute",
            position: Vec3::new(0.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
            chasing: true,
        },
    ];

    let frame = sys.tick(&mut base, &build, &attackers, player, cfg.hit_interval);
    let hits = frame
        .events
        .iter()
        .filter(|e| matches!(e, SiegeEvent::Hit { part_id, .. } if *part_id == wall_id))
        .count();
    let breaks = frame
        .events
        .iter()
        .filter(|e| matches!(e, SiegeEvent::Broken { part_id, .. } if *part_id == wall_id))
        .count();

    assert_eq!(hits, 1);
    assert_eq!(breaks, 1);
    assert!(frame.geometry_changed);
}

#[test]
fn timer_map_is_cleaned_when_attackers_disappear() {
    let build = load_build();
    let cfg = load_siege();
    let mut base = BaseState::default();
    place_part(
        &mut base,
        &build,
        "wall",
        Vec3::new(1.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
        None,
    );

    let mut sys = SiegeSystem::new(cfg.clone());
    let player = Vec3::new(5.0, 0.0, 0.0); // new: DECISIONS 2026-11-23
    let half_step = cfg.hit_interval * 0.5; // new: DECISIONS 2026-11-23

    let attackers = [
        SiegeAttacker {
            id: 1,
            kind: "walker",
            position: Vec3::ZERO,
            chasing: true,
        },
        SiegeAttacker {
            id: 2,
            kind: "walker",
            position: Vec3::new(0.0, 0.0, 0.2), // new: DECISIONS 2026-11-23
            chasing: true,
        },
    ];
    sys.tick(&mut base, &build, &attackers, player, half_step);
    assert_eq!(sys.tracked_attackers(), 2);

    // Drop attacker 2: only attacker 1 remains tracked.
    sys.tick(&mut base, &build, &attackers[..1], player, 0.1); // new: DECISIONS 2026-11-23
    assert_eq!(sys.tracked_attackers(), 1);

    // Re-adding attacker 2 starts its timer fresh, so a 0.6s tick must not trigger a hit yet.
    let frame = sys.tick(&mut base, &build, &attackers[1..], player, half_step);
    assert!(frame.events.is_empty());
    assert_eq!(sys.tracked_attackers(), 1);
}

#[test]
fn alert_fades_to_zero_and_clear_resets_state() {
    let build = load_build();
    let cfg = load_siege();
    let mut base = BaseState::default();
    place_part(
        &mut base,
        &build,
        "wall",
        Vec3::new(1.0, 0.0, 0.0), // new: DECISIONS 2026-11-23
        None,
    );

    let mut sys = SiegeSystem::new(cfg.clone());
    let player = Vec3::new(5.0, 0.0, 0.0); // new: DECISIONS 2026-11-23
    let attackers = [SiegeAttacker {
        id: 1,
        kind: "walker",
        position: Vec3::ZERO,
        chasing: true,
    }];

    sys.tick(&mut base, &build, &attackers, player, cfg.hit_interval);
    assert!(sys.under_attack());
    assert!((sys.alert_fraction() - 1.0).abs() < 1e-4);

    // Fade halfway with no attackers.
    sys.tick(&mut base, &build, &[], player, cfg.alert_seconds * 0.5); // new: DECISIONS 2026-11-23
    assert!(sys.under_attack());
    assert!((sys.alert_fraction() - 0.5).abs() < 1e-4);

    // Fade all the way to 0.
    sys.tick(&mut base, &build, &[], player, cfg.alert_seconds);
    assert!(!sys.under_attack());
    assert!(sys.alert_fraction().abs() < 1e-4);

    // Trigger again and verify `clear` wipes both alert and timers.
    sys.tick(&mut base, &build, &attackers, player, cfg.hit_interval);
    assert!(sys.under_attack());
    assert_eq!(sys.tracked_attackers(), 1);
    sys.clear();
    assert!(!sys.under_attack());
    assert!(sys.alert_fraction().abs() < 1e-4);
    assert_eq!(sys.tracked_attackers(), 0);
}

#[test]
fn siege_json_loads_and_covers_all_zombie_types_and_balance_targets() {
    let cfg = load_siege();
    let build = load_build();

    let zombies_text = std::fs::read_to_string(paths::repo_path("data/zombies.json"))
        .expect("data/zombies.json should load");
    let zombies_json: Value =
        serde_json::from_str(&zombies_text).expect("data/zombies.json should parse");
    let types = zombies_json["types"]
        .as_object()
        .expect("zombies.json should have a types object");

    for kind in types.keys() {
        assert!(
            cfg.damage.contains_key(kind),
            "data/siege.json is missing damage entry for zombie type '{kind}'"
        );
        assert!(
            cfg.damage_for(kind) > 0.0,
            "damage for zombie type '{kind}' must be positive"
        );
    }

    // Balance verification against basic wall HP in data/build_parts.json:
    // ~30s vs 3 walkers and ~10s vs 1 brute.
    let wall_hp = build.get("wall").expect("wall exists").hp;
    let three_walker_dps = 3.0 * cfg.damage_for("walker") / cfg.hit_interval; // new: DECISIONS 2026-11-23
    let brute_dps = cfg.damage_for("brute") / cfg.hit_interval;
    let walker_ttl = wall_hp / three_walker_dps;
    let brute_ttl = wall_hp / brute_dps;

    assert!(
        (25.0..=35.0).contains(&walker_ttl), // new: DECISIONS 2026-11-23
        "3 walkers should break a basic wall in ~30s, got {walker_ttl:.1}s"
    );
    assert!(
        (8.0..=12.0).contains(&brute_ttl), // new: DECISIONS 2026-11-23
        "1 brute should break a basic wall in ~10s, got {brute_ttl:.1}s"
    );
}
