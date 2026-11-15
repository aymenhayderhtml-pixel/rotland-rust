//! Pure functions of the hour ported from Godot's DayNightPalette.cs and DayNightVisuals.cs.
//! Where the sun and moon are, how bright they are, and what colors the sky, horizon and fog are.

use std::f32::consts::PI;

pub const MAX_SUN_ENERGY: f32 = 0.9;
pub const MOON_ENERGY: f32 = 0.12;
pub const MOON_COLOR: [f32; 3] = [0.6, 0.7, 1.0];
pub const SHADOW_MIN_ENERGY: f32 = 0.05;
pub const NIGHT_AMBIENT: f32 = 0.18;
pub const DAY_AMBIENT: f32 = 0.6;

pub const NIGHT_TOP: [f32; 3] = [11.0 / 255.0, 18.0 / 255.0, 48.0 / 255.0];
pub const NIGHT_HORIZON: [f32; 3] = [28.0 / 255.0, 37.0 / 255.0, 64.0 / 255.0];
pub const DAY_TOP: [f32; 3] = [91.0 / 255.0, 134.0 / 255.0, 196.0 / 255.0];
pub const DAY_HORIZON: [f32; 3] = [201.0 / 255.0, 214.0 / 255.0, 224.0 / 255.0];
pub const DUSK_DAWN_TOP: [f32; 3] = [62.0 / 255.0, 74.0 / 255.0, 122.0 / 255.0];
pub const DUSK_DAWN_HORIZON: [f32; 3] = [240.0 / 255.0, 165.0 / 255.0, 126.0 / 255.0];
pub const DUSK_SUNSET_TOP: [f32; 3] = [61.0 / 255.0, 63.0 / 255.0, 110.0 / 255.0];
pub const DUSK_SUNSET_HORIZON: [f32; 3] = [232.0 / 255.0, 135.0 / 255.0, 90.0 / 255.0];

/// Sky colours for one moment of the day.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyColors {
    pub top: [f32; 3],
    pub horizon: [f32; 3],
}

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_color(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [lerp_f32(a[0], b[0], t), lerp_f32(a[1], b[1], t), lerp_f32(a[2], b[2], t)]
}

/// Angle around the day: 0 at sunrise (06:00), PI/2 at midday, PI at sunset (19:00).
pub fn sun_angle(hour: f32) -> f32 {
    let x = (hour - 6.0).rem_euclid(24.0);
    if x < 13.0 {
        PI * x / 13.0
    } else {
        PI + PI * (x - 13.0) / 11.0
    }
}

/// Height of the sun above the horizon, from -1 (midnight) to 1 (top of the day).
pub fn sun_elevation(hour: f32) -> f32 {
    sun_angle(hour).sin()
}

/// Unit direction from the ground to the sun.
pub fn sun_direction(hour: f32) -> [f32; 3] {
    let angle = sun_angle(hour);
    let x = -angle.cos();
    let y = angle.sin();
    let z = 0.4;
    let len = (x * x + y * y + z * z).sqrt();
    [x / len, y / len, z / len]
}

/// Sun light energy. Fades out as the sun drops below about -0.3 of its height.
pub fn sun_energy(hour: f32) -> f32 {
    let t = ((sun_elevation(hour) + 0.3) / 0.4).clamp(0.0, 1.0);
    MAX_SUN_ENERGY * t * t * (3.0 - 2.0 * t)
}

/// 0 at night, 1 in full daylight.
pub fn day_factor(hour: f32) -> f32 {
    (sun_energy(hour) / MAX_SUN_ENERGY).clamp(0.0, 1.0)
}

/// Sky top and horizon colours, blended between key hours.
pub fn sky_at(hour: f32) -> SkyColors {
    let h = hour.rem_euclid(24.0);
    let keys: [(f32, [f32; 3], [f32; 3]); 8] = [
        (0.0, NIGHT_TOP, NIGHT_HORIZON),
        (5.0, NIGHT_TOP, NIGHT_HORIZON),
        (6.5, DUSK_DAWN_TOP, DUSK_DAWN_HORIZON),
        (8.5, DAY_TOP, DAY_HORIZON),
        (17.0, DAY_TOP, DAY_HORIZON),
        (19.0, DUSK_SUNSET_TOP, DUSK_SUNSET_HORIZON),
        (20.5, NIGHT_TOP, NIGHT_HORIZON),
        (24.0, NIGHT_TOP, NIGHT_HORIZON),
    ];
    for i in 1..keys.len() {
        if h <= keys[i].0 {
            let from = keys[i - 1];
            let to = keys[i];
            let t = (h - from.0) / (to.0 - from.0);
            return SkyColors {
                top: lerp_color(from.1, to.1, t),
                horizon: lerp_color(from.2, to.2, t),
            };
        }
    }
    SkyColors {
        top: NIGHT_TOP,
        horizon: NIGHT_HORIZON,
    }
}

/// Warm sun colour near the horizon, white-ish at the top of the day.
pub fn sun_color(hour: f32) -> [f32; 3] {
    let warm = 1.0 - (sun_elevation(hour) / 0.6).clamp(0.0, 1.0);
    let t = warm * 0.7;
    lerp_color([1.0, 0.98, 0.93], [1.0, 0.6, 0.35], t)
}

/// All lighting and environment numbers for a single hour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DayNightLook {
    pub sun_angle: f32,
    pub sun_elevation: f32,
    pub sun_direction: [f32; 3],
    pub sun_energy: f32,
    pub sun_color: [f32; 3],
    pub day_factor: f32,
    pub moon_direction: [f32; 3],
    pub moon_energy: f32,
    pub moon_color: [f32; 3],
    pub ambient_energy: f32,
    pub sky_top: [f32; 3],
    pub sky_horizon: [f32; 3],
    pub ground_horizon: [f32; 3],
    pub ground_bottom: [f32; 3],
    pub fog_color: [f32; 3],
    pub shadow_min_energy: f32,
    pub shadow_enabled: bool,
}

impl DayNightLook {
    pub fn at(hour: f32) -> Self {
        let angle = sun_angle(hour);
        let elevation = sun_elevation(hour);
        let s_dir = sun_direction(hour);
        let s_energy = sun_energy(hour);
        let s_color = sun_color(hour);
        let day = day_factor(hour);

        let m_dir = [-s_dir[0], -s_dir[1], -s_dir[2]];
        let m_energy = MOON_ENERGY * (1.0 - day);
        let m_color = MOON_COLOR;

        let sky = sky_at(hour);
        let ambient = lerp_f32(NIGHT_AMBIENT, DAY_AMBIENT, day);

        Self {
            sun_angle: angle,
            sun_elevation: elevation,
            sun_direction: s_dir,
            sun_energy: s_energy,
            sun_color: s_color,
            day_factor: day,
            moon_direction: m_dir,
            moon_energy: m_energy,
            moon_color: m_color,
            ambient_energy: ambient,
            sky_top: sky.top,
            sky_horizon: sky.horizon,
            ground_horizon: sky.horizon,
            ground_bottom: sky.horizon,
            fog_color: sky.horizon,
            shadow_min_energy: SHADOW_MIN_ENERGY,
            shadow_enabled: s_energy >= SHADOW_MIN_ENERGY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() <= eps * (1.0 + b.abs())
    }

    fn approx3(a: [f32; 3], b: [f32; 3], eps: f32) -> bool {
        approx(a[0], b[0], eps) && approx(a[1], b[1], eps) && approx(a[2], b[2], eps)
    }

    #[test]
    fn compare_against_csharp_formulas() {
        let test_hours = [0.0f32, 5.5, 6.5, 8.0, 12.0, 18.0, 19.5, 23.0];
        for &h in &test_hours {
            let look = DayNightLook::at(h);

            // C# math computation
            let x = (h - 6.0).rem_euclid(24.0);
            let expected_angle = if x < 13.0 { PI * x / 13.0 } else { PI + PI * (x - 13.0) / 11.0 };
            assert!(approx(look.sun_angle, expected_angle, 1e-5), "hour {h} angle");

            let expected_elevation = expected_angle.sin();
            assert!(approx(look.sun_elevation, expected_elevation, 1e-5), "hour {h} elevation");

            let s_len = (expected_angle.cos().powi(2) + expected_angle.sin().powi(2) + 0.16f32).sqrt();
            let expected_s_dir = [-expected_angle.cos() / s_len, expected_angle.sin() / s_len, 0.4 / s_len];
            assert!(approx3(look.sun_direction, expected_s_dir, 1e-5), "hour {h} s_dir");

            let expected_m_dir = [-expected_s_dir[0], -expected_s_dir[1], -expected_s_dir[2]];
            assert!(approx3(look.moon_direction, expected_m_dir, 1e-5), "hour {h} m_dir");

            let t = ((expected_elevation + 0.3) / 0.4).clamp(0.0, 1.0);
            let expected_energy = 0.9 * t * t * (3.0 - 2.0 * t);
            assert!(approx(look.sun_energy, expected_energy, 1e-5), "hour {h} sun_energy");

            let expected_day = (expected_energy / 0.9).clamp(0.0, 1.0);
            assert!(approx(look.day_factor, expected_day, 1e-5), "hour {h} day_factor");

            let expected_moon_energy = 0.12 * (1.0 - expected_day);
            assert!(approx(look.moon_energy, expected_moon_energy, 1e-5), "hour {h} moon_energy");

            let expected_ambient = 0.18 + (0.6 - 0.18) * expected_day;
            assert!(approx(look.ambient_energy, expected_ambient, 1e-5), "hour {h} ambient");

            let warm = 1.0 - (expected_elevation / 0.6).clamp(0.0, 1.0);
            let k = warm * 0.7;
            let expected_sun_color = [1.0, 0.98 + (0.6 - 0.98) * k, 0.93 + (0.35 - 0.93) * k];
            assert!(approx3(look.sun_color, expected_sun_color, 1e-5), "hour {h} sun_color");

            let expected_sky = sky_at(h);
            assert_eq!(look.sky_top, expected_sky.top, "hour {h} sky_top");
            assert_eq!(look.sky_horizon, expected_sky.horizon, "hour {h} sky_horizon");
            assert_eq!(look.ground_horizon, expected_sky.horizon, "hour {h} ground_horizon");
            assert_eq!(look.ground_bottom, expected_sky.horizon, "hour {h} ground_bottom");
            assert_eq!(look.fog_color, expected_sky.horizon, "hour {h} fog_color");

            assert_eq!(look.shadow_enabled, look.sun_energy >= 0.05, "hour {h} shadow");
        }
    }

    #[test]
    fn hour_0_night() {
        let look = DayNightLook::at(0.0);
        assert_eq!(look.sun_energy, 0.0);
        assert_eq!(look.day_factor, 0.0);
        assert_eq!(look.moon_energy, 0.12);
        assert_eq!(look.ambient_energy, 0.18);
        assert!(!look.shadow_enabled);
        assert_eq!(look.sky_top, NIGHT_TOP);
        assert_eq!(look.sky_horizon, NIGHT_HORIZON);
    }

    #[test]
    fn hour_12_daylight() {
        let look = DayNightLook::at(12.0);
        assert!(look.sun_energy > 0.85);
        assert_eq!(look.day_factor, 1.0);
        assert_eq!(look.moon_energy, 0.0);
        assert_eq!(look.ambient_energy, 0.6);
        assert!(look.shadow_enabled);
        assert_eq!(look.sky_top, DAY_TOP);
        assert_eq!(look.sky_horizon, DAY_HORIZON);
    }
}
