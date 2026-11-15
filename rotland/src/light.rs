//! The light of the day, ported from Godot's DayNightPalette (sun path, sun colour, sky colours).

use engine::render::Lighting;
use glam::Vec3;
use world::daynight::DayNightLook;

/// Lighting at `hour` (0..24). Builds the engine Lighting from DayNightLook.
/// Uses the sun only for now (moon light comes later).
pub fn at_hour(hour: f32) -> Lighting {
    let look = DayNightLook::at(hour);
    // Note: moon light comes later; for now we only set up the sun.
    Lighting {
        sun_toward: Vec3::from(look.sun_direction),
        sun_energy: look.sun_energy,
        sun_color: Vec3::from(look.sun_color),
        sky_top: Vec3::from(look.sky_top),
        sky_horizon: Vec3::from(look.sky_horizon),
        ground_horizon: Vec3::from(look.ground_horizon),
        ground_bottom: Vec3::from(look.ground_bottom),
        ambient_energy: look.ambient_energy,
        fog_begin: 120.0,
        fog_end: 190.0,
        fog_density: 1.0,
        fog_color: Vec3::from(look.fog_color),
    }
}
