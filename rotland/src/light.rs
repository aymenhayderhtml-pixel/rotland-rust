//! The light of the day, ported from Godot's DayNightPalette (sun path, sun colour, sky colours).

use engine::render::{Lighting, SpotLight};
use glam::Vec3;
use world::daynight::DayNightLook;

#[allow(dead_code)] // unused by the humanoid_view example, which includes this file
/// The flashlight (scripts/Items/Flashlight.cs): 25 m range, 30 degree cone, brightness 3, warm white.
pub fn flashlight(position: Vec3, direction: Vec3) -> SpotLight {
    SpotLight { position, direction, range: 25.0, angle_deg: 30.0, color: Vec3::new(1.0, 0.93, 0.78), energy: 3.0 }
}

/// Lighting at `hour` (0..24) from DayNightLook: sun, moon, sky, fog, and an optional flashlight.
pub fn at_hour(hour: f32, spot: Option<SpotLight>) -> Lighting {
    let look = DayNightLook::at(hour);
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
        moon_toward: Vec3::from(look.moon_direction),
        moon_energy: look.moon_energy,
        moon_color: Vec3::from(look.moon_color),
        spot,
    }
}
