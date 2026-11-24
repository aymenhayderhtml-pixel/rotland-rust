use std::f32::consts::PI;

use game::{FpMotion, FpMotionConfig, FpMotionInput};

fn rest_input() -> FpMotionInput {
    FpMotionInput {
        on_floor: true,
        weapon_key: 1,
        stamina_frac: 1.0,
        ..Default::default()
    }
}

fn step_for(m: &mut FpMotion, cfg: &FpMotionConfig, input: &FpMotionInput, secs: f32, hz: u32) {
    let steps = (secs * hz as f32).round() as u32;
    let dt = 1.0 / hz as f32;
    for _ in 0..steps {
        m.update(dt, input, cfg);
    }
}

#[test]
fn data_parses_and_matches_default() {
    let loaded = FpMotionConfig::from_repo().expect("data/fp_motion.json should parse");
    assert_eq!(loaded, FpMotionConfig::default());
}

#[test]
fn still_player_has_tiny_breathing_and_more_when_tired() {
    let cfg = FpMotionConfig::default();
    let mut fresh = FpMotion::default();
    let mut tired = FpMotion::default();
    let fresh_in = rest_input();
    let tired_in = FpMotionInput { stamina_frac: 0.0, ..rest_input() };

    let mut max_fresh_y = 0.0_f32;
    let mut max_tired_y = 0.0_f32;
    for _ in 0..240 {
        fresh.update(1.0 / 60.0, &fresh_in, &cfg);
        tired.update(1.0 / 60.0, &tired_in, &cfg);
        max_fresh_y = max_fresh_y.max(fresh.offset().y.abs());
        max_tired_y = max_tired_y.max(tired.offset().y.abs());
    }
    assert!(max_fresh_y > 0.001 && max_fresh_y <= 0.005);
    assert!(max_tired_y > max_fresh_y * 2.0);
    assert!(fresh.rot_deg().length() < 1e-4);
}

#[test]
fn fast_turn_lags_then_returns_and_wraps_pi() {
    let cfg = FpMotionConfig::default();
    let mut m = FpMotion::default();
    let mut input = FpMotionInput { yaw: PI - 0.1, ..rest_input() };
    m.update(1.0 / 60.0, &input, &cfg);

    for _ in 0..8 {
        input.yaw = (input.yaw + 0.05 + PI).rem_euclid(2.0 * PI) - PI;
        m.update(1.0 / 60.0, &input, &cfg);
    }
    let peak_x = m.offset().x;
    let peak_roll = m.rot_deg().z;
    assert!(peak_x > 0.01, "expected positive lag on +yaw turn, got {peak_x}");
    assert!(peak_roll.abs() > 0.5, "expected tilt toward turn, got {peak_roll}");

    step_for(&mut m, &cfg, &input, 1.0, 60);
    assert!(m.offset().x.abs() < 0.005);
    assert!(m.rot_deg().length() < 0.2);
}

#[test]
fn snap_turn_is_ignored_and_first_update_has_no_jump() {
    let cfg = FpMotionConfig::default();
    let mut m = FpMotion::default();
    let mut input = FpMotionInput { yaw: 2.4, pitch: -0.4, weapon_key: 99, ..rest_input() };
    m.update(1.0 / 60.0, &input, &cfg);
    assert!(m.offset().length() < 0.002, "first update jumped: {:?}", m.offset());
    assert!(m.rot_deg().length() < 1e-4);

    input.yaw = -0.2;
    m.update(1.0 / 60.0, &input, &cfg);
    assert!(m.offset().x.abs() < 0.002, "snap turn leaked lag: {:?}", m.offset());
}

#[test]
fn sprint_and_crouch_lower_weapon() {
    let cfg = FpMotionConfig::default();
    let mut sprint = FpMotion::default();
    let sprint_in = FpMotionInput { sprinting: true, horizontal_speed: 7.0, ..rest_input() };
    step_for(&mut sprint, &cfg, &sprint_in, 0.5, 60);
    assert!(sprint.offset().y < -0.03);
    assert!(sprint.rot_deg().y > 10.0);

    let mut crouch = FpMotion::default();
    let crouch_in = FpMotionInput { crouching: true, ..rest_input() };
    step_for(&mut crouch, &cfg, &crouch_in, 0.5, 60);
    assert!(crouch.offset().y < -0.02);
    assert!(crouch.offset().z > 0.02);
}

#[test]
fn landing_dips_more_for_harder_impact_then_recovers() {
    let cfg = FpMotionConfig::default();
    let measure_dip = |fall_vy: f32| {
        let mut m = FpMotion::default();
        let air = FpMotionInput { on_floor: false, vertical_velocity: fall_vy, ..rest_input() };
        m.update(1.0 / 60.0, &air, &cfg);
        let land = rest_input();
        let mut min_y = 0.0_f32;
        for _ in 0..18 {
            m.update(1.0 / 60.0, &land, &cfg);
            min_y = min_y.min(m.offset().y);
        }
        step_for(&mut m, &cfg, &land, 1.0, 60);
        (min_y, m.offset().y)
    };
    let (soft_min, soft_end) = measure_dip(-2.5);
    let (hard_min, hard_end) = measure_dip(-7.5);
    assert!(soft_min < -0.015);
    assert!(hard_min < soft_min - 0.02, "hard {hard_min} vs soft {soft_min}");
    assert!(soft_end.abs() < 0.006 && hard_end.abs() < 0.006);
}

#[test]
fn reload_dips_and_returns() {
    let cfg = FpMotionConfig::default();
    let mut m = FpMotion::default();
    let mut input = rest_input();
    m.update(1.0 / 60.0, &input, &cfg);

    input.reloading = true;
    step_for(&mut m, &cfg, &input, 0.4, 60);
    assert!(m.offset().y < -0.06);
    assert!(m.rot_deg().z > 15.0);

    input.reloading = false;
    step_for(&mut m, &cfg, &input, 0.8, 60);
    assert!(m.offset().y.abs() < 0.008);
    assert!(m.rot_deg().z.abs() < 0.5);
}

#[test]
fn switching_weapons_starts_low_and_rises() {
    let cfg = FpMotionConfig::default();
    let mut m = FpMotion::default();
    let mut input = rest_input();
    m.update(1.0 / 60.0, &input, &cfg);

    input.weapon_key = 2;
    m.update(1.0 / 60.0, &input, &cfg);
    let start_y = m.offset().y;
    assert!(start_y < -0.10, "expected low start on switch, got {start_y}");

    step_for(&mut m, &cfg, &input, 0.5, 60);
    assert!(m.offset().y > start_y + 0.09);
    assert!(m.offset().y.abs() < 0.006);
}

#[test]
fn frame_rate_independence_and_extreme_dt_finite() {
    let cfg = FpMotionConfig::default();
    let run = |hz: u32| {
        let mut m = FpMotion::default();
        let dt = 1.0 / hz as f32;
        let mut input = FpMotionInput { on_floor: false, vertical_velocity: -5.0, ..rest_input() };
        m.update(dt, &input, &cfg);
        input.on_floor = true;
        input.vertical_velocity = 0.0;
        input.sprinting = true;
        input.horizontal_speed = 6.0;
        // 0.2 s of running at both rates (30 Hz gives 6 frames, 120 Hz gives 24 frames).
        for i in 0..(hz / 5) {
            if i == hz / 10 {
                input.weapon_key = 3;
            }
            input.yaw += 2.0 * dt;
            m.update(dt, &input, &cfg);
        }
        (m.offset(), m.rot_deg())
    };
    let (off30, rot30) = run(30);
    let (off120, rot120) = run(120);
    assert!((off30 - off120).length() < 0.005, "off30={off30:?} off120={off120:?}");
    assert!((rot30 - rot120).length() < 0.5, "rot30={rot30:?} rot120={rot120:?}");

    let mut edge = FpMotion::default();
    let input = FpMotionInput { yaw: 0.5, sprinting: true, ..rest_input() };
    edge.update(0.0, &input, &cfg);
    assert!(edge.offset().is_finite() && edge.rot_deg().is_finite());
    edge.update(0.25, &FpMotionInput { yaw: 1.0, ..input }, &cfg);
    assert!(edge.offset().is_finite() && edge.rot_deg().is_finite());
}
