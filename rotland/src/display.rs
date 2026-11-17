//! Display settings logic: FOV scaling/application and building engine DisplayRequests.
//! Ported from Godot's CameraRig.cs, GraphicsApplier.cs, and GameSettings.cs.

pub use engine::DisplayRequest;

use crate::settings::{self, Settings};

/// Godot's base camera field of view in degrees (CameraRig.cs DefaultZoom / fov default).
pub const DEFAULT_CAMERA_FOV: f32 = 75.0;

/// Returns the player's configured base FOV in degrees, clamped to 60.0..=100.0 (Godot GameSettings).
pub fn base_fov(settings: &Settings) -> f32 {
    settings
        .field_of_view
        .clamp(settings::MIN_FOV, settings::MAX_FOV)
}

/// Applies the player's FOV setting to the FOV computed by CameraRig.
/// When the rig is at default FOV (75.0 deg), this replaces it with `base_fov`.
/// When aiming, it scales proportionally so aim zoom still narrows the field of view.
pub fn apply_fov(camera_fov_from_rig: f32, settings: &Settings) -> f32 {
    let base = base_fov(settings);
    if (camera_fov_from_rig - DEFAULT_CAMERA_FOV).abs() < 1e-4 {
        base
    } else {
        camera_fov_from_rig * (base / DEFAULT_CAMERA_FOV)
    }
}

/// Builds an engine `DisplayRequest` from game `Settings`.
/// Render scale is clamped to 0.5..=1.0 (Godot GraphicsApplier / settings).
pub fn display_request(settings: &Settings) -> DisplayRequest {
    DisplayRequest {
        render_scale: settings
            .render_scale
            .clamp(settings::MIN_RENDER_SCALE, settings::MAX_RENDER_SCALE),
        vsync: settings.vsync,
    }
}

/// Convenience function to build a `DisplayRequest` from `Settings`.
pub fn from_settings(settings: &Settings) -> DisplayRequest {
    display_request(settings)
}

impl From<&Settings> for DisplayRequest {
    fn from(s: &Settings) -> Self {
        display_request(s)
    }
}

impl From<Settings> for DisplayRequest {
    fn from(s: Settings) -> Self {
        display_request(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fov_90_setting_gives_godot_camera_fov() {
        let mut settings = Settings::default();
        settings.field_of_view = 90.0;
        assert_eq!(base_fov(&settings), 90.0);
        // Default Godot un-aimed camera FOV is 75.0 deg; with FOV 90 it must produce 90.0 deg
        let fov = apply_fov(DEFAULT_CAMERA_FOV, &settings);
        assert!((fov - 90.0).abs() < 1e-4);
    }

    #[test]
    fn base_fov_clamps_to_valid_range() {
        let mut settings = Settings::default();
        settings.field_of_view = 30.0;
        assert_eq!(base_fov(&settings), settings::MIN_FOV); // 60.0

        settings.field_of_view = 120.0;
        assert_eq!(base_fov(&settings), settings::MAX_FOV); // 100.0

        settings.field_of_view = 75.0;
        assert_eq!(base_fov(&settings), 75.0);
    }

    #[test]
    fn apply_fov_preserves_default_when_setting_is_default() {
        let settings = Settings::default(); // field_of_view is 75.0
        let fov = apply_fov(DEFAULT_CAMERA_FOV, &settings);
        assert!((fov - DEFAULT_CAMERA_FOV).abs() < 1e-4);
    }

    #[test]
    fn apply_fov_scales_aim_fov() {
        let mut settings = Settings::default();
        settings.field_of_view = 90.0; // 1.2x scale
        // Third-person aim FOV is 55.0 deg
        let aim_fov = 55.0;
        let scaled = apply_fov(aim_fov, &settings);
        let expected = 55.0 * (90.0 / 75.0); // 66.0
        assert!((scaled - expected).abs() < 1e-4);
    }

    #[test]
    fn render_scale_clamps_to_valid_range() {
        let mut settings = Settings::default();
        settings.render_scale = 0.2;
        let req = display_request(&settings);
        assert_eq!(req.render_scale, 0.5);

        settings.render_scale = 1.5;
        let req = display_request(&settings);
        assert_eq!(req.render_scale, 1.0);

        settings.render_scale = 0.75;
        let req = display_request(&settings);
        assert_eq!(req.render_scale, 0.75);
    }

    #[test]
    fn display_request_reflects_vsync() {
        let mut settings = Settings::default();
        settings.vsync = false;
        assert!(!display_request(&settings).vsync);

        settings.vsync = true;
        assert!(display_request(&settings).vsync);
    }

    #[test]
    fn from_settings_and_trait_conversion() {
        let mut settings = Settings::default();
        settings.render_scale = 0.8;
        settings.vsync = true;

        let req1 = from_settings(&settings);
        let req2: DisplayRequest = (&settings).into();
        let req3: DisplayRequest = settings.clone().into();
        assert_eq!(req1, req2);
        assert_eq!(req1, req3);
        assert_eq!(req1.render_scale, 0.8);
        assert!(req1.vsync);
    }
}
