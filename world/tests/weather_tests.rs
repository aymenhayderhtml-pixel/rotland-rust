use world::{BiomeId, WeatherData, WeatherKind, WeatherState};

#[test]
fn data_loads_and_every_biome_has_non_empty_list() {
    let data = WeatherData::load().expect("failed to load data/weather.json");
    assert!(data.fade_seconds > 0.0);
    assert!(data.min_spell_seconds > 0.0);
    assert!(data.max_spell_seconds >= data.min_spell_seconds);

    for biome in BiomeId::ALL {
        let list = data.biome_weights(biome);
        assert!(
            !list.is_empty(),
            "biome '{}' must have a non-empty weather list",
            biome.key()
        );
        let weight_sum: f32 = list.iter().map(|entry| entry.weight).sum();
        assert!(
            weight_sum > 0.0,
            "biome '{}' weather weights must sum to > 0",
            biome.key()
        );
    }

    for kind in WeatherKind::ALL {
        assert_eq!(WeatherKind::from_key(kind.key()), Some(kind));
        let def = data.kind(kind);
        assert!((0.0..=1.0).contains(&def.fog_mult));
        assert!((0.0..=1.0).contains(&def.visibility));
        assert!((0.0..=1.0).contains(&def.noise_mask));
        assert!(def.wind >= 0.0);
        assert!(def.particles_per_m3 >= 0.0);
    }

    assert_eq!(WeatherKind::from_key("tornado"), None);
    assert_eq!(WeatherKind::from_key(""), None);
}

#[test]
fn from_json_rejects_missing_biome_or_kind() {
    let path = paths::repo_path("data/weather.json");
    let raw = std::fs::read_to_string(path).expect("read data/weather.json");

    let mut missing_biome: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
    missing_biome["biomes"]
        .as_object_mut()
        .expect("biomes object")
        .remove("swamp");
    let err = WeatherData::from_json(&serde_json::to_string(&missing_biome).expect("serialize"))
        .expect_err("expected missing biome error");
    assert!(
        err.contains("swamp"),
        "error should name missing biome, got: {err}"
    );

    let mut missing_kind: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
    missing_kind["kinds"]
        .as_object_mut()
        .expect("kinds object")
        .remove("sandstorm");
    let err = WeatherData::from_json(&serde_json::to_string(&missing_kind).expect("serialize"))
        .expect_err("expected missing kind error");
    assert!(
        err.contains("sandstorm"),
        "error should name missing kind, got: {err}"
    );
}

#[test]
fn same_seed_and_steps_produce_identical_weather() {
    let data = WeatherData::load().expect("load data/weather.json");
    let mut a = WeatherState::new(1337);
    let mut b = WeatherState::new(1337);

    let schedule = [
        (5.0_f32, BiomeId::Plains),
        (12.5, BiomeId::Forest),
        (25.0, BiomeId::Desert),
        (10.0, BiomeId::Snow),
        (15.0, BiomeId::Mountains),
        (8.0, BiomeId::Swamp),
    ];

    for step in 0..400 {
        let (dt, biome) = schedule[(step / 50) % schedule.len()];
        a.update(dt, biome, &data);
        b.update(dt, biome, &data);

        assert_eq!(a.kind, b.kind, "kind mismatch at step {step}");
        assert!(
            (a.intensity - b.intensity).abs() < 1e-6,
            "intensity mismatch at step {step}"
        );
        assert!(
            (a.wind - b.wind).length() < 1e-6,
            "wind mismatch at step {step}"
        );
        assert!((a.fog_mult() - b.fog_mult()).abs() < 1e-6);
        assert!((a.visibility() - b.visibility()).abs() < 1e-6);
        assert!((a.noise_mask() - b.noise_mask()).abs() < 1e-6);
    }
}

#[test]
fn sandstorm_never_starts_in_snow() {
    let data = WeatherData::load().expect("load data/weather.json");
    let mut saw_snow = false;

    for seed in [1_i32, 42, 999, -12345] {
        let mut state = WeatherState::new(seed);
        for step in 0..1000 {
            state.update(15.0, BiomeId::Snow, &data);
            assert_ne!(
                state.kind,
                WeatherKind::Sandstorm,
                "sandstorm started in snow for seed={seed} step={step}"
            );
            assert!(
                data.biome_allows(BiomeId::Snow, state.kind),
                "disallowed weather {:?} in snow",
                state.kind
            );
            if state.kind == WeatherKind::Snow {
                saw_snow = true;
            }
        }
    }

    assert!(saw_snow, "expected snow weather to occur in Snow biome");
}

#[test]
fn walking_from_desert_into_snow_ends_sandstorm_within_fade_seconds() {
    let data = WeatherData::load().expect("load data/weather.json");

    // 1. Reach a full-intensity sandstorm naturally in Desert, then cross into Snow.
    let mut state = WeatherState::new(42);
    let mut reached_sandstorm = false;
    for _ in 0..2000 {
        state.update(20.0, BiomeId::Desert, &data);
        if state.kind == WeatherKind::Sandstorm && state.intensity >= 0.99 {
            reached_sandstorm = true;
            break;
        }
    }
    assert!(
        reached_sandstorm,
        "expected a sandstorm spell to occur in Desert"
    );

    let step_dt = 0.5_f32;
    let steps = (data.fade_seconds / step_dt).ceil() as usize;
    for _ in 0..steps {
        state.update(step_dt, BiomeId::Snow, &data);
        assert!((0.0..=1.0).contains(&state.intensity));
    }
    assert_ne!(
        state.kind,
        WeatherKind::Sandstorm,
        "sandstorm should end within fade_seconds after entering Snow"
    );
    assert!(data.biome_allows(BiomeId::Snow, state.kind));

    // 2. Verify at 60 FPS ticks from forced full-intensity sandstorm.
    let mut forced = WeatherState::new(7);
    forced.set_weather(WeatherKind::Sandstorm, 1.0, &data);
    let frame_dt = 1.0_f32 / 60.0;
    let frames = (data.fade_seconds * 60.0).ceil() as usize;
    for _ in 0..frames {
        forced.update(frame_dt, BiomeId::Snow, &data);
    }
    assert_ne!(
        forced.kind,
        WeatherKind::Sandstorm,
        "sandstorm should end within fade_seconds at 60 Hz ticks"
    );
}

#[test]
fn intensity_always_stays_in_unit_range() {
    let data = WeatherData::load().expect("load data/weather.json");
    let mut state = WeatherState::new(2026);

    // Non-positive dt should leave intensity valid.
    state.update(0.0, BiomeId::Plains, &data);
    assert!((0.0..=1.0).contains(&state.intensity));
    state.update(-5.0, BiomeId::Desert, &data);
    assert!((0.0..=1.0).contains(&state.intensity));

    for i in 0..1500 {
        let biome = BiomeId::ALL[(i / 37) % BiomeId::COUNT];
        let dt = if i % 29 == 0 { 180.0 } else { 3.5 };
        state.update(dt, biome, &data);
        assert!(
            state.intensity.is_finite() && (0.0..=1.0).contains(&state.intensity),
            "intensity out of range at step {i}: {}",
            state.intensity
        );
        assert!((0.0..=1.0).contains(&state.fog_mult()));
        assert!((0.0..=1.0).contains(&state.visibility()));
        assert!((0.0..=1.0).contains(&state.noise_mask()));
    }
}

#[test]
fn fog_mult_is_one_when_clear_and_blends_with_intensity() {
    let data = WeatherData::load().expect("load data/weather.json");
    let mut state = WeatherState::new(42);

    assert!((state.fog_mult() - 1.0).abs() < 1e-6);
    assert!((state.visibility() - 1.0).abs() < 1e-6);
    assert!(state.noise_mask().abs() < 1e-6);

    state.update(1.0, BiomeId::Plains, &data);
    assert_eq!(state.kind, WeatherKind::Clear);
    assert!((state.fog_mult() - 1.0).abs() < 1e-6);

    state.set_weather(WeatherKind::Sandstorm, 0.0, &data);
    assert!((state.fog_mult() - 1.0).abs() < 1e-6);
    assert!((state.visibility() - 1.0).abs() < 1e-6);
    assert!(state.noise_mask().abs() < 1e-6);

    state.set_weather(WeatherKind::Sandstorm, 0.5, &data);
    assert!((state.fog_mult() - 0.65).abs() < 1e-5);
    assert!((state.visibility() - 0.7).abs() < 1e-5);
    assert!((state.noise_mask() - 0.25).abs() < 1e-5);

    state.set_weather(WeatherKind::Sandstorm, 1.0, &data);
    let sand = data.kind(WeatherKind::Sandstorm);
    assert!((state.fog_mult() - sand.fog_mult).abs() < 1e-6);
    assert!((state.visibility() - sand.visibility).abs() < 1e-6);
    assert!((state.noise_mask() - sand.noise_mask).abs() < 1e-6);
}

#[test]
fn wind_scales_with_intensity_and_drifts_slowly() {
    let data = WeatherData::load().expect("load data/weather.json");
    let mut state = WeatherState::new(77);
    state.set_weather(WeatherKind::Rain, 1.0, &data);

    let initial_dir = state.wind.normalize();
    let rain_wind = data.kind(WeatherKind::Rain).wind;
    assert!((state.wind.length() - rain_wind).abs() < 1e-5);

    state.update(60.0, BiomeId::Plains, &data);
    let after_dir = state.wind.normalize();
    let angle_deg = initial_dir.dot(after_dir).clamp(-1.0, 1.0).acos().to_degrees();
    assert!(
        (1.0..=6.0).contains(&angle_deg),
        "expected wind to drift a few degrees per minute, got {angle_deg} deg"
    );
    assert!((state.wind.length() - rain_wind * state.intensity).abs() < 1e-5);
}

#[test]
fn weather_state_serde_round_trip_preserves_determinism() {
    let data = WeatherData::load().expect("load data/weather.json");
    let mut original = WeatherState::new(555);
    for _ in 0..80 {
        original.update(10.0, BiomeId::Swamp, &data);
    }

    let json = serde_json::to_string(&original).expect("serialize WeatherState");
    let mut restored: WeatherState = serde_json::from_str(&json).expect("deserialize WeatherState");

    assert_eq!(restored.kind, original.kind);
    assert!((restored.intensity - original.intensity).abs() < 1e-6);
    assert!((restored.wind - original.wind).length() < 1e-6);
    assert!((restored.fog_mult() - original.fog_mult()).abs() < 1e-6);

    for _ in 0..120 {
        original.update(12.0, BiomeId::Mountains, &data);
        restored.update(12.0, BiomeId::Mountains, &data);
        assert_eq!(restored.kind, original.kind);
        assert!((restored.intensity - original.intensity).abs() < 1e-6);
        assert!((restored.wind - original.wind).length() < 1e-6);
    }
}
