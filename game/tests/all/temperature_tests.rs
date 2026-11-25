use game_player::survival::temperature::{BodyTemp, TempState, TemperatureData, WarmthSources};

fn load_data() -> TemperatureData {
    TemperatureData::from_repo().expect("failed to load data/temperature.json")
}

fn step_seconds(
    body: &mut BodyTemp,
    seconds: u32,
    ambient_c: f32,
    sources: &WarmthSources,
    data: &TemperatureData,
) {
    for _ in 0..seconds {
        body.update(1.0, ambient_c, sources, data);
    }
}

#[test]
fn comfy_at_18c() {
    let data = load_data();
    let mut body = BodyTemp::new(&data);
    let none = WarmthSources::default();

    step_seconds(&mut body, 600, 18.0, &none, &data);

    assert!((body.value() - 50.0).abs() < 1e-3);
    assert_eq!(body.state(), TempState::Comfy);
    assert!((body.thirst_mult() - 1.0).abs() < 1e-5);
    assert!((body.stamina_regen_mult() - 1.0).abs() < 1e-5);
    assert!((body.health_drain_per_sec() - 0.0).abs() < 1e-5);
}

#[test]
fn drifts_to_cold_in_snow_within_three_minutes_not_thirty_seconds() {
    let data = load_data();
    let mut body = BodyTemp::new(&data);
    let none = WarmthSources::default();
    // Snow daytime temperature from data/biomes.json (-12 C).
    let snow_day_c = -12.0;

    step_seconds(&mut body, 30, snow_day_c, &none, &data);
    assert_eq!(
        body.state(),
        TempState::Comfy,
        "should NOT drop to Cold within 30 s, value was {}",
        body.value()
    );

    step_seconds(&mut body, 150, snow_day_c, &none, &data);
    assert_eq!(
        body.state(),
        TempState::Cold,
        "should drift to Cold within 3 game minutes, value was {}",
        body.value()
    );
    assert!((body.stamina_regen_mult() - 0.7).abs() < 1e-5);
}

#[test]
fn near_fire_stays_comfy_in_cold_and_caps_at_55() {
    let data = load_data();
    let near_fire = WarmthSources {
        near_fire: true,
        ..WarmthSources::default()
    };

    let mut snow_body = BodyTemp::new(&data);
    step_seconds(&mut snow_body, 600, -22.0, &near_fire, &data);
    assert_eq!(
        snow_body.state(),
        TempState::Comfy,
        "near fire should stay Comfy in -22 C snow, value was {}",
        snow_body.value()
    );

    let mut mild_body = BodyTemp::new(&data);
    step_seconds(&mut mild_body, 600, 18.0, &near_fire, &data);
    assert!(
        mild_body.value() <= 55.0 + 1e-3,
        "near fire should cap warmth at 55, got {}",
        mild_body.value()
    );
    assert_eq!(mild_body.state(), TempState::Comfy);
}

#[test]
fn insulation_07_keeps_at_least_cold_at_minus_10c() {
    let data = load_data();
    let warm_clothes = WarmthSources {
        insulation: 0.7,
        ..WarmthSources::default()
    };

    let mut body = BodyTemp::new(&data);
    step_seconds(&mut body, 1200, -10.0, &warm_clothes, &data);
    assert_ne!(
        body.state(),
        TempState::Freezing,
        "insulation 0.7 should prevent Freezing at -10 C, value was {}",
        body.value()
    );
    assert!(body.value() >= data.thresholds.freezing_max);

    // Even in -22 C snow night, 0.7 insulation stays Cold rather than Freezing.
    let mut night_body = BodyTemp::new(&data);
    step_seconds(&mut night_body, 1200, -22.0, &warm_clothes, &data);
    assert_eq!(
        night_body.state(),
        TempState::Cold,
        "insulation 0.7 should stay Cold in -22 C snow night, value was {}",
        night_body.value()
    );
}

#[test]
fn freezing_drains_health_but_never_below_ten() {
    let data = load_data();
    let mut body = BodyTemp::new(&data);
    let none = WarmthSources::default();

    step_seconds(&mut body, 900, -22.0, &none, &data);
    assert_eq!(body.state(), TempState::Freezing);
    assert!((body.stamina_regen_mult() - 0.5).abs() < 1e-5);
    assert!((body.health_drain_per_sec() - 0.25).abs() < 1e-5);

    let mut health = 100.0_f32;
    for _ in 0..1000 {
        health -= body.health_drain_for(health, 1.0);
    }
    assert!(
        (health - 10.0).abs() < 1e-4,
        "freezing should stop draining at 10 HP, got {health}"
    );

    // If health is already below 10 from another source, cold drains 0 and does not heal.
    assert!((body.health_drain_for(5.0, 10.0) - 0.0).abs() < 1e-5);
}

#[test]
fn desert_noon_gives_thirst_mult_above_one() {
    let data = load_data();
    let none = WarmthSources::default();
    let mut body = BodyTemp::new(&data);

    // Desert at noon (~35.5 C from data/biomes.json 38 C day / 12 C night).
    step_seconds(&mut body, 600, 35.5, &none, &data);
    assert_eq!(body.state(), TempState::Hot);
    assert!(
        body.thirst_mult() > 1.0,
        "desert noon should raise thirst_mult above 1, got {}",
        body.thirst_mult()
    );
    assert!((body.thirst_mult() - 1.3).abs() < 1e-5);

    // Sun protection counteracts desert heat.
    let scarf = WarmthSources {
        sun_protection: 0.5,
        ..WarmthSources::default()
    };
    let mut shaded = BodyTemp::new(&data);
    step_seconds(&mut shaded, 600, 35.5, &scarf, &data);
    assert_eq!(shaded.state(), TempState::Comfy);
    assert!((shaded.thirst_mult() - 1.0).abs() < 1e-5);
}

#[test]
fn chill_lowers_value_and_clamps_to_zero() {
    let data = load_data();
    let mut body = BodyTemp::new(&data);

    body.chill(8.0);
    assert!((body.value() - 42.0).abs() < 1e-4);

    // Non-positive chill values do not warm the player.
    body.chill(-10.0);
    assert!((body.value() - 42.0).abs() < 1e-4);

    body.chill(100.0);
    assert!((body.value() - 0.0).abs() < 1e-4);
    assert_eq!(body.state(), TempState::Freezing);
}

#[test]
fn state_boundaries_and_overheated_effects_match_data() {
    let data = load_data();
    let mut body = BodyTemp::from_config(&data);

    let cases = [
        (14.9, TempState::Freezing, 1.0, 0.5, 0.25),
        (15.0, TempState::Cold, 1.0, 0.7, 0.0),
        (34.9, TempState::Cold, 1.0, 0.7, 0.0),
        (35.0, TempState::Comfy, 1.0, 1.0, 0.0),
        (65.0, TempState::Comfy, 1.0, 1.0, 0.0),
        (65.1, TempState::Hot, 1.3, 1.0, 0.0),
        (85.0, TempState::Hot, 1.3, 1.0, 0.0),
        (85.1, TempState::Overheated, 1.8, 0.7, 0.0),
    ];

    for (val, expected_state, expected_thirst, expected_stam, expected_drain) in cases {
        body.set(val);
        assert_eq!(body.state(), expected_state, "state mismatch at {val}");
        assert!((body.thirst_mult() - expected_thirst).abs() < 1e-5);
        assert!((body.stamina_regen_mult() - expected_stam).abs() < 1e-5);
        assert!((body.health_drain_per_sec() - expected_drain).abs() < 1e-5);
    }

    let saved = body.to_save();
    let mut restored = BodyTemp::new(&data);
    restored.from_save(&saved);
    assert!((restored.value() - 85.1).abs() < 1e-4);
}
