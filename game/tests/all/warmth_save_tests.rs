use game_player::player::Stamina;
use game_player::survival::temperature::{BodyTemp, TempState, TemperatureData};

#[test]
fn body_temp_save_round_trip_and_old_save_default() {
    let data = TemperatureData::from_repo().expect("failed to load data/temperature.json");
    let mut body = BodyTemp::new(&data);
    body.set(22.0);
    let saved = body.to_save();

    let mut loaded = BodyTemp::new(&data);
    loaded.from_save(&saved);
    assert!((loaded.value() - 22.0).abs() < 1e-4);

    // Old save without a value keeps comfy default (50.0).
    let mut from_old = BodyTemp::new(&data);
    from_old.from_save(&serde_json::json!({}));
    assert!((from_old.value() - 50.0).abs() < 1e-4);
}

#[test]
fn warmth_stamina_regen_multiplier_halves_regen() {
    let mut normal = Stamina {
        current: 20.0,
        time_since_use: 10.0,
        ..Default::default()
    };
    let mut halved = Stamina {
        current: 20.0,
        time_since_use: 10.0,
        warmth_regen_multiplier: 0.5,
        ..Default::default()
    };

    normal.update(1.0);
    halved.update(1.0);

    let gained_normal = normal.current - 20.0;
    let gained_halved = halved.current - 20.0;
    assert!((gained_halved - gained_normal * 0.5).abs() < 1e-4);
}

#[test]
fn freezing_body_temp_halves_stamina_regen() {
    let data = TemperatureData::from_repo().expect("failed to load data/temperature.json");
    let mut body = BodyTemp::new(&data);
    body.set(10.0);
    assert_eq!(body.state(), TempState::Freezing);
    assert!((body.stamina_regen_mult() - 0.5).abs() < 1e-4);

    let mut stamina = Stamina {
        current: 0.0,
        time_since_use: 10.0,
        warmth_regen_multiplier: body.stamina_regen_mult(),
        ..Default::default()
    };
    stamina.update(1.0);
    assert!((stamina.current - stamina.regen_per_second * 0.5).abs() < 1e-4);
}
