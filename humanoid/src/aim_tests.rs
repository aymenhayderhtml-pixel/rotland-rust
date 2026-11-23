use std::f32::consts::FRAC_PI_2;

use glam::{Quat, Vec3};

use crate::aim::{AimConfig, AimController, AimJoints};
use crate::pose_buffer::LocalTransform;
use crate::stance::{StanceClass, StanceTable};

const TEST_JOINTS: &[&str] = &["hips", "spine", "chest", "neck", "head", "upper_arm_R"];
const IDENTITY_LOCAL: LocalTransform = (Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);
const FORWARD: Vec3 = Vec3::new(0.0, 0.0, -1.0); // godot: forward = -Z
const DT_60: f32 = 1.0 / 60.0; // new: DECISIONS 2026-11-23
const DT_SLOW: f32 = 0.1; // new: DECISIONS 2026-11-23
const TOL: f32 = 1e-3; // new: DECISIONS 2026-11-23

fn load_aim_config() -> AimConfig {
    let text = std::fs::read_to_string(paths::repo_path("data/aim.json"))
        .expect("read data/aim.json");
    AimConfig::from_json(&text).expect("parse data/aim.json")
}

fn test_rig() -> (AimJoints, Vec<LocalTransform>) {
    (AimJoints::from_names(TEST_JOINTS), vec![IDENTITY_LOCAL; TEST_JOINTS.len()])
}

fn controller_for(class: Option<StanceClass>) -> AimController {
    let mut ctrl = AimController::with_config(load_aim_config());
    ctrl.set_class(class);
    ctrl
}

#[test]
fn aim_json_parses_and_clips_exist_in_poses_aim_py() {
    let cfg = load_aim_config();
    assert_eq!(cfg.version, 1);
    let py = std::fs::read_to_string(paths::repo_path("tools/blender/characters/poses_aim.py"))
        .expect("read tools/blender/characters/poses_aim.py");

    for (class_name, class_cfg) in &cfg.classes {
        if let Some(clip) = &class_cfg.reload_clip {
            assert!(
                py.contains(&format!("\"{clip}\"")),
                "reload clip '{clip}' for class '{class_name}' missing from poses_aim.py"
            );
        }
    }

    let stance_text = std::fs::read_to_string(paths::repo_path("data/stances.json"))
        .expect("read data/stances.json");
    let stances = StanceTable::from_json(&stance_text).expect("parse data/stances.json");
    for weapon in ["pistol", "rifle", "ak47", "smg", "shotgun", "bow"] {
        let def = stances.for_weapon(weapon).expect("weapon in stances.json");
        let clip = def.aim_clip.as_deref().expect("weapon has aimClip");
        assert!(
            py.contains(&format!("\"{clip}\"")),
            "stance aimClip '{clip}' for '{weapon}' missing from poses_aim.py"
        );
    }
}

#[test]
fn pitch_clamps_to_up_and_down_limits() {
    let cfg = load_aim_config();
    let max_up = cfg.max_pitch_up_deg.to_radians();
    let max_down = cfg.max_pitch_down_deg.to_radians();
    let mut ctrl = AimController::with_config(cfg);
    ctrl.set_class(Some(StanceClass::Rifle));

    ctrl.set_look_pitch(FRAC_PI_2);
    for _ in 0..120 {
        ctrl.tick(DT_60);
    }
    assert!((ctrl.smoothed_pitch() - max_up).abs() < TOL);

    ctrl.set_look_pitch(-FRAC_PI_2);
    for _ in 0..120 {
        ctrl.tick(DT_60);
    }
    assert!((ctrl.smoothed_pitch() + max_down).abs() < TOL);
}

#[test]
fn pitch_up_and_down_rotate_chest_in_opposite_directions() {
    let (joints, base) = test_rig();
    let chest = joints.chest.expect("chest joint");

    let mut up = controller_for(Some(StanceClass::Rifle));
    up.set_look_pitch(30.0_f32.to_radians()); // new: DECISIONS 2026-11-23
    up.tick(2.0); // new: DECISIONS 2026-11-23
    let mut pose_up = base.clone();
    up.apply(&joints, &mut pose_up, 1.0);

    let mut down = controller_for(Some(StanceClass::Rifle));
    down.set_look_pitch(-30.0_f32.to_radians()); // new: DECISIONS 2026-11-23
    down.tick(2.0); // new: DECISIONS 2026-11-23
    let mut pose_down = base;
    down.apply(&joints, &mut pose_down, 1.0);

    let fwd_up = pose_up[chest].1 * FORWARD;
    let fwd_down = pose_down[chest].1 * FORWARD;
    assert!(fwd_up.y > 0.05, "positive pitch should pitch forward upward, got {fwd_up:?}");
    assert!(fwd_down.y < -0.05, "negative pitch should pitch forward downward, got {fwd_down:?}");
}

#[test]
fn carry_uses_carry_pitch_fraction() {
    let cfg = load_aim_config();
    let frac = cfg.carry_pitch_fraction;
    let (joints, base) = test_rig();
    let chest = joints.chest.expect("chest joint");

    let mut ctrl = AimController::with_config(cfg);
    ctrl.set_class(Some(StanceClass::Rifle));
    ctrl.set_look_pitch(40.0_f32.to_radians()); // new: DECISIONS 2026-11-23
    ctrl.tick(2.0); // new: DECISIONS 2026-11-23

    let mut aim_pose = base.clone();
    ctrl.apply(&joints, &mut aim_pose, 1.0);
    let mut carry_pose = base;
    ctrl.apply(&joints, &mut carry_pose, 0.0);

    let aim_angle = Quat::IDENTITY.angle_between(aim_pose[chest].1);
    let carry_angle = Quat::IDENTITY.angle_between(carry_pose[chest].1);
    assert!(aim_angle > 0.0);
    assert!((carry_angle / aim_angle - frac).abs() < TOL);
}

#[test]
fn recoil_rises_returns_to_zero_and_caps_at_max_kick() {
    let cfg = load_aim_config();
    let max_kick = cfg.max_kick_deg;
    let (joints, base) = test_rig();
    let chest = joints.chest.expect("chest joint");
    let mut ctrl = AimController::with_config(cfg);
    ctrl.set_class(Some(StanceClass::Shotgun));

    assert_eq!(ctrl.kick_deg(), 0.0);
    ctrl.fire();
    assert!(ctrl.kick_deg() > 0.0);

    let mut kicked = base.clone();
    ctrl.apply(&joints, &mut kicked, 1.0);
    assert!(kicked[chest].0.z > 0.0, "recoil should move chest back (+Z)");
    assert!((kicked[chest].1 * FORWARD).y > 0.0, "recoil should tilt chest upward");

    for _ in 0..180 {
        ctrl.tick(DT_60);
    }
    assert!(ctrl.kick_deg().abs() < TOL, "recoil did not settle: {}", ctrl.kick_deg());

    for _ in 0..25 {
        ctrl.fire();
        assert!(ctrl.kick_deg() <= max_kick + TOL);
        ctrl.tick(DT_60);
        assert!(ctrl.kick_deg() <= max_kick + TOL);
    }
}

#[test]
fn recoil_stable_at_slow_dt_and_framerate_independent() {
    let cfg = load_aim_config();
    let max_kick = cfg.max_kick_deg;

    let mut slow = AimController::with_config(cfg.clone());
    slow.set_class(Some(StanceClass::Shotgun));
    slow.set_look_pitch(25.0_f32.to_radians()); // new: DECISIONS 2026-11-23
    slow.fire();

    let mut fast = slow.clone();
    for _ in 0..10 {
        slow.tick(DT_SLOW);
        assert!(slow.kick_deg().is_finite());
        assert!(slow.kick_deg().abs() <= max_kick + TOL);
    }
    for _ in 0..100 {
        fast.tick(DT_SLOW / 10.0); // new: DECISIONS 2026-11-23
    }
    assert!((slow.kick_deg() - fast.kick_deg()).abs() < TOL);
    assert!((slow.smoothed_pitch() - fast.smoothed_pitch()).abs() < TOL);

    for _ in 0..20 {
        slow.tick(DT_SLOW);
    }
    assert!(slow.kick_deg().abs() < TOL);
}

#[test]
fn reload_speed_matches_duration_and_weapon_change_cancels_recoil_and_reload() {
    let mut ctrl = controller_for(Some(StanceClass::Rifle));
    let target_secs = 2.4; // data: weapons.json rifle.reloadTime
    let clip_dur = 1.0; // new: DECISIONS 2026-11-23
    let (clip, speed) = ctrl
        .start_reload(target_secs, |name| (name == "reload_rifle").then_some(clip_dur))
        .expect("rifle reload clip");
    assert_eq!(clip, "reload_rifle");
    assert!((clip_dur / speed - target_secs).abs() < TOL);
    assert!(ctrl.is_reloading());

    ctrl.tick(1.2); // new: DECISIONS 2026-11-23
    assert!(ctrl.is_reloading());
    ctrl.tick(1.25); // new: DECISIONS 2026-11-23
    assert!(!ctrl.is_reloading());

    ctrl.fire();
    assert!(ctrl.start_reload(target_secs, |_| Some(clip_dur)).is_some());
    assert!(ctrl.kick_deg() > 0.0 && ctrl.is_reloading());

    ctrl.set_class(Some(StanceClass::Pistol));
    assert_eq!(ctrl.kick_deg(), 0.0);
    assert!(!ctrl.is_reloading());
}

#[test]
fn no_class_has_no_effect_and_zero_or_nan_dt_is_safe() {
    let (joints, base) = test_rig();
    let mut none_ctrl = controller_for(None);
    none_ctrl.set_look_pitch(35.0_f32.to_radians()); // new: DECISIONS 2026-11-23
    none_ctrl.fire();
    none_ctrl.tick(DT_60);
    assert_eq!(none_ctrl.kick_deg(), 0.0);
    assert!(none_ctrl.start_reload(1.5, |_| Some(1.0)).is_none());

    let mut pose = base.clone();
    none_ctrl.apply(&joints, &mut pose, 1.0);
    assert_eq!(pose, base);

    let mut pistol = controller_for(Some(StanceClass::Pistol));
    pistol.set_look_pitch(f32::NAN);
    pistol.fire();
    pistol.tick(0.0);
    pistol.tick(-0.1);
    pistol.tick(f32::NAN);
    assert!(pistol.kick_deg().is_finite());
    assert!(pistol.smoothed_pitch().is_finite());

    let mut pose2 = base;
    pistol.apply(&joints, &mut pose2, f32::NAN);
    for (t, r, s) in &pose2 {
        assert!(t.is_finite() && r.is_finite() && s.is_finite());
    }
}
