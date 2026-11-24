//! Unit tests for ladder climbing animation state, config and clip definitions (A09).

use crate::anim_config::AnimConfig;
use crate::ladder_anim::{
    is_ladder_clip, ladder_event_clip, LadderAnimConfig, LadderAnimState, LadderEvent,
    LADDER_CLIMB_CLIP, LADDER_ENTER_BOTTOM_CLIP, LADDER_ENTER_TOP_CLIP, LADDER_EXIT_BOTTOM_CLIP,
    LADDER_EXIT_TOP_CLIP,
};
use crate::layers::{LayerId, LayerStack};

const EPS: f32 = 1e-4; // new: DECISIONS 2026-11-24
const STEP_DT: f32 = 0.016; // new: DECISIONS 2026-11-24
const BIG_DT: f32 = 0.1; // new: DECISIONS 2026-11-24
const CLIMB_CLIP_LEN: f32 = 0.8; // new: DECISIONS 2026-11-24

#[test]
fn data_json_parses() {
    let path = paths::repo_path("data/ladder_anim.json");
    let text = std::fs::read_to_string(&path).expect("read data/ladder_anim.json");
    let cfg = LadderAnimConfig::from_json(&text).expect("parse data/ladder_anim.json");
    assert_eq!(cfg, LadderAnimConfig::default());
    assert!(cfg.metres_per_cycle > 0.0);
    assert!(cfg.teleport_distance > cfg.metres_per_cycle);
}

#[test]
fn all_five_clips_defined_in_poses_ladder() {
    let path = paths::repo_path("tools/blender/characters/poses_ladder.py");
    let text = std::fs::read_to_string(&path).expect("read poses_ladder.py");
    for name in [
        LADDER_CLIMB_CLIP,
        LADDER_ENTER_BOTTOM_CLIP,
        LADDER_EXIT_TOP_CLIP,
        LADDER_ENTER_TOP_CLIP,
        LADDER_EXIT_BOTTOM_CLIP,
    ] {
        let needle = format!("\"{name}\"");
        assert!(text.contains(&needle), "missing clip {name} in poses_ladder.py");
        assert!(is_ladder_clip(name));
    }
    assert!(!is_ladder_clip("jump"));
}

#[test]
fn never_driven_does_nothing() {
    let mut state = LadderAnimState::with_config(LadderAnimConfig::default());
    let mut layers = LayerStack::new(&AnimConfig::default());
    assert!(!state.is_on_ladder());
    assert_eq!(state.tick(STEP_DT), None);
    assert_eq!(state.tick(BIG_DT), None);
    assert_eq!(state.climb_time(CLIMB_CLIP_LEN), 0.0);
    state.release_air(&mut layers);
    assert_eq!(layers.get(LayerId::Air).clip.as_deref(), Some("jump"));
}

#[test]
fn never_fires_events_on_land() {
    let mut state = LadderAnimState::with_config(LadderAnimConfig::default());
    for y in [0.0, 0.4, 0.9, 0.2, 0.0] { // new: DECISIONS 2026-11-24
        state.set_input(false, false, y);
        assert_eq!(state.tick(STEP_DT), None);
        assert!(!state.is_on_ladder());
    }
}

#[test]
fn enter_bottom_and_exit_top_fire_once_each() {
    let mut state = LadderAnimState::with_config(LadderAnimConfig::default());
    state.set_input(false, false, 0.0);
    assert_eq!(state.tick(STEP_DT), None);

    state.set_input(true, true, 0.0);
    assert_eq!(state.tick(STEP_DT), Some(LadderEvent::EnterBottom));
    assert!(state.is_on_ladder());

    state.set_input(true, true, 0.3); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(STEP_DT), None);
    state.set_input(true, true, 0.6); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(STEP_DT), None);

    state.set_input(true, false, 0.65); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(STEP_DT), Some(LadderEvent::ExitTop));
    assert!(state.is_on_ladder());

    state.set_input(true, false, 0.85); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(STEP_DT), None);

    state.set_input(false, false, 0.9); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(STEP_DT), None);
    assert!(!state.is_on_ladder());
}

#[test]
fn enter_top_and_exit_bottom_fire_once_each() {
    let mut state = LadderAnimState::with_config(LadderAnimConfig::default());
    state.set_input(true, false, 3.0); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(STEP_DT), Some(LadderEvent::EnterTop));
    assert!(state.is_on_ladder());

    state.set_input(true, false, 2.5); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(STEP_DT), None);

    state.set_input(true, true, 2.1); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(STEP_DT), None);
    assert_eq!(state.climb_time(CLIMB_CLIP_LEN), 0.0);

    state.set_input(true, true, 1.8); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(STEP_DT), None);
    state.set_input(true, true, 1.5); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(STEP_DT), None);

    state.set_input(false, false, 1.5); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(STEP_DT), Some(LadderEvent::ExitBottom));
    assert!(!state.is_on_ladder());

    state.set_input(false, false, 1.5); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(STEP_DT), None);
}

#[test]
fn climb_time_advances_reverses_holds_and_wraps() {
    let mut state = LadderAnimState::with_config(LadderAnimConfig::default());
    state.set_input(true, true, 0.0);
    let _ = state.tick(STEP_DT);
    assert!(state.climb_time(CLIMB_CLIP_LEN).abs() < EPS);

    // Up 0.3 m = half cycle -> 0.4 s
    state.set_input(true, true, 0.3); // new: DECISIONS 2026-11-24
    let _ = state.tick(STEP_DT);
    assert!((state.climb_time(CLIMB_CLIP_LEN) - 0.4).abs() < EPS); // new: DECISIONS 2026-11-24

    // Hold still -> keeps exact pose time
    state.set_input(true, true, 0.3); // new: DECISIONS 2026-11-24
    let _ = state.tick(STEP_DT);
    assert!((state.climb_time(CLIMB_CLIP_LEN) - 0.4).abs() < EPS); // new: DECISIONS 2026-11-24

    // Up to 0.75 m = 1.25 cycles -> wraps to 0.2 s
    state.set_input(true, true, 0.75); // new: DECISIONS 2026-11-24
    let _ = state.tick(STEP_DT);
    assert!((state.climb_time(CLIMB_CLIP_LEN) - 0.2).abs() < EPS); // new: DECISIONS 2026-11-24

    // Down 0.3 m -> 0.75 cycles -> wraps backwards to 0.6 s
    state.set_input(true, true, 0.45); // new: DECISIONS 2026-11-24
    let _ = state.tick(STEP_DT);
    assert!((state.climb_time(CLIMB_CLIP_LEN) - 0.6).abs() < EPS); // new: DECISIONS 2026-11-24
}

#[test]
fn teleport_ignored_and_dt_edge_cases_finite() {
    let mut state = LadderAnimState::with_config(LadderAnimConfig::default());
    state.set_input(true, true, 0.0);
    let _ = state.tick(0.0);
    state.set_input(true, true, 0.3); // new: DECISIONS 2026-11-24
    let _ = state.tick(BIG_DT);
    let before = state.climb_time(CLIMB_CLIP_LEN);

    // Jump of 5.0 m exceeds teleportDistance (1.5 m) and must be ignored.
    state.set_input(true, true, 5.3); // new: DECISIONS 2026-11-24
    let _ = state.tick(BIG_DT);
    assert!((state.climb_time(CLIMB_CLIP_LEN) - before).abs() < EPS);

    // Non-finite inputs and dt 0 / 0.1 stay finite.
    state.set_input(true, true, f32::NAN);
    let _ = state.tick(0.0);
    let _ = state.tick(BIG_DT);
    let t = state.climb_time(CLIMB_CLIP_LEN);
    assert!(t.is_finite() && (0.0..CLIMB_CLIP_LEN).contains(&t));
}

#[test]
fn layers_and_event_helpers_work() {
    let mut state = LadderAnimState::with_config(LadderAnimConfig::default());
    let mut layers = LayerStack::new(&AnimConfig::default());
    layers.get_mut(LayerId::Stance).target = 1.0; // new: DECISIONS 2026-11-24
    layers.get_mut(LayerId::Aim).target = 1.0; // new: DECISIONS 2026-11-24

    state.set_input(true, true, 0.0);
    let _ = state.tick(STEP_DT);
    state.set_input(true, true, 0.15); // new: DECISIONS 2026-11-24
    let _ = state.tick(STEP_DT);
    state.apply_ladder(&mut layers, CLIMB_CLIP_LEN);
    state.suppress_upper(&mut layers);

    let air = layers.get(LayerId::Air);
    assert_eq!(air.clip.as_deref(), Some(LADDER_CLIMB_CLIP));
    assert_eq!(air.weight, 1.0); // new: DECISIONS 2026-11-24
    assert_eq!(air.speed, 0.0);
    assert!((air.time - 0.2).abs() < EPS); // new: DECISIONS 2026-11-24
    assert_eq!(layers.get(LayerId::Stance).target, 0.0);
    assert_eq!(layers.get(LayerId::Aim).target, 0.0);
    assert_eq!(ladder_event_clip(LadderEvent::ExitTop), LADDER_EXIT_TOP_CLIP);
    assert!((state.event_speed(LadderEvent::ExitTop, 0.9) - 2.0).abs() < EPS); // new: DECISIONS 2026-11-24

    state.release_air(&mut layers);
    assert_eq!(layers.get(LayerId::Air).clip.as_deref(), Some("jump"));
    assert_eq!(layers.get(LayerId::Air).weight, 0.0);
}
