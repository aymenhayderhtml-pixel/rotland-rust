use game::zombie::specials::*;
use glam::Vec3;

// new: DECISIONS 2026-11-23
const DT: f32 = 0.016;
// new: DECISIONS 2026-11-23
const FLOAT_EPS: f32 = 1e-4;
// new: DECISIONS 2026-11-23
const LUNGE_TRAVEL_TOLERANCE: f32 = 0.05;

fn load_cfg() -> SpecialConfig {
    SpecialConfig::from_repo().expect("data/special_zombies.json should load")
}

fn assert_finite_directive(d: SpecialDirective) {
    if let Some(v) = d.velocity {
        assert!(v.is_finite(), "velocity must be finite: {v:?}");
    }
    if let Some(f) = d.face {
        assert!(f.is_finite(), "face must be finite: {f:?}");
    }
    if let Some(hit) = d.damage {
        assert!(hit.amount.is_finite(), "damage amount must be finite");
        assert!(hit.knockback.is_finite(), "knockback must be finite");
    }
    if let Some(call) = d.call {
        assert!(call.target.is_finite(), "call target must be finite");
        assert!(call.radius.is_finite(), "call radius must be finite");
    }
}

#[test]
fn brute_windup_is_visible_for_full_duration() {
    let cfg = load_cfg();
    let mut state = SpecialState::new(SpecialKind::Brute);
    let own = Vec3::ZERO;
    let player = Vec3::new(
        (cfg.brute.charge_min_range + cfg.brute.charge_max_range) * 0.5,
        0.0,
        0.0,
    );

    let mut windup_ticks = 0u32;
    let first_after = loop {
        let d = state.tick(&cfg, own, player, true, false, DT, 0.0);
        if d.pose == SpecialPose::Windup {
            assert_eq!(d.velocity, Some(Vec3::ZERO));
            assert_eq!(d.face, Some(Vec3::X));
            windup_ticks += 1;
            assert!(windup_ticks < 1000, "windup never ended");
        } else {
            break d;
        }
    };

    let elapsed = windup_ticks as f32 * DT;
    assert!(
        (elapsed - cfg.brute.windup_seconds).abs() <= DT,
        "brute windup {elapsed}s not within {DT}s of {}s",
        cfg.brute.windup_seconds
    );
    assert_eq!(first_after.pose, SpecialPose::Charge);
}

#[test]
fn screamer_windup_is_visible_for_full_duration() {
    let cfg = load_cfg();
    let mut state = SpecialState::new(SpecialKind::Screamer);
    let own = Vec3::ZERO;
    let player = Vec3::new(cfg.screamer.trigger_range * 0.5, 0.0, 0.0);

    let mut windup_ticks = 0u32;
    let first_after = loop {
        let d = state.tick(&cfg, own, player, true, false, DT, 0.0);
        if d.pose == SpecialPose::Windup {
            assert_eq!(d.velocity, Some(Vec3::ZERO));
            assert_eq!(d.face, Some(Vec3::X));
            assert!(d.call.is_none());
            windup_ticks += 1;
            assert!(windup_ticks < 1000, "windup never ended");
        } else {
            break d;
        }
    };

    let elapsed = windup_ticks as f32 * DT;
    assert!(
        (elapsed - cfg.screamer.windup_seconds).abs() <= DT,
        "screamer windup {elapsed}s not within {DT}s of {}s",
        cfg.screamer.windup_seconds
    );
    assert_eq!(first_after.pose, SpecialPose::Scream);
    assert!(first_after.call.is_some());
}

#[test]
fn brute_charge_range_bounds() {
    let cfg = load_cfg();
    let own = Vec3::ZERO;
    // new: DECISIONS 2026-11-23
    let margin = 0.5_f32;

    let mut close = SpecialState::new(SpecialKind::Brute);
    let d_close = close.tick(
        &cfg,
        own,
        Vec3::new(cfg.brute.charge_min_range - margin, 0.0, 0.0),
        true,
        false,
        DT,
        0.0,
    );
    assert_eq!(d_close.pose, SpecialPose::Normal);
    assert_eq!(d_close.velocity, None);

    let mut far = SpecialState::new(SpecialKind::Brute);
    let d_far = far.tick(
        &cfg,
        own,
        Vec3::new(cfg.brute.charge_max_range + margin, 0.0, 0.0),
        true,
        false,
        DT,
        0.0,
    );
    assert_eq!(d_far.pose, SpecialPose::Normal);
    assert_eq!(d_far.velocity, None);

    let mut at_min = SpecialState::new(SpecialKind::Brute);
    let d_min = at_min.tick(
        &cfg,
        own,
        Vec3::new(cfg.brute.charge_min_range, 0.0, 0.0),
        true,
        false,
        DT,
        0.0,
    );
    assert_eq!(d_min.pose, SpecialPose::Windup);

    let mut at_max = SpecialState::new(SpecialKind::Brute);
    let d_max = at_max.tick(
        &cfg,
        own,
        Vec3::new(cfg.brute.charge_max_range, 0.0, 0.0),
        true,
        false,
        DT,
        0.0,
    );
    assert_eq!(d_max.pose, SpecialPose::Windup);
}

#[test]
fn brute_cooldown_blocks_new_charge_until_elapsed() {
    let cfg = load_cfg();
    let mut state = SpecialState::new(SpecialKind::Brute);
    let own = Vec3::ZERO;
    let in_range = Vec3::new(
        (cfg.brute.charge_min_range + cfg.brute.charge_max_range) * 0.5,
        0.0,
        0.0,
    );
    let far = Vec3::new(cfg.brute.charge_max_range * 4.0, 0.0, 0.0);

    // Run windup and charge until the charge ends (timeout -> stunned).
    loop {
        let d = state.tick(&cfg, own, in_range, true, false, DT, 0.0);
        if d.pose == SpecialPose::Charge {
            break;
        }
    }
    loop {
        let d = state.tick(&cfg, own, far, true, false, DT, 0.0);
        if d.pose == SpecialPose::Stunned {
            break;
        }
    }

    // One tick of cooldown has already passed on the first Stunned tick.
    let mut ticks_since_charge_end = 1u32;
    loop {
        let d = state.tick(&cfg, own, in_range, true, false, DT, 0.0);
        ticks_since_charge_end += 1;
        if d.pose == SpecialPose::Windup {
            break;
        }
        assert!(ticks_since_charge_end < 5000, "brute cooldown never ended");
    }

    let waited = ticks_since_charge_end as f32 * DT;
    assert!(
        (waited - cfg.brute.cooldown_seconds).abs() <= DT,
        "brute waited {waited}s vs cooldown {}s",
        cfg.brute.cooldown_seconds
    );
}

#[test]
fn brute_charge_timeout_causes_stun_and_vulnerability() {
    let cfg = load_cfg();
    let mut state = SpecialState::new(SpecialKind::Brute);
    let own = Vec3::ZERO;
    let trigger_pos = Vec3::new(
        (cfg.brute.charge_min_range + cfg.brute.charge_max_range) * 0.5,
        0.0,
        0.0,
    );
    let far = Vec3::new(cfg.brute.charge_max_range * 5.0, 0.0, 0.0);

    loop {
        let d = state.tick(&cfg, own, trigger_pos, true, false, DT, 0.0);
        if d.pose == SpecialPose::Charge {
            break;
        }
    }

    let mut charge_ticks = 1u32;
    let first_stun = loop {
        let d = state.tick(&cfg, own, far, true, false, DT, 0.0);
        if d.pose == SpecialPose::Charge {
            assert!(!d.stunned && !d.vulnerable);
            charge_ticks += 1;
        } else {
            break d;
        }
    };
    let charge_time = charge_ticks as f32 * DT;
    assert!((charge_time - cfg.brute.charge_seconds).abs() <= DT);
    assert_eq!(first_stun.pose, SpecialPose::Stunned);
    assert_eq!(first_stun.velocity, Some(Vec3::ZERO));
    assert!(first_stun.stunned && first_stun.vulnerable);

    let mut stun_ticks = 1u32;
    let after_stun = loop {
        let d = state.tick(&cfg, own, far, true, false, DT, 0.0);
        if d.pose == SpecialPose::Stunned {
            assert_eq!(d.velocity, Some(Vec3::ZERO));
            assert!(d.stunned && d.vulnerable);
            stun_ticks += 1;
        } else {
            break d;
        }
    };
    let stun_time = stun_ticks as f32 * DT;
    assert!((stun_time - cfg.brute.stun_seconds).abs() <= DT);
    assert_eq!(after_stun.pose, SpecialPose::Normal);
    assert!(!after_stun.stunned && !after_stun.vulnerable);
}

#[test]
fn brute_wall_hit_causes_stun_and_vulnerability() {
    let cfg = load_cfg();
    let mut state = SpecialState::new(SpecialKind::Brute);
    let own = Vec3::ZERO;
    let player = Vec3::new(
        (cfg.brute.charge_min_range + cfg.brute.charge_max_range) * 0.5,
        0.0,
        0.0,
    );

    loop {
        let d = state.tick(&cfg, own, player, true, false, DT, 0.0);
        if d.pose == SpecialPose::Charge {
            break;
        }
    }

    let d_wall = state.tick(&cfg, own, player, true, true, DT, 0.0);
    assert_eq!(d_wall.pose, SpecialPose::Stunned);
    assert_eq!(d_wall.velocity, Some(Vec3::ZERO));
    assert!(d_wall.stunned && d_wall.vulnerable);

    let mut stun_ticks = 1u32;
    loop {
        let d = state.tick(&cfg, own, player, true, false, DT, 0.0);
        if d.pose == SpecialPose::Stunned {
            assert!(d.stunned && d.vulnerable);
            stun_ticks += 1;
        } else {
            break;
        }
    }
    let stun_time = stun_ticks as f32 * DT;
    assert!((stun_time - cfg.brute.stun_seconds).abs() <= DT);
}

#[test]
fn brute_charge_hit_deals_damage_and_knockback_without_stun() {
    let cfg = load_cfg();
    let mut state = SpecialState::new(SpecialKind::Brute);
    let mut own = Vec3::ZERO;
    let player = Vec3::new(cfg.brute.charge_min_range + 1.0, 0.0, 0.0);

    let mut hits = Vec::new();
    for _ in 0..400 {
        let d = state.tick(&cfg, own, player, true, false, DT, 0.0);
        assert!(!d.stunned, "hitting the player must not stun the brute");
        assert!(!d.vulnerable, "hitting the player must not make brute vulnerable");
        assert_ne!(d.pose, SpecialPose::Stunned);
        if let Some(hit) = d.damage {
            hits.push(hit);
        }
        if let Some(v) = d.velocity {
            own += v * DT;
        }
    }

    assert_eq!(hits.len(), 1, "expected exactly one hit");
    assert!((hits[0].amount - cfg.brute.hit_damage).abs() <= FLOAT_EPS);
    assert!(hits[0].knockback.x > 0.0, "knockback must point away from brute");
    assert_eq!(hits[0].knockback.y, 0.0);
    assert!((hits[0].knockback.length() - cfg.brute.knockback_speed).abs() <= FLOAT_EPS);
}

#[test]
fn brute_charge_turn_rate_is_clamped() {
    let cfg = load_cfg();
    let mut state = SpecialState::new(SpecialKind::Brute);
    let own = Vec3::ZERO;
    let start_player = Vec3::new(
        (cfg.brute.charge_min_range + cfg.brute.charge_max_range) * 0.5,
        0.0,
        0.0,
    );
    let side_player = Vec3::new(0.0, 0.0, start_player.x);

    // Finish windup facing +X.
    loop {
        let d = state.tick(&cfg, own, start_player, true, false, DT, 0.0);
        if d.pose == SpecialPose::Charge {
            break;
        }
    }

    let max_turn = cfg.brute.turn_rate_deg.to_radians() * DT;
    let mut prev_dir = Vec3::X;
    let mut turned = false;
    loop {
        let d = state.tick(&cfg, own, side_player, true, false, DT, 0.0);
        if d.pose != SpecialPose::Charge {
            break;
        }
        let dir = d.velocity.expect("charge must set velocity").normalize();
        let angle = prev_dir.dot(dir).clamp(-1.0, 1.0).acos();
        assert!(
            angle <= max_turn + FLOAT_EPS,
            "turn {angle} exceeded max_turn {max_turn}"
        );
        if dir.z > 0.0 {
            turned = true;
        }
        prev_dir = dir;
    }
    assert!(turned, "brute should gradually turn toward the sidestepped player");
}

#[test]
fn screamer_killed_during_windup_never_calls() {
    let cfg = load_cfg();
    let own = Vec3::ZERO;
    let player = Vec3::new(cfg.screamer.trigger_range * 0.5, 0.0, 0.0);

    // Case 1: killed midway through windup.
    let mut mid = SpecialState::new(SpecialKind::Screamer);
    let half_ticks = ((cfg.screamer.windup_seconds * 0.5) / DT) as u32;
    for _ in 0..half_ticks {
        let d = mid.tick(&cfg, own, player, true, false, DT, 0.0);
        assert_eq!(d.pose, SpecialPose::Windup);
        assert!(d.call.is_none());
    }
    mid.on_hit(true);
    for _ in 0..300 {
        let d = mid.tick(&cfg, own, player, true, false, DT, 0.0);
        assert!(d.call.is_none());
        assert_ne!(d.pose, SpecialPose::Scream);
    }

    // Case 2: killed right after the final windup tick before Scream fires.
    let mut late = SpecialState::new(SpecialKind::Screamer);
    let full_ticks = (cfg.screamer.windup_seconds / DT).round() as u32;
    for _ in 0..full_ticks {
        let d = late.tick(&cfg, own, player, true, false, DT, 0.0);
        assert_eq!(d.pose, SpecialPose::Windup);
    }
    late.on_hit(true);
    for _ in 0..100 {
        let d = late.tick(&cfg, own, player, true, false, DT, 0.0);
        assert!(d.call.is_none());
    }
}

#[test]
fn screamer_call_and_retreat_sequence() {
    let cfg = load_cfg();
    let mut state = SpecialState::new(SpecialKind::Screamer);
    let own = Vec3::new(cfg.screamer.trigger_range * 0.5, 0.0, 0.0);
    let player = Vec3::ZERO;

    // Trigger windup, then lose sight partway through — windup must still complete.
    let d0 = state.tick(&cfg, own, player, true, false, DT, 0.0);
    assert_eq!(d0.pose, SpecialPose::Windup);
    let mut calls = Vec::new();
    let scream_tick = loop {
        let d = state.tick(&cfg, own, player, false, false, DT, 0.0);
        assert!(!d.vulnerable);
        if let Some(c) = d.call {
            calls.push(c);
        }
        if d.pose != SpecialPose::Windup {
            break d;
        }
    };

    assert_eq!(scream_tick.pose, SpecialPose::Scream);
    assert_eq!(scream_tick.velocity, Some(Vec3::ZERO));
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].target, player);
    assert!((calls[0].radius - cfg.screamer.call_radius).abs() <= FLOAT_EPS);
    assert_eq!(calls[0].max_zombies, cfg.screamer.call_max_zombies);

    let mut retreat_ticks = 0u32;
    let after_retreat = loop {
        let d = state.tick(&cfg, own, player, true, false, DT, 0.0);
        assert!(!d.vulnerable);
        assert!(d.call.is_none());
        if d.pose == SpecialPose::Retreat {
            let v = d.velocity.expect("retreat must set velocity");
            assert!(v.x > 0.0, "retreat must move away from player (+X)");
            assert!((v.length() - cfg.screamer.retreat_speed).abs() <= FLOAT_EPS);
            retreat_ticks += 1;
        } else {
            break d;
        }
    };
    let retreat_time = retreat_ticks as f32 * DT;
    assert!((retreat_time - cfg.screamer.retreat_seconds).abs() <= DT);
    assert_eq!(after_retreat.pose, SpecialPose::Normal);
}

#[test]
fn screamer_call_cooldown_blocks_then_allows_second_call() {
    let cfg = load_cfg();
    let mut state = SpecialState::new(SpecialKind::Screamer);
    let own = Vec3::ZERO;
    let player = Vec3::new(cfg.screamer.trigger_range * 0.5, 0.0, 0.0);

    // Wait for first call.
    loop {
        let d = state.tick(&cfg, own, player, true, false, DT, 0.0);
        if d.call.is_some() {
            break;
        }
    }

    // Measure time until second windup starts, then confirm second call fires.
    let mut ticks_after_call = 0u32;
    loop {
        let d = state.tick(&cfg, own, player, true, false, DT, 0.0);
        ticks_after_call += 1;
        assert!(d.call.is_none(), "no call should fire before second windup finishes");
        if d.pose == SpecialPose::Windup {
            break;
        }
        assert!(ticks_after_call < 5000, "screamer cooldown never expired");
    }
    let cooldown_elapsed = ticks_after_call as f32 * DT;
    assert!(
        (cooldown_elapsed - cfg.screamer.call_cooldown_seconds).abs() <= DT,
        "second sighting triggered after {cooldown_elapsed}s vs {}s",
        cfg.screamer.call_cooldown_seconds
    );

    let mut second_calls = 0u32;
    for _ in 0..200 {
        let d = state.tick(&cfg, own, player, true, false, DT, 0.0);
        if d.call.is_some() {
            second_calls += 1;
        }
    }
    assert_eq!(second_calls, 1);
}

#[test]
fn crawler_lurks_without_sight_or_beyond_lurk_range() {
    let cfg = load_cfg();
    let mut state = SpecialState::new(SpecialKind::Crawler);
    let own = Vec3::ZERO;

    let d_no_los = state.tick(
        &cfg,
        own,
        Vec3::new(cfg.crawler.lunge_trigger_range * 0.5, 0.0, 0.0),
        false,
        false,
        DT,
        0.0,
    );
    assert_eq!(d_no_los.pose, SpecialPose::Lurk);
    assert_eq!(d_no_los.velocity, Some(Vec3::ZERO));

    let d_far = state.tick(
        &cfg,
        own,
        Vec3::new(cfg.crawler.lurk_range + 1.0, 0.0, 0.0),
        true,
        false,
        DT,
        0.0,
    );
    assert_eq!(d_far.pose, SpecialPose::Lurk);
    assert_eq!(d_far.velocity, Some(Vec3::ZERO));

    let d_mid = state.tick(
        &cfg,
        own,
        Vec3::new(
            (cfg.crawler.lunge_trigger_range + cfg.crawler.lurk_range) * 0.5,
            0.0,
            0.0,
        ),
        true,
        false,
        DT,
        0.0,
    );
    assert_eq!(d_mid.pose, SpecialPose::Normal);
    assert_eq!(d_mid.velocity, None);
}

#[test]
fn crawler_lunge_range_and_travel_distance() {
    let cfg = load_cfg();
    let own = Vec3::ZERO;

    // 3.5 m (> 3.0 m lungeTriggerRange): does not lunge.
    let mut far = SpecialState::new(SpecialKind::Crawler);
    // new: DECISIONS 2026-11-23
    let beyond_lunge = cfg.crawler.lunge_trigger_range + 0.5;
    let d_far = far.tick(&cfg, own, Vec3::new(beyond_lunge, 0.0, 0.0), true, false, DT, 0.0);
    assert_eq!(d_far.pose, SpecialPose::Normal);
    assert_eq!(d_far.velocity, None);

    // 3.0 m (== lungeTriggerRange) with roll 0.0: lunges.
    let mut state = SpecialState::new(SpecialKind::Crawler);
    let mut pos = own;
    let d_start = state.tick(
        &cfg,
        pos,
        Vec3::new(cfg.crawler.lunge_trigger_range, 0.0, 0.0),
        true,
        false,
        DT,
        0.0,
    );
    assert_eq!(d_start.pose, SpecialPose::Lunge);
    pos += d_start.velocity.expect("lunge velocity") * DT;

    // Player steps out of the lunge path so the lunge travels its full duration.
    let sidestepped = Vec3::new(0.0, 0.0, cfg.crawler.lurk_range);
    loop {
        let d = state.tick(&cfg, pos, sidestepped, true, false, DT, 0.0);
        if d.pose != SpecialPose::Lunge {
            break;
        }
        pos += d.velocity.expect("lunge velocity") * DT;
    }

    let expected_travel = cfg.crawler.lunge_speed * cfg.crawler.lunge_seconds;
    assert!(
        (pos.length() - expected_travel).abs() <= LUNGE_TRAVEL_TOLERANCE,
        "lunge travel {}m vs expected {expected_travel}m",
        pos.length()
    );
}

#[test]
fn crawler_recovers_with_vulnerability_after_lunge() {
    let cfg = load_cfg();
    let mut state = SpecialState::new(SpecialKind::Crawler);
    let own = Vec3::ZERO;
    let trigger = Vec3::new(cfg.crawler.lunge_trigger_range, 0.0, 0.0);
    let sidestepped = Vec3::new(0.0, 0.0, cfg.crawler.lurk_range);

    let d0 = state.tick(&cfg, own, trigger, true, false, DT, 0.0);
    assert_eq!(d0.pose, SpecialPose::Lunge);

    let first_recover = loop {
        let d = state.tick(&cfg, own, sidestepped, true, false, DT, 0.0);
        if d.pose != SpecialPose::Lunge {
            break d;
        }
    };
    assert_eq!(first_recover.pose, SpecialPose::Recover);
    assert_eq!(first_recover.velocity, Some(Vec3::ZERO));
    assert!(first_recover.vulnerable);

    let mut recover_ticks = 1u32;
    let after_recover = loop {
        let d = state.tick(&cfg, own, trigger, true, false, DT, 0.0);
        if d.pose == SpecialPose::Recover {
            assert_eq!(d.velocity, Some(Vec3::ZERO));
            assert!(d.vulnerable);
            recover_ticks += 1;
        } else {
            break d;
        }
    };
    let recover_time = recover_ticks as f32 * DT;
    assert!((recover_time - cfg.crawler.recover_seconds).abs() <= DT);
    assert_eq!(after_recover.pose, SpecialPose::Normal);
    assert!(!after_recover.vulnerable);
}

#[test]
fn crawler_lunge_damage_hits_once_or_misses_if_player_steps_away() {
    let cfg = load_cfg();

    // Player stays in path -> one hit of lungeDamage with zero knockback.
    let mut hitter = SpecialState::new(SpecialKind::Crawler);
    let mut pos = Vec3::ZERO;
    let player = Vec3::new(cfg.crawler.lunge_trigger_range * 0.5, 0.0, 0.0);
    let mut hits = Vec::new();
    for _ in 0..150 {
        let d = hitter.tick(&cfg, pos, player, true, false, DT, 0.0);
        if let Some(hit) = d.damage {
            hits.push(hit);
        }
        if let Some(v) = d.velocity {
            pos += v * DT;
        }
    }
    assert_eq!(hits.len(), 1);
    assert!((hits[0].amount - cfg.crawler.lunge_damage).abs() <= FLOAT_EPS);
    assert_eq!(hits[0].knockback, Vec3::ZERO);

    // Player steps away after lunge starts -> zero hits.
    let mut misser = SpecialState::new(SpecialKind::Crawler);
    let mut pos2 = Vec3::ZERO;
    let d0 = misser.tick(&cfg, pos2, player, true, false, DT, 0.0);
    assert_eq!(d0.pose, SpecialPose::Lunge);
    assert!(d0.damage.is_none());
    pos2 += d0.velocity.expect("lunge velocity") * DT;

    let stepped_away = Vec3::new(player.x, 0.0, cfg.crawler.hit_range * 4.0);
    for _ in 0..150 {
        let d = misser.tick(&cfg, pos2, stepped_away, true, false, DT, 0.0);
        assert!(d.damage.is_none(), "player stepped away, no hit expected");
        if let Some(v) = d.velocity {
            pos2 += v * DT;
        }
    }
}

#[test]
fn same_position_never_nans_or_panics_for_any_kind_or_pose() {
    let cfg = load_cfg();
    // new: DECISIONS 2026-11-23
    let same = Vec3::new(5.0, 0.0, -3.0);
    let mut seen_poses = Vec::new();
    let mut record = |d: SpecialDirective| {
        assert_finite_directive(d);
        if !seen_poses.contains(&d.pose) {
            seen_poses.push(d.pose);
        }
    };

    // Brute: Normal, Windup, Charge, Stunned at identical position.
    let mut brute = SpecialState::new(SpecialKind::Brute);
    record(brute.tick(&cfg, same, same, true, false, DT, 0.0));
    let brute_trigger = same + Vec3::new(cfg.brute.charge_min_range + 1.0, 0.0, 0.0);
    record(brute.tick(&cfg, same, brute_trigger, true, false, DT, 0.0));
    loop {
        let d = brute.tick(&cfg, same, same, true, false, DT, 0.0);
        record(d);
        if d.pose == SpecialPose::Charge {
            break;
        }
    }
    let mut brute_wall = SpecialState::new(SpecialKind::Brute);
    loop {
        let d = brute_wall.tick(&cfg, same, brute_trigger, true, false, DT, 0.0);
        if d.pose == SpecialPose::Charge {
            break;
        }
    }
    // The wall tick uses the real player spot: at zero distance the brute reaches the player first (hit wins).
    record(brute_wall.tick(&cfg, same, brute_trigger, true, true, DT, 0.0));
    record(brute_wall.tick(&cfg, same, same, true, false, DT, 0.0));

    // Screamer: Normal, Windup, Scream, Retreat at identical position.
    let mut screamer = SpecialState::new(SpecialKind::Screamer);
    record(screamer.tick(&cfg, same, same, false, false, DT, 0.0));
    for _ in 0..250 {
        record(screamer.tick(&cfg, same, same, true, false, DT, 0.0));
    }

    // Crawler: Lurk, Normal, Lunge, Recover at identical position.
    let mut crawler = SpecialState::new(SpecialKind::Crawler);
    record(crawler.tick(&cfg, same, same, false, false, DT, 0.0));
    record(crawler.tick(&cfg, same, same, true, false, DT, 1.0));
    let mut crawler_lunge = SpecialState::new(SpecialKind::Crawler);
    record(crawler_lunge.tick(&cfg, same, same, true, false, DT, 0.0));
    record(crawler_lunge.tick(&cfg, same, same, true, false, DT, 0.0));

    for pose in [
        SpecialPose::Normal,
        SpecialPose::Windup,
        SpecialPose::Charge,
        SpecialPose::Stunned,
        SpecialPose::Lurk,
        SpecialPose::Lunge,
        SpecialPose::Recover,
        SpecialPose::Scream,
        SpecialPose::Retreat,
    ] {
        assert!(seen_poses.contains(&pose), "missing pose in same-point test: {pose:?}");
    }
}

#[test]
fn phase_durations_stay_stable_across_dt_range() {
    let cfg = load_cfg();
    // new: DECISIONS 2026-11-23
    let dts = [0.001_f32, 0.005, 0.016, 0.033, 0.05, 0.1];
    let own = Vec3::ZERO;
    let brute_in = Vec3::new(
        (cfg.brute.charge_min_range + cfg.brute.charge_max_range) * 0.5,
        0.0,
        0.0,
    );
    let far = Vec3::new(100.0, 0.0, 0.0);
    let screamer_in = Vec3::new(cfg.screamer.trigger_range * 0.5, 0.0, 0.0);
    let crawler_in = Vec3::new(cfg.crawler.lunge_trigger_range, 0.0, 0.0);

    for dt in dts {
        // Brute: windup -> charge -> stunned.
        let mut brute = SpecialState::new(SpecialKind::Brute);
        let mut b_windup = 0u32;
        let mut b_charge = 0u32;
        let mut b_stun = 0u32;
        loop {
            let target = if b_charge == 0 && b_stun == 0 {
                brute_in
            } else {
                far
            };
            let d = brute.tick(&cfg, own, target, true, false, dt, 0.0);
            assert_finite_directive(d);
            match d.pose {
                SpecialPose::Windup => b_windup += 1,
                SpecialPose::Charge => b_charge += 1,
                SpecialPose::Stunned => b_stun += 1,
                SpecialPose::Normal if b_stun > 0 => break,
                _ => {}
            }
        }
        assert!((b_windup as f32 * dt - cfg.brute.windup_seconds).abs() <= dt + FLOAT_EPS);
        assert!((b_charge as f32 * dt - cfg.brute.charge_seconds).abs() <= dt + FLOAT_EPS);
        assert!((b_stun as f32 * dt - cfg.brute.stun_seconds).abs() <= dt + FLOAT_EPS);

        // Screamer: windup -> scream -> retreat.
        let mut screamer = SpecialState::new(SpecialKind::Screamer);
        let mut s_windup = 0u32;
        let mut s_scream = 0u32;
        let mut s_retreat = 0u32;
        loop {
            let d = screamer.tick(&cfg, own, screamer_in, true, false, dt, 0.0);
            assert_finite_directive(d);
            match d.pose {
                SpecialPose::Windup => s_windup += 1,
                SpecialPose::Scream => s_scream += 1,
                SpecialPose::Retreat => s_retreat += 1,
                SpecialPose::Normal if s_retreat > 0 => break,
                _ => {}
            }
        }
        assert_eq!(s_scream, 1);
        assert!((s_windup as f32 * dt - cfg.screamer.windup_seconds).abs() <= dt + FLOAT_EPS);
        assert!((s_retreat as f32 * dt - cfg.screamer.retreat_seconds).abs() <= dt + FLOAT_EPS);

        // Crawler: lunge -> recover.
        let mut crawler = SpecialState::new(SpecialKind::Crawler);
        let mut c_lunge = 0u32;
        let mut c_recover = 0u32;
        loop {
            let target = if c_lunge == 0 && c_recover == 0 {
                crawler_in
            } else {
                far
            };
            let d = crawler.tick(&cfg, own, target, true, false, dt, 0.0);
            assert_finite_directive(d);
            match d.pose {
                SpecialPose::Lunge => c_lunge += 1,
                SpecialPose::Recover => c_recover += 1,
                _ if c_recover > 0 => break,
                _ => {}
            }
        }
        assert!((c_lunge as f32 * dt - cfg.crawler.lunge_seconds).abs() <= dt + FLOAT_EPS);
        assert!((c_recover as f32 * dt - cfg.crawler.recover_seconds).abs() <= dt + FLOAT_EPS);
    }
}

#[test]
fn non_lethal_hit_changes_nothing() {
    let cfg = load_cfg();
    let own = Vec3::ZERO;

    for (kind, player) in [
        (
            SpecialKind::Brute,
            Vec3::new(cfg.brute.charge_min_range + 1.0, 0.0, 0.0),
        ),
        (
            SpecialKind::Screamer,
            Vec3::new(cfg.screamer.trigger_range * 0.5, 0.0, 0.0),
        ),
        (
            SpecialKind::Crawler,
            Vec3::new(cfg.crawler.lunge_trigger_range, 0.0, 0.0),
        ),
    ] {
        let mut baseline = SpecialState::new(kind);
        let mut hit_state = SpecialState::new(kind);
        for _ in 0..250 {
            hit_state.on_hit(false);
            let d_base = baseline.tick(&cfg, own, player, true, false, DT, 0.0);
            let d_hit = hit_state.tick(&cfg, own, player, true, false, DT, 0.0);
            assert_eq!(d_base, d_hit, "on_hit(false) altered {kind:?} behaviour");
        }
    }
}

#[test]
fn special_kind_from_id_maps_only_specials() {
    assert_eq!(SpecialKind::from_id("brute"), Some(SpecialKind::Brute));
    assert_eq!(SpecialKind::from_id("screamer"), Some(SpecialKind::Screamer));
    assert_eq!(SpecialKind::from_id("crawler"), Some(SpecialKind::Crawler));
    assert_eq!(SpecialKind::from_id("walker"), None);
    assert_eq!(SpecialKind::from_id("runner"), None);
    assert_eq!(SpecialKind::from_id("unknown"), None);

    assert_eq!(SpecialState::new(SpecialKind::Brute).kind(), SpecialKind::Brute);
    assert_eq!(
        SpecialState::new(SpecialKind::Screamer).kind(),
        SpecialKind::Screamer
    );
    assert_eq!(
        SpecialState::new(SpecialKind::Crawler).kind(),
        SpecialKind::Crawler
    );
}

#[test]
fn from_repo_loads_and_matches_json_numbers() {
    let cfg = load_cfg();

    // data: special_zombies.json vulnerableMultiplier
    assert!((cfg.vulnerable_multiplier - 2.0).abs() <= FLOAT_EPS);

    // data: special_zombies.json brute
    assert!((cfg.brute.charge_min_range - 6.0).abs() <= FLOAT_EPS);
    assert!((cfg.brute.charge_max_range - 16.0).abs() <= FLOAT_EPS);
    assert!((cfg.brute.windup_seconds - 0.9).abs() <= FLOAT_EPS);
    assert!((cfg.brute.charge_seconds - 1.8).abs() <= FLOAT_EPS);
    assert!((cfg.brute.charge_speed - 6.0).abs() <= FLOAT_EPS);
    assert!((cfg.brute.turn_rate_deg - 25.0).abs() <= FLOAT_EPS);
    assert!((cfg.brute.hit_range - 1.6).abs() <= FLOAT_EPS);
    assert!((cfg.brute.hit_damage - 25.0).abs() <= FLOAT_EPS);
    assert!((cfg.brute.knockback_speed - 5.0).abs() <= FLOAT_EPS);
    assert!((cfg.brute.stun_seconds - 2.0).abs() <= FLOAT_EPS);
    assert!((cfg.brute.cooldown_seconds - 14.0).abs() <= FLOAT_EPS);

    // data: special_zombies.json screamer
    assert!((cfg.screamer.trigger_range - 24.0).abs() <= FLOAT_EPS);
    assert!((cfg.screamer.windup_seconds - 1.2).abs() <= FLOAT_EPS);
    assert!((cfg.screamer.call_radius - 25.0).abs() <= FLOAT_EPS);
    assert_eq!(cfg.screamer.call_max_zombies, 6);
    assert!((cfg.screamer.call_cooldown_seconds - 20.0).abs() <= FLOAT_EPS);
    assert!((cfg.screamer.retreat_seconds - 2.5).abs() <= FLOAT_EPS);
    assert!((cfg.screamer.retreat_speed - 1.8).abs() <= FLOAT_EPS);

    // data: special_zombies.json crawler
    assert!((cfg.crawler.lurk_range - 14.0).abs() <= FLOAT_EPS);
    assert!((cfg.crawler.lunge_trigger_range - 3.0).abs() <= FLOAT_EPS);
    assert!((cfg.crawler.lunge_speed - 7.0).abs() <= FLOAT_EPS);
    assert!((cfg.crawler.lunge_seconds - 0.35).abs() <= FLOAT_EPS);
    assert!((cfg.crawler.hit_range - 1.0).abs() <= FLOAT_EPS);
    assert!((cfg.crawler.lunge_damage - 8.0).abs() <= FLOAT_EPS);
    assert!((cfg.crawler.lunge_chance - 0.6).abs() <= FLOAT_EPS);
    assert!((cfg.crawler.recover_seconds - 1.2).abs() <= FLOAT_EPS);
    assert!((cfg.crawler.retry_seconds - 2.0).abs() <= FLOAT_EPS);
}
