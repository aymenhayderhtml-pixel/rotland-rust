//! Pure crow flock simulation and flapper animation timing (R41-05).

pub mod flock;
mod circlers;
#[cfg(test)]
mod tests;

use std::path::Path;

use serde::Deserialize;

pub use flock::{Circlers, CrowFlock, CrowModel, CrowPose, FlockState};

/// Configuration loaded from `data/crows.json`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrowsConfig {
    pub startle_radius: f32,
    pub startle_radius_running: f32,
    pub gunshot_startle_radius: f32,
    pub takeoff_speed: f32,
    pub fly_speed: f32,
    pub climb_rate: f32,
    pub gone_height: f32,
    pub return_after: [f32; 2],
    pub flap_rate: f32,
    pub glide_share: f32,
    pub circlers: usize,
    pub circle_radius: [f32; 2],
    pub circle_height: [f32; 2],
    pub draw_radius: f32,
    pub flapper_rate: f32,
    pub flapper_draw_radius: f32,
}

impl Default for CrowsConfig {
    fn default() -> Self {
        Self {
            startle_radius: 12.0,          // data: crows.json startleRadius
            startle_radius_running: 20.0,  // data: crows.json startleRadiusRunning
            gunshot_startle_radius: 60.0,  // data: crows.json gunshotStartleRadius
            takeoff_speed: 4.0,            // data: crows.json takeoffSpeed
            fly_speed: 9.0,                // data: crows.json flySpeed
            climb_rate: 3.0,               // data: crows.json climbRate
            gone_height: 45.0,             // data: crows.json goneHeight
            return_after: [40.0, 90.0],    // data: crows.json returnAfter
            flap_rate: 9.0,                // data: crows.json flapRate
            glide_share: 0.35,             // data: crows.json glideShare
            circlers: 6,                   // data: crows.json circlers
            circle_radius: [40.0, 90.0],   // data: crows.json circleRadius
            circle_height: [60.0, 110.0],  // data: crows.json circleHeight
            draw_radius: 90.0,             // data: crows.json drawRadius
            flapper_rate: 2.2,             // data: crows.json flapperRate
            flapper_draw_radius: 70.0,     // data: crows.json flapperDrawRadius
        }
    }
}

impl CrowsConfig {
    /// Loads `data/crows.json` from the repository path.
    pub fn from_repo() -> Result<Self, String> {
        let path = paths::repo_path("data/crows.json");
        Self::from_path(&path)
    }

    /// Loads `CrowsConfig` from a JSON file at `path`.
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&text)
    }

    /// Parses `CrowsConfig` from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("invalid crows.json: {e}"))
    }
}

/// Returns the 0..4 animation frame index for flapping flags, plastic, or wings.
/// Cycles `((time * rate + phase) * 4) mod 4`.
pub fn flap_frame(phase: f32, time: f32, rate: f32) -> usize {
    let steps = ((time * rate + phase) * 4.0).floor() as i64; // new: DECISIONS 2026-11-27
    steps.rem_euclid(4) as usize // new: DECISIONS 2026-11-27
}
