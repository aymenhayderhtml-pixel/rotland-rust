use glam::Vec2;
use world::biome::ambient_temperature;
use world::biome::layout::{WorldLayout, WorldLayoutData};
use world::biome::registry::BiomeRegistry;
use world::weather::{WeatherKind, WeatherState};
use world::BiomeId;

fn weather_of(kind: WeatherKind, intensity: f32) -> WeatherState {
    let mut state = WeatherState::new(1);
    state.kind = kind;
    state.intensity = intensity;
    state
}

#[test]
fn desert_at_14_is_over_30c() {
    let registry = BiomeRegistry::load().expect("failed to load data/biomes.json");
    let layout_data = WorldLayoutData::load().expect("failed to load data/world_layout.json");
    let layout = WorldLayout::new(&layout_data, 42, Vec2::ZERO, Vec2::X);

    let pos = layout.direction_at_deg(0.0) * 4000.0;
    let sample = layout.sample(pos.x, pos.y);
    assert_eq!(sample.primary, BiomeId::Desert);
    assert_eq!(sample.outer, 1.0);

    let clear = weather_of(WeatherKind::Clear, 0.0);
    let temp = ambient_temperature(&registry, &sample, 14.0, &clear);
    assert!(
        temp > 30.0,
        "expected desert 14:00 temperature > 30 C, got {temp}"
    );
}

#[test]
fn snow_at_04_is_below_minus_15c() {
    let registry = BiomeRegistry::load().expect("failed to load data/biomes.json");
    let layout_data = WorldLayoutData::load().expect("failed to load data/world_layout.json");
    let layout = WorldLayout::new(&layout_data, 42, Vec2::ZERO, Vec2::X);

    let pos = layout.direction_at_deg(180.0) * 4000.0;
    let sample = layout.sample(pos.x, pos.y);
    assert_eq!(sample.primary, BiomeId::Snow);
    assert_eq!(sample.outer, 1.0);

    let clear = weather_of(WeatherKind::Clear, 0.0);
    let temp = ambient_temperature(&registry, &sample, 4.0, &clear);
    assert!(
        temp < -15.0,
        "expected snow 04:00 temperature < -15 C, got {temp}"
    );
}

#[test]
fn home_biome_at_noon_is_between_12_and_26c() {
    let registry = BiomeRegistry::load().expect("failed to load data/biomes.json");
    let layout_data = WorldLayoutData::load().expect("failed to load data/world_layout.json");
    let layout = WorldLayout::new(&layout_data, 42, Vec2::ZERO, Vec2::X);

    let sample = layout.sample(0.0, 0.0);
    assert_eq!(sample.outer, 0.0);

    let clear = weather_of(WeatherKind::Clear, 0.0);
    let temp = ambient_temperature(&registry, &sample, 12.0, &clear);
    assert!(
        (12.0..=26.0).contains(&temp),
        "expected home biome at 12:00 between 12 and 26 C, got {temp}"
    );
}

#[test]
fn blend_band_midpoint_lies_between_home_and_outer_biomes() {
    let registry = BiomeRegistry::load().expect("failed to load data/biomes.json");
    let layout_data = WorldLayoutData::load().expect("failed to load data/world_layout.json");
    let layout = WorldLayout::new(&layout_data, 42, Vec2::ZERO, Vec2::X);
    let clear = weather_of(WeatherKind::Clear, 0.0);

    let home_sample = layout.sample(0.0, 0.0);
    let mid_dist = layout_data.core_radius + layout_data.blend_band * 0.5;
    let far_dist = layout_data.core_radius + layout_data.blend_band + 1000.0;

    // Home -> Desert at 14:00 (warmer outer biome).
    let desert_mid_pos = layout.direction_at_deg(0.0) * mid_dist;
    let desert_far_pos = layout.direction_at_deg(0.0) * far_dist;
    let home_14 = ambient_temperature(&registry, &home_sample, 14.0, &clear);
    let mid_14 = ambient_temperature(
        &registry,
        &layout.sample(desert_mid_pos.x, desert_mid_pos.y),
        14.0,
        &clear,
    );
    let desert_14 = ambient_temperature(
        &registry,
        &layout.sample(desert_far_pos.x, desert_far_pos.y),
        14.0,
        &clear,
    );
    assert!(
        mid_14 > home_14 && mid_14 < desert_14,
        "expected midpoint {mid_14} strictly between home {home_14} and desert {desert_14}"
    );

    // Home -> Snow at 04:00 (colder outer biome).
    let snow_mid_pos = layout.direction_at_deg(180.0) * mid_dist;
    let snow_far_pos = layout.direction_at_deg(180.0) * far_dist;
    let home_04 = ambient_temperature(&registry, &home_sample, 4.0, &clear);
    let mid_04 = ambient_temperature(
        &registry,
        &layout.sample(snow_mid_pos.x, snow_mid_pos.y),
        4.0,
        &clear,
    );
    let snow_04 = ambient_temperature(
        &registry,
        &layout.sample(snow_far_pos.x, snow_far_pos.y),
        4.0,
        &clear,
    );
    assert!(
        mid_04 < home_04 && mid_04 > snow_04,
        "expected midpoint {mid_04} strictly between snow {snow_04} and home {home_04}"
    );

    // Unwarped sector border between Desert and Swamp at 14:00 lies between both biomes.
    let mut unwarped = layout_data.clone();
    unwarped.warp_deg = 0.0;
    let flat_layout = WorldLayout::new(&unwarped, 42, Vec2::ZERO, Vec2::X);
    let swamp_pos = flat_layout.direction_at_deg(90.0) * 4000.0;
    let border_pos = flat_layout.direction_at_deg(45.0) * 4000.0;
    let swamp_14 = ambient_temperature(
        &registry,
        &flat_layout.sample(swamp_pos.x, swamp_pos.y),
        14.0,
        &clear,
    );
    let border_14 = ambient_temperature(
        &registry,
        &flat_layout.sample(border_pos.x, border_pos.y),
        14.0,
        &clear,
    );
    assert!(
        border_14 > swamp_14 && border_14 < desert_14,
        "expected border {border_14} between swamp {swamp_14} and desert {desert_14}"
    );
}

#[test]
fn weather_offsets_apply_and_scale_with_intensity() {
    let registry = BiomeRegistry::load().expect("failed to load data/biomes.json");
    let layout_data = WorldLayoutData::load().expect("failed to load data/world_layout.json");
    let layout = WorldLayout::new(&layout_data, 42, Vec2::ZERO, Vec2::X);
    let sample = layout.sample(0.0, 0.0);

    let clear = ambient_temperature(&registry, &sample, 14.0, &weather_of(WeatherKind::Clear, 1.0));
    let rain = ambient_temperature(&registry, &sample, 14.0, &weather_of(WeatherKind::Rain, 1.0));
    let snow = ambient_temperature(&registry, &sample, 14.0, &weather_of(WeatherKind::Snow, 1.0));
    let sand = ambient_temperature(
        &registry,
        &sample,
        14.0,
        &weather_of(WeatherKind::Sandstorm, 1.0),
    );
    let fog = ambient_temperature(&registry, &sample, 14.0, &weather_of(WeatherKind::Fog, 1.0));
    let half_rain =
        ambient_temperature(&registry, &sample, 14.0, &weather_of(WeatherKind::Rain, 0.5));

    assert!((rain - (clear - 3.0)).abs() < 1e-4);
    assert!((snow - (clear - 3.0)).abs() < 1e-4);
    assert!((sand - (clear + 2.0)).abs() < 1e-4);
    assert!((fog - (clear - 1.0)).abs() < 1e-4);
    assert!((half_rain - (clear - 1.5)).abs() < 1e-4);
}
