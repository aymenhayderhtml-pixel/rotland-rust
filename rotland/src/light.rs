//! The light of the day, ported from Godot's DayNightPalette (sun path, sun colour, sky colours).

use engine::render::Lighting;
use glam::Vec3;
use std::f32::consts::PI;

fn hex(h: u32) -> Vec3 {
    Vec3::new(((h >> 16) & 255) as f32, ((h >> 8) & 255) as f32, (h & 255) as f32) / 255.0
}

fn sun_angle(hour: f32) -> f32 {
    let x = (hour - 6.0).rem_euclid(24.0);
    if x < 13.0 { PI * x / 13.0 } else { PI + PI * (x - 13.0) / 11.0 }
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Lighting at `hour` (0..24). Daytime only for now; night (moon, flashlight) comes with R1.
pub fn at_hour(hour: f32) -> Lighting {
    let angle = sun_angle(hour);
    let elevation = angle.sin();
    let energy = 0.9 * smooth(((elevation + 0.3) / 0.4).clamp(0.0, 1.0));
    let warm = 1.0 - (elevation / 0.6).clamp(0.0, 1.0);
    let sun_color = Vec3::new(1.0, 0.98, 0.93).lerp(Vec3::new(1.0, 0.6, 0.35), warm * 0.7);
    let day = (energy / 0.9).clamp(0.0, 1.0);
    let (top, horizon) = (hex(0x5b86c4), hex(0xc9d6e0));
    Lighting {
        sun_toward: Vec3::new(-angle.cos(), angle.sin(), 0.4).normalize(),
        sun_energy: energy,
        sun_color,
        sky_top: top,
        sky_horizon: horizon,
        ground_horizon: horizon,
        ground_bottom: horizon,
        ambient_energy: 0.18 + (0.6 - 0.18) * day,
        fog_begin: 120.0,
        fog_end: 190.0,
        fog_density: 1.0,
        fog_color: horizon,
    }
}
