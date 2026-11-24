//! Tests for wounded body language animation state (A15).

use engine::anim::Rig;
use glam::Mat4;
use pack::{Clip, Joint, Skeleton};

use crate::anim_config::AnimConfig;
use crate::animator::HumanoidAnimator;
use crate::blend::Locomotion;
use crate::wound_anim::{
    DROOP_CLIP, HOLD_SIDE_CLIP, LIMP_WALK_CLIP, WoundAnimConfig, WoundAnimState, WoundContext,
};

fn state() -> WoundAnimState {
    WoundAnimState::with_config(WoundAnimConfig::default())
}

fn walk_ctx() -> WoundContext {
    WoundContext { speed: 2.0, ..Default::default() }
}

fn clip(name: &str, duration: f32) -> Clip {
    Clip { name: name.into(), duration, channels: vec![] }
}

fn wound_clips() -> Vec<Clip> {
    [
        ("idle", 2.0),
        ("walk", 1.0),
        ("run", 0.7),
        ("die", 1.2),
        (LIMP_WALK_CLIP, 1.1),
        (HOLD_SIDE_CLIP, 1.6),
        (DROOP_CLIP, 2.4),
    ]
    .into_iter()
    .map(|(name, duration)| clip(name, duration))
    .collect()
}

fn skeleton() -> Skeleton {
    let joint = |name: &str, parent: Option<u16>| Joint {
        name: name.into(),
        parent,
        inverse_bind: Mat4::IDENTITY.to_cols_array(),
        rest_translation: [0.0, 0.0, 0.0],
        rest_rotation: [0.0, 0.0, 0.0, 1.0],
        rest_scale: [1.0, 1.0, 1.0],
    };
    Skeleton {
        joints: vec![
            joint("root", None),
            joint("hips", Some(0)),
            joint("spine", Some(1)),
            joint("chest", Some(2)),
            joint("neck", Some(3)),
            joint("head", Some(4)),
            joint("shoulder_L", Some(3)),
            joint("upper_arm_L", Some(6)),
            joint("forearm_L", Some(7)),
            joint("hand_L", Some(8)),
            joint("shoulder_R", Some(3)),
            joint("upper_arm_R", Some(10)),
            joint("forearm_R", Some(11)),
            joint("hand_R", Some(12)),
            joint("thigh_L", Some(1)),
            joint("shin_L", Some(14)),
            joint("foot_L", Some(15)),
            joint("thigh_R", Some(1)),
            joint("shin_R", Some(17)),
            joint("foot_R", Some(18)),
        ],
    }
}

#[test]
fn data_parses_and_matches_defaults() {
    let path = paths::repo_path("data/wound_anim.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    let parsed = WoundAnimConfig::from_json(&text).expect("data/wound_anim.json must parse");
    assert_eq!(parsed, WoundAnimConfig::default());
    assert!(parsed.droop_threshold < parsed.hold_side_threshold);
    assert!(parsed.hold_side_threshold < parsed.limp_threshold);
}

#[test]
fn never_called_changes_nothing() {
    let skel = skeleton();
    let rig = Rig::new(&skel);
    let mut untouched = HumanoidAnimator::with_config(&skel, &wound_clips(), AnimConfig::default());
    let mut healthy = HumanoidAnimator::with_config(&skel, &wound_clips(), AnimConfig::default());

    assert!(!state().is_driven());

    let mut a = vec![Mat4::IDENTITY; rig.joint_count()];
    let mut b = vec![Mat4::IDENTITY; rig.joint_count()];
    let loco = Locomotion { speed: 2.0, on_floor: true, crouching: false };
    for _ in 0..10 {
        healthy.set_health(1.0);
        healthy.update(0.05, loco);
        untouched.update(0.05, loco);
        healthy.write_skin(&rig, &mut a);
        untouched.write_skin(&rig, &mut b);
        assert_eq!(a, b, "full health must not alter skin");
    }
}

#[test]
fn weights_zero_at_full_health_and_rise_below_thresholds() {
    let mut s = state();
    s.set_health(1.0);
    for _ in 0..20 {
        s.tick(0.05, walk_ctx());
    }
    assert_eq!(s.limp_weight(), 0.0);
    assert_eq!(s.hold_side_weight(), 0.0);
    assert_eq!(s.droop_weight(), 0.0);

    // Below limp & hold_side thresholds, above droop threshold
    s.set_health(0.30);
    for _ in 0..30 {
        s.tick(0.05, walk_ctx());
    }
    assert!(s.limp_weight() > 0.2, "limp should rise below threshold");
    assert!(s.hold_side_weight() > 0.1, "hold_side should rise below threshold");
    assert_eq!(s.droop_weight(), 0.0, "droop stays 0 above droop threshold");

    // Very low health: droop rises too
    s.set_health(0.10);
    for _ in 0..30 {
        s.tick(0.05, walk_ctx());
    }
    assert!(s.droop_weight() > 0.2, "droop should rise at very low health");
}

#[test]
fn weights_smooth_over_time() {
    let mut s = state();
    s.set_health(0.10);
    s.tick(0.016, walk_ctx());
    let first = s.limp_weight();
    assert!(first > 0.0 && first < 0.2, "first step should be small: {first}");

    s.tick(0.016, walk_ctx());
    let second = s.limp_weight();
    assert!(second > first, "weight should increase smoothly: {first} -> {second}");

    // Recovering to full health fades back smoothly
    s.set_health(1.0);
    s.tick(0.016, walk_ctx());
    let fading = s.limp_weight();
    assert!(fading < second && fading > 0.0, "recovery should fade smoothly: {fading}");
}

#[test]
fn weights_fade_to_zero_while_dying_or_swimming() {
    let mut s = state();
    s.set_health(0.10);
    for _ in 0..30 {
        s.tick(0.05, walk_ctx());
    }
    assert!(s.limp_weight() > 0.3);
    assert!(s.hold_side_weight() > 0.3);
    assert!(s.droop_weight() > 0.2);

    // Swimming suppresses all wound layers
    let swim_ctx = WoundContext { swimming: true, ..walk_ctx() };
    for _ in 0..30 {
        s.tick(0.05, swim_ctx);
    }
    assert!(s.limp_weight() < 0.01, "limp must fade out while swimming");
    assert!(s.hold_side_weight() < 0.01, "hold_side must fade out while swimming");
    assert!(s.droop_weight() < 0.01, "droop must fade out while swimming");

    // Re-build weights, then test dying via HumanoidAnimator
    let skel = skeleton();
    let mut anim = HumanoidAnimator::with_config(&skel, &wound_clips(), AnimConfig::default());
    anim.set_health(0.10);
    let loco = Locomotion { speed: 2.0, on_floor: true, crouching: false };
    for _ in 0..30 {
        anim.update(0.05, loco);
    }
    assert!(anim.wound_state().limp_weight() > 0.3);

    anim.play_die();
    for _ in 0..30 {
        anim.update(0.05, loco);
    }
    assert!(anim.wound_state().limp_weight() < 0.01, "limp must fade out while dying");
    assert!(anim.wound_state().hold_side_weight() < 0.01, "hold_side must fade out while dying");
    assert!(anim.wound_state().droop_weight() < 0.01, "droop must fade out while dying");
}
