use game::camera_rig::MOUSE_SENSITIVITY;
use game::input::{keyboard_look_delta, KEYBOARD_LOOK_SPEED_DEG};

#[test]
fn one_second_of_left_equals_120_degrees_worth_of_pixels() {
    let dt = 1.0;
    let delta = keyboard_look_delta(true, false, false, false, dt);
    let expected_pixels = KEYBOARD_LOOK_SPEED_DEG.to_radians() / MOUSE_SENSITIVITY;

    assert_eq!(delta.1, 0.0);
    // Left arrow turns view left (negative x delta, same as moving mouse left)
    assert!(
        (delta.0 - (-expected_pixels)).abs() < 1e-4,
        "delta.0 was {}, expected -{}",
        delta.0,
        expected_pixels
    );
}

#[test]
fn left_and_right_cancel() {
    let delta = keyboard_look_delta(true, true, false, false, 1.0);
    assert_eq!(delta, (0.0, 0.0));
}

#[test]
fn up_gives_negative_y_delta_like_moving_mouse_up() {
    let dt = 1.0;
    let delta = keyboard_look_delta(false, false, true, false, dt);
    let expected_pixels = KEYBOARD_LOOK_SPEED_DEG.to_radians() / MOUSE_SENSITIVITY;

    assert_eq!(delta.0, 0.0);
    // Up arrow looks up (negative y delta, same as moving mouse up)
    assert!(
        (delta.1 - (-expected_pixels)).abs() < 1e-4,
        "delta.1 was {}, expected -{}",
        delta.1,
        expected_pixels
    );
}

#[test]
fn down_gives_positive_y_delta_and_cancels_with_up() {
    let dt = 1.0;
    let delta_down = keyboard_look_delta(false, false, false, true, dt);
    let expected_pixels = KEYBOARD_LOOK_SPEED_DEG.to_radians() / MOUSE_SENSITIVITY;

    assert_eq!(delta_down.0, 0.0);
    assert!(
        (delta_down.1 - expected_pixels).abs() < 1e-4,
        "delta.1 was {}, expected {}",
        delta_down.1,
        expected_pixels
    );

    let canceled = keyboard_look_delta(false, false, true, true, dt);
    assert_eq!(canceled, (0.0, 0.0));
}

#[test]
fn nothing_held_gives_zero() {
    let delta = keyboard_look_delta(false, false, false, false, 1.0);
    assert_eq!(delta, (0.0, 0.0));
}

#[test]
fn non_positive_dt_gives_zero() {
    let zero_dt = keyboard_look_delta(true, false, true, false, 0.0);
    assert_eq!(zero_dt, (0.0, 0.0));

    let neg_dt = keyboard_look_delta(true, false, true, false, -1.0);
    assert_eq!(neg_dt, (0.0, 0.0));
}

#[test]
fn frame_delta_scales_linearly() {
    let dt = 0.016; // ~60 fps
    let delta = keyboard_look_delta(true, false, false, false, dt);
    let expected_pixels = (KEYBOARD_LOOK_SPEED_DEG.to_radians() / MOUSE_SENSITIVITY) * dt;

    assert_eq!(delta.1, 0.0);
    assert!((delta.0 - (-expected_pixels)).abs() < 1e-4);
}
