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
    assert_eq!(bearing_to_view_deg(180.0, 0.0), 180.0);

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
    assert_eq!(is_crosshair_visible(false, false, HeldKind::Melee), false);
    assert_eq!(is_crosshair_visible(false, false, HeldKind::Gun), true);
    assert_eq!(is_crosshair_visible(false, false, HeldKind::None), true);
    assert_eq!(is_crosshair_visible(false, false, HeldKind::Other), true);

    // Dead player -> never show crosshair
    assert_eq!(is_crosshair_visible(true, false, HeldKind::Gun), false);
    assert_eq!(is_crosshair_visible(true, false, HeldKind::None), false);

    // Scoped rifle -> never show crosshair (covered by scope overlay)
    assert_eq!(is_crosshair_visible(false, true, HeldKind::Gun), false);
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
