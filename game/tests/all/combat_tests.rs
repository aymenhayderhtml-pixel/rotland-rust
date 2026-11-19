use game::{DamageInfo, Health, HealthEvent, IgnoredReason, WeaponKind, WeaponRegistry};

fn load_weapons() -> (serde_json::Value, WeaponRegistry) {
    let path = paths::repo_path("data/weapons.json");
    let text = std::fs::read_to_string(&path).expect("data/weapons.json");
    let json: serde_json::Value = serde_json::from_str(&text).expect("weapons.json is valid JSON");
    let registry = WeaponRegistry::load(&path).expect("registry loads weapons.json");
    (json, registry)
}


/// f32 fields compared with a tolerance: 0.15 is not exact in f32, so exact equality would fail.
fn assert_close(actual: f32, expected: f64) {
    assert!((actual as f64 - expected).abs() < 1e-6, "{actual} != {expected}");
}

#[test]
fn damage_reduces_health() {
    let mut health = Health::new(100.0);
    let result = health.take_damage(DamageInfo::new(30.0));
    assert_eq!(result.dealt, 30.0);
    assert!(!result.died_now);
    assert_eq!(result.ignored, None);
    assert_eq!(health.current(), 70.0);
    assert!(!health.is_dead());
}

#[test]
fn damage_at_zero_kills_once() {
    let mut health = Health::new(50.0);
    let killing = health.take_damage(DamageInfo::new(50.0));
    assert!(killing.died_now);
    assert_eq!(killing.dealt, 50.0);
    assert!(health.is_dead());
    assert_eq!(health.current(), 0.0);

    // A second hit on the corpse reports nothing: not dealt, not a new death.
    let again = health.take_damage(DamageInfo::new(10.0));
    assert_eq!(again.dealt, 0.0);
    assert!(!again.died_now);
    assert_eq!(again.ignored, Some(IgnoredReason::Dead));

    let died_events = health.drain_events().iter().filter(|e| **e == HealthEvent::Died).count();
    println!("died events counted: {died_events}");
    assert_eq!(died_events, 1);
}

#[test]
fn damage_is_capped_at_remaining_health() {
    let mut health = Health::new(20.0);
    let result = health.take_damage(DamageInfo::new(35.0));
    assert_eq!(result.dealt, 20.0);
    assert!(result.died_now);
}

#[test]
fn invulnerability_ignores_damage_then_wears_off() {
    let mut health = Health::new(100.0).with_invulnerability(0.5);
    assert_eq!(health.take_damage(DamageInfo::new(10.0)).dealt, 10.0);

    let blocked = health.take_damage(DamageInfo::new(10.0));
    assert_eq!(blocked.ignored, Some(IgnoredReason::Invulnerable));
    assert_eq!(health.current(), 90.0);

    health.tick(0.3);
    assert_eq!(health.take_damage(DamageInfo::new(10.0)).ignored, Some(IgnoredReason::Invulnerable));

    health.tick(0.3); // 0.6 s since the hit, past the 0.5 s window
    let after = health.take_damage(DamageInfo::new(10.0));
    assert_eq!(after.ignored, None);
    assert_eq!(health.current(), 80.0);
}

#[test]
fn fresh_health_is_not_invulnerable() {
    let mut health = Health::new(100.0).with_invulnerability(5.0);
    assert_eq!(health.take_damage(DamageInfo::new(10.0)).dealt, 10.0);
}

#[test]
fn heal_caps_at_max_and_not_on_the_dead() {
    let mut health = Health::new(100.0);
    health.take_damage(DamageInfo::new(30.0));
    health.heal(50.0);
    assert_eq!(health.current(), 100.0);

    health.take_damage(DamageInfo::new(100.0));
    health.heal(10.0);
    assert_eq!(health.current(), 0.0);
    assert!(health.is_dead());
}

#[test]
fn revive_restores_health_and_clears_invulnerability() {
    let mut health = Health::new(100.0).with_invulnerability(5.0);
    health.take_damage(DamageInfo::new(100.0));
    assert!(health.is_dead());

    health.revive(None);
    assert!(!health.is_dead());
    assert_eq!(health.current(), 100.0);
    // Revive clears the hit timer, so the next hit lands at once.
    assert_eq!(health.take_damage(DamageInfo::new(10.0)).dealt, 10.0);
}

#[test]
fn kill_ignores_invulnerability() {
    let mut health = Health::new(100.0).with_invulnerability(5.0);
    health.take_damage(DamageInfo::new(1.0));
    health.kill();
    assert!(health.is_dead());
    assert_eq!(health.current(), 0.0);
}

#[test]
fn events_follow_the_hit() {
    let mut health = Health::new(10.0);
    health.take_damage(DamageInfo::new(4.0));
    let events = health.drain_events();
    assert_eq!(
        events,
        vec![
            HealthEvent::Damaged { amount: 4.0, source: None },
            HealthEvent::Changed { current: 6.0, max: 10.0 },
        ]
    );
    assert!(health.drain_events().is_empty());
}

#[test]
fn registry_loads_every_weapon_in_the_json() {
    let (json, registry) = load_weapons();
    let json_count = json.as_object().expect("weapons.json is an object").len();
    println!("weapons in json: {json_count}, loaded: {}", registry.len());
    assert!(json_count > 0);
    assert_eq!(registry.len(), json_count);
    for id in json.as_object().unwrap().keys() {
        let def = registry.get(id).unwrap_or_else(|| panic!("{id} not loaded"));
        assert_eq!(&def.id, id);
    }
    assert!(registry.get("no_such_weapon").is_none());
}

#[test]
fn bat_numbers_equal_the_json() {
    let (json, registry) = load_weapons();
    let raw = &json["bat"];
    let bat = registry.get("bat").expect("bat");

    assert_eq!(bat.name, raw["name"].as_str().unwrap());
    assert_eq!(bat.kind, WeaponKind::Melee);
    assert!(!bat.is_gun());
    assert_close(bat.damage, raw["damage"].as_f64().unwrap());
    assert_close(bat.range, raw["range"].as_f64().unwrap());
    assert_close(bat.arc_deg, raw["arcDeg"].as_f64().unwrap());
    assert_close(bat.windup, raw["windup"].as_f64().unwrap());
    assert_close(bat.cooldown, raw["cooldown"].as_f64().unwrap());
    assert_close(bat.stamina_cost, raw["staminaCost"].as_f64().unwrap());
    assert_close(bat.knockback, raw["knockback"].as_f64().unwrap());
    assert_close(bat.stagger, raw["stagger"].as_f64().unwrap());
    assert_eq!(bat.max_targets as u64, raw["maxTargets"].as_u64().unwrap());
    assert_close(bat.noise_radius, raw["noiseRadius"].as_f64().unwrap());
    assert_eq!(bat.anim.attack.as_deref(), raw["anim"]["attack"].as_str());
    assert_close(bat.anim.impact, raw["anim"]["impact"].as_f64().unwrap());
    assert_eq!(bat.model, raw["model"].as_str().unwrap());
    assert_eq!(bat.color, raw["color"].as_str().unwrap());
    let size: Vec<f64> = raw["size"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect();
    assert_eq!(size.len(), bat.size.len());
    for (actual, expected) in bat.size.iter().zip(&size) {
        assert_close(*actual, *expected);
    }
    assert_eq!(bat.damage, 25.0);
    assert_eq!(bat.max_targets, 2);
}

#[test]
fn guns_read_their_fields() {
    let (_json, registry) = load_weapons();
    let pistol = registry.get("pistol").expect("pistol");
    assert!(pistol.is_gun());
    assert_eq!(pistol.magazine_size, 12);
    assert_eq!(pistol.ammo_item, "pistol_ammo");
    assert_eq!(pistol.headshot_multiplier, 2.0);
    assert_eq!(pistol.pellets, 1);
    assert_eq!(pistol.scope, None);
    assert_eq!(pistol.mag_swap, None);

    assert_eq!(registry.get("shotgun").expect("shotgun").pellets, 8);
    let rifle = registry.get("rifle").expect("rifle");
    let scope = rifle.scope.as_ref().expect("rifle has a scope");
    assert_eq!(scope.fov, 25.0);
    assert_eq!(scope.sensitivity, 0.4);
    assert_eq!(registry.get("ak47").expect("ak47").mag_swap, Some([0.7, 1.35]));
    assert!(registry.get("ak47").unwrap().auto);
}
