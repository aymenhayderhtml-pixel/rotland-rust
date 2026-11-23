//! Unit tests for foot placement IK, hip drop, slope alignment, and smoothing.

use glam::{Mat4, Quat, Vec3};

use crate::foot_ik::{FootIkConfig, FootIkState, FootJoints, GroundHit};
use crate::ik::{forward_kinematics, parent_first_order};
use crate::pose_buffer::LocalTransform;

const HIPS_Y: f32 = 0.90; // new: DECISIONS 2026-11-23
const HIP_HALF_X: f32 = 0.10; // new: DECISIONS 2026-11-23
const THIGH_LEN: f32 = 0.42; // new: DECISIONS 2026-11-23
const SHIN_LEN: f32 = 0.41; // new: DECISIONS 2026-11-23
const SPINE_OFFSET_Y: f32 = 0.25; // new: DECISIONS 2026-11-23
const REST_ANKLE_Y: f32 = 0.07; // data: foot_ik.json footHeight
const FRAME_DT: f32 = 1.0 / 60.0; // new: DECISIONS 2026-11-23
const SETTLE_STEPS: usize = 90; // new: DECISIONS 2026-11-23

struct TestRig {
    joints: FootJoints,
    parents: Vec<Option<usize>>,
    order: Vec<usize>,
    rest: Vec<LocalTransform>,
    spine: usize,
}

impl TestRig {
    fn new() -> Self {
        let names = ["hips", "thigh_L", "shin_L", "foot_L", "thigh_R", "shin_R", "foot_R", "spine"];
        let parents = vec![None, Some(0), Some(1), Some(2), Some(0), Some(4), Some(5), Some(0)];
        let order = parent_first_order(&parents);
        let joints = FootJoints::from_names(&names).expect("foot joints");
        let rest = vec![
            (Vec3::new(0.0, HIPS_Y, 0.0), Quat::IDENTITY, Vec3::ONE),
            (Vec3::new(-HIP_HALF_X, 0.0, 0.0), Quat::IDENTITY, Vec3::ONE),
            (Vec3::new(0.0, -THIGH_LEN, 0.0), Quat::IDENTITY, Vec3::ONE),
            (Vec3::new(0.0, -SHIN_LEN, 0.0), Quat::IDENTITY, Vec3::ONE),
            (Vec3::new(HIP_HALF_X, 0.0, 0.0), Quat::IDENTITY, Vec3::ONE),
            (Vec3::new(0.0, -THIGH_LEN, 0.0), Quat::IDENTITY, Vec3::ONE),
            (Vec3::new(0.0, -SHIN_LEN, 0.0), Quat::IDENTITY, Vec3::ONE),
            (Vec3::new(0.0, SPINE_OFFSET_Y, 0.0), Quat::IDENTITY, Vec3::ONE),
        ];
        Self { joints, parents, order, rest, spine: 7 }
    }

    #[allow(clippy::too_many_arguments)] // test helper; inputs are unrelated
    fn run(
        &self,
        ik: &mut FootIkState,
        base: &[LocalTransform],
        left: Option<GroundHit>,
        right: Option<GroundHit>,
        char_world: Mat4,
        steps: usize,
        dt: f32,
    ) -> (Vec<LocalTransform>, Vec<Mat4>) {
        let mut local = base.to_vec();
        let mut globals = vec![Mat4::IDENTITY; self.rest.len()];
        for _ in 0..steps {
            local.copy_from_slice(base);
            ik.set_ground(left, right, char_world);
            ik.tick(dt, 0.0);
            ik.apply(&self.joints, &self.parents, &self.order, &mut local, &mut globals);
        }
        forward_kinematics(&self.parents, &self.order, &local, &mut globals);
        (local, globals)
    }
}

fn hit(x: f32, y: f32, normal: Vec3) -> GroundHit {
    GroundHit { point: Vec3::new(x, y, 0.0), normal }
}

fn pos(globals: &[Mat4], idx: usize) -> Vec3 {
    globals[idx].w_axis.truncate()
}

#[test]
fn json_config_parses() {
    let cfg = FootIkConfig::from_json(include_str!("../../data/foot_ik.json")).expect("parse");
    assert_eq!(cfg.version, 1); // data: foot_ik.json version
    assert!(cfg.enabled);
    assert!((cfg.foot_height - 0.07).abs() < 1e-6); // data: foot_ik.json footHeight
    assert!((cfg.max_hip_drop - 0.32).abs() < 1e-6); // data: foot_ik.json maxHipDrop
    assert!((cfg.max_foot_tilt_deg - 28.0).abs() < 1e-6); // data: foot_ik.json maxFootTiltDeg
    assert_eq!(FootIkConfig::load_shared().version, 1);
}

#[test]
fn never_called_does_nothing() {
    let rig = TestRig::new();
    let mut ik = FootIkState::with_config(FootIkConfig::default());
    let mut local = rig.rest.clone();
    let mut globals = vec![Mat4::IDENTITY; rig.rest.len()];
    assert!(ik.probe_points().is_none());
    ik.tick(0.05, 0.0); // new: DECISIONS 2026-11-23
    ik.apply(&rig.joints, &rig.parents, &rig.order, &mut local, &mut globals);
    assert_eq!(local, rig.rest);
    assert!(ik.probe_points().is_none());
}

#[test]
fn flat_ground_preserves_pose() {
    let rig = TestRig::new();
    let mut ik = FootIkState::with_config(FootIkConfig::default());
    let (_, g) = rig.run(
        &mut ik, &rig.rest, Some(hit(-HIP_HALF_X, 0.0, Vec3::Y)),
        Some(hit(HIP_HALF_X, 0.0, Vec3::Y)), Mat4::IDENTITY, SETTLE_STEPS, FRAME_DT,
    );
    let fl = rig.joints.left.unwrap().foot;
    let fr = rig.joints.right.unwrap().foot;
    assert!((pos(&g, fl) - Vec3::new(-HIP_HALF_X, REST_ANKLE_Y, 0.0)).length() < 1e-3); // new: DECISIONS 2026-11-23
    assert!((pos(&g, fr) - Vec3::new(HIP_HALF_X, REST_ANKLE_Y, 0.0)).length() < 1e-3);
    assert!((pos(&g, rig.joints.hips).y - HIPS_Y).abs() < 1e-3);
    assert!(ik.probe_points().is_some());
}

#[test]
fn raised_step_reaches_without_hip_drop() {
    let rig = TestRig::new();
    let mut ik = FootIkState::with_config(FootIkConfig::default());
    let step = 0.20; // new: DECISIONS 2026-11-23
    let (_, g) = rig.run(
        &mut ik, &rig.rest, Some(hit(-HIP_HALF_X, step, Vec3::Y)),
        Some(hit(HIP_HALF_X, 0.0, Vec3::Y)), Mat4::IDENTITY, SETTLE_STEPS, FRAME_DT,
    );
    let l = rig.joints.left.unwrap();
    let r = rig.joints.right.unwrap();
    assert!((pos(&g, rig.joints.hips).y - HIPS_Y).abs() < 2e-3);
    assert!((pos(&g, l.foot).y - (step + REST_ANKLE_Y)).abs() < 3e-3);
    assert!(pos(&g, l.shin).z < -0.05); // new: DECISIONS 2026-11-23 (knee bends forward -Z)
    assert!((pos(&g, r.foot).y - REST_ANKLE_Y).abs() < 2e-3);
}

#[test]
fn lowered_step_drops_hips_within_limit() {
    let rig = TestRig::new();
    let mut ik = FootIkState::with_config(FootIkConfig::default());
    let drop = 0.20; // new: DECISIONS 2026-11-23
    let (_, g) = rig.run(
        &mut ik, &rig.rest, Some(hit(-HIP_HALF_X, -drop, Vec3::Y)),
        Some(hit(HIP_HALF_X, 0.0, Vec3::Y)), Mat4::IDENTITY, SETTLE_STEPS, FRAME_DT,
    );
    assert!((pos(&g, rig.joints.hips).y - (HIPS_Y - drop)).abs() < 3e-3);
    assert!((pos(&g, rig.spine).y - (HIPS_Y + SPINE_OFFSET_Y - drop)).abs() < 3e-3);
    assert!((pos(&g, rig.joints.left.unwrap().foot).y - (REST_ANKLE_Y - drop)).abs() < 4e-3);
    assert!((pos(&g, rig.joints.right.unwrap().foot).y - REST_ANKLE_Y).abs() < 4e-3);

    let deep = -0.60; // new: DECISIONS 2026-11-23
    let (_, g2) = rig.run(
        &mut ik, &rig.rest, Some(hit(-HIP_HALF_X, deep, Vec3::Y)),
        Some(hit(HIP_HALF_X, 0.0, Vec3::Y)), Mat4::IDENTITY, SETTLE_STEPS, FRAME_DT,
    );
    let actual_drop = HIPS_Y - pos(&g2, rig.joints.hips).y;
    assert!(actual_drop <= FootIkConfig::default().max_hip_drop + 2e-3);
    assert!(actual_drop > 0.30); // new: DECISIONS 2026-11-23
}

#[test]
fn mid_swing_keeps_lift() {
    let rig = TestRig::new();
    let mut ik = FootIkState::with_config(FootIkConfig::default());
    let mut swing = rig.rest.clone();
    let l = rig.joints.left.unwrap();
    let bend = 55.0_f32.to_radians(); // new: DECISIONS 2026-11-23
    swing[l.thigh].1 = Quat::from_rotation_x(bend);
    swing[l.shin].1 = Quat::from_rotation_x(-bend);
    let mut base_g = vec![Mat4::IDENTITY; rig.rest.len()];
    forward_kinematics(&rig.parents, &rig.order, &swing, &mut base_g);
    let orig_y = pos(&base_g, l.foot).y;
    assert!(orig_y > REST_ANKLE_Y + FootIkConfig::default().swing_lift_threshold);

    let step = 0.10; // new: DECISIONS 2026-11-23
    let (_, g) = rig.run(
        &mut ik, &swing, Some(hit(-HIP_HALF_X, step, Vec3::Y)),
        Some(hit(HIP_HALF_X, 0.0, Vec3::Y)), Mat4::IDENTITY, SETTLE_STEPS, FRAME_DT,
    );
    assert!((pos(&g, l.foot).y - (orig_y + step)).abs() < 5e-3);
}

#[test]
fn slope_tilts_foot_within_limit() {
    let rig = TestRig::new();
    let mut ik = FootIkState::with_config(FootIkConfig::default());
    let a18 = 18.0_f32.to_radians(); // new: DECISIONS 2026-11-23
    let n18 = Vec3::new(0.0, a18.cos(), a18.sin());
    let (_, g1) = rig.run(
        &mut ik, &rig.rest, Some(hit(-HIP_HALF_X, 0.0, n18)),
        Some(hit(HIP_HALF_X, 0.0, Vec3::Y)), Mat4::IDENTITY, SETTLE_STEPS, FRAME_DT,
    );
    let fl = rig.joints.left.unwrap().foot;
    let deg1 = g1[fl].transform_vector3(Vec3::Y).normalize().y.clamp(-1.0, 1.0).acos().to_degrees();
    assert!((deg1 - 18.0).abs() < 0.6); // new: DECISIONS 2026-11-23

    let a50 = 50.0_f32.to_radians(); // new: DECISIONS 2026-11-23
    let n50 = Vec3::new(0.0, a50.cos(), a50.sin());
    let (_, g2) = rig.run(
        &mut ik, &rig.rest, Some(hit(-HIP_HALF_X, 0.0, n50)),
        Some(hit(HIP_HALF_X, 0.0, Vec3::Y)), Mat4::IDENTITY, SETTLE_STEPS, FRAME_DT,
    );
    let deg2 = g2[fl].transform_vector3(Vec3::Y).normalize().y.clamp(-1.0, 1.0).acos().to_degrees();
    assert!(deg2 <= FootIkConfig::default().max_foot_tilt_deg + 0.5);
    assert!(deg2 > 26.5); // new: DECISIONS 2026-11-23
}

#[test]
fn out_of_reach_stays_finite() {
    let rig = TestRig::new();
    let mut ik = FootIkState::with_config(FootIkConfig::default());
    let (local, g) = rig.run(
        &mut ik, &rig.rest, Some(hit(-HIP_HALF_X, -5.0, Vec3::Y)), // new: DECISIONS 2026-11-23
        Some(hit(HIP_HALF_X, 5.0, Vec3::Y)), Mat4::IDENTITY, SETTLE_STEPS, FRAME_DT,
    );
    assert!(local.iter().all(|(t, r, s)| t.is_finite() && r.is_finite() && s.is_finite()));
    assert!(g.iter().all(|m| m.is_finite()));
}

#[test]
fn missing_hit_fades_out() {
    let rig = TestRig::new();
    let mut ik = FootIkState::with_config(FootIkConfig::default());
    let step = 0.20; // new: DECISIONS 2026-11-23
    let (_, g1) = rig.run(
        &mut ik, &rig.rest, Some(hit(-HIP_HALF_X, step, Vec3::Y)),
        Some(hit(HIP_HALF_X, 0.0, Vec3::Y)), Mat4::IDENTITY, SETTLE_STEPS, FRAME_DT,
    );
    let fl = rig.joints.left.unwrap().foot;
    assert!(pos(&g1, fl).y > 0.24); // new: DECISIONS 2026-11-23
    let (_, g2) = rig.run(
        &mut ik, &rig.rest, None, Some(hit(HIP_HALF_X, 0.0, Vec3::Y)),
        Mat4::IDENTITY, SETTLE_STEPS, FRAME_DT,
    );
    assert!((pos(&g2, fl).y - REST_ANKLE_Y).abs() < 3e-3);
}

#[test]
fn inactive_fades_out() {
    let rig = TestRig::new();
    let mut ik = FootIkState::with_config(FootIkConfig::default());
    let l_hit = Some(hit(-HIP_HALF_X, 0.20, Vec3::Y)); // new: DECISIONS 2026-11-23
    let r_hit = Some(hit(HIP_HALF_X, 0.0, Vec3::Y));
    rig.run(&mut ik, &rig.rest, l_hit, r_hit, Mat4::IDENTITY, SETTLE_STEPS, FRAME_DT);
    assert!(ik.weight() > 0.99); // new: DECISIONS 2026-11-23
    ik.set_active(false);
    let (_, g) = rig.run(&mut ik, &rig.rest, l_hit, r_hit, Mat4::IDENTITY, SETTLE_STEPS, FRAME_DT);
    assert!(ik.weight() < 0.01); // new: DECISIONS 2026-11-23
    assert!((pos(&g, rig.joints.left.unwrap().foot).y - REST_ANKLE_Y).abs() < 2e-3);
}

#[test]
fn teleport_resets_smoothing() {
    let rig = TestRig::new();
    let mut ik = FootIkState::with_config(FootIkConfig::default());
    rig.run(
        &mut ik, &rig.rest, Some(hit(-HIP_HALF_X, 0.25, Vec3::Y)), // new: DECISIONS 2026-11-23
        Some(hit(HIP_HALF_X, 0.0, Vec3::Y)), Mat4::IDENTITY, SETTLE_STEPS, FRAME_DT,
    );
    assert!(ik.weight() > 0.99);
    let far = Vec3::new(10.0, 0.0, 0.0); // new: DECISIONS 2026-11-23
    let far_world = Mat4::from_translation(far);
    ik.set_ground(Some(hit(far.x - HIP_HALF_X, 0.0, Vec3::Y)), Some(hit(far.x + HIP_HALF_X, 0.0, Vec3::Y)), far_world);
    assert!(ik.weight() < 1e-6);
    let mut local = rig.rest.clone();
    let mut g = vec![Mat4::IDENTITY; rig.rest.len()];
    ik.apply(&rig.joints, &rig.parents, &rig.order, &mut local, &mut g);
    assert!((pos(&g, rig.joints.left.unwrap().foot).y - REST_ANKLE_Y).abs() < 1e-4);
}

#[test]
fn frame_rate_independent() {
    let rig = TestRig::new();
    let a15 = 15.0_f32.to_radians(); // new: DECISIONS 2026-11-23
    let l_hit = Some(hit(-HIP_HALF_X, 0.18, Vec3::new(0.0, a15.cos(), a15.sin()))); // new: DECISIONS 2026-11-23
    let r_hit = Some(hit(HIP_HALF_X, -0.15, Vec3::Y)); // new: DECISIONS 2026-11-23
    let mut ik_fine = FootIkState::with_config(FootIkConfig::default());
    let mut ik_coarse = FootIkState::with_config(FootIkConfig::default());
    let (_, g_fine) = rig.run(&mut ik_fine, &rig.rest, l_hit, r_hit, Mat4::IDENTITY, 10, 0.01); // new: DECISIONS 2026-11-23
    let (_, g_coarse) = rig.run(&mut ik_coarse, &rig.rest, l_hit, r_hit, Mat4::IDENTITY, 1, 0.10); // new: DECISIONS 2026-11-23
    assert!((ik_fine.weight() - ik_coarse.weight()).abs() < 1e-4);
    for idx in [rig.joints.hips, rig.joints.left.unwrap().foot, rig.joints.right.unwrap().foot] {
        assert!((pos(&g_fine, idx) - pos(&g_coarse, idx)).length() < 1e-3);
    }
}

#[test]
fn zero_dt_no_nan_and_speed_reduces_weight() {
    let rig = TestRig::new();
    let mut ik = FootIkState::with_config(FootIkConfig::default());
    ik.set_ground(Some(hit(-HIP_HALF_X, 0.15, Vec3::Y)), Some(hit(HIP_HALF_X, 0.0, Vec3::Y)), Mat4::IDENTITY);
    ik.tick(0.0, 0.0);
    ik.tick(-0.1, f32::NAN); // new: DECISIONS 2026-11-23
    let mut local = rig.rest.clone();
    let mut g = vec![Mat4::IDENTITY; rig.rest.len()];
    ik.apply(&rig.joints, &rig.parents, &rig.order, &mut local, &mut g);
    assert!(ik.weight().is_finite() && g.iter().all(|m| m.is_finite()));

    for _ in 0..SETTLE_STEPS {
        ik.tick(FRAME_DT, crate::blend::RUN_BLEND.max(10.0)); // new: DECISIONS 2026-11-23
    }
    assert!((ik.weight() - FootIkConfig::default().min_weight_at_run).abs() < 0.01);
}
