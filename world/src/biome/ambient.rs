use std::f32::consts::PI;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::biome::{BiomeDef, BiomeId, BiomeRegistry, BiomeSample};
use crate::weather::{WeatherKind, WeatherState};

/// Weather temperature offsets in degrees Celsius loaded from `data/temperature.json`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(default)]
struct WeatherOffsets {
    clear: f32,
    rain: f32,
    snow: f32,
    sandstorm: f32,
    fog: f32,
}

impl Default for WeatherOffsets {
    fn default() -> Self {
        Self {
            // data: temperature.json weather_offset_c.clear
            clear: 0.0,
            // data: temperature.json weather_offset_c.rain
            rain: -3.0,
            // data: temperature.json weather_offset_c.snow
            snow: -3.0,
            // data: temperature.json weather_offset_c.sandstorm
            sandstorm: 2.0,
            // data: temperature.json weather_offset_c.fog
            fog: -1.0,
        }
    }
}

impl WeatherOffsets {
    fn for_kind(self, kind: WeatherKind) -> f32 {
        match kind {
            WeatherKind::Clear => self.clear,
            WeatherKind::Rain => self.rain,
            WeatherKind::Snow => self.snow,
            WeatherKind::Sandstorm => self.sandstorm,
            WeatherKind::Fog => self.fog,
        }
    }
}

/// Subset of `data/temperature.json` needed for ambient air temperature.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(default)]
struct AmbientConfig {
    sun_peak_hour: f32,
    sun_low_hour: f32,
    weather_offset_c: WeatherOffsets,
}

impl Default for AmbientConfig {
    fn default() -> Self {
        Self {
            // data: temperature.json sun_peak_hour
            sun_peak_hour: 14.0,
            // data: temperature.json sun_low_hour
            sun_low_hour: 4.0,
            weather_offset_c: WeatherOffsets::default(),
        }
    }
}

fn ambient_config() -> &'static AmbientConfig {
    static CONFIG: OnceLock<AmbientConfig> = OnceLock::new();
    CONFIG.get_or_init(load_ambient_config)
}

fn load_ambient_config() -> AmbientConfig {
    if let Ok(path) = paths::try_repo_path("data/temperature.json")
        && let Ok(text) = std::fs::read_to_string(path)
        && let Ok(cfg) = serde_json::from_str::<AmbientConfig>(&text)
    {
        return cfg;
    }
    AmbientConfig::default()
}

/// Smooth piecewise cosine sun curve in `0.0..=1.0`, peaking at `peak_hour` (14:00)
/// and reaching its minimum at `low_hour` (04:00).
fn sun_factor(hour: f32, peak_hour: f32, low_hour: f32) -> f32 {
    // new: DECISIONS 2026-11-25
    const HOURS_PER_DAY: f32 = 24.0;
    let h = hour.rem_euclid(HOURS_PER_DAY);
    let rise_span = (peak_hour - low_hour).rem_euclid(HOURS_PER_DAY);
    let fall_span = HOURS_PER_DAY - rise_span;
    let since_low = (h - low_hour).rem_euclid(HOURS_PER_DAY);

    if since_low <= rise_span && rise_span > 0.0 {
        let u = since_low / rise_span;
        0.5 * (1.0 - (PI * u).cos())
    } else if fall_span > 0.0 {
        let since_peak = (h - peak_hour).rem_euclid(HOURS_PER_DAY);
        let u = (since_peak / fall_span).clamp(0.0, 1.0);
        0.5 * (1.0 + (PI * u).cos())
    } else {
        1.0
    }
}

fn biome_temp(def: &BiomeDef, sun: f32) -> f32 {
    def.temperature_night + (def.temperature_day - def.temperature_night) * sun
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Computes ambient air temperature in degrees Celsius for `sample` at `hour` (0..24)
/// and active `weather`.
///
/// Blends `temperature_day` / `temperature_night` via the sun curve (peak 14:00, low 04:00),
/// primary/secondary outer biomes by `sample.blend`, the home biome (Plains) by `sample.outer`,
/// and adds the intensity-scaled weather offset from `data/temperature.json`.
pub fn ambient_temperature(
    registry: &BiomeRegistry,
    sample: &BiomeSample,
    hour: f32,
    weather: &WeatherState,
) -> f32 {
    let cfg = ambient_config();
    let sun = sun_factor(hour, cfg.sun_peak_hour, cfg.sun_low_hour);
    let home_temp = biome_temp(registry.get(BiomeId::Plains), sun);
    let primary_temp = biome_temp(registry.get(sample.primary), sun);
    let secondary_temp = biome_temp(registry.get(sample.secondary), sun);
    let outer_temp = lerp(primary_temp, secondary_temp, sample.blend.clamp(0.0, 1.0));
    let base_temp = lerp(home_temp, outer_temp, sample.outer.clamp(0.0, 1.0));
    let weather_offset =
        cfg.weather_offset_c.for_kind(weather.kind) * weather.intensity.clamp(0.0, 1.0);
    base_temp + weather_offset
}
