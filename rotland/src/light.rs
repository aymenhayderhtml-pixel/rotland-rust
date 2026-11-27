//! The light of the day, ported from Godot's DayNightPalette (sun path, sun colour, sky colours).

use engine::render::{Lighting, SpotLight};
use glam::Vec3;
use world::daynight::DayNightLook;

const BASE_FOG_BEGIN: f32 = 120.0; // moved: rotland/src/light.rs
const BASE_FOG_END: f32 = 190.0; // moved: rotland/src/light.rs
const MIN_FOG_BEGIN: f32 = 25.0; // new: DECISIONS 2026-11-25
const MIN_FOG_END: f32 = 45.0; // new: DECISIONS 2026-11-25
const WEATHER_TINT_MIX: f32 = 0.5; // new: DECISIONS 2026-11-25
const SKY_TOP_TINT_SCALE: f32 = 0.5; // new: DECISIONS 2026-11-25
const LUMA_R: f32 = 0.2126; // new: DECISIONS 2026-11-25
const LUMA_G: f32 = 0.7152; // new: DECISIONS 2026-11-25
const LUMA_B: f32 = 0.0722; // new: DECISIONS 2026-11-25
const MIN_TINT_LUMA: f32 = 1e-4; // new: DECISIONS 2026-11-25
const FLASH_ANGLE_DEG: f32 = 120.0; // new: DECISIONS 2026-11-25
const FLASH_COLOR: Vec3 = Vec3::new(1.0, 0.85, 0.45); // new: DECISIONS 2026-11-25
const FLASH_ENERGY: f32 = 4.0; // new: DECISIONS 2026-11-25

/// Weather and biome atmosphere inputs for [`at_hour_weather`].
#[allow(dead_code)] // unused by the humanoid_view example, which includes this file
#[derive(Clone, Copy, Debug)]
pub struct WeatherLight {
    /// Multiplier applied to base fog begin/end distances (1.0 = normal, <1.0 = denser fog).
    pub fog_mult: f32,
    /// Weather tint colour (0..1) from `data/weather.json`.
    pub weather_tint: Vec3,
    /// Current weather intensity (0..1).
    pub intensity: f32,
    /// Biome fog tint (0..1) from `BiomeRegistry::get(primary).fog_tint`.
    pub biome_fog_tint: Vec3,
    /// Outer biome blend weight (0 = home core, 1 = outer biome) from `BiomeSample::outer`.
    pub biome_outer: f32,
}

impl Default for WeatherLight {
    fn default() -> Self {
        Self {
            fog_mult: 1.0,
            weather_tint: Vec3::ONE,
            intensity: 0.0,
            biome_fog_tint: Vec3::ONE,
            biome_outer: 0.0,
        }
    }
}

#[allow(dead_code)] // unused by the humanoid_view example, which includes this file
/// Converts an 8-bit RGB colour (`[0..255; 3]`) into normalized `0.0..=1.0` `Vec3`.
pub fn rgb_u8_to_vec3(rgb: [u8; 3]) -> Vec3 {
    Vec3::new(
        f32::from(rgb[0]) / 255.0,
        f32::from(rgb[1]) / 255.0,
        f32::from(rgb[2]) / 255.0,
    )
}

#[allow(dead_code)] // unused by the humanoid_view example, which includes this file
/// The flashlight (scripts/Items/Flashlight.cs): 25 m range, 30 degree cone, brightness 3, warm white.
pub fn flashlight(position: Vec3, direction: Vec3) -> SpotLight {
    SpotLight {
        position,
        direction,
        range: 25.0,
        angle_deg: 30.0,
        color: Vec3::new(1.0, 0.93, 0.78),
        energy: 3.0,
    }
}

#[allow(dead_code)] // unused by the humanoid_view example, which includes this file
/// Short-lived warm muzzle flash light with radius `range` in metres.
pub fn muzzle_flash(position: Vec3, direction: Vec3, range: f32) -> SpotLight {
    SpotLight {
        position,
        direction: direction.normalize_or_zero(),
        range,
        angle_deg: FLASH_ANGLE_DEG,
        color: FLASH_COLOR,
        energy: FLASH_ENERGY,
    }
}

#[allow(dead_code)] // unused by the humanoid_view example, which includes this file
/// Inserts a transient flash light into the first empty slot of `lighting.spots`, or slot 2 if full.
pub fn add_flash_light(lighting: &mut Lighting, flash: SpotLight) {
    if let Some(slot) = lighting.spots.iter_mut().find(|s| s.is_none()) {
        *slot = Some(flash);
    } else {
        lighting.spots[2] = Some(flash);
    }
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
        fog_begin: BASE_FOG_BEGIN,
        fog_end: BASE_FOG_END,
        fog_density: 1.0,
        fog_color: Vec3::from(look.fog_color),
        moon_toward: Vec3::from(look.moon_direction),
        moon_energy: look.moon_energy,
        moon_color: Vec3::from(look.moon_color),
        spots: [spot, None, None],
        nvg: [0.0; 4],
        nvg_tint: Vec3::new(0.35, 1.0, 0.45), // data: boosts.json nightVision.tint
    }
}

/// Lighting at `hour` (0..24) with biome fog tint and weather fog/sky scaling.
///
/// Fog begin/end scale by `weather.fog_mult` and clamp to at least `25 m` / `45 m`.
/// Fog colour mixes toward `biome_fog_tint` by `biome_outer`, then toward `weather_tint`
/// by `intensity * 0.5`. Tints are luminance-capped to the base night/day colour so night
/// fog and sky stay dark.
#[allow(dead_code)] // unused by the humanoid_view example, which includes this file
pub fn at_hour_weather(hour: f32, spot: Option<SpotLight>, weather: WeatherLight) -> Lighting {
    let mut lighting = at_hour(hour, spot);

    let fog_mult = weather.fog_mult.max(0.0);
    lighting.fog_begin = (BASE_FOG_BEGIN * fog_mult).max(MIN_FOG_BEGIN);
    lighting.fog_end = (BASE_FOG_END * fog_mult).max(MIN_FOG_END);

    let base_fog = lighting.fog_color;
    let outer = weather.biome_outer.clamp(0.0, 1.0);
    let biome_target = darken_tint_to_base(base_fog, weather.biome_fog_tint);
    let fog_after_biome = base_fog.lerp(biome_target, outer);

    let weather_mix = weather.intensity.clamp(0.0, 1.0) * WEATHER_TINT_MIX;
    let weather_fog_target = darken_tint_to_base(fog_after_biome, weather.weather_tint);
    lighting.fog_color = fog_after_biome.lerp(weather_fog_target, weather_mix);

    if weather_mix > 0.0 {
        let horizon_target = darken_tint_to_base(lighting.sky_horizon, weather.weather_tint);
        lighting.sky_horizon = lighting.sky_horizon.lerp(horizon_target, weather_mix);
        let top_target = darken_tint_to_base(lighting.sky_top, weather.weather_tint);
        lighting.sky_top = lighting
            .sky_top
            .lerp(top_target, weather_mix * SKY_TOP_TINT_SCALE);
    }

    lighting
}

fn luminance(color: Vec3) -> f32 {
    color.x * LUMA_R + color.y * LUMA_G + color.z * LUMA_B
}

/// Scales `tint` down if it is brighter than `base`, keeping hue while never brightening dark fog/sky.
fn darken_tint_to_base(base: Vec3, tint: Vec3) -> Vec3 {
    let base_luma = luminance(base).max(0.0);
    let tint_luma = luminance(tint);
    if tint_luma <= MIN_TINT_LUMA {
        return Vec3::ZERO;
    }
    let scale = (base_luma / tint_luma).min(1.0);
    tint * scale
}

/// Height above the ground (m) where the view starts to clear. // new: DECISIONS 2026-11-26
#[allow(dead_code)] // unused by the humanoid_view example, which includes this file
const FOG_LIFT_START_M: f32 = 10.0;
/// Extra fog end per metre of height above that (begin gets half). // new: DECISIONS 2026-11-26
#[allow(dead_code)] // unused by the humanoid_view example, which includes this file
const FOG_LIFT_PER_M: f32 = 2.0;

/// The higher the camera above the ground, the farther it sees (rooftops, the `--metro=top` view).
/// On the ground (camera under ~10 m up) nothing changes.
#[allow(dead_code)] // unused by the humanoid_view example, which includes this file
pub fn lift_fog_for_height(lighting: &mut Lighting, height_above_ground: f32) {
    let lift = (height_above_ground - FOG_LIFT_START_M).max(0.0) * FOG_LIFT_PER_M;
    lighting.fog_begin += lift * 0.5;
    lighting.fog_end += lift;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_weather_matches_plain_at_hour() {
        let base = at_hour(13.0, None);
        let with_default = at_hour_weather(13.0, None, WeatherLight::default());
        assert_eq!(base.fog_begin, with_default.fog_begin);
        assert_eq!(base.fog_end, with_default.fog_end);
        assert!((base.fog_color - with_default.fog_color).length() < 1e-5);
        assert!((base.sky_horizon - with_default.sky_horizon).length() < 1e-5);
        assert!((base.sky_top - with_default.sky_top).length() < 1e-5);
    }

    #[test]
    fn fog_distances_scale_and_clamp_to_minimums() {
        let half = at_hour_weather(
            12.0,
            None,
            WeatherLight {
                fog_mult: 0.5,
                ..WeatherLight::default()
            },
        );
        assert!((half.fog_begin - 60.0).abs() < 1e-4);
        assert!((half.fog_end - 95.0).abs() < 1e-4);

        let extreme = at_hour_weather(
            12.0,
            None,
            WeatherLight {
                fog_mult: 0.05,
                ..WeatherLight::default()
            },
        );
        assert!((extreme.fog_begin - MIN_FOG_BEGIN).abs() < 1e-4);
        assert!((extreme.fog_end - MIN_FOG_END).abs() < 1e-4);
    }

    #[test]
    fn night_fog_and_sky_never_brighten_under_bright_tints() {
        let night = at_hour(2.0, None);
        let tinted = at_hour_weather(
            2.0,
            None,
            WeatherLight {
                fog_mult: 0.3,
                weather_tint: Vec3::new(0.95, 0.98, 1.0),
                intensity: 1.0,
                biome_fog_tint: rgb_u8_to_vec3([215, 228, 242]),
                biome_outer: 1.0,
            },
        );
        assert!(
            luminance(tinted.fog_color) <= luminance(night.fog_color) + 1e-4,
            "night fog brightened: base={} tinted={}",
            luminance(night.fog_color),
            luminance(tinted.fog_color)
        );
        assert!(
            luminance(tinted.sky_horizon) <= luminance(night.sky_horizon) + 1e-4,
            "night horizon brightened"
        );
        assert!(
            luminance(tinted.sky_top) <= luminance(night.sky_top) + 1e-4,
            "night sky top brightened"
        );
    }

    #[test]
    fn day_fog_mixes_toward_biome_and_weather_tints() {
        let base = at_hour(12.0, None);
        let desert_sandstorm = at_hour_weather(
            12.0,
            None,
            WeatherLight {
                fog_mult: 0.35,
                weather_tint: Vec3::new(0.85, 0.65, 0.38),
                intensity: 1.0,
                biome_fog_tint: rgb_u8_to_vec3([235, 215, 170]),
                biome_outer: 1.0,
            },
        );
        // Warm sandy tint should increase red relative to blue compared with clear midday fog.
        let base_rb = base.fog_color.x - base.fog_color.z;
        let tinted_rb = desert_sandstorm.fog_color.x - desert_sandstorm.fog_color.z;
        assert!(tinted_rb > base_rb + 0.1);
    }

    #[test]
    fn add_flash_light_uses_first_empty_slot_then_replaces_last() {
        let mut lighting = at_hour(22.0, Some(flashlight(Vec3::ZERO, Vec3::NEG_Z)));
        let flash = muzzle_flash(Vec3::Y, Vec3::NEG_Z, 4.0);
        add_flash_light(&mut lighting, flash);
        assert!(lighting.spots[0].is_some());
        assert_eq!(lighting.spots[1], Some(flash));
        assert!(lighting.spots[2].is_none());

        let head = flashlight(Vec3::X, Vec3::NEG_Z);
        lighting.spots[2] = Some(head);
        let flash2 = muzzle_flash(Vec3::Z, Vec3::NEG_Z, 5.0);
        add_flash_light(&mut lighting, flash2);
        assert_eq!(lighting.spots[2], Some(flash2));
    }

    #[test]
    fn fog_lifts_only_high_above_the_ground() {
        let mut low = at_hour(12.0, None);
        let (b, e) = (low.fog_begin, low.fog_end);
        lift_fog_for_height(&mut low, 3.0);
        assert_eq!((low.fog_begin, low.fog_end), (b, e));
        let mut high = at_hour(12.0, None);
        lift_fog_for_height(&mut high, 210.0);
        assert!((high.fog_end - (e + 400.0)).abs() < 1e-3);
        assert!((high.fog_begin - (b + 200.0)).abs() < 1e-3);
    }
}
