//! Tests for pressure director pacing, waves, night hordes, and save state.

use game::director::*;

#[test]
fn director_cycles_states_over_simulated_time() {
    let cfg = DirectorConfig::load_default().expect("data/director.json must load");
    let mut director = Director::new(cfg);

    assert_eq!(director.state(), DirectorState::Calm);
    assert_eq!(director.spawn_rate_multiplier(), 0.5);
    assert_eq!(director.max_active_multiplier(), 0.8);
    assert_eq!(director.special_chance(), 0.0);

    // Tick through Calm into Building
    let mut input = DirectorInput::new(1.0);
    input.is_night = false;
    input.player_health = 1.0;

    for _ in 0..41 {
        let out = director.update(&input);
        assert!(out.events.is_empty());
    }
    assert_eq!(director.state(), DirectorState::Building);
    assert!(director.spawn_rate_multiplier() > 1.0);
    assert!(director.special_chance() > 0.0);

    // Tick through Building into Wave
    let mut wave_started = false;
    for _ in 0..26 {
        let out = director.update(&input);
        for event in out.events {
            if let DirectorEvent::WaveStart { size, kind } = event {
                assert!(size > 0);
                assert_eq!(kind, WaveKind::Noise);
                wave_started = true;
            }
        }
    }
    assert!(wave_started, "Wave event must be emitted when entering Wave");
    assert_eq!(director.state(), DirectorState::Wave);
    assert_eq!(director.spawn_rate_multiplier(), 2.0);

    // Tick through Wave into Relax
    for _ in 0..21 {
        director.update(&input);
    }
    assert_eq!(director.state(), DirectorState::Relax);
    assert!(director.spawn_rate_multiplier() < 0.5);

    // Tick through Relax back to Calm
    for _ in 0..26 {
        director.update(&input);
    }
    assert_eq!(director.state(), DirectorState::Calm);
}

#[test]
fn director_no_waves_in_the_camp_or_at_low_health() {
    let cfg = DirectorConfig::load_default().expect("data/director.json must load");
    let mut director = Director::new(cfg);

    // Fast-forward to Building state
    let mut input = DirectorInput::new(41.0);
    director.update(&input);
    assert_eq!(director.state(), DirectorState::Building);

    // While player is in camp safe zone, building timer expires but wave does NOT start
    input.dt = 30.0;
    input.player_in_camp = true;
    let out = director.update(&input);
    assert_eq!(director.state(), DirectorState::Building);
    assert!(out.events.is_empty(), "No wave should trigger while player is in camp");

    // While player is at low health (< 0.3), wave also does NOT start
    input.player_in_camp = false;
    input.player_health = 0.25;
    let out = director.update(&input);
    assert_eq!(director.state(), DirectorState::Building);
    assert!(out.events.is_empty(), "No wave should trigger while player health is low");

    // Once player heals and leaves camp, wave triggers immediately
    input.player_health = 0.8;
    input.dt = 1.0;
    let out = director.update(&input);
    assert_eq!(director.state(), DirectorState::Wave);
    assert_eq!(out.events.len(), 1);
    match &out.events[0] {
        DirectorEvent::WaveStart { size, kind } => {
            assert!(*size > 0);
            assert_eq!(*kind, WaveKind::Noise);
        }
        _ => panic!("Expected WaveStart event"),
    }
}

#[test]
fn director_one_night_horde_per_night() {
    let cfg = DirectorConfig::load_default().expect("data/director.json must load");
    let mut director = Director::new(cfg);

    let mut input = DirectorInput::new(10.0);
    input.is_night = true;
    input.player_in_camp = true; // Night horde attacks camp even if player is in safe zone

    let mut camp_attacks = 0;

    // Simulate 120 seconds of night (12 steps of 10s)
    for _ in 0..12 {
        let out = director.update(&input);
        for event in out.events {
            if let DirectorEvent::CampAttack { size } = event {
                assert_eq!(size, 20);
                camp_attacks += 1;
            }
        }
    }

    assert_eq!(camp_attacks, 1, "Exactly one CampAttack night horde per night");

    // Transition to day
    input.is_night = false;
    for _ in 0..6 {
        let out = director.update(&input);
        for event in out.events {
            if let DirectorEvent::CampAttack { .. } = event {
                panic!("Camp attack should not happen during day");
            }
        }
    }

    // Second night triggers another night horde
    input.is_night = true;
    for _ in 0..12 {
        let out = director.update(&input);
        for event in out.events {
            if let DirectorEvent::CampAttack { .. } = event {
                camp_attacks += 1;
            }
        }
    }
    assert_eq!(camp_attacks, 2, "Second night should trigger another CampAttack");
}

#[test]
fn director_city_wave_has_city_kind_and_size() {
    let cfg = DirectorConfig::load_default().expect("data/director.json must load");
    let mut director = Director::new(cfg);

    let mut input = DirectorInput::new(1.0);
    input.in_city = true;

    // Advance through calm
    for _ in 0..16 {
        director.update(&input);
    }
    assert_eq!(director.state(), DirectorState::Building);

    // Advance through building to wave
    let mut city_wave_found = false;
    for _ in 0..14 {
        let out = director.update(&input);
        for event in out.events {
            if let DirectorEvent::WaveStart { size, kind } = event {
                assert_eq!(kind, WaveKind::City);
                assert_eq!(size, 14);
                city_wave_found = true;
            }
        }
    }
    assert!(city_wave_found);
}

#[test]
fn director_save_state_roundtrip_and_default() {
    let empty_save: DirectorSaveState = serde_json::from_str("{}").expect("empty json loads as default");
    assert_eq!(empty_save, DirectorSaveState::default());

    let cfg = DirectorConfig::load_default().expect("data/director.json must load");
    let mut director = Director::new(cfg);
    let mut input = DirectorInput::new(45.0);
    input.is_night = true;
    director.update(&input);

    let save = director.to_save_state();
    let json = serde_json::to_string(&save).expect("serialize director save");
    let loaded: DirectorSaveState = serde_json::from_str(&json).expect("deserialize director save");
    assert_eq!(save, loaded);

    let mut restored = Director::new(director.config().clone());
    restored.from_save_state(&loaded);
    assert_eq!(restored.state(), director.state());
    assert_eq!(restored.to_save_state(), save);
}

#[test]
fn director_config_custom_json_and_load_file() {
    let json_text = r#"{
        "states": {
            "calm": { "durationDay": 10.0, "durationNight": 5.0, "durationCity": 5.0, "spawnRateMultiplier": 0.5, "maxActiveMultiplier": 0.5, "specialChance": 0.0 },
            "building": { "durationDay": 10.0, "durationNight": 5.0, "durationCity": 5.0, "spawnRateMultiplier": 1.0, "maxActiveMultiplier": 1.0, "specialChance": 0.1 },
            "wave": { "durationDay": 10.0, "durationNight": 5.0, "durationCity": 5.0, "spawnRateMultiplier": 2.0, "maxActiveMultiplier": 2.0, "specialChance": 0.2 },
            "relax": { "durationDay": 10.0, "durationNight": 5.0, "durationCity": 5.0, "spawnRateMultiplier": 0.2, "maxActiveMultiplier": 0.2, "specialChance": 0.0 }
        },
        "rules": {
            "lowHealthThreshold": 0.3, "nightHordeDelay": 15.0, "nightHordeSize": 10,
            "waveSizeDay": 5, "waveSizeNight": 8, "waveSizeCity": 12, "waveSizeNoise": 6
        },
        "specialZombies": {
            "brute": { "structureDamage": 50.0 },
            "screamer": { "callRadius": 60.0 },
            "crawler": { "hitboxScale": 0.5 }
        },
        "horde": {
            "speed": 2.0, "spacing": 1.5, "maxDistance": 10.0, "mergeDistance": 5.0,
            "cohesionWeight": 1.0, "separationWeight": 1.0, "alignmentWeight": 1.0
        }
    }"#;
    let custom = DirectorConfig::from_json(json_text).expect("custom json parse");
    assert_eq!(custom.rules.low_health_threshold, 0.3);
    assert_eq!(custom.rules.night_horde_size, 10);
    assert_eq!(custom.horde.speed, 2.0);

    let from_file = DirectorConfig::load(&paths::repo_path("data/director.json")).expect("load from path");
    assert_eq!(from_file.rules.night_horde_size, 20);
    assert_eq!(from_file.special_zombies.get("brute").unwrap().structure_damage, Some(50.0));
    assert_eq!(from_file.special_zombies.get("screamer").unwrap().call_radius, Some(60.0));
    assert_eq!(from_file.special_zombies.get("crawler").unwrap().hitbox_scale, Some(0.5));
}
