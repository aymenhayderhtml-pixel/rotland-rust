use glam::{Mat4, Vec3};
use pack::Marker;

use game::ladders::{
    climb_tick, ladder_prompt, ladders_from_markers, nearest_ladder_end, ClimbInput, ClimbPhase,
    LadderClimb, LadderEnd, LadderRail, CLIMB_SPEED, GRAB_RANGE, SETTLE_LIFT, START_GRACE,
    TRANSITION_SECONDS,
};
use game::Player;

#[test]
fn test_marker_pairing_by_suffix_and_missing_top_skipped() {
    let markers = vec![
        Marker {
            name: "ladder_bottom".to_string(),
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        },
        Marker {
            name: "ladder_top".to_string(),
            translation: [0.0, 10.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        },
        Marker {
            name: "ladder_bottom_1".to_string(),
            translation: [5.0, 1.0, 5.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        },
        Marker {
            name: "ladder_top_1".to_string(),
            translation: [5.0, 7.0, 5.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        },
        // Missing matching top: must be skipped
        Marker {
            name: "ladder_bottom_2".to_string(),
            translation: [10.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        },
        // Missing matching bottom: must be skipped
        Marker {
            name: "ladder_top_3".to_string(),
            translation: [0.0, 8.0, 10.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        },
        // Unrelated marker
        Marker {
            name: "loot_crate_1".to_string(),
            translation: [1.0, 0.0, 1.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        },
    ];

    let rails = ladders_from_markers(&markers, Mat4::IDENTITY);
    assert_eq!(rails.len(), 2, "Must produce exactly 2 paired ladders");

    // Ladder 0: watchtower (no suffix)
    assert_eq!(rails[0].suffix, "");
    assert_eq!(rails[0].foot, Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(rails[0].deck, Vec3::new(0.0, 10.0, 0.0));
    assert_eq!(rails[0].facing(), Vec3::NEG_Z);

    // Ladder 1: settlement platform (_1 suffix)
    assert_eq!(rails[1].suffix, "_1");
    assert_eq!(rails[1].foot, Vec3::new(5.0, 1.0, 5.0));
    assert_eq!(rails[1].deck, Vec3::new(5.0, 7.0, 5.0));

    // Test with model_matrix transform
    let transform = Mat4::from_translation(Vec3::new(20.0, 3.0, -15.0));
    let transformed_rails = ladders_from_markers(&markers, transform);
    assert_eq!(transformed_rails.len(), 2);
    assert_eq!(transformed_rails[0].foot, Vec3::new(20.0, 3.0, -15.0));
    assert_eq!(transformed_rails[0].deck, Vec3::new(20.0, 13.0, -15.0));

    let compared = rails.len();
    assert!(compared > 0, "Comparison test must verify > 0 items");
}

#[test]
fn test_climbing_up_reaches_the_deck() {
    let foot = Vec3::new(0.0, 0.0, 0.0);
    let deck = Vec3::new(0.0, 5.0, 0.0);
    let rail = LadderRail::new(foot, deck, "");
    let rails = [rail];

    let mut player = Player::new(foot);
    let mut climb = LadderClimb::new();

    // Start climb at foot via E interact
    let e_input = ClimbInput::new(0.0, false, true);
    let grabbed = climb_tick(&mut climb, &mut player, &e_input, &rails, 0.016);
    assert!(grabbed, "E at foot within range must start climb");
    assert!(climb.is_active());
    assert_eq!(climb.phase(), ClimbPhase::Climbing);

    // Hold W to climb up at CLIMB_SPEED (2.5 m/s). Deck is at Y=5.0m.
    // 20 ticks of dt=0.1s = 2.0s: 2.0 * 2.5 = 5.0m.
    let climb_input = ClimbInput::new(1.0, false, false);
    let dt = 0.1;
    for tick in 1..=20 {
        let on_ladder = climb_tick(&mut climb, &mut player, &climb_input, &rails, dt);
        assert!(on_ladder, "Should remain on ladder during ascent tick {tick}");
    }

    // At Y=5.0m, transition onto deck begins (finishing = true)
    assert!(climb.is_active());
    assert_eq!(climb.phase(), ClimbPhase::Transition);

    // Transition path: 4 points, length = 0.4 + 0.0 + 0.4 = 0.8m.
    // TransitionSeconds = 0.3s. Over 3 ticks of dt=0.1s (0.3s), transition finishes.
    let on_1 = climb_tick(&mut climb, &mut player, &climb_input, &rails, 0.1);
    assert!(on_1, "Still in transition after 0.1s");
    let on_2 = climb_tick(&mut climb, &mut player, &climb_input, &rails, 0.1);
    assert!(on_2, "Still in transition after 0.2s");
    let on_3 = climb_tick(&mut climb, &mut player, &climb_input, &rails, 0.1);
    assert!(!on_3, "Climb must end when transition completes onto deck");

    assert!(!climb.is_active(), "Ladder climb must be inactive");
    // Position settled on deck with SETTLE_LIFT (0.05m)
    let dist_to_deck = player.position.distance(deck);
    assert!(
        dist_to_deck <= 0.1,
        "Player must land within 0.1m of deck, got {dist_to_deck:.3}m"
    );
    assert_eq!(player.position.y, deck.y + SETTLE_LIFT);
}

#[test]
fn test_letting_go_falls() {
    let foot = Vec3::new(0.0, 0.0, 0.0);
    let deck = Vec3::new(0.0, 10.0, 0.0);
    let rail = LadderRail::new(foot, deck, "");
    let rails = [rail];

    let mut player = Player::new(foot);
    let mut climb = LadderClimb::new();

    // Start climb at foot
    let e_input = ClimbInput::new(0.0, false, true);
    assert!(climb_tick(&mut climb, &mut player, &e_input, &rails, 0.016));

    // Climb for 1.0s (10 ticks of 0.1s) -> reaches 2.5m (MinClimbHeight 1.5m in Godot test)
    let up_input = ClimbInput::new(1.0, false, false);
    for _ in 0..10 {
        climb_tick(&mut climb, &mut player, &up_input, &rails, 0.1);
    }
    let height = player.position.y - foot.y;
    assert!(
        (height - 2.5).abs() < 1e-4,
        "Player should reach 2.5m height, got {height}"
    );
    assert!(climb.is_active());

    // Space to let go: releasePressed terminates climb
    let space_input = ClimbInput::new(0.0, true, false);
    let on_ladder = climb_tick(&mut climb, &mut player, &space_input, &rails, 0.016);
    assert!(!on_ladder, "Pressing space must let go of the ladder");
    assert!(!climb.is_active(), "Climb must no longer be active");

    // Gravity falls to ground over 1.5s
    for _ in 0..15 {
        player.velocity.y -= 9.8 * 0.1;
        player.position.y += player.velocity.y * 0.1;
        if player.position.y <= foot.y {
            player.position.y = foot.y;
            player.velocity.y = 0.0;
            player.on_floor = true;
            break;
        }
    }
    assert!(player.on_floor, "Player must fall to ground");
    assert_eq!(player.position.y, foot.y);
}

#[test]
fn test_climb_down_from_deck() {
    let foot = Vec3::new(0.0, 0.0, 0.0);
    let deck = Vec3::new(0.0, 5.0, 0.0);
    let rail = LadderRail::new(foot, deck, "");
    let rails = [rail];

    let mut player = Player::new(deck);
    let mut climb = LadderClimb::new();

    // Start climb down from deck
    let e_input = ClimbInput::new(0.0, false, true);
    let grabbed = climb_tick(&mut climb, &mut player, &e_input, &rails, 0.016);
    assert!(grabbed, "E at deck within range must start climb down");
    assert!(climb.is_active());
    assert_eq!(climb.phase(), ClimbPhase::Transition);

    // Transition off deck onto rungs: 0.3s (3 ticks of 0.1s)
    let idle_input = ClimbInput::default();
    climb_tick(&mut climb, &mut player, &idle_input, &rails, 0.1);
    climb_tick(&mut climb, &mut player, &idle_input, &rails, 0.1);
    climb_tick(&mut climb, &mut player, &idle_input, &rails, 0.1);
    assert_eq!(
        climb.phase(),
        ClimbPhase::Climbing,
        "Transition off deck must finish and switch to Climbing"
    );

    // Climb down with S (up = -1.0) at 2.5 m/s. 20 ticks of dt=0.1s = 2.0s: 5.0m to 0.0m.
    let down_input = ClimbInput::new(-1.0, false, false);
    for tick in 1..=19 {
        let on = climb_tick(&mut climb, &mut player, &down_input, &rails, 0.1);
        assert!(on, "Still climbing down at tick {tick}");
    }

    // Tick 20 reaches foot.y (0.0): stops climb and places player on ground at foot
    let on_20 = climb_tick(&mut climb, &mut player, &down_input, &rails, 0.1);
    assert!(!on_20, "Reaching bottom with S held must end the climb");
    assert!(!climb.is_active());
    assert_eq!(player.position, foot);
}

#[test]
fn test_constants_and_interaction_geometry() {
    assert_eq!(CLIMB_SPEED, 2.5);
    assert_eq!(TRANSITION_SECONDS, 0.3);
    assert_eq!(START_GRACE, 0.3);
    assert_eq!(SETTLE_LIFT, 0.05);
    assert_eq!(GRAB_RANGE, 2.5);

    let rail = LadderRail::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 4.0, 2.0), "");
    assert_eq!(rail.foot_interaction_point(), Vec3::new(0.0, 0.9, 0.0));
    assert_eq!(rail.deck_interaction_point(), Vec3::new(0.0, 4.9, 2.0));
    assert_eq!(rail.facing(), Vec3::new(0.0, 0.0, 1.0));
}

#[test]
fn test_ladder_prompt_at_ends_far_away_and_while_climbing() {
    let foot = Vec3::new(0.0, 0.0, 0.0);
    let deck = Vec3::new(0.0, 5.0, 0.0);
    let rail1 = LadderRail::new(foot, deck, "");
    let rail2 = LadderRail::new(Vec3::new(10.0, 0.0, 0.0), Vec3::new(10.0, 5.0, 0.0), "_2");
    let rails = [rail1.clone(), rail2];

    // 1. Standing near ladder foot (not climbing): "[E] Climb ladder"
    let prompt_foot = ladder_prompt(foot, &rails, false);
    assert_eq!(prompt_foot, Some("[E] Climb ladder"));

    let near_foot = foot + Vec3::new(0.8, 0.0, 0.5);
    assert_eq!(
        ladder_prompt(near_foot, &rails, false),
        Some("[E] Climb ladder")
    );

    let foot_interact = rail1.foot_interaction_point();
    assert_eq!(
        ladder_prompt(foot_interact, &rails, false),
        Some("[E] Climb ladder")
    );

    // 2. Standing near ladder deck (not climbing): "[E] Climb down"
    let prompt_deck = ladder_prompt(deck, &rails, false);
    assert_eq!(prompt_deck, Some("[E] Climb down"));

    let near_deck = deck + Vec3::new(0.5, 0.0, 0.5);
    assert_eq!(
        ladder_prompt(near_deck, &rails, false),
        Some("[E] Climb down")
    );

    let deck_interact = rail1.deck_interaction_point();
    assert_eq!(
        ladder_prompt(deck_interact, &rails, false),
        Some("[E] Climb down")
    );

    // 3. Far away from all ladders: None
    let far_away = Vec3::new(50.0, 0.0, 50.0);
    assert_eq!(ladder_prompt(far_away, &rails, false), None);

    // Just outside GRAB_RANGE (2.5m)
    let beyond_range = foot + Vec3::new(3.0, 0.0, 0.0);
    assert_eq!(ladder_prompt(beyond_range, &rails, false), None);

    // 4. While climbing: None
    assert_eq!(ladder_prompt(foot, &rails, true), None);
    assert_eq!(ladder_prompt(deck, &rails, true), None);
    let mid_rung = foot + Vec3::new(0.0, 2.5, 0.0);
    assert_eq!(ladder_prompt(mid_rung, &rails, true), None);

    // 5. Multiple rails: picks the nearest ladder end
    let near_rail2_foot = Vec3::new(9.8, 0.0, 0.1);
    assert_eq!(
        ladder_prompt(near_rail2_foot, &rails, false),
        Some("[E] Climb ladder")
    );
    let (nearest_end, dist) = nearest_ladder_end(near_rail2_foot, &rails, false).unwrap();
    assert_eq!(nearest_end.rail.suffix, "_2");
    assert!(!nearest_end.is_top);
    assert!(dist < 0.5);

    let near_rail2_deck = Vec3::new(10.1, 5.0, 0.0);
    assert_eq!(
        ladder_prompt(near_rail2_deck, &rails, false),
        Some("[E] Climb down")
    );
    let (nearest_end2, dist2) = nearest_ladder_end(near_rail2_deck, &rails, false).unwrap();
    assert_eq!(nearest_end2.rail.suffix, "_2");
    assert!(nearest_end2.is_top);
    assert!(dist2 < 0.5);

    // Ensure LadderEnd properties and methods match
    assert_eq!(LadderEnd::new(rail1.clone(), false).prompt(), "[E] Climb ladder");
    assert_eq!(LadderEnd::new(rail1.clone(), true).prompt(), "[E] Climb down");
    assert!(LadderEnd::new(rail1.clone(), false).can_interact(false));
    assert!(!LadderEnd::new(rail1, false).can_interact(true));
}
