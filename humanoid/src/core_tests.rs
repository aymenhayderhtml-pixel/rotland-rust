//! Unit tests for the animation core foundation (masks, pose pool, layers, full-body, events, config).

use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use engine::anim::Rig;
use glam::{Mat4, Quat, Vec3};
use pack::{Channel, ChannelKind, Clip, Interpolation, Joint, Skeleton};

use crate::action::ActionState;
use crate::anim_config::{AnimConfig, ClipEventDef};
use crate::animator::HumanoidAnimator;
use crate::blend::{
    blend_local_poses, sample_locomotion, tempo_for, Locomotion, ACTION_FADE_IN, ACTION_FADE_OUT,
    BLEND_RATE, RUN_BLEND, UPPER_BONES,
};
use crate::clip_events::{ClipEvent, MAX_QUEUED_EVENTS};
use crate::full_body::FullBodyOptions;
use crate::mask::{BoneMasks, MaskId};
use crate::pose_buffer::{add_pose_masked, blend_pose_masked, LocalTransform};

fn make_skeleton() -> Skeleton {
    let joint = |name: &str, parent: Option<u16>, y: f32| Joint {
        name: name.into(),
        parent,
        inverse_bind: Mat4::from_translation(Vec3::new(0.0, -y, 0.0)).to_cols_array(),
        rest_translation: [0.0, if parent.is_some() { 0.4 } else { 0.0 }, 0.0],
        rest_rotation: [0.0, 0.0, 0.0, 1.0],
        rest_scale: [1.0, 1.0, 1.0],
    };
    Skeleton {
        joints: vec![
            joint("root", None, 0.0),
            joint("hips", Some(0), 0.4),
            joint("thigh_L", Some(1), 0.8),
            joint("spine", Some(1), 0.8),
            joint("chest", Some(3), 1.2),
            joint("head", Some(4), 1.6),
            joint("hand_R", Some(4), 1.6),
        ],
    }
}

fn rot_clip(name: &str, duration: f32, joint: u16, angle_start: f32, angle_end: f32) -> Clip {
    let q0 = Quat::from_rotation_y(angle_start).to_array();
    let q1 = Quat::from_rotation_y(angle_end).to_array();
    Clip {
        name: name.into(),
        duration,
        channels: vec![
            Channel {
                joint,
                kind: ChannelKind::Rotation,
                interpolation: Interpolation::Linear,
                times: vec![0.0, duration],
                values: vec![q0, q1],
            },
            Channel {
                joint: 1,
                kind: ChannelKind::Translation,
                interpolation: Interpolation::Linear,
                times: vec![0.0, duration],
                values: vec![[0.0, 0.4, 0.0, 0.0], [0.0, 0.4 + angle_end * 0.1, 0.0, 0.0]],
            },
        ],
    }
}

fn assert_skins_close(a: &[Mat4], b: &[Mat4], eps: f32, label: &str) {
    assert_eq!(a.len(), b.len(), "{label}: length mismatch");
    for (i, (ma, mb)) in a.iter().zip(b).enumerate() {
        assert!(ma.abs_diff_eq(*mb, eps), "{label}: joint {i} mismatch\n  a={ma:?}\n  b={mb:?}");
    }
}

#[test]
fn local_pose_into_matches_local_pose() {
    let skel = make_skeleton();
    let rig = Rig::new(&skel);
    let clip = rot_clip("walk", 1.0, 3, 0.0, FRAC_PI_2);
    let mut buf = vec![(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE); rig.joint_count()];
    for t in [0.0, 0.25, 0.75, 1.35] {
        let expected = rig.local_pose(&clip, t);
        rig.local_pose_into(&clip, t, &mut buf);
        for (a, b) in expected.iter().zip(&buf) {
            assert!(a.0.abs_diff_eq(b.0, 1e-6));
            assert!(a.1.abs_diff_eq(b.1, 1e-6));
            assert!(a.2.abs_diff_eq(b.2, 1e-6));
        }
    }
}

#[test]
fn masks_match_upper_bones_and_handle_missing_or_unknown_bones() {
    let names = ["root", "hips", "spine", "chest", "head", "hand_R", "mystery_tail"];
    let masks = BoneMasks::new(&names);
    let upper = masks.weights(MaskId::Upper);
    for (name, &w) in names.iter().zip(upper) {
        let expected = if UPPER_BONES.contains(name) { 1.0 } else { 0.0 };
        assert_eq!(w, expected, "Upper mask mismatch for {name}");
    }
    // Unknown bone gets 1.0 in Full and 0.0 in every other mask.
    let unknown_idx = names.len() - 1;
    assert_eq!(masks.weights(MaskId::Full)[unknown_idx], 1.0);
    for id in [MaskId::Upper, MaskId::Lower, MaskId::Spine, MaskId::Arms, MaskId::LeftArm, MaskId::RightArm, MaskId::Head] {
        assert_eq!(masks.weights(id)[unknown_idx], 0.0, "unknown bone should be 0 in {id:?}");
    }
    // Partial skeleton missing most bones is safe.
    let tiny = BoneMasks::new(&["hips", "hand_L"]);
    assert_eq!(tiny.weights(MaskId::Upper), &[0.0, 1.0]);
    assert_eq!(tiny.weights(MaskId::LeftArm), &[0.0, 1.0]);
    assert_eq!(tiny.weights(MaskId::RightArm), &[0.0, 0.0]);
}

#[test]
fn blend_pose_masked_weights_shortest_path_and_matches_legacy() {
    let q_base = Quat::from_rotation_y(0.2).normalize();
    let q_over = Quat::from_rotation_y(1.0).normalize();
    let base_src: Vec<LocalTransform> = vec![
        (Vec3::new(1.0, 2.0, 3.0), q_base, Vec3::ONE),
        (Vec3::new(4.0, 5.0, 6.0), q_base, Vec3::ONE),
    ];
    let over: Vec<LocalTransform> = vec![
        (Vec3::new(3.0, 6.0, 9.0), q_over, Vec3::splat(2.0)),
        (Vec3::new(8.0, 10.0, 12.0), q_over, Vec3::splat(3.0)),
    ];
    let full = [1.0, 1.0];

    // Weight 0 leaves base unchanged; weight 1 equals over.
    let mut p0 = base_src.clone();
    blend_pose_masked(&mut p0, &over, 0.0, &full);
    assert!(p0[0].0.abs_diff_eq(base_src[0].0, 1e-6));

    let mut p1 = base_src.clone();
    blend_pose_masked(&mut p1, &over, 1.0, &full);
    assert!(p1[0].0.abs_diff_eq(over[0].0, 1e-6));
    assert!(p1[0].1.abs_diff_eq(over[0].1, 1e-6));

    // Weight 0.5 matches legacy blend_local_poses.
    let mut p_new = base_src.clone();
    let mut p_old = base_src.clone();
    blend_pose_masked(&mut p_new, &over, 0.5, &full);
    blend_local_poses(&mut p_old, &over, 0.5, None);
    for (a, b) in p_new.iter().zip(&p_old) {
        assert!(a.0.abs_diff_eq(b.0, 1e-5));
        assert!(a.1.abs_diff_eq(b.1, 1e-5));
        assert!(a.2.abs_diff_eq(b.2, 1e-5));
    }

    // Shortest path with a negated target quaternion matches the non-negated result.
    let over_neg = vec![(over[0].0, -q_over, over[0].2)];
    let mut p_neg = vec![base_src[0]];
    blend_pose_masked(&mut p_neg, &over_neg, 0.5, &[1.0]);
    assert!((p_neg[0].1.dot(p_new[0].1)).abs() > 0.9999);
}

#[test]
fn additive_with_rest_delta_changes_nothing() {
    let rest: Vec<LocalTransform> = vec![(Vec3::new(0.0, 0.4, 0.0), Quat::from_rotation_x(0.3), Vec3::ONE)];
    let base_orig: Vec<LocalTransform> = vec![(Vec3::new(0.1, 0.5, -0.2), Quat::from_rotation_y(FRAC_PI_4), Vec3::ONE)];
    let mut base = base_orig.clone();
    add_pose_masked(&mut base, &rest, &rest, 1.0, &[1.0]);
    assert!(base[0].0.abs_diff_eq(base_orig[0].0, 1e-6));
    assert!(base[0].1.abs_diff_eq(base_orig[0].1, 1e-6));
    assert!(base[0].2.abs_diff_eq(base_orig[0].2, 1e-6));

    // Applying a non-zero delta at weight 1.0 onto rest reproduces delta.
    let delta: Vec<LocalTransform> = vec![(Vec3::new(0.2, 0.6, 0.0), Quat::from_rotation_x(0.8), Vec3::splat(1.2))];
    let mut from_rest = rest.clone();
    add_pose_masked(&mut from_rest, &delta, &rest, 1.0, &[1.0]);
    assert!(from_rest[0].0.abs_diff_eq(delta[0].0, 1e-5));
    assert!(from_rest[0].1.abs_diff_eq(delta[0].1, 1e-5));
    assert!(from_rest[0].2.abs_diff_eq(delta[0].2, 1e-5));
}

#[derive(Default)]
struct LegacySim {
    blend_pos: f32,
    air: f32,
    crouch: f32,
    aim: f32,
    aim_target: f32,
    aim_clip: Option<String>,
    t_idle: f32,
    t_loco: f32,
    t_air: f32,
    t_crouch: f32,
    t_aim: f32,
    action: Option<ActionState>,
}

impl LegacySim {
    fn step(&mut self, dt: f32, loco: Locomotion) {
        let k = 1.0 - (-BLEND_RATE * dt).exp();
        self.blend_pos += (loco.speed.clamp(0.0, RUN_BLEND) - self.blend_pos) * k;
        self.air += ((if loco.on_floor { 0.0 } else { 1.0 }) - self.air) * k;
        self.crouch += ((if loco.crouching { 1.0 } else { 0.0 }) - self.crouch) * k;
        self.aim += (self.aim_target - self.aim) * k;
        let tempo = tempo_for(self.blend_pos);
        self.t_idle += dt;
        self.t_loco += dt * tempo;
        self.t_air += dt;
        self.t_crouch += dt;
        self.t_aim += dt;
        if let Some(a) = &mut self.action {
            a.time += dt;
            if a.is_finished() {
                self.action = None;
            }
        }
    }

    fn skin(&self, rig: &Rig, clips: &HashMap<String, Clip>, is_upper: &[bool]) -> Vec<Mat4> {
        let mut local = sample_locomotion(
            rig,
            clips.get("idle"),
            clips.get("walk"),
            clips.get("run"),
            self.blend_pos,
            self.t_idle,
            self.t_loco,
        );
        if self.air > 0.001 && let Some(c) = clips.get("jump") {
            let p = rig.local_pose(c, self.t_air);
            blend_local_poses(&mut local, &p, self.air, None);
        }
        if self.crouch > 0.001 && let Some(c) = clips.get("crouch") {
            let p = rig.local_pose(c, self.t_crouch);
            blend_local_poses(&mut local, &p, self.crouch, None);
        }
        if self.aim > 0.001
            && let Some(name) = &self.aim_clip
            && let Some(c) = clips.get(name)
        {
            let p = rig.local_pose(c, self.t_aim);
            blend_local_poses(&mut local, &p, self.aim, Some(is_upper));
        }
        if let Some(a) = &self.action {
            let w = a.weight();
            if w > 0.001 && let Some(c) = clips.get(&a.clip_name) {
                let p = rig.local_pose(c, a.time * a.speed);
                blend_local_poses(&mut local, &p, w, Some(is_upper));
            }
        }
        let mut out = vec![Mat4::IDENTITY; rig.joint_count()];
        rig.skin(&local, &mut out);
        out
    }
}

#[test]
fn golden_pipeline_matches_legacy_math() {
    let skel = make_skeleton();
    let rig = Rig::new(&skel);
    let clips = vec![
        rot_clip("idle", 1.0, 1, 0.0, 0.2),
        rot_clip("walk", 1.0, 2, 0.0, FRAC_PI_4),
        rot_clip("run", 0.8, 2, 0.1, FRAC_PI_2),
        rot_clip("jump", 0.6, 1, 0.2, 0.6),
        rot_clip("crouch", 1.0, 1, -0.3, -0.3),
        rot_clip("aim_rifle", 1.0, 4, 0.4, 0.4),
        rot_clip("reload", 0.8, 6, 0.0, FRAC_PI_2),
    ];
    let clip_map: HashMap<String, Clip> = clips.iter().map(|c| (c.name.clone(), c.clone())).collect();
    let is_upper: Vec<bool> = skel.joints.iter().map(|j| UPPER_BONES.contains(&j.name.as_str())).collect();

    let mut anim = HumanoidAnimator::with_config(&skel, &clips, AnimConfig::default());
    let mut legacy = LegacySim::default();
    let mut skin_new = vec![Mat4::IDENTITY; rig.joint_count()];

    let scenarios = [
        ("idle", Locomotion { speed: 0.0, on_floor: true, crouching: false }, None, false),
        ("walk", Locomotion { speed: 3.5, on_floor: true, crouching: false }, None, false),
        ("run", Locomotion { speed: 6.5, on_floor: true, crouching: false }, None, false),
        ("air", Locomotion { speed: 4.0, on_floor: false, crouching: false }, None, false),
        ("crouch_aim", Locomotion { speed: 2.0, on_floor: true, crouching: true }, Some("aim_rifle"), false),
        ("action", Locomotion { speed: 4.5, on_floor: true, crouching: false }, Some("aim_rifle"), true),
    ];

    for (label, loco, aim, trigger_action) in scenarios {
        anim.set_aim_pose(aim);
        if let Some(n) = aim {
            legacy.aim_clip = Some(n.into());
            legacy.aim_target = 1.0;
        } else {
            legacy.aim_target = 0.0;
        }
        if trigger_action {
            anim.play_action("reload", 1.0);
            legacy.action = Some(ActionState::new("reload".into(), 0.8, 1.0));
        }
        for _ in 0..5 {
            anim.update(0.05, loco);
            legacy.step(0.05, loco);
            anim.write_skin(&rig, &mut skin_new);
            let skin_old = legacy.skin(&rig, &clip_map, &is_upper);
            assert_skins_close(&skin_new, &skin_old, 1e-4, label);
        }
    }
}

#[test]
fn full_body_lifecycle_crossfade_and_stop() {
    let skel = make_skeleton();
    let rig = Rig::new(&skel);
    let clips = vec![
        rot_clip("idle", 1.0, 1, 0.0, 0.0),
        rot_clip("vault", 1.0, 3, FRAC_PI_4, FRAC_PI_2),
        rot_clip("climb", 1.0, 3, -FRAC_PI_2, -FRAC_PI_4),
    ];
    let mut anim = HumanoidAnimator::with_config(&skel, &clips, AnimConfig::default());
    let mut skin = vec![Mat4::IDENTITY; rig.joint_count()];

    // Missing clip returns false.
    assert!(!anim.play_full_body("missing_clip", FullBodyOptions::default()));
    assert!(!anim.is_full_body_active());

    // Fade in and finish automatically.
    let opts = FullBodyOptions { speed: 1.0, looping: false, hold_last: false, fade_in: 0.2, fade_out: 0.2 };
    assert!(anim.play_full_body("vault", opts));
    assert!(anim.is_full_body_active());
    anim.update(0.1, Locomotion::default());
    assert!((anim.full_body_progress() - 0.1).abs() < 1e-4);
    anim.update(0.4, Locomotion::default());
    assert!((anim.full_body_progress() - 0.5).abs() < 1e-4);
    anim.update(0.6, Locomotion::default());
    assert!(!anim.is_full_body_active());
    assert_eq!(anim.full_body_progress(), 0.0);

    // Hold last stays active at progress 1.0 until stopped.
    let hold_opts = FullBodyOptions { speed: 1.0, looping: false, hold_last: true, fade_in: 0.1, fade_out: 0.2 };
    assert!(anim.play_full_body("vault", hold_opts));
    anim.update(1.5, Locomotion::default());
    assert!(anim.is_full_body_active());
    assert!((anim.full_body_progress() - 1.0).abs() < 1e-4);

    // Crossfade from vault to climb blends smoothly between both clips.
    anim.write_skin(&rig, &mut skin);
    let vault_end = skin.clone();
    assert!(anim.play_full_body("climb", hold_opts));
    anim.update(0.05, Locomotion::default());
    anim.write_skin(&rig, &mut skin);
    assert!(!skin[3].abs_diff_eq(vault_end[3], 1e-3), "crossfade should move away from vault pose");

    // stop_full_body fades out over custom duration then clears.
    anim.stop_full_body(Some(0.2));
    anim.update(0.1, Locomotion::default());
    assert!(anim.is_full_body_active());
    anim.update(0.15, Locomotion::default());
    assert!(!anim.is_full_body_active());
}

#[test]
fn play_die_fades_in_and_reset_alive_clears() {
    let skel = make_skeleton();
    let rig = Rig::new(&skel);
    let clips = vec![
        rot_clip("idle", 1.0, 3, 0.0, 0.0),
        rot_clip("die", 1.0, 3, FRAC_PI_2, FRAC_PI_2),
    ];
    let mut anim = HumanoidAnimator::with_config(&skel, &clips, AnimConfig::default());
    let mut idle_skin = vec![Mat4::IDENTITY; rig.joint_count()];
    anim.write_skin(&rig, &mut idle_skin);

    anim.play_die();
    assert!(anim.is_dying());
    assert!(anim.is_full_body_active());

    // Immediately after play_die (t=0), weight is 0 so pose did not snap.
    let mut die_t0 = vec![Mat4::IDENTITY; rig.joint_count()];
    anim.write_skin(&rig, &mut die_t0);
    assert_skins_close(&die_t0, &idle_skin, 1e-4, "die t=0");

    // Halfway through dieFadeIn (0.075 / 0.15), pose is strictly between idle and die.
    anim.update(0.075, Locomotion::default());
    let mut die_mid = vec![Mat4::IDENTITY; rig.joint_count()];
    anim.write_skin(&rig, &mut die_mid);
    assert!(!die_mid[3].abs_diff_eq(idle_skin[3], 1e-3));

    // Past dieFadeIn, fully in die pose and holds last frame.
    anim.update(2.0, Locomotion::default());
    assert!(anim.is_dying());
    assert!((anim.full_body_progress() - 1.0).abs() < 1e-4);

    // reset_alive clears dying and restores normal animation.
    anim.reset_alive();
    assert!(!anim.is_dying());
    assert!(!anim.is_full_body_active());
    let mut alive_skin = vec![Mat4::IDENTITY; rig.joint_count()];
    anim.write_skin(&rig, &mut alive_skin);
    assert_skins_close(&alive_skin, &idle_skin, 1e-4, "reset_alive");
}

#[test]
fn clip_events_fire_once_handle_dt_speed_loops_lod_and_limit() {
    let skel = make_skeleton();
    let clips = vec![
        rot_clip("reload", 1.0, 6, 0.0, 0.5),
        rot_clip("climb", 1.0, 3, 0.0, 0.5),
    ];
    let mut cfg = AnimConfig::default();
    cfg.events.insert(
        "reload".into(),
        vec![
            ClipEventDef { name: "mag_out".into(), time: 0.3 },
            ClipEventDef { name: "mag_in".into(), time: 0.8 },
        ],
    );
    cfg.events.insert("climb".into(), vec![ClipEventDef { name: "step".into(), time: 0.5 }]);
    cfg.events.insert("ghost_clip".into(), vec![ClipEventDef { name: "never".into(), time: 0.1 }]);

    let mut anim = HumanoidAnimator::with_config(&skel, &clips, cfg);
    let mut out: Vec<ClipEvent> = Vec::new();

    // Once each with small steps.
    anim.play_action("reload", 1.0);
    for _ in 0..10 {
        anim.update(0.1, Locomotion::default());
    }
    anim.drain_events(&mut out);
    assert_eq!(out.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), vec!["mag_out", "mag_in"]);
    out.clear();

    // Large dt in one step fires both events in order.
    anim.play_action("reload", 1.0);
    anim.update(2.0, Locomotion::default());
    anim.drain_events(&mut out);
    assert_eq!(out.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), vec!["mag_out", "mag_in"]);
    out.clear();

    // Speed 2.0 reaches 0.8 clip-time in 0.4 real seconds.
    anim.play_action("reload", 2.0);
    anim.update(0.2, Locomotion::default());
    anim.drain_events(&mut out);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].name, "mag_out");
    out.clear();
    anim.update(0.25, Locomotion::default());
    anim.drain_events(&mut out);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].name, "mag_in");
    out.clear();

    // Looping full-body clip wraps 3 times in one large step.
    let loop_opts = FullBodyOptions { speed: 1.0, looping: true, hold_last: false, fade_in: 0.0, fade_out: 0.0 };
    assert!(anim.play_full_body("climb", loop_opts));
    anim.update(3.2, Locomotion::default());
    anim.drain_events(&mut out);
    assert_eq!(out.len(), 3);
    assert!(out.iter().all(|e| e.clip == "climb" && e.name == "step"));
    out.clear();
    anim.stop_full_body(Some(0.0));

    // LOD throttle preserves events when accumulated_dt fires.
    anim.set_update_interval(0.5);
    anim.play_action("reload", 1.0);
    anim.update(0.2, Locomotion::default());
    anim.update(0.2, Locomotion::default());
    anim.update(0.2, Locomotion::default()); // accumulated 0.6 >= 0.5 -> fires mag_out
    anim.drain_events(&mut out);
    assert_eq!(out.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), vec!["mag_out"]);
    out.clear();
    anim.set_update_interval(0.0);

    // Bounded buffer never exceeds MAX_QUEUED_EVENTS when undrained.
    assert!(anim.play_full_body("climb", loop_opts));
    for _ in 0..(MAX_QUEUED_EVENTS + 40) {
        anim.update(1.0, Locomotion::default());
    }
    anim.drain_events(&mut out);
    assert_eq!(out.len(), MAX_QUEUED_EVENTS);
}

#[test]
fn anim_config_defaults_json_and_repo_file() {
    let def = AnimConfig::default();
    assert_eq!(def.blend_rate, BLEND_RATE);
    assert_eq!(def.action_fade_in, ACTION_FADE_IN);
    assert_eq!(def.action_fade_out, ACTION_FADE_OUT);

    let from_empty = AnimConfig::from_json("{}").expect("empty JSON should use defaults");
    assert_eq!(from_empty, def);
    assert!(AnimConfig::from_json("{not valid json").is_err());

    let path = paths::repo_path("data/animation.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let repo_cfg = AnimConfig::from_json(&text).expect("data/animation.json must parse");
    assert_eq!(repo_cfg, def);
}

#[test]
fn no_nan_across_dt_values() {
    let skel = make_skeleton();
    let rig = Rig::new(&skel);
    let clips = vec![
        rot_clip("idle", 1.0, 1, 0.0, 0.2),
        rot_clip("walk", 1.0, 2, 0.0, FRAC_PI_4),
        rot_clip("run", 0.8, 2, 0.1, FRAC_PI_2),
        rot_clip("die", 1.0, 3, 0.0, FRAC_PI_2),
    ];
    let mut anim = HumanoidAnimator::with_config(&skel, &clips, AnimConfig::default());
    let mut skin = vec![Mat4::IDENTITY; rig.joint_count()];

    for dt in [0.0, 0.001, 0.5] {
        anim.update(dt, Locomotion { speed: 5.0, on_floor: true, crouching: false });
        anim.write_skin(&rig, &mut skin);
        for m in &skin {
            assert!(!m.is_nan(), "NaN skin matrix for dt={dt}");
        }
    }
    anim.play_die();
    for dt in [0.0, 0.001, 0.5] {
        anim.update(dt, Locomotion::default());
        anim.write_skin(&rig, &mut skin);
        for m in &skin {
            assert!(!m.is_nan(), "NaN skin matrix during die for dt={dt}");
        }
    }
}
