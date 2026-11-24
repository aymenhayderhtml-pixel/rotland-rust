use crate::swim::{
    SwimAnimConfig, SwimState, WaterPhase, SWIM_CLIMB_OUT_CLIP, SWIM_FORWARD_CLIP,
    SWIM_TIRED_CLIP, SWIM_TREAD_CLIP, WADE_WALK_CLIP,
};

#[test]
fn config_parses_from_data_file() {
    let path = paths::repo_path("data/swim_anim.json");
    let text = std::fs::read_to_string(&path).expect("read data/swim_anim.json");
    let cfg = SwimAnimConfig::from_json(&text).expect("parse data/swim_anim.json");
    assert_eq!(cfg, SwimAnimConfig::default());
}

#[test]
fn all_five_swim_clips_defined_in_poses_swim() {
    let path = paths::repo_path("tools/blender/characters/poses_swim.py");
    let text = std::fs::read_to_string(&path).expect("read poses_swim.py");
    for clip in [
        WADE_WALK_CLIP,
        SWIM_TREAD_CLIP,
        SWIM_FORWARD_CLIP,
        SWIM_TIRED_CLIP,
        SWIM_CLIMB_OUT_CLIP,
    ] {
        let needle = format!("\"{clip}\"");
        assert!(text.contains(&needle), "missing clip {clip} in poses_swim.py");
    }
}

#[test]
fn never_driven_state_is_inert() {
    let mut s = SwimState::with_config(SwimAnimConfig::default());
    assert!(!s.tick(0.1)); // new: DECISIONS 2026-11-24
    assert_eq!(s.phase(), WaterPhase::Dry);
    assert!(!s.is_swimming());
    assert_eq!(s.swim_clip(), None);
    assert_eq!(s.swim_tempo(), 0.0);
    assert_eq!(s.wade_weight(), 0.0);
    assert!(!s.wants_wade());
}

#[test]
fn wade_weight_follows_depth_and_zeroes_while_swimming() {
    let mut s = SwimState::with_config(SwimAnimConfig::default());
    s.set_input(0.2, false, 1.0, 1.0); // new: DECISIONS 2026-11-24
    for _ in 0..20 {
        assert!(!s.tick(0.05)); // new: DECISIONS 2026-11-24
    }
    assert_eq!(s.phase(), WaterPhase::Dry);
    assert!(s.wade_weight() < 0.01); // new: DECISIONS 2026-11-24

    s.set_input(0.675, false, 1.0, 1.0); // new: DECISIONS 2026-11-24
    for _ in 0..30 {
        s.tick(0.05); // new: DECISIONS 2026-11-24
    }
    assert_eq!(s.phase(), WaterPhase::Wading);
    let mid = s.wade_weight();
    assert!((mid - 0.5).abs() < 0.05, "expected ~0.5, got {mid}"); // new: DECISIONS 2026-11-24

    s.set_input(1.0, false, 1.0, 1.0); // data: swim_anim.json wadeFullDepth
    for _ in 0..30 {
        s.tick(0.05); // new: DECISIONS 2026-11-24
    }
    assert!(s.wade_weight() > 0.95); // new: DECISIONS 2026-11-24

    s.set_input(1.25, true, 0.2, 1.0); // new: DECISIONS 2026-11-24
    s.tick(0.05); // new: DECISIONS 2026-11-24
    assert!(s.is_swimming());
    assert_eq!(s.wade_weight(), 0.0);
}

#[test]
fn swimming_phases_and_hysteresis_prevent_flicker() {
    let mut s = SwimState::with_config(SwimAnimConfig::default());

    s.set_input(1.25, true, 0.2, 1.0); // new: DECISIONS 2026-11-24
    s.tick(0.05); // new: DECISIONS 2026-11-24
    assert_eq!(s.phase(), WaterPhase::Treading);
    assert_eq!(s.swim_clip(), Some(SWIM_TREAD_CLIP));

    // Speed reaches swimMoveSpeed -> Swimming; small dip stays Swimming.
    s.set_input(1.25, true, 0.6, 1.0); // data: swim_anim.json swimMoveSpeed
    s.tick(0.05); // new: DECISIONS 2026-11-24
    assert_eq!(s.phase(), WaterPhase::Swimming);
    assert_eq!(s.swim_clip(), Some(SWIM_FORWARD_CLIP));

    s.set_input(1.25, true, 0.52, 1.0); // new: DECISIONS 2026-11-24
    s.tick(0.05); // new: DECISIONS 2026-11-24
    assert_eq!(s.phase(), WaterPhase::Swimming);

    s.set_input(1.25, true, 0.4, 1.0); // new: DECISIONS 2026-11-24
    s.tick(0.05); // new: DECISIONS 2026-11-24
    assert_eq!(s.phase(), WaterPhase::Treading);

    // Stamina hits tiredStamina -> Tired; small bounce stays Tired.
    s.set_input(1.25, true, 1.5, 0.25); // data: swim_anim.json tiredStamina
    s.tick(0.05); // new: DECISIONS 2026-11-24
    assert_eq!(s.phase(), WaterPhase::Tired);
    assert_eq!(s.swim_clip(), Some(SWIM_TIRED_CLIP));

    s.set_input(1.25, true, 1.5, 0.28); // new: DECISIONS 2026-11-24
    s.tick(0.05); // new: DECISIONS 2026-11-24
    assert_eq!(s.phase(), WaterPhase::Tired);

    s.set_input(1.25, true, 1.5, 0.35); // new: DECISIONS 2026-11-24
    s.tick(0.05); // new: DECISIONS 2026-11-24
    assert_eq!(s.phase(), WaterPhase::Swimming);
}

#[test]
fn climb_out_fires_once_on_exit_and_never_on_land() {
    let mut s = SwimState::with_config(SwimAnimConfig::default());
    s.set_input(0.0, false, 1.0, 1.0); // new: DECISIONS 2026-11-24
    assert!(!s.tick(0.05)); // new: DECISIONS 2026-11-24
    assert!(!s.tick(0.05)); // new: DECISIONS 2026-11-24

    s.set_input(1.25, true, 0.8, 0.9); // new: DECISIONS 2026-11-24
    assert!(!s.tick(0.05)); // new: DECISIONS 2026-11-24
    assert!(!s.tick(0.05)); // new: DECISIONS 2026-11-24

    s.set_input(0.4, false, 0.8, 0.9); // new: DECISIONS 2026-11-24
    assert!(s.tick(0.05)); // new: DECISIONS 2026-11-24
    assert!(!s.tick(0.05)); // new: DECISIONS 2026-11-24
    assert!(!s.tick(0.05)); // new: DECISIONS 2026-11-24
}

#[test]
fn tempo_grows_with_speed_and_stays_finite() {
    let mut s = SwimState::with_config(SwimAnimConfig::default());
    s.set_input(1.25, true, 0.6, 1.0); // data: swim_anim.json swimMoveSpeed
    let slow = s.swim_tempo();
    s.set_input(1.25, true, 1.5, 1.0); // new: DECISIONS 2026-11-24
    let mid = s.swim_tempo();
    s.set_input(1.25, true, 2.5, 1.0); // data: swim_anim.json swimFullSpeed
    let fast = s.swim_tempo();
    s.set_input(1.25, true, 20.0, 1.0); // new: DECISIONS 2026-11-24
    let clamped = s.swim_tempo();

    assert!(slow.is_finite() && mid.is_finite() && fast.is_finite() && clamped.is_finite());
    assert!(slow < mid && mid < fast);
    assert_eq!(fast, clamped);
}

#[test]
fn no_nan_at_zero_and_tenth_second_dt() {
    let mut s = SwimState::with_config(SwimAnimConfig::default());
    s.set_input(0.7, false, 1.2, 0.5); // new: DECISIONS 2026-11-24
    for dt in [0.0, 0.1] { // new: DECISIONS 2026-11-24
        s.tick(dt);
        assert!(s.wade_weight().is_finite());
        assert!(s.swim_tempo().is_finite());
    }
    s.set_input(f32::NAN, false, f32::INFINITY, f32::NAN);
    for dt in [0.0, 0.1] { // new: DECISIONS 2026-11-24
        s.tick(dt);
        assert!(s.wade_weight().is_finite());
        assert!(s.swim_tempo().is_finite());
    }
}
