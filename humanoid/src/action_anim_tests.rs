//! Unit tests for `action_anim` config, clip coverage, cancellation and timing.

use std::path::Path;

use crate::action_anim::{ActionAnimCommand, ActionAnimConfig, ActionAnimState, ActionCue};
use crate::anim_config::AnimConfig;
use crate::layers::{LayerId, LayerStack};

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("humanoid crate should live under workspace root")
}

#[test]
fn data_parses_and_has_all_eight_cues() {
    let text = std::fs::read_to_string(workspace_root().join("data/action_anim.json"))
        .expect("data/action_anim.json should exist");
    let cfg = ActionAnimConfig::from_json(&text).expect("action_anim.json should parse");
    assert_eq!(cfg.version, 1); // data: action_anim.json version
    assert!((cfg.cancel_speed - 1.2).abs() < 1e-5); // data: action_anim.json cancelSpeed
    let all = [
        ActionCue::PickUp,
        ActionCue::OpenCrate,
        ActionCue::Eat,
        ActionCue::Drink,
        ActionCue::Bandage,
        ActionCue::Build,
        ActionCue::FishCast,
        ActionCue::FishReel,
    ];
    for cue in all {
        let spec = cfg.spec(cue).expect("cue should be present");
        assert!(!spec.clip.is_empty());
    }
    assert!(cfg.spec(ActionCue::FishStop).is_none());
}

#[test]
fn every_clip_name_exists_in_blender_use_files() {
    let root = workspace_root();
    let cfg_text = std::fs::read_to_string(root.join("data/action_anim.json")).unwrap();
    let cfg = ActionAnimConfig::from_json(&cfg_text).unwrap();
    let use_v1 = std::fs::read_to_string(root.join("tools/blender/characters/poses_use.py"))
        .expect("poses_use.py should exist");
    let use_v2 = std::fs::read_to_string(root.join("tools/blender/characters/poses_use_v2.py"))
        .expect("poses_use_v2.py should exist");
    let combined = format!("{use_v1}\n{use_v2}");
    let all = [
        ActionCue::PickUp,
        ActionCue::OpenCrate,
        ActionCue::Eat,
        ActionCue::Drink,
        ActionCue::Bandage,
        ActionCue::Build,
        ActionCue::FishCast,
        ActionCue::FishReel,
    ];
    for cue in all {
        let clip = &cfg.spec(cue).unwrap().clip;
        let quoted = format!("\"{clip}\"");
        assert!(combined.contains(&quoted), "missing clip {clip} in poses_use*.py");
    }
}

#[test]
fn never_called_changes_nothing() {
    let mut state = ActionAnimState::new();
    let mut layers = LayerStack::new(AnimConfig::load_shared());
    assert!(!state.is_driven());
    assert_eq!(state.tick(0.05, false), ActionAnimCommand::None); // new: DECISIONS 2026-11-24
    state.apply_upper(&mut layers);
    assert!(layers.get(LayerId::Action).clip.is_none());
    assert_eq!(layers.get(LayerId::Action).weight, 0.0); // new: DECISIONS 2026-11-24
}

#[test]
fn full_body_cancels_above_cancel_speed_and_not_below() {
    let mut state = ActionAnimState::new();
    let cmd = state.play_cue(ActionCue::OpenCrate, false);
    assert!(matches!(cmd, ActionAnimCommand::StartFullBody { cue: ActionCue::OpenCrate, .. }));

    state.set_move_speed(1.1); // new: DECISIONS 2026-11-24
    assert_eq!(state.tick(0.1, false), ActionAnimCommand::None); // new: DECISIONS 2026-11-24
    assert!(state.is_full_body_active());

    state.set_move_speed(1.5); // new: DECISIONS 2026-11-24
    let stop = state.tick(0.1, false); // new: DECISIONS 2026-11-24
    assert!(matches!(stop, ActionAnimCommand::StopFullBody { fade_out } if fade_out > 0.0));
    assert!(!state.is_full_body_active());
}

#[test]
fn upper_body_not_cancelled_by_speed() {
    let mut state = ActionAnimState::new();
    let mut layers = LayerStack::new(AnimConfig::load_shared());
    state.play_cue(ActionCue::Drink, false);
    state.set_move_speed(6.0); // new: DECISIONS 2026-11-24
    state.tick(0.3, false); // new: DECISIONS 2026-11-24
    state.apply_upper(&mut layers);
    assert!(state.is_upper_active());
    assert_eq!(layers.get(LayerId::Action).clip.as_deref(), Some("drink"));
    assert!(layers.get(LayerId::Action).weight > 0.5); // new: DECISIONS 2026-11-24
}

#[test]
fn timed_cue_ends_after_its_seconds() {
    let mut state = ActionAnimState::new();
    let mut layers = LayerStack::new(AnimConfig::load_shared());
    state.play_cue(ActionCue::Build, false); // 0.7s
    for _ in 0..8 {
        state.tick(0.1, false); // new: DECISIONS 2026-11-24
        state.apply_upper(&mut layers);
    }
    assert!(!state.is_upper_active());
    assert_eq!(layers.get(LayerId::Action).weight, 0.0); // new: DECISIONS 2026-11-24
    assert!(layers.get(LayerId::Action).clip.is_none());
}

#[test]
fn fish_reel_loops_until_fish_stop() {
    let mut state = ActionAnimState::new();
    let mut layers = LayerStack::new(AnimConfig::load_shared());
    state.play_cue(ActionCue::FishReel, false);
    for _ in 0..50 {
        state.tick(0.1, false); // new: DECISIONS 2026-11-24
        state.apply_upper(&mut layers);
    }
    assert!(state.is_upper_active());
    assert!((state.weight() - 1.0).abs() < 1e-5); // new: DECISIONS 2026-11-24

    state.play_cue(ActionCue::FishStop, false);
    for _ in 0..4 {
        state.tick(0.1, false); // new: DECISIONS 2026-11-24
        state.apply_upper(&mut layers);
    }
    assert!(!state.is_upper_active());
    assert_eq!(layers.get(LayerId::Action).weight, 0.0); // new: DECISIONS 2026-11-24
}

#[test]
fn new_cue_replaces_old_cue() {
    let mut state = ActionAnimState::new();
    state.play_cue(ActionCue::PickUp, false);
    assert_eq!(state.active_cue(), Some(ActionCue::PickUp));

    let cmd = state.play_cue(ActionCue::Eat, false);
    assert!(matches!(cmd, ActionAnimCommand::StopFullBody { .. }));
    assert_eq!(state.active_cue(), Some(ActionCue::Eat));
}

#[test]
fn stable_and_finite_at_zero_and_large_dt() {
    let mut state = ActionAnimState::new();
    let mut layers = LayerStack::new(AnimConfig::load_shared());
    state.play_cue(ActionCue::Bandage, false);
    for dt in [0.0_f32, 0.1, 0.0, 0.1] {
        state.tick(dt, false);
        state.apply_upper(&mut layers);
        assert!(state.weight().is_finite());
        assert!(layers.get(LayerId::Action).weight.is_finite());
        assert!(layers.get(LayerId::Action).time.is_finite());
    }
}
