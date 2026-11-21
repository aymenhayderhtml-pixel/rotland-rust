//! Unit and parity tests for HUD math, scaling, compass bearing, crosshair visibility,
//! and non-overlapping layout calculations.
//!
//! Ported with exact reference values from Godot C#:
//! - `scripts/UI/CompassMath.cs`
//! - `scripts/UI/Compass.cs`
//! - `scripts/UI/Crosshair.cs`
//! - `scripts/UI/ScopeOverlay.cs`
//! - GitHub issues #2, #4, #18 (UI-001, UI-002)

use std::f32::consts::PI;
use game::hud_math::*;

#[test]
fn test_hud_scaling() {
    // Baseline 1080p
    assert!((hud_scale(1080.0) - 1.0).abs() < 1e-4);
    // 720p
    assert!((hud_scale(720.0) - (720.0 / 1080.0)).abs() < 1e-4);
    // 600p clamped to MIN_HUD_SCALE (0.6)
    assert_eq!(hud_scale(600.0), MIN_HUD_SCALE);
    // Extreme low clamped to 0.6
    assert_eq!(hud_scale(480.0), MIN_HUD_SCALE);
    // 1440p
    assert!((hud_scale(1440.0) - (1440.0 / 1080.0)).abs() < 1e-4);
    // 4K clamped to MAX_HUD_SCALE (1.5)
    assert_eq!(hud_scale(2160.0), MAX_HUD_SCALE);
}

#[test]
fn test_compass_math_bearing_to_view() {
    // Yaw 0 looking North
    assert_eq!(bearing_to_view_deg(0.0, 0.0), 0.0);
    assert_eq!(bearing_to_view_deg(90.0, 0.0), 90.0);
    assert_eq!(bearing_to_view_deg(270.0, 0.0), -90.0);
    assert_eq!(bearing_to_view_deg(180.0, 0.0).abs(), 180.0);

    // Wrap around at 0/360 boundary
    assert!((bearing_to_view_deg(355.0, 0.0) - (-5.0)).abs() < 1e-4);
    assert!((bearing_to_view_deg(5.0, 0.0) - 5.0).abs() < 1e-4);

    // With yaw: looking East (yaw = -PI/2 in Godot conventions)
    let yaw_east = -PI / 2.0;
    // East (90 deg) viewed facing East should be at center 0 deg
    assert!((bearing_to_view_deg(90.0, yaw_east) - 0.0).abs() < 1e-4);
    // North (0 deg) viewed facing East should be to the left (-90 deg)
    assert!((bearing_to_view_deg(0.0, yaw_east) - (-90.0)).abs() < 1e-4);
}

#[test]
fn test_compass_marker_x_and_clamping() {
    let strip_w = 600.0;
    // Center
    assert_eq!(marker_x(0.0, strip_w), 300.0);
    // Edges
    assert_eq!(marker_x(-90.0, strip_w), 0.0);
    assert_eq!(marker_x(90.0, strip_w), 600.0);
    // Clamping beyond 90 degrees
    assert_eq!(marker_x(-120.0, strip_w), 0.0);
    assert_eq!(marker_x(150.0, strip_w), 600.0);
}

#[test]
fn test_compass_direction_names() {
    assert_eq!(direction_name(0), "N");
    assert_eq!(direction_name(45), "NE");
    assert_eq!(direction_name(90), "E");
    assert_eq!(direction_name(135), "SE");
    assert_eq!(direction_name(180), "S");
    assert_eq!(direction_name(225), "SW");
    assert_eq!(direction_name(270), "W");
    assert_eq!(direction_name(315), "NW");
    assert_eq!(direction_name(360), "N");
    assert_eq!(direction_name(-45), "NW");
}

#[test]
fn test_compass_relative_angle_and_behind() {
    // Player at origin, yaw 0 (facing -Z North)
    // Target straight ahead: (0, -10)
    let angle_front = relative_angle_deg(0.0, 0.0, 0.0, -10.0, 0.0);
    assert!(angle_front.abs() < 1e-4);

    // Target to the right (East): (10, 0)
    let angle_right = relative_angle_deg(0.0, 0.0, 10.0, 0.0, 0.0);
    assert!((angle_right - 90.0).abs() < 1e-4);

    // Target to the left (West): (-10, 0)
    let angle_left = relative_angle_deg(0.0, 0.0, -10.0, 0.0, 0.0);
    assert!((angle_left - (-90.0)).abs() < 1e-4);

    // Shows behind logic (Compass.cs BehindMinDistance = 25m)
    // Camp at 180 deg, distance 15m (< 25m) -> should NOT show "(behind)"
    assert!(!shows_behind(true, 180.0, 15.0));
    // Camp at 180 deg, distance 30m (>= 25m) -> SHOULD show "(behind)"
    assert!(shows_behind(true, 180.0, 30.0));
    // Camp in front at 30m -> should NOT show "(behind)"
    assert!(!shows_behind(true, 45.0, 30.0));
    // Marker hidden -> false
    assert!(!shows_behind(false, 180.0, 30.0));
}

#[test]
fn test_crosshair_visibility_rules() {
    // Issue #4: Melee in hand -> no crosshair; Gun -> crosshair; None -> crosshair
    assert!(!is_crosshair_visible(false, false, HeldKind::Melee));
    assert!(is_crosshair_visible(false, false, HeldKind::Gun));
    assert!(is_crosshair_visible(false, false, HeldKind::None));
    assert!(is_crosshair_visible(false, false, HeldKind::Other));

    // Dead player -> never show crosshair
    assert!(!is_crosshair_visible(true, false, HeldKind::Gun));
    assert!(!is_crosshair_visible(true, false, HeldKind::None));

    // Scoped rifle -> never show crosshair (covered by scope overlay)
    assert!(!is_crosshair_visible(false, true, HeldKind::Gun));
}

#[test]
fn test_scope_overlay_math() {
    // 1920x1080: min dim is 1080. ClearRadius = 0.36
    let r = scope_clear_radius(1920.0, 1080.0);
    assert!((r - (1080.0 * 0.36)).abs() < 1e-4);
    assert!((scope_reticle_gap(r) - (r * 0.06)).abs() < 1e-4);
    assert!((scope_reticle_ends(r) - (r * 0.96)).abs() < 1e-4);
}

#[test]
fn test_hud_layout_no_overlaps_all_resolutions() {
    let resolutions = [
        (800.0, 600.0),
        (1280.0, 720.0),
        (1920.0, 1080.0),
        (2560.0, 1440.0),
    ];

    for (w, h) in resolutions {
        assert!(
            !hud_elements_overlap(w, h),
            "HUD elements overlapped at resolution {w}x{h}"
        );

        let l = compute_hud_layout(w, h);
        // Specifically verify issue #18 UI-001 (hotbar vs vitals)
        assert!(
            !l.vitals.overlaps(&l.hotbar),
            "Vitals overlapped with hotbar at {w}x{h}"
        );
        // Verify minimap box and FPS do not collide
        assert!(
            !l.minimap_reserved.overlaps(&l.fps),
            "Minimap box overlapped with FPS at {w}x{h}"
        );
        // Verify compass and clock do not collide
        assert!(
            !l.compass.overlaps(&l.clock),
            "Compass overlapped with clock at {w}x{h}"
        );
    }
}

#[test]
fn test_damage_direction_angles() {
    // Player at (0, 0), looking North (yaw = 0.0)
    let p_x = 0.0;
    let p_z = 0.0;
    let yaw_north = 0.0;

    // Attacker straight ahead North (0, -10)
    let a_front = damage_direction_angle_rad(p_x, p_z, 0.0, -10.0, yaw_north);
    assert!(a_front.abs() < 1e-4, "Expected 0.0 rad, got {a_front}");

    // Attacker to the right East (10, 0) -> +PI/2
    let a_right = damage_direction_angle_rad(p_x, p_z, 10.0, 0.0, yaw_north);
    assert!((a_right - (PI / 2.0)).abs() < 1e-4, "Expected PI/2 rad, got {a_right}");

    // Attacker behind South (0, 10) -> PI
    let a_behind = damage_direction_angle_rad(p_x, p_z, 0.0, 10.0, yaw_north);
    assert!((a_behind.abs() - PI).abs() < 1e-4, "Expected PI rad, got {a_behind}");

    // Attacker to the left West (-10, 0) -> -PI/2
    let a_left = damage_direction_angle_rad(p_x, p_z, -10.0, 0.0, yaw_north);
    assert!((a_left - (-PI / 2.0)).abs() < 1e-4, "Expected -PI/2 rad, got {a_left}");

    // Camera facing East (yaw = -PI/2 in Godot convention)
    let yaw_east = -PI / 2.0;

    // Attacker at East (10, 0) is now straight ahead in view -> 0.0 rad
    let a_east_facing_east = damage_direction_angle_rad(p_x, p_z, 10.0, 0.0, yaw_east);
    assert!(a_east_facing_east.abs() < 1e-4, "Expected 0.0, got {a_east_facing_east}");

    // Attacker at North (0, -10) is to the left of East view -> -PI/2 rad
    let a_north_facing_east = damage_direction_angle_rad(p_x, p_z, 0.0, -10.0, yaw_east);
    assert!((a_north_facing_east - (-PI / 2.0)).abs() < 1e-4);

    // Attacker at South (0, 10) is to the right of East view -> +PI/2 rad
    let a_south_facing_east = damage_direction_angle_rad(p_x, p_z, 0.0, 10.0, yaw_east);
    assert!((a_south_facing_east - (PI / 2.0)).abs() < 1e-4);
}

#[test]
fn test_damage_screen_direction_and_point() {
    // 0 rad -> Up (0, -1)
    let (dx_up, dy_up) = damage_screen_direction(0.0);
    assert!(dx_up.abs() < 1e-4 && (dy_up - (-1.0)).abs() < 1e-4);

    // PI/2 -> Right (1, 0)
    let (dx_r, dy_r) = damage_screen_direction(PI / 2.0);
    assert!((dx_r - 1.0).abs() < 1e-4 && dy_r.abs() < 1e-4);

    // PI -> Down (0, 1)
    let (dx_d, dy_d) = damage_screen_direction(PI);
    assert!(dx_d.abs() < 1e-4 && (dy_d - 1.0).abs() < 1e-4);

    // -PI/2 -> Left (-1, 0)
    let (dx_l, dy_l) = damage_screen_direction(-PI / 2.0);
    assert!((dx_l - (-1.0)).abs() < 1e-4 && dy_l.abs() < 1e-4);

    // Screen point: 1920x1080 screen with 20px margin
    let (px, py) = damage_screen_point(1920.0, 1080.0, 0.0, 20.0);
    assert_eq!(px, 960.0);
    assert_eq!(py, 20.0);

    let (px_r, py_r) = damage_screen_point(1920.0, 1080.0, PI / 2.0, 20.0);
    assert_eq!(px_r, 1920.0 - 20.0);
    assert_eq!(py_r, 540.0);
}

#[test]
fn test_damage_arc_points() {
    let pts = damage_arc_points(100.0, 100.0, 50.0, 50.0, 0.0, PI / 4.0, 4);
    assert_eq!(pts.len(), 5);
    // Midpoint should be at top (100.0, 50.0)
    let mid = pts[2];
    assert!((mid.0 - 100.0).abs() < 1e-4);
    assert!((mid.1 - 50.0).abs() < 1e-4);
}

#[test]
fn test_damage_alpha_fade() {
    assert_eq!(damage_alpha(0.0, 1.2), 1.0);
    assert!((damage_alpha(0.6, 1.2) - 0.5).abs() < 1e-4);
    assert_eq!(damage_alpha(1.2, 1.2), 0.0);
    assert_eq!(damage_alpha(2.0, 1.2), 0.0);
}

#[test]
fn test_vitals_trail_update() {
    // Current trail at 100, health dropped to 70
    let mut trail = 100.0;
    let target = 70.0;
    let shrink_rate = 30.0; // 30 hp per second

    // After 0.5s, trail should be 85.0
    trail = vitals_trail_update(trail, target, 0.5, shrink_rate);
    assert!((trail - 85.0).abs() < 1e-4);

    // After another 1.0s, trail should reach target 70.0 (clamped)
    trail = vitals_trail_update(trail, target, 1.0, shrink_rate);
    assert_eq!(trail, 70.0);

    // If player heals to 90, trail snaps to 90
    trail = vitals_trail_update(trail, 90.0, 0.1, shrink_rate);
    assert_eq!(trail, 90.0);
}

#[test]
fn test_bar_pulse_factor() {
    let p0 = bar_pulse_factor(0.0, 6.0);
    assert!((p0 - 0.5).abs() < 1e-4);
    for t in [0.1, 0.5, 1.0, 2.5] {
        let p = bar_pulse_factor(t, 6.0);
        assert!((0.0..=1.0).contains(&p));
    }
}

#[test]
fn test_hud_math_defaults() {
    assert_eq!(DEFAULT_DAMAGE_FADE_S, 1.2);
    assert_eq!(DEFAULT_PICKUP_FADE_S, 2.5);
    assert_eq!(DEFAULT_NOTIFICATION_DURATION_S, 4.0);
    assert_eq!(DEFAULT_TRAIL_SHRINK_RATE, 0.35);
    assert_eq!(DEFAULT_TRAIL_DELAY_S, 0.4);
    assert_eq!(DEFAULT_PULSE_SPEED, 6.0);
}

