use std::f32::consts::FRAC_PI_2;

use game::{HitInput, HitReactConfig, HitReactState, HitSide, hit_side};
use glam::Vec3;

#[test]
fn config_from_repo_matches_default() {
    let cfg = HitReactConfig::from_repo().expect("load data/hit_react.json");
    assert_eq!(cfg, HitReactConfig::default());
}

#[test]
fn hit_side_yaw_zero_cardinal_directions() {
    let player = Vec3::ZERO;
    let yaw = 0.0; // new: DECISIONS 2026-11-24
    assert_eq!(hit_side(player, yaw, Vec3::new(0.0, 0.0, -2.0)), HitSide::Front);
    assert_eq!(hit_side(player, yaw, Vec3::new(0.0, 0.0, 2.0)), HitSide::Back);
    assert_eq!(hit_side(player, yaw, Vec3::new(2.0, 0.0, 0.0)), HitSide::Right);
    assert_eq!(hit_side(player, yaw, Vec3::new(-2.0, 0.0, 0.0)), HitSide::Left);
}

#[test]
fn hit_side_yaw_facing_east_and_zero_distance() {
    let player = Vec3::ZERO;
    let yaw = -FRAC_PI_2;
    assert_eq!(hit_side(player, yaw, Vec3::new(2.0, 0.0, 0.0)), HitSide::Front);
    assert_eq!(hit_side(player, yaw, Vec3::new(0.0, 0.0, 2.0)), HitSide::Right);
    assert_eq!(hit_side(player, yaw, Vec3::new(-2.0, 0.0, 0.0)), HitSide::Back);
    assert_eq!(hit_side(player, yaw, Vec3::new(0.0, 0.0, -2.0)), HitSide::Left);
    assert_eq!(hit_side(player, yaw, player), HitSide::Front);
}

#[test]
fn on_hit_unknown_attacker_defaults_to_hit_front() {
    let cfg = HitReactConfig::default();
    let mut state = HitReactState::default();
    let input = HitInput {
        time: 1.0, // new: DECISIONS 2026-11-24
        player_pos: Vec3::ZERO,
        player_yaw: 0.0,
        attacker: None,
        knockback: Vec3::ZERO,
    };

    let reaction = state.on_hit(&input, &cfg).expect("first hit plays");
    assert_eq!(reaction.clip, "hit_front");
    assert!(!reaction.full_body);
}

#[test]
fn on_hit_respects_min_gap_seconds() {
    let cfg = HitReactConfig::default();
    let mut state = HitReactState::default();
    let mut input = HitInput {
        time: 2.0, // new: DECISIONS 2026-11-24
        player_pos: Vec3::ZERO,
        player_yaw: 0.0,
        attacker: Some(Vec3::new(2.0, 0.0, 0.0)),
        knockback: Vec3::ZERO,
    };

    let first = state.on_hit(&input, &cfg).expect("first hit plays");
    assert_eq!(first.clip, "hit_right");

    input.time = 2.1; // new: DECISIONS 2026-11-24
    assert!(state.on_hit(&input, &cfg).is_none());

    input.time = 2.35; // new: DECISIONS 2026-11-24
    let after = state.on_hit(&input, &cfg).expect("hit after gap plays");
    assert_eq!(after.clip, "hit_right");
}

#[test]
fn big_push_backward_plays_full_body_stagger_even_inside_gap() {
    let cfg = HitReactConfig::default();
    let mut state = HitReactState::default();
    let normal = HitInput {
        time: 5.0, // new: DECISIONS 2026-11-24
        player_pos: Vec3::ZERO,
        player_yaw: 0.0,
        attacker: Some(Vec3::new(0.0, 0.0, -1.0)),
        knockback: Vec3::ZERO,
    };
    assert!(state.on_hit(&normal, &cfg).is_some());

    // Facing -Z, so +Z knockback pushes the player backward (hit from front).
    let big = HitInput {
        time: 5.05, // new: DECISIONS 2026-11-24
        player_pos: Vec3::ZERO,
        player_yaw: 0.0,
        attacker: None,
        knockback: Vec3::new(0.0, 0.0, cfg.stagger_knockback),
    };
    let reaction = state.on_hit(&big, &cfg).expect("big hit ignores gap");
    assert_eq!(reaction.clip, "stagger_back");
    assert!(reaction.full_body);
    assert_eq!(reaction.fade_in, cfg.stagger_fade_in);
    assert_eq!(reaction.fade_out, cfg.stagger_fade_out);
}

#[test]
fn big_push_forward_plays_upper_body_hit_back() {
    let cfg = HitReactConfig::default();
    let mut state = HitReactState::default();
    // Facing -Z, so -Z knockback pushes the player forward (hit from behind).
    let big = HitInput {
        time: 1.0, // new: DECISIONS 2026-11-24
        player_pos: Vec3::ZERO,
        player_yaw: 0.0,
        attacker: None,
        knockback: Vec3::new(0.0, 0.0, -cfg.stagger_knockback - 1.0),
    };
    let reaction = state.on_hit(&big, &cfg).expect("big hit from behind plays");
    assert_eq!(reaction.clip, "hit_back");
    assert!(!reaction.full_body);
}

#[test]
fn small_push_below_threshold_plays_normal_directional_flinch() {
    let cfg = HitReactConfig::default();
    let mut state = HitReactState::default();
    let small = HitInput {
        time: 1.0, // new: DECISIONS 2026-11-24
        player_pos: Vec3::ZERO,
        player_yaw: 0.0,
        attacker: Some(Vec3::new(-2.0, 0.0, 0.0)),
        knockback: Vec3::new(0.0, 0.0, cfg.stagger_knockback - 0.5),
    };
    let reaction = state.on_hit(&small, &cfg).expect("small push plays flinch");
    assert_eq!(reaction.clip, "hit_left");
    assert!(!reaction.full_body);
}
