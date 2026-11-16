use engine::egui::vec2;
use glam::Vec2;

use super::state::MapUiState;

#[test]
fn map_ui_state_zoom_clamps() {
    let mut state = MapUiState::new();
    assert_eq!(state.zoom_index, game::DEFAULT_ZOOM);
    assert!((state.scale() - 0.5).abs() < 1e-6);

    // Zoom in (+1)
    state.zoom(1);
    assert_eq!(state.zoom_index, 2);
    assert!((state.scale() - 1.0).abs() < 1e-6);

    // Zoom in beyond max clamps to 2
    state.zoom(1);
    assert_eq!(state.zoom_index, 2);
    assert!((state.scale() - 1.0).abs() < 1e-6);

    state.zoom(10);
    assert_eq!(state.zoom_index, 2);
    assert!((state.scale() - 1.0).abs() < 1e-6);

    // Zoom out to default
    state.zoom(-1);
    assert_eq!(state.zoom_index, 1);
    assert!((state.scale() - 0.5).abs() < 1e-6);

    // Zoom out to min
    state.zoom(-1);
    assert_eq!(state.zoom_index, 0);
    assert!((state.scale() - 0.25).abs() < 1e-6);

    // Zoom out beyond min clamps to 0
    state.zoom(-1);
    assert_eq!(state.zoom_index, 0);
    assert!((state.scale() - 0.25).abs() < 1e-6);

    state.zoom(-10);
    assert_eq!(state.zoom_index, 0);
    assert!((state.scale() - 0.25).abs() < 1e-6);
}

#[test]
fn map_ui_state_pan_and_centre() {
    let mut state = MapUiState::new();
    assert_eq!(state.centre, Vec2::ZERO);

    // Pan at scale 0.5 (zoom index 1)
    state.pan(vec2(100.0, 50.0));
    // screen_delta (100, 50) / 0.5 = (200, 100), centre = 0 - (200, 100) = (-200, -100)
    assert!((state.centre.x - (-200.0)).abs() < 1e-5);
    assert!((state.centre.y - (-100.0)).abs() < 1e-5);

    // Zoom in to scale 1.0
    state.zoom(1);
    assert_eq!(state.zoom_index, 2);
    assert!((state.scale() - 1.0).abs() < 1e-6);

    // Pan at scale 1.0
    state.pan(vec2(50.0, -50.0));
    // screen_delta (50, -50) / 1.0 = (50, -50), centre = (-200, -100) - (50, -50) = (-250, -50)
    assert!((state.centre.x - (-250.0)).abs() < 1e-5);
    assert!((state.centre.y - (-50.0)).abs() < 1e-5);

    // Direct centre_on
    state.centre_on(Vec2::new(123.0, 456.0));
    assert_eq!(state.centre, Vec2::new(123.0, 456.0));

    // centre_on_player
    assert!(state.centre_on_player(Some(Vec2::new(10.0, 20.0))));
    assert_eq!(state.centre, Vec2::new(10.0, 20.0));

    // centre_on_player with None does not change centre
    assert!(!state.centre_on_player(None));
    assert_eq!(state.centre, Vec2::new(10.0, 20.0));
}
