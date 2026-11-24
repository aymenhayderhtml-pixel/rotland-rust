//! Tests for the airborne jump/fall/landing state machine and its animator wiring.

use glam::{Mat4, Vec3};
use pack::{Clip, Joint, Skeleton};

use crate::airborne::{AirConfig, AirPhase, AirState, LandingKind, landing_clip};
use crate::anim_config::AnimConfig;
use crate::animator::HumanoidAnimator;
use crate::blend::Locomotion;
use crate::layers::LayerId;
use crate::mask::MaskId;

fn state() -> AirState {
    AirState::with_config(AirConfig::default())
}

fn clip(name: &str, duration: f32) -> Clip {
    Clip { name: name.into(), duration, channels: vec![] }
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
            joint("thigh_L", Some(1)),
            joint("shin_L", Some(4)),
            joint("thigh_R", Some(1)),
            joint("shin_R", Some(6)),
        ],
    }
}

fn air_clips() -> Vec<Clip> {
    [
        ("jump", 0.5),
        ("air_takeoff", 0.18),
        ("air_rise", 0.5),
        ("air_fall", 0.6),
        ("land_soft", 0.28),
        ("land_hard", 0.65),
        ("land_roll", 0.75),
        ("vault", 2.0),
    ]
    .into_iter()
    .map(|(n, d)| clip(n, d))
    .collect()
}

fn airborne(anim: &mut HumanoidAnimator) {
    let loco = Locomotion { speed: 0.0, on_floor: false, crouching: false };
    for _ in 0..4 {
        anim.update(0.05, loco);
    }
}

/// Feeds one fall from `fall_speed` and returns the landing kind.
fn land_with(fall_speed: f32, horizontal: f32) -> (Option<LandingKind>, AirState) {
    let mut s = state();
    s.set_velocity(Vec3::new(horizontal, -fall_speed, 0.0));
    assert!(s.tick(0.0, true).is_none());
    for _ in 0..3 {
        assert!(s.tick(0.05, false).is_none());
    }
    let kind = s.tick(0.016, true);
    (kind, s)
}

#[test]
fn data_parses_and_matches_defaults() {
    let path = paths::repo_path("data/airborne.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let cfg = AirConfig::from_json(&text).expect("data/airborne.json must parse");
    assert_eq!(cfg, AirConfig::default());
    assert!(AirConfig::from_json("{not valid json").is_err());
}

#[test]
fn poses_file_defines_the_six_clips() {
    let path = paths::repo_path("tools/blender/characters/poses_air.py");
    let text = std::fs::read_to_string(&path).expect("poses_air.py must exist");
    assert!(text.contains("CLIPS = {"));
    for name in ["air_takeoff", "air_rise", "air_fall", "land_soft", "land_hard", "land_roll"] {
        assert!(text.contains(&format!("\"{name}\"")), "poses_air.py is missing {name}");
    }
    assert_eq!(landing_clip(LandingKind::Soft), "land_soft");
    assert_eq!(landing_clip(LandingKind::Hard), "land_hard");
    assert_eq!(landing_clip(LandingKind::Roll), "land_roll");
}

#[test]
fn short_bump_does_nothing() {
    let mut s = state();
    s.set_velocity(Vec3::new(0.0, 0.0, -2.0));
    assert!(s.tick(0.016, true).is_none());
    assert!(s.tick(0.05, false).is_none());
    assert!(s.tick(0.05, true).is_none());
    assert_eq!(s.phase(), AirPhase::Grounded);
    assert_eq!(s.air_clip(), None);
    assert_eq!(s.air_target(), 0.0);
}

#[test]
fn jump_goes_takeoff_rise_fall() {
    let mut s = state();
    s.set_velocity(Vec3::new(0.0, 4.0, 0.0));
    s.tick(0.0, true);
    s.tick(0.016, false);
    assert_eq!(s.phase(), AirPhase::Takeoff);
    assert_eq!(s.air_clip(), Some("air_takeoff"));
    assert_eq!(s.air_target(), 1.0);
    for _ in 0..3 {
        s.tick(0.05, false);
    }
    assert_eq!(s.phase(), AirPhase::Rise);
    assert_eq!(s.air_clip(), Some("air_rise"));
    s.set_velocity(Vec3::new(0.0, -3.0, 0.0));
    s.tick(0.016, false);
    assert_eq!(s.phase(), AirPhase::Fall);
    assert_eq!(s.air_clip(), Some("air_fall"));
}

#[test]
fn ledge_falls_only_after_min_air_seconds() {
    let mut s = state();
    s.set_velocity(Vec3::new(0.0, -0.2, 0.0));
    s.tick(0.0, true);
    s.tick(0.05, false);
    assert_eq!(s.phase(), AirPhase::Grounded);
    assert_eq!(s.air_target(), 0.0);
    s.tick(0.05, false);
    assert_eq!(s.phase(), AirPhase::Grounded);
    s.tick(0.05, false);
    assert_eq!(s.phase(), AirPhase::Fall);
    assert_eq!(s.air_clip(), Some("air_fall"));
    assert_eq!(s.air_target(), 1.0);
}

#[test]
fn top_of_the_jump_does_not_flicker() {
    let mut s = state();
    s.set_velocity(Vec3::new(0.0, 4.0, 0.0));
    s.tick(0.0, true);
    for _ in 0..5 {
        s.tick(0.05, false);
    }
    assert_eq!(s.phase(), AirPhase::Rise);
    for v in [0.3, 0.05, -0.5, -1.0] {
        s.set_velocity(Vec3::new(0.0, v, 0.0));
        s.tick(0.016, false);
        assert_eq!(s.phase(), AirPhase::Rise, "flickered at v={v}");
    }
    s.set_velocity(Vec3::new(0.0, -1.3, 0.0));
    s.tick(0.016, false);
    assert_eq!(s.phase(), AirPhase::Fall);
    for v in [-0.8, -1.1] {
        s.set_velocity(Vec3::new(0.0, v, 0.0));
        s.tick(0.016, false);
        assert_eq!(s.phase(), AirPhase::Fall, "flickered at v={v}");
    }
}

#[test]
fn soft_hard_and_roll_by_speed() {
    assert_eq!(land_with(0.0, 0.0).0, Some(LandingKind::Soft), "slow step down is soft");
    assert_eq!(land_with(3.0, 0.0).0, Some(LandingKind::Soft));
    assert_eq!(land_with(7.4, 0.0).0, Some(LandingKind::Soft));
    assert_eq!(land_with(7.5, 0.0).0, Some(LandingKind::Hard));
    assert_eq!(land_with(8.0, 3.4).0, Some(LandingKind::Hard));
    assert_eq!(land_with(8.0, 3.5).0, Some(LandingKind::Roll));
    assert_eq!(land_with(8.0, 4.0).0, Some(LandingKind::Roll));
    assert_eq!(land_with(0.0, 9.0).0, Some(LandingKind::Soft), "no roll without a hard fall");
}

#[test]
fn landing_fires_exactly_once_and_cools_down() {
    let (kind, mut s) = land_with(8.0, 0.0);
    assert_eq!(kind, Some(LandingKind::Hard));
    assert!(s.tick(0.016, true).is_none());
    assert_eq!(s.phase(), AirPhase::Grounded);

    // A second fall inside landCooldown stays silent, then one after it fires.
    s.set_velocity(Vec3::new(0.0, -8.0, 0.0));
    for _ in 0..3 {
        s.tick(0.05, false);
    }
    assert!(s.tick(0.016, true).is_none(), "cooldown should block the landing");
    s.set_velocity(Vec3::new(0.0, -8.0, 0.0));
    for _ in 0..3 {
        s.tick(0.05, false);
    }
    assert_eq!(s.tick(0.016, true), Some(LandingKind::Hard));
}

#[test]
fn without_set_velocity_nothing_changes() {
    let mut s = state();
    for _ in 0..10 {
        assert!(s.tick(0.05, false).is_none());
    }
    assert_eq!(s.phase(), AirPhase::Grounded);
    assert_eq!(s.air_clip(), None);
    assert_eq!(s.air_target(), 0.0);
    assert!(s.tick(0.05, true).is_none());
}

#[test]
fn finite_at_zero_and_large_dt() {
    let mut s = state();
    s.set_velocity(Vec3::new(f32::NAN, f32::INFINITY, 0.0));
    s.tick(0.0, false);
    s.tick(0.1, false);
    s.tick(f32::NAN, false);
    s.tick(-1.0, false);
    assert!(s.air_target().is_finite());
    assert!(s.phase() == AirPhase::Grounded || s.phase() == AirPhase::Fall);
    assert_eq!(s.tick(0.1, true), None);
}

#[test]
fn animator_keeps_jump_until_driven() {
    let skel = skeleton();
    let mut anim = HumanoidAnimator::with_config(&skel, &air_clips(), AnimConfig::default());
    anim.update(0.1, Locomotion { speed: 0.0, on_floor: false, crouching: false });
    let air = anim.layers().get(LayerId::Air);
    assert_eq!(air.clip.as_deref(), Some("jump"));
    assert_eq!(air.target, 1.0);
    assert!(!anim.is_full_body_active());
}

#[test]
fn animator_plays_landing_clips_and_keeps_stance_arms_on_soft() {
    let skel = skeleton();
    let loco_floor = Locomotion { speed: 0.0, on_floor: true, crouching: false };

    let mut hard = HumanoidAnimator::with_config(&skel, &air_clips(), AnimConfig::default());
    hard.set_move_velocity(Vec3::new(0.0, -8.0, 0.0), 0.0);
    airborne(&mut hard);
    hard.update(0.016, loco_floor);
    assert_eq!(hard.layers().get(LayerId::FullBody).clip.as_deref(), Some("land_hard"));
    assert_eq!(hard.layers().get(LayerId::FullBody).mask, MaskId::Full);

    let mut soft = HumanoidAnimator::with_config(&skel, &air_clips(), AnimConfig::default());
    soft.set_move_velocity(Vec3::new(0.0, -4.0, 0.0), 0.0);
    airborne(&mut soft);
    soft.update(0.016, loco_floor);
    assert_eq!(soft.layers().get(LayerId::FullBody).clip.as_deref(), Some("land_soft"));
    assert_eq!(soft.layers().get(LayerId::FullBody).mask, MaskId::Lower);
    assert_eq!(soft.layers().get(LayerId::Air).target, 0.0);

    let mut roll = HumanoidAnimator::with_config(&skel, &air_clips(), AnimConfig::default());
    roll.set_move_velocity(Vec3::new(4.0, -8.0, 0.0), 0.0);
    airborne(&mut roll);
    roll.update(0.016, Locomotion { speed: 4.0, on_floor: true, crouching: false });
    assert_eq!(roll.layers().get(LayerId::FullBody).clip.as_deref(), Some("land_roll"));
}

#[test]
fn animator_never_interrupts_another_full_body_clip() {
    let skel = skeleton();
    let mut anim = HumanoidAnimator::with_config(&skel, &air_clips(), AnimConfig::default());
    assert!(anim.play_full_body("vault", Default::default()));
    anim.set_move_velocity(Vec3::new(0.0, -8.0, 0.0), 0.0);
    airborne(&mut anim);
    anim.update(0.016, Locomotion { speed: 0.0, on_floor: true, crouching: false });
    assert_eq!(anim.layers().get(LayerId::FullBody).clip.as_deref(), Some("vault"));
}
