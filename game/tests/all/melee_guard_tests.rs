use game::melee::MeleeTarget;
use game::{GuardConfig, GuardEvent, GuardInput, GuardState, Stamina, shove_targets};
use glam::Vec3;

fn full_stamina() -> Stamina {
    Stamina { current: 100.0, ..Stamina::default() } // new: DECISIONS 2026-11-24
}

#[test]
fn config_parses_and_loads_from_repo() {
    let cfg = GuardConfig::from_repo().expect("data/melee_guard.json should load");
    assert_eq!(cfg, GuardConfig::default());
}

#[test]
fn tap_shoves_and_respects_cooldown_and_stamina() {
    let cfg = GuardConfig::default();
    let mut state = GuardState::default();
    let mut stamina = full_stamina();
    let start = stamina.current;

    let ev = state.update(0.05, GuardInput { held: true, can_guard: true }, &mut stamina, &cfg);
    assert_eq!(ev, GuardEvent::None);
    let ev = state.update(0.05, GuardInput { held: false, can_guard: true }, &mut stamina, &cfg);
    assert_eq!(ev, GuardEvent::Shove);
    assert!((stamina.current - (start - cfg.shove.stamina_cost)).abs() < 1e-4);

    // Second tap inside cooldown does nothing.
    state.update(0.05, GuardInput { held: true, can_guard: true }, &mut stamina, &cfg);
    let ev = state.update(0.05, GuardInput { held: false, can_guard: true }, &mut stamina, &cfg);
    assert_eq!(ev, GuardEvent::None);

    // Wait out cooldown, then try with too little stamina.
    state.update(cfg.shove.cooldown, GuardInput { held: false, can_guard: true }, &mut stamina, &cfg);
    stamina.current = cfg.shove.stamina_cost - 1.0;
    state.update(0.05, GuardInput { held: true, can_guard: true }, &mut stamina, &cfg);
    let ev = state.update(0.05, GuardInput { held: false, can_guard: true }, &mut stamina, &cfg);
    assert_eq!(ev, GuardEvent::None);
}

#[test]
fn hold_blocks_and_release_ends_without_shove() {
    let cfg = GuardConfig::default();
    let mut state = GuardState::default();
    let mut stamina = full_stamina();

    let ev = state.update(0.1, GuardInput { held: true, can_guard: true }, &mut stamina, &cfg);
    assert_eq!(ev, GuardEvent::None);
    assert!(!state.is_blocking());

    let ev = state.update(0.15, GuardInput { held: true, can_guard: true }, &mut stamina, &cfg);
    assert_eq!(ev, GuardEvent::BlockStart);
    assert!(state.is_blocking());

    let ev = state.update(0.05, GuardInput { held: false, can_guard: true }, &mut stamina, &cfg);
    assert_eq!(ev, GuardEvent::BlockEnd);
    assert!(!state.is_blocking());
}

#[test]
fn block_drains_and_guard_break_locks_out_until_break_seconds() {
    let cfg = GuardConfig::default();
    let mut state = GuardState::default();
    let mut stamina = full_stamina();
    stamina.current = 6.0; // new: DECISIONS 2026-11-24

    assert_eq!(
        state.update(cfg.tap_seconds + 0.01, GuardInput { held: true, can_guard: true }, &mut stamina, &cfg),
        GuardEvent::BlockStart
    );

    let ev = state.update(1.0, GuardInput { held: true, can_guard: true }, &mut stamina, &cfg);
    assert_eq!(ev, GuardEvent::None);
    assert!((stamina.current - (6.0 - cfg.block.drain_per_second)).abs() < 1e-4);

    let ev = state.update(1.0, GuardInput { held: true, can_guard: true }, &mut stamina, &cfg);
    assert_eq!(ev, GuardEvent::GuardBroken);
    assert!(!state.is_blocking());
    assert_eq!(stamina.current, 0.0);

    // Refill stamina, but still inside break_seconds: cannot block.
    stamina.current = 50.0;
    state.update(0.05, GuardInput { held: false, can_guard: true }, &mut stamina, &cfg);
    let ev = state.update(cfg.tap_seconds + 0.1, GuardInput { held: true, can_guard: true }, &mut stamina, &cfg);
    assert_eq!(ev, GuardEvent::None);
    assert!(!state.is_blocking());

    // After break_seconds elapses, holding blocks again.
    let ev = state.update(cfg.block.break_seconds, GuardInput { held: true, can_guard: true }, &mut stamina, &cfg);
    assert_eq!(ev, GuardEvent::BlockStart);
    assert!(state.is_blocking());
}

#[test]
fn can_guard_false_ends_block() {
    let cfg = GuardConfig::default();
    let mut state = GuardState::default();
    let mut stamina = full_stamina();

    state.update(cfg.tap_seconds + 0.05, GuardInput { held: true, can_guard: true }, &mut stamina, &cfg);
    assert!(state.is_blocking());

    let ev = state.update(0.05, GuardInput { held: true, can_guard: false }, &mut stamina, &cfg);
    assert_eq!(ev, GuardEvent::BlockEnd);
    assert!(!state.is_blocking());
}

#[test]
fn filter_bite_checks_arc_blocking_and_stamina() {
    let cfg = GuardConfig::default();
    let mut state = GuardState::default();
    let mut stamina = full_stamina();
    let pos = Vec3::ZERO;
    let dmg = 20.0; // new: DECISIONS 2026-11-24

    // Not blocking -> full damage.
    assert_eq!(state.filter_bite(dmg, pos, 0.0, Some(Vec3::new(0.0, 0.0, -1.0)), &mut stamina, &cfg), (dmg, false));

    // Start blocking (yaw 0 faces -Z).
    state.update(cfg.tap_seconds + 0.05, GuardInput { held: true, can_guard: true }, &mut stamina, &cfg);
    let before = stamina.current;

    // Front attacker at -Z -> blocked.
    let (out_dmg, blocked) = state.filter_bite(dmg, pos, 0.0, Some(Vec3::new(0.0, 0.0, -1.0)), &mut stamina, &cfg);
    assert!(blocked);
    assert!((out_dmg - dmg * cfg.block.damage_scale).abs() < 1e-4);
    assert!((stamina.current - (before - cfg.block.stamina_per_hit)).abs() < 1e-4);

    // Behind attacker at +Z -> full damage, not blocked.
    assert_eq!(state.filter_bite(dmg, pos, 0.0, Some(Vec3::new(0.0, 0.0, 1.0)), &mut stamina, &cfg), (dmg, false));

    // Unknown attacker -> full damage, not blocked.
    assert_eq!(state.filter_bite(dmg, pos, 0.0, None, &mut stamina, &cfg), (dmg, false));

    // Not enough stamina -> full damage and block broken.
    stamina.current = cfg.block.stamina_per_hit - 1.0;
    assert_eq!(state.filter_bite(dmg, pos, 0.0, Some(Vec3::new(0.0, 0.0, -1.0)), &mut stamina, &cfg), (dmg, false));
    assert!(!state.is_blocking());
}

#[test]
fn shove_targets_filters_sorts_caps_and_clears_out() {
    let cfg = GuardConfig::default();
    let pos = Vec3::ZERO;
    let facing = Vec3::NEG_Z;
    let targets = [
        MeleeTarget { id: 1, position: Vec3::new(0.0, 0.0, -1.6), radius: 0.3 },
        MeleeTarget { id: 2, position: Vec3::new(0.0, 0.0, -0.8), radius: 0.3 },
        MeleeTarget { id: 3, position: Vec3::new(0.3, 0.0, -1.2), radius: 0.3 },
        MeleeTarget { id: 4, position: Vec3::new(-0.3, 0.0, -1.4), radius: 0.3 },
        MeleeTarget { id: 5, position: Vec3::new(0.0, 0.0, 1.0), radius: 0.3 },  // behind
        MeleeTarget { id: 6, position: Vec3::new(0.0, 0.0, -3.0), radius: 0.3 }, // too far
    ];

    let mut out = vec![(99, Vec3::X)];
    shove_targets(pos, facing, &targets, &cfg, &mut out);

    let ids: Vec<u64> = out.iter().map(|&(id, _)| id).collect();
    assert_eq!(ids, vec![2, 3, 4]);
    for &(_, push) in &out {
        assert!((push.length() - cfg.shove.push).abs() < 1e-4);
        assert!(push.z < 0.0);
    }
}
