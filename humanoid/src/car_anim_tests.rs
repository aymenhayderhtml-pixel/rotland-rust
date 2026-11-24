use std::f32::consts::FRAC_PI_2;

use glam::{Mat4, Vec3};

use crate::anim_config::AnimConfig;
use crate::car_anim::{
    car_event_clip, seat_world, CarAnimConfig, CarAnimState, CarEvent, CAR_GET_OUT_CLIP,
    CAR_SIT_DRIVE_CLIP, CAR_STEER_LEFT_CLIP, CAR_STEER_RIGHT_CLIP, CAR_THROWN_OUT_CLIP,
};
use crate::layers::{LayerId, LayerStack};
use crate::mask::MaskId;

const EPS: f32 = 1e-5; // new: DECISIONS 2026-11-24
const DT_ZERO: f32 = 0.0; // new: DECISIONS 2026-11-24
const DT_FRAME: f32 = 0.05; // new: DECISIONS 2026-11-24
const DT_TENTH: f32 = 0.1; // new: DECISIONS 2026-11-24
const STEER_OVER: f32 = 2.0; // new: DECISIONS 2026-11-24

#[test]
fn data_file_parses() {
    let path = paths::repo_path("data/car_anim.json");
    let text = std::fs::read_to_string(&path).expect("read data/car_anim.json");
    let cfg = CarAnimConfig::from_json(&text).expect("parse data/car_anim.json");
    assert_eq!(cfg, CarAnimConfig::default());
    assert!(cfg.enter_fade > 0.0); // data: car_anim.json enterFade
    assert!(cfg.steer_full_rad > 0.0); // data: car_anim.json steerFullRad
    assert!(cfg.steer_rate > 0.0); // data: car_anim.json steerRate
    assert!(cfg.exit_fade > 0.0); // data: car_anim.json exitFade
    assert!(cfg.thrown_fade > 0.0); // data: car_anim.json thrownFade
}

#[test]
fn clip_names_exist_in_blender_poses_car() {
    let path = paths::repo_path("tools/blender/characters/poses_car.py");
    let text = std::fs::read_to_string(&path).expect("read poses_car.py");
    for clip in [
        CAR_SIT_DRIVE_CLIP,
        CAR_STEER_LEFT_CLIP,
        CAR_STEER_RIGHT_CLIP,
        CAR_GET_OUT_CLIP,
        CAR_THROWN_OUT_CLIP,
    ] {
        let double_q = format!("\"{clip}\"");
        let single_q = format!("'{clip}'");
        assert!(
            text.contains(&double_q) || text.contains(&single_q),
            "missing clip {clip} in poses_car.py"
        );
    }
}

#[test]
fn not_driven_does_nothing() {
    let mut state = CarAnimState::with_config(CarAnimConfig::default());
    assert_eq!(state.tick(DT_ZERO), None);
    assert_eq!(state.tick(DT_TENTH), None);
    assert!(!state.is_seated());
    assert!(state.steer_weight().abs() < EPS);
    assert!(state.seat_weight().abs() < EPS);

    let mut layers = LayerStack::new(AnimConfig::load_shared());
    let air_before = layers.get(LayerId::Air).clip.clone();
    state.release_layers(&mut layers);
    assert_eq!(layers.get(LayerId::Air).clip, air_before);
}

#[test]
fn seated_on_off_fires_got_out_once() {
    let mut state = CarAnimState::with_config(CarAnimConfig::default());
    state.set_input(true, 0.0, false); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(DT_TENTH), None);
    assert!(state.is_seated());
    assert!(state.seat_weight() > 0.0 && state.seat_weight() < 1.0); // data: car_anim.json enterFade

    state.set_input(false, 0.0, false); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(DT_TENTH), Some(CarEvent::GotOut));
    assert!(!state.is_seated());
    assert_eq!(car_event_clip(CarEvent::GotOut), CAR_GET_OUT_CLIP);
    assert_eq!(state.tick(DT_TENTH), None);
}

#[test]
fn flipped_exit_fires_thrown_out_and_clears() {
    let mut state = CarAnimState::with_config(CarAnimConfig::default());
    state.set_input(true, 0.0, true); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(DT_TENTH), None);

    // Caller no longer sees driven car on exit frame and passes flipped = false.
    state.set_input(false, 0.0, false); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(DT_TENTH), Some(CarEvent::ThrownOut));
    assert_eq!(car_event_clip(CarEvent::ThrownOut), CAR_THROWN_OUT_CLIP);
    assert_eq!(state.tick(DT_TENTH), None);

    // Next upright drive exits normally.
    state.set_input(true, 0.0, false); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(DT_TENTH), None);
    state.set_input(false, 0.0, false); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(DT_TENTH), Some(CarEvent::GotOut));
}

#[test]
fn steer_weight_is_smoothed_signed_and_clamped() {
    let cfg = CarAnimConfig::default();
    let mut state = CarAnimState::with_config(cfg.clone());
    state.set_input(true, cfg.steer_full_rad, false);
    state.tick(DT_FRAME);
    let mid = state.steer_weight();
    assert!(mid > 0.0 && mid < 1.0, "expected smoothed steer, got {mid}"); // new: DECISIONS 2026-11-24

    state.set_input(true, STEER_OVER, false);
    for _ in 0..20 {
        state.tick(DT_TENTH);
    }
    assert!(state.steer_weight() > 0.99 && state.steer_weight() <= 1.0); // data: car_anim.json steerMaxWeight

    state.set_input(true, -STEER_OVER, false);
    for _ in 0..20 {
        state.tick(DT_TENTH);
    }
    assert!(state.steer_weight() < -0.99 && state.steer_weight() >= -1.0); // data: car_anim.json steerMaxWeight
}

#[test]
fn seat_world_translates_and_rotates_offset() {
    let cfg = CarAnimConfig::default();
    let offset = Vec3::from_array(cfg.seat_offset);

    let id_seat = seat_world(Mat4::IDENTITY, &cfg);
    assert!(id_seat.transform_point3(Vec3::ZERO).distance(offset) < EPS);

    let rot = Mat4::from_rotation_y(FRAC_PI_2);
    let rot_seat = seat_world(rot, &cfg);
    let expected = rot.transform_point3(offset);
    assert!(rot_seat.transform_point3(Vec3::ZERO).distance(expected) < EPS);
}

#[test]
fn apply_and_release_layers_and_finite_at_zero_and_tenth_dt() {
    let cfg = CarAnimConfig::default();
    let mut state = CarAnimState::with_config(cfg.clone());
    let mut layers = LayerStack::new(AnimConfig::load_shared());
    let orig_crouch_clip = layers.get(LayerId::Crouch).clip.clone();
    let orig_crouch_mask = layers.get(LayerId::Crouch).mask;

    state.set_input(true, cfg.steer_full_rad, false);
    state.tick(DT_ZERO);
    assert!(state.steer_weight().is_finite());
    assert!(state.seat_weight().is_finite());

    for _ in 0..5 {
        state.tick(DT_TENTH);
    }
    state.apply_car(&mut layers);
    assert_eq!(layers.get(LayerId::Air).clip.as_deref(), Some(CAR_SIT_DRIVE_CLIP));
    assert_eq!(layers.get(LayerId::Crouch).clip.as_deref(), Some(CAR_STEER_LEFT_CLIP));
    assert_eq!(layers.get(LayerId::Crouch).mask, MaskId::Upper);
    assert!(layers.get(LayerId::Crouch).weight > 0.5); // new: DECISIONS 2026-11-24

    state.set_input(false, 0.0, false); // new: DECISIONS 2026-11-24
    state.tick(DT_TENTH);
    state.release_layers(&mut layers);
    assert_eq!(layers.get(LayerId::Crouch).clip, orig_crouch_clip);
    assert_eq!(layers.get(LayerId::Crouch).mask, orig_crouch_mask);
    assert!(seat_world(Mat4::IDENTITY, &cfg).is_finite());
}
