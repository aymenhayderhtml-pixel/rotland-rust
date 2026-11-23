use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};
use std::path::Path;

use glam::{Quat, Vec2, Vec3};

use crate::motion::{
    ClipSlots, LegSampleParams, MotionConfig, MotionJoints, MotionState, direction_weights,
    local_velocity,
};
use crate::pose_buffer::LocalTransform;

const JOINT_NAMES: &[&str] = &["hips", "thigh_L", "thigh_R", "spine", "chest"];
const LOWER_MASK: &[f32] = &[1.0, 1.0, 1.0, 0.0, 0.0]; // new: DECISIONS 2026-11-23
const TEST_DT: f32 = 0.016; // new: DECISIONS 2026-11-23
const WALK_SPEED: f32 = 4.0; // godot: PlayerController.cs:walk_speed
const EPS: f32 = 1e-4; // new: DECISIONS 2026-11-23

fn repo_file(rel: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    std::fs::read_to_string(root.join(rel))
        .unwrap_or_else(|e| panic!("failed to read {rel}: {e}"))
}

fn rest_pose() -> Vec<LocalTransform> {
    vec![(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE); JOINT_NAMES.len()]
}

fn make_leg_pose(angle_rad: f32) -> Vec<LocalTransform> {
    let mut pose = rest_pose();
    pose[1].1 = Quat::from_rotation_x(angle_rad);
    pose[2].1 = Quat::from_rotation_x(-angle_rad);
    pose[4].1 = Quat::from_rotation_y(angle_rad); // upper body must stay masked out
    pose
}

fn step_motion(state: &mut MotionState, vel: Vec3, yaw: f32, dt: f32, steps: usize, on_floor: bool) {
    for _ in 0..steps {
        state.set_input(vel, yaw);
        state.tick(dt, on_floor);
    }
}

fn all_finite(s: &MotionState) -> bool {
    let w = s.weights();
    w.forward.is_finite()
        && w.back.is_finite()
        && w.left.is_finite()
        && w.right.is_finite()
        && s.directional_amount().is_finite()
        && s.turn_amount().is_finite()
        && s.lean_roll_deg().is_finite()
        && s.lean_pitch_deg().is_finite()
        && s.tempo_scale().is_finite()
}

#[test]
fn motion_json_parses_and_matches_defaults() {
    let text = repo_file("data/motion.json");
    let cfg = MotionConfig::from_json(&text).expect("data/motion.json must parse");
    assert_eq!(cfg, MotionConfig::default());
    assert_eq!(cfg.version, 1); // data: motion.json version
}

#[test]
fn poses_move_py_defines_all_eight_clips() {
    let src = repo_file("tools/blender/characters/poses_move.py");
    for clip in [
        "move_walk_back",
        "move_run_back",
        "move_strafe_l",
        "move_strafe_r",
        "move_run_strafe_l",
        "move_run_strafe_r",
        "move_turn_l",
        "move_turn_r",
    ] {
        let key = format!("\"{clip}\"");
        assert!(src.contains(&key), "poses_move.py missing clip {clip}");
    }
}

#[test]
fn direction_weights_eight_directions_sum_to_one() {
    let zero = direction_weights(Vec2::ZERO);
    assert_eq!(zero.forward + zero.back + zero.left + zero.right, 0.0);

    for i in 0..8 {
        let angle = (i as f32) * FRAC_PI_4;
        let v = Vec2::new(angle.sin(), angle.cos());
        let w = direction_weights(v);
        let sum = w.forward + w.back + w.left + w.right;
        assert!((sum - 1.0).abs() < EPS, "dir {i} sum={sum}");
        assert!(w.forward >= 0.0 && w.back >= 0.0 && w.left >= 0.0 && w.right >= 0.0);
    }
    let diag = direction_weights(Vec2::new(-1.0, 1.0));
    assert!((diag.forward - 0.5).abs() < EPS && (diag.left - 0.5).abs() < EPS);
}

#[test]
fn local_velocity_north_and_east_yaw_convention() {
    // Facing north (yaw = 0): forward = (0, 0, -1), right = (1, 0, 0).
    let n_fwd = local_velocity(Vec3::new(0.0, 0.0, -WALK_SPEED), 0.0);
    assert!(n_fwd.x.abs() < EPS && (n_fwd.y - WALK_SPEED).abs() < EPS);
    let n_right = local_velocity(Vec3::new(WALK_SPEED, 0.0, 0.0), 0.0);
    assert!((n_right.x - WALK_SPEED).abs() < EPS && n_right.y.abs() < EPS);

    // Facing east (yaw = -PI/2): forward = (+1, 0, 0), right = (0, 0, +1).
    let e_fwd = local_velocity(Vec3::new(WALK_SPEED, 0.0, 0.0), -FRAC_PI_2);
    assert!(e_fwd.x.abs() < EPS && (e_fwd.y - WALK_SPEED).abs() < EPS);
    let e_left = local_velocity(Vec3::new(0.0, 0.0, -WALK_SPEED), -FRAC_PI_2);
    assert!((e_left.x + WALK_SPEED).abs() < EPS && e_left.y.abs() < EPS);
}

#[test]
fn straight_forward_and_uninitialized_have_no_effect_on_pose() {
    let base = make_leg_pose(0.3); // new: DECISIONS 2026-11-23
    let clip_pose = make_leg_pose(1.1); // new: DECISIONS 2026-11-23
    let slots = ClipSlots {
        walk_back: Some(0usize),
        strafe_l: Some(0usize),
        strafe_r: Some(0usize),
        ..ClipSlots::default()
    };
    let params = LegSampleParams {
        time_loco: 0.25, // new: DECISIONS 2026-11-23
        run_fraction: 0.0,
        lower_mask: LOWER_MASK,
    };

    // Before set_input is called, MotionState must leave poses and tempo untouched.
    let mut idle_state = MotionState::with_config(MotionConfig::default());
    idle_state.tick(TEST_DT, true);
    assert_eq!(idle_state.tempo_scale(), 1.0);
    let mut local = base.clone();
    let mut scratch = rest_pose();
    idle_state.apply_legs_sampled(slots, params, &mut local, &mut scratch, |_, _, out| {
        out.copy_from_slice(&clip_pose);
    });
    assert_eq!(local, base);

    // Moving straight forward leaves directional_amount at 0 and pose untouched.
    let mut fwd_state = MotionState::with_config(MotionConfig::default());
    step_motion(&mut fwd_state, Vec3::new(0.0, 0.0, -WALK_SPEED), 0.0, TEST_DT, 40, true);
    assert!(fwd_state.directional_amount() < EPS);
    assert_eq!(fwd_state.turn_amount(), 0.0);
    assert!((fwd_state.tempo_scale() - 1.0).abs() < EPS);
    fwd_state.apply_legs_sampled(slots, params, &mut local, &mut scratch, |_, _, out| {
        out.copy_from_slice(&clip_pose);
    });
    assert_eq!(local, base);
}

#[test]
fn yaw_wrap_and_body_snap_rejection() {
    // Crossing +PI -> -PI smoothly (increasing yaw = turning left) must lean left, not snap.
    let mut wrap_state = MotionState::with_config(MotionConfig::default());
    let omega = 1.5; // new: DECISIONS 2026-11-23
    let mut yaw = PI - 0.15; // new: DECISIONS 2026-11-23
    for _ in 0..20 {
        let vel = Vec3::new(-yaw.sin() * WALK_SPEED, 0.0, -yaw.cos() * WALK_SPEED);
        wrap_state.set_input(vel, yaw);
        wrap_state.tick(TEST_DT, true);
        yaw += omega * TEST_DT;
        if yaw > PI {
            yaw -= 2.0 * PI;
        }
    }
    assert!(wrap_state.lean_roll_deg() > 1.0, "expected left lean across PI wrap");

    // A single-frame 1.5 rad body snap (> maxYawRateRad * dt) must cause no lean or turn step.
    let mut snap_state = MotionState::with_config(MotionConfig::default());
    snap_state.set_input(Vec3::ZERO, 0.0);
    snap_state.tick(TEST_DT, true);
    snap_state.set_input(Vec3::ZERO, 1.5); // new: DECISIONS 2026-11-23
    snap_state.tick(TEST_DT, true);
    assert_eq!(snap_state.lean_roll_deg(), 0.0);
    assert_eq!(snap_state.lean_pitch_deg(), 0.0);
    assert_eq!(snap_state.turn_amount(), 0.0);
}

#[test]
fn lean_sign_left_right_and_fades_in_air() {
    let joints = MotionJoints::from_names(JOINT_NAMES);
    let mut left_turn = MotionState::with_config(MotionConfig::default());
    let mut right_turn = MotionState::with_config(MotionConfig::default());
    let omega = 1.6; // new: DECISIONS 2026-11-23
    let (mut yaw_l, mut yaw_r) = (0.0_f32, 0.0_f32);
    for _ in 0..25 {
        let vl = Vec3::new(-yaw_l.sin() * WALK_SPEED, 0.0, -yaw_l.cos() * WALK_SPEED);
        let vr = Vec3::new(-yaw_r.sin() * WALK_SPEED, 0.0, -yaw_r.cos() * WALK_SPEED);
        left_turn.set_input(vl, yaw_l);
        left_turn.tick(TEST_DT, true);
        right_turn.set_input(vr, yaw_r);
        right_turn.tick(TEST_DT, true);
        yaw_l += omega * TEST_DT;
        yaw_r -= omega * TEST_DT;
    }
    assert!(left_turn.lean_roll_deg() > 2.0, "left turn must have positive roll lean");
    assert!(right_turn.lean_roll_deg() < -2.0, "right turn must have negative roll lean");

    // Verify skeleton lean tilts +Y toward -X for left turn and +X for right turn.
    let mut pose_l = rest_pose();
    left_turn.apply_lean(&joints, &mut pose_l);
    let mut pose_r = rest_pose();
    right_turn.apply_lean(&joints, &mut pose_r);
    assert!((pose_l[3].1 * Vec3::Y).x < -0.01);
    assert!((pose_r[3].1 * Vec3::Y).x > 0.01);

    // Lean fades cleanly toward zero in the air.
    let peak = left_turn.lean_roll_deg().abs();
    step_motion(&mut left_turn, Vec3::new(0.0, 0.0, -WALK_SPEED), yaw_l, TEST_DT, 40, false);
    assert!(left_turn.lean_roll_deg().abs() < peak * 0.1);
}

#[test]
fn turn_in_place_only_when_stopped() {
    let mut stopped = MotionState::with_config(MotionConfig::default());
    let mut moving = MotionState::with_config(MotionConfig::default());
    let omega = 1.6; // new: DECISIONS 2026-11-23 (~92 deg/s > 45 deg/s threshold)
    let mut yaw = 0.0_f32;
    for _ in 0..20 {
        stopped.set_input(Vec3::ZERO, yaw);
        stopped.tick(TEST_DT, true);
        moving.set_input(Vec3::new(0.0, 0.0, -WALK_SPEED), yaw);
        moving.tick(TEST_DT, true);
        yaw += omega * TEST_DT;
    }
    assert!(stopped.turn_amount() > 0.5, "expected positive left turn step when stopped");
    assert_eq!(moving.turn_amount(), 0.0, "must not turn in place while moving");
}

#[test]
fn missing_clips_fallback_and_lower_mask_respected() {
    let mut state = MotionState::with_config(MotionConfig::default());
    step_motion(&mut state, Vec3::new(-WALK_SPEED, 0.0, 0.0), 0.0, TEST_DT, 50, true);
    assert!(state.directional_amount() > 0.9);
    assert!((state.tempo_scale() - MotionConfig::default().strafe_speed_scale).abs() < 0.02);

    let base = rest_pose();
    let strafe_pose = make_leg_pose(0.8); // new: DECISIONS 2026-11-23
    let params = LegSampleParams {
        time_loco: 0.25, // new: DECISIONS 2026-11-23
        run_fraction: 0.0,
        lower_mask: LOWER_MASK,
    };

    // All clips missing: falls back to base pose safely.
    let mut local = base.clone();
    let mut scratch = rest_pose();
    state.apply_legs_sampled(ClipSlots::<usize>::default(), params, &mut local, &mut scratch, |_, _, _| {
        panic!("sampler should not be called when clips are missing");
    });
    assert_eq!(local, base);

    // Strafe clip present: lower body blends, upper body (chest index 4) untouched.
    let present = ClipSlots {
        strafe_l: Some(0usize),
        ..ClipSlots::default()
    };
    state.apply_legs_sampled(present, params, &mut local, &mut scratch, |_, _, out| {
        out.copy_from_slice(&strafe_pose);
    });
    assert!(local[1].1.angle_between(strafe_pose[1].1) < 0.05);
    assert_eq!(local[4].1, base[4].1);
}

#[test]
fn frame_rate_independence_and_edge_dt_safety() {
    let mut fine = MotionState::with_config(MotionConfig::default());
    let mut coarse = MotionState::with_config(MotionConfig::default());
    let omega = 1.2; // new: DECISIONS 2026-11-23
    let total_time = 0.5_f32; // new: DECISIONS 2026-11-23

    let run = |s: &mut MotionState, steps: usize| {
        let dt = total_time / (steps as f32);
        s.set_input(Vec3::new(-WALK_SPEED, 0.0, 0.0), 0.0);
        s.tick(dt, true);
        for i in 1..=steps {
            let t = (i as f32) * dt;
            let yaw = omega * t;
            // Constant body-local velocity (-WALK_SPEED right, 0 forward) rotated by yaw.
            let vel = Vec3::new(-WALK_SPEED * yaw.cos(), 0.0, WALK_SPEED * yaw.sin());
            s.set_input(vel, yaw);
            s.tick(dt, true);
        }
    };
    run(&mut fine, 50);
    run(&mut coarse, 10);

    assert!((fine.weights().left - coarse.weights().left).abs() < 0.02);
    assert!((fine.lean_roll_deg() - coarse.lean_roll_deg()).abs() < 0.25);

    // dt = 0, dt = 0.1, zero velocity, identical positions and NaN inputs stay finite.
    let mut edge = MotionState::with_config(MotionConfig::default());
    edge.set_input(Vec3::ZERO, 0.0);
    edge.tick(0.0, true);
    assert!(all_finite(&edge));
    edge.tick(0.1, true); // new: DECISIONS 2026-11-23
    assert!(all_finite(&edge));
    edge.set_input(Vec3::new(f32::NAN, 0.0, f32::INFINITY), f32::NAN);
    edge.tick(0.0, true);
    edge.tick(0.1, true); // new: DECISIONS 2026-11-23
    assert!(all_finite(&edge));
}
