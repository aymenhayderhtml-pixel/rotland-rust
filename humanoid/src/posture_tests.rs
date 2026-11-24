//! Tests for the crouch-walk, sprint, tired and catch-breath posture state (A07).

use engine::anim::Rig;
use glam::Mat4;
use pack::{Clip, Joint, Skeleton};

use crate::anim_config::AnimConfig;
use crate::animator::HumanoidAnimator;
use crate::blend::Locomotion;
use crate::layers::LayerId;
use crate::mask::MaskId;
use crate::posture::{
    CATCH_BREATH_CLIP, CROUCH_WALK_CLIP, PostureConfig, PostureState, SPRINT_ARMS_WEAPON_CLIP,
    SPRINT_CLIP, TIRED_BREATH_CLIP,
};

fn config() -> PostureConfig {
    PostureConfig::default()
}

fn state() -> PostureState {
    PostureState::with_config(config())
}

fn clip(name: &str, duration: f32) -> Clip {
    Clip { name: name.into(), duration, channels: vec![] }
}

fn posture_clips() -> Vec<Clip> {
    [
        ("idle", 2.0),
        ("walk", 1.0),
        ("run", 0.7),
        ("crouch", 1.0),
        (CROUCH_WALK_CLIP, 0.9),
        (SPRINT_CLIP, 0.55),
        (SPRINT_ARMS_WEAPON_CLIP, 0.5),
        (TIRED_BREATH_CLIP, 1.4),
        (CATCH_BREATH_CLIP, 2.0),
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

fn stand(speed: f32) -> Locomotion {
    Locomotion { speed, on_floor: true, crouching: false }
}

fn crouch(speed: f32) -> Locomotion {
    Locomotion { speed, on_floor: true, crouching: true }
}

#[test]
fn data_parses_and_matches_defaults() {
    let path = paths::repo_path("data/posture.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    let parsed = PostureConfig::from_json(&text).expect("data/posture.json must parse");
    assert_eq!(parsed, PostureConfig::default());
    assert!(parsed.crouch_walk_start_speed < parsed.crouch_walk_full_speed);
    assert!(parsed.tired_stamina_enter < parsed.tired_stamina_exit);
}

#[test]
fn poses_posture_py_defines_all_five_clips() {
    let path = paths::repo_path("tools/blender/characters/poses_posture.py");
    let text = std::fs::read_to_string(&path).expect("poses_posture.py must exist");
    for name in [
        CROUCH_WALK_CLIP,
        SPRINT_CLIP,
        SPRINT_ARMS_WEAPON_CLIP,
        TIRED_BREATH_CLIP,
        CATCH_BREATH_CLIP,
    ] {
        assert!(text.contains(&format!("\"{name}\"")), "poses_posture.py is missing {name}");
    }
}

#[test]
fn never_called_changes_nothing() {
    let skel = skeleton();
    let rig = Rig::new(&skel);
    let mut untouched = HumanoidAnimator::with_config(&skel, &posture_clips(), AnimConfig::default());
    let mut neutral = HumanoidAnimator::with_config(&skel, &posture_clips(), AnimConfig::default());

    // Inert by default: no posture layer is played.
    assert!(!state().is_driven());
    assert_eq!(neutral.layers().get(LayerId::Crouch).clip.as_deref(), Some("crouch"));
    assert_eq!(neutral.layers().get(LayerId::Aim).clip, None);
    assert_eq!(neutral.layers().get(LayerId::Additive).clip, None);

    let mut a = vec![Mat4::IDENTITY; rig.joint_count()];
    let mut b = vec![Mat4::IDENTITY; rig.joint_count()];
    for _ in 0..12 {
        // Neutral input means "rested, standing still": every posture weight stays 0.
        neutral.set_posture(0.0, false, false, 1.0);
        neutral.update(0.05, stand(0.0));
        untouched.update(0.05, stand(0.0));
        neutral.write_skin(&rig, &mut a);
        untouched.write_skin(&rig, &mut b);
        assert_eq!(a, b, "neutral posture input must not change the skin");
    }
    assert_eq!(neutral.layers().get(LayerId::Aim).clip, None);
    assert_eq!(neutral.layers().get(LayerId::Additive).clip, None);
}

#[test]
fn crouch_walk_weight_follows_speed() {
    let mut s = state();
    let mut previous = -1.0;
    for speed in [0.0, 0.5, 0.9, 1.4, 1.8] {
        s.set_input(speed, true, false, 1.0);
        for _ in 0..20 {
            s.tick(0.05);
        }
        let w = s.crouch_walk_weight();
        assert!(w.is_finite() && (0.0..=1.0).contains(&w));
        assert!(w >= previous - 1e-4, "crouch walk must grow with speed: {previous} -> {w}");
        if speed == 0.0 {
            assert!(w < 0.01);
        }
        previous = w;
    }
    assert!(previous > 0.99, "full crouch walk expected at full speed, got {previous}");

    // Standing back up drops crouch walk back to zero.
    s.set_input(0.0, false, false, 1.0);
    for _ in 0..20 {
        s.tick(0.05);
    }
    assert!(s.crouch_walk_weight() < 0.01);
}

#[test]
fn sprint_only_above_threshold() {
    let mut s = state();
    // The sprint key is held but the body is still slower than the threshold.
    s.set_input(5.0, false, true, 1.0);
    for _ in 0..30 {
        s.tick(0.05);
    }
    assert!(s.sprint_weight() < 0.01);

    s.set_input(7.0, false, true, 1.0);
    for _ in 0..20 {
        s.tick(0.05);
    }
    assert!(s.sprint_weight() > 0.99);

    s.set_input(7.0, false, false, 1.0);
    for _ in 0..20 {
        s.tick(0.05);
    }
    assert!(s.sprint_weight() < 0.01);
}

#[test]
fn tired_fades_without_flicker_at_the_edge() {
    let mut s = state();
    s.set_input(0.0, false, false, 0.2);
    for _ in 0..40 {
        s.tick(0.05);
    }
    assert!(s.tired_weight() > 0.9, "tired must fade in below the enter threshold");

    // Between the two thresholds the state holds, so a wobbling stamina bar cannot flicker.
    for stamina in [0.45, 0.36, 0.49, 0.34, 0.48] {
        s.set_input(0.0, false, false, stamina);
        for _ in 0..10 {
            s.tick(0.05);
        }
        assert!(s.tired_weight() > 0.8, "tired flickered at stamina {stamina}");
    }

    s.set_input(0.0, false, false, 0.6);
    for _ in 0..40 {
        s.tick(0.05);
    }
    assert!(s.tired_weight() < 0.1, "tired must fade out above the exit threshold");
}

#[test]
fn catch_breath_only_after_tired_sprint_and_ends() {
    let cfg = config();
    let mut s = PostureState::with_config(cfg.clone());

    // A rested sprint does not end in a catch breath.
    s.set_input(7.0, false, true, 0.9);
    for _ in 0..20 {
        s.tick(0.05);
    }
    s.set_input(0.0, false, false, 0.9);
    s.tick(0.05);
    assert!(!s.catch_breath_active());

    // A tired sprint that ends does.
    s.set_input(7.0, false, true, 0.2);
    for _ in 0..20 {
        s.tick(0.05);
    }
    assert!(!s.catch_breath_active(), "no catch breath while still sprinting");
    assert!(s.catch_breath_weight() < 0.01);

    s.set_input(0.0, false, false, 0.2);
    s.tick(0.05);
    assert!(s.catch_breath_active(), "a tired sprint end must start a catch breath");
    for _ in 0..6 {
        s.tick(0.05);
    }
    assert!(s.catch_breath_weight() > 0.5, "catch breath must be visible after the fade in");

    // It ends by itself after its configured time.
    for _ in 0..60 {
        s.tick(0.05);
    }
    assert!(!s.catch_breath_active());
    assert!(s.catch_breath_weight() < 0.05);
}

#[test]
fn catch_breath_cancels_when_the_player_moves_again() {
    let mut s = state();
    s.set_input(7.0, false, true, 0.2);
    for _ in 0..10 {
        s.tick(0.05);
    }
    s.set_input(0.0, false, false, 0.2);
    s.tick(0.05);
    assert!(s.catch_breath_active());

    s.set_input(3.0, false, false, 0.2);
    s.tick(0.05);
    assert!(!s.catch_breath_active(), "walking off cancels the catch breath");
    for _ in 0..20 {
        s.tick(0.05);
    }
    assert!(s.catch_breath_weight() < 0.05);
}

#[test]
fn stable_and_finite_at_zero_and_large_dt() {
    let mut s = state();
    s.set_input(2.0, true, false, 0.2);
    for _ in 0..10 {
        s.tick(0.1);
    }
    let before = s.crouch_walk_weight();
    for _ in 0..100 {
        s.tick(0.0);
    }
    assert_eq!(s.crouch_walk_weight(), before, "dt = 0 must not change any weight");

    for _ in 0..10 {
        s.tick(0.1);
    }
    for w in [
        s.crouch_walk_weight(),
        s.sprint_weight(),
        s.tired_weight(),
        s.catch_breath_weight(),
    ] {
        assert!(w.is_finite() && (0.0..=1.0).contains(&w));
    }

    // Broken inputs are sanitised.
    s.set_input(f32::NAN, false, false, f32::NAN);
    for _ in 0..5 {
        s.tick(0.1);
    }
    assert!(s.tired_weight().is_finite());
    assert!(s.crouch_walk_weight().is_finite());
}

#[test]
fn animator_plays_posture_layers() {
    let skel = skeleton();
    let mut anim = HumanoidAnimator::with_config(&skel, &posture_clips(), AnimConfig::default());

    // Sprint: the whole body goes on the Crouch layer, replacing the crouch idle.
    anim.set_posture(7.0, false, true, 1.0);
    for _ in 0..10 {
        anim.update(0.1, stand(7.0));
    }
    let crouch_layer = anim.layers().get(LayerId::Crouch);
    assert_eq!(crouch_layer.clip.as_deref(), Some(SPRINT_CLIP));
    assert_eq!(crouch_layer.mask, MaskId::Full);
    assert!(crouch_layer.weight > 0.9);

    // Sprint ends: the Crouch layer goes back to the crouch clip and fades out.
    anim.set_posture(2.0, false, false, 1.0);
    for _ in 0..10 {
        anim.update(0.1, stand(2.0));
    }
    let crouch_layer = anim.layers().get(LayerId::Crouch);
    assert_eq!(crouch_layer.clip.as_deref(), Some("crouch"));
    assert!(crouch_layer.weight < 0.05);

    // Crouch walk: legs come from the Aim layer, crouch idle stays on the Crouch layer.
    anim.set_posture(1.8, true, false, 1.0);
    for _ in 0..10 {
        anim.update(0.1, crouch(1.8));
    }
    let aim = anim.layers().get(LayerId::Aim);
    assert_eq!(aim.clip.as_deref(), Some(CROUCH_WALK_CLIP));
    assert_eq!(aim.mask, MaskId::Lower);
    assert!(aim.weight > 0.9);
    assert!(anim.layers().get(LayerId::Crouch).weight > 0.9);

    // Tired breathing rides on the additive layer.
    anim.set_posture(0.0, false, false, 0.2);
    for _ in 0..40 {
        anim.update(0.1, stand(0.0));
    }
    let additive = anim.layers().get(LayerId::Additive);
    assert_eq!(additive.clip.as_deref(), Some(TIRED_BREATH_CLIP));
    assert_eq!(additive.mask, MaskId::Upper);
    assert!(additive.weight > 0.5);

    // Catch breath takes over the whole body after a tired sprint.
    anim.set_posture(7.0, false, true, 0.2);
    for _ in 0..10 {
        anim.update(0.1, stand(7.0));
    }
    anim.set_posture(0.0, false, false, 0.2);
    for _ in 0..6 {
        anim.update(0.1, stand(0.0));
    }
    let aim = anim.layers().get(LayerId::Aim);
    assert_eq!(aim.clip.as_deref(), Some(CATCH_BREATH_CLIP));
    assert_eq!(aim.mask, MaskId::Full);
    assert!(aim.weight > 0.5);
    assert!(anim.layers().get(LayerId::Additive).weight < 0.1, "breathing fades during catch breath");
}

#[test]
fn weapon_sprint_uses_the_arms_clip() {
    let skel = skeleton();

    // Unarmed: the sprint clip already pumps the arms, nothing goes on the Aim layer.
    let mut unarmed = HumanoidAnimator::with_config(&skel, &posture_clips(), AnimConfig::default());
    unarmed.set_posture(7.0, false, true, 1.0);
    for _ in 0..10 {
        unarmed.update(0.1, stand(7.0));
    }
    assert_eq!(unarmed.layers().get(LayerId::Aim).clip, None);

    // Armed: the weapon is tucked low across the chest by the upper-body clip.
    let mut armed = HumanoidAnimator::with_config(&skel, &posture_clips(), AnimConfig::default());
    armed.set_weapon(Some("rifle"));
    armed.set_posture(7.0, false, true, 1.0);
    for _ in 0..10 {
        armed.update(0.1, stand(7.0));
    }
    let aim = armed.layers().get(LayerId::Aim);
    assert_eq!(aim.clip.as_deref(), Some(SPRINT_ARMS_WEAPON_CLIP));
    assert_eq!(aim.mask, MaskId::Upper);
    assert!(aim.weight > 0.9);
    assert_eq!(armed.layers().get(LayerId::Crouch).clip.as_deref(), Some(SPRINT_CLIP));
}
