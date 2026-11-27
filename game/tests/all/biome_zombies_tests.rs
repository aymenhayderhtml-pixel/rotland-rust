use game::spawner::{Spawner, SPECIAL_KINDS};
use game::spawner_config::SpawnConfig;
use game::zombie::defs::ZombieDefs;
use game::zombie::specials::*;
use glam::Vec3;

// new: DECISIONS 2026-11-25
const DT: f32 = 0.016;
// new: DECISIONS 2026-11-25
const FLOAT_EPS: f32 = 1e-4;
// new: DECISIONS 2026-11-25
const SAMPLE_COUNT: usize = 1000;

fn load_special_cfg() -> SpecialConfig {
    SpecialConfig::from_repo().expect("data/special_zombies.json should load")
}

#[test]
fn biome_zombie_data_loads() {
    let defs = ZombieDefs::load_default().expect("data/zombies.json should load");
    assert!(defs.get("frozen").is_some(), "frozen missing from zombies.json");
    assert!(defs.get("husk").is_some(), "husk missing from zombies.json");
    assert!(defs.get("bloater").is_some(), "bloater missing from zombies.json");

    let specials = load_special_cfg();
    // data: special_zombies.json frozen
    assert!((specials.frozen.chill_on_bite - 8.0).abs() <= FLOAT_EPS);
    assert!((specials.frozen.speed_mult - 0.75).abs() <= FLOAT_EPS);
    assert!((specials.frozen.health_mult - 1.6).abs() <= FLOAT_EPS);
    // data: special_zombies.json husk
    assert!((specials.husk.speed_mult - 1.3).abs() <= FLOAT_EPS);
    assert!((specials.husk.health_mult - 0.7).abs() <= FLOAT_EPS);
    // data: special_zombies.json bloater
    assert!((specials.bloater.cloud_radius - 4.0).abs() <= FLOAT_EPS);
    assert!((specials.bloater.cloud_seconds - 6.0).abs() <= FLOAT_EPS);
    assert!((specials.bloater.player_dps - 4.0).abs() <= FLOAT_EPS);
    assert!((specials.bloater.zombie_dps - 2.0).abs() <= FLOAT_EPS);
    assert!(specials.bloater.burst_on_fire);

    let spawn_cfg = SpawnConfig::load_default().expect("data/spawning.json should load");
    assert!(spawn_cfg.biome_specials.contains_key("snow"));
    assert!(spawn_cfg.biome_specials.contains_key("mountains"));
    assert!(spawn_cfg.biome_specials.contains_key("desert"));
    assert!(spawn_cfg.biome_specials.contains_key("swamp"));
    assert!(!spawn_cfg.biome_specials.contains_key("plains"));
    assert!(!spawn_cfg.biome_specials.contains_key("forest"));

    for kind in ["frozen", "husk", "bloater"] {
        assert!(SPECIAL_KINDS.contains(&kind), "SPECIAL_KINDS missing {kind}");
        assert!(SpecialKind::from_id(kind).is_some(), "SpecialKind::from_id missing {kind}");
    }
}

#[test]
fn snow_spawns_frozen_about_35_percent_and_plains_zero() {
    let cfg = SpawnConfig::load_default().expect("data/spawning.json should load");
    // new: DECISIONS 2026-11-25
    let mut snow_spawner = Spawner::new(cfg.clone(), 42);
    let mut snow_frozen = 0usize;
    for _ in 0..SAMPLE_COUNT {
        let kind = snow_spawner.pick_spawn_type_id(Some("snow"), false, false, false, 0.0, None);
        if kind == "frozen" {
            snow_frozen += 1;
        }
    }
    let snow_ratio = snow_frozen as f32 / SAMPLE_COUNT as f32;
    // data: spawning.json biomeSpecials.snow
    let expected = 0.35_f32;
    // new: DECISIONS 2026-11-25
    let tolerance = 0.05_f32;
    assert!(
        (snow_ratio - expected).abs() <= tolerance,
        "snow frozen share {snow_ratio:.3} outside {expected} +/- {tolerance}"
    );

    // new: DECISIONS 2026-11-25
    let mut plains_spawner = Spawner::new(cfg, 42);
    let mut plains_frozen = 0usize;
    for _ in 0..SAMPLE_COUNT {
        let kind = plains_spawner.pick_spawn_type_id(Some("plains"), true, false, false, 0.25, None);
        if kind == "frozen" {
            plains_frozen += 1;
        }
    }
    assert_eq!(plains_frozen, 0, "plains must not spawn frozen zombies");
}

#[test]
fn bloater_death_emits_toxic_burst_once() {
    let cfg = load_special_cfg();
    let mut bloater = SpecialState::new(SpecialKind::Bloater);
    // new: DECISIONS 2026-11-25
    let death_pos = Vec3::new(10.0, 0.0, -5.0);

    let burst = bloater
        .on_death(&cfg, death_pos)
        .expect("bloater death should emit a ToxicBurst");
    assert_eq!(burst.pos, death_pos);
    assert!((burst.radius - cfg.bloater.cloud_radius).abs() <= FLOAT_EPS);
    assert!((burst.seconds - cfg.bloater.cloud_seconds).abs() <= FLOAT_EPS);
    assert!((burst.player_dps - cfg.bloater.player_dps).abs() <= FLOAT_EPS);
    assert!((burst.zombie_dps - cfg.bloater.zombie_dps).abs() <= FLOAT_EPS);

    assert!(
        bloater.on_death(&cfg, death_pos).is_none(),
        "bloater should not emit a second burst"
    );

    // Fire hit also bursts a bloater when burstOnFire is true.
    let mut burned = SpecialState::new(SpecialKind::Bloater);
    let fire_event = burned.on_hit_at(&cfg, death_pos, false, true);
    assert!(matches!(fire_event, Some(SpecialEvent::ToxicBurst(_))));
    assert!(burned.on_death(&cfg, death_pos).is_none());
}

#[test]
fn toxic_cloud_damages_inside_radius_and_expires_after_six_seconds() {
    let cfg = load_special_cfg();
    let mut clouds = ToxicClouds::new();
    let center = Vec3::new(4.0, 0.0, 8.0);
    clouds.spawn(ToxicBurst {
        pos: center,
        radius: cfg.bloater.cloud_radius,
        seconds: cfg.bloater.cloud_seconds,
        player_dps: cfg.bloater.player_dps,
        zombie_dps: cfg.bloater.zombie_dps,
    });
    assert_eq!(clouds.len(), 1);

    // Inside radius: full DPS.
    let inside = center + Vec3::new(cfg.bloater.cloud_radius * 0.5, 0.0, 0.0);
    let (p_dps, z_dps) = clouds.damage_at(inside);
    assert!((p_dps - cfg.bloater.player_dps).abs() <= FLOAT_EPS);
    assert!((z_dps - cfg.bloater.zombie_dps).abs() <= FLOAT_EPS);

    // Outside radius: zero DPS.
    // new: DECISIONS 2026-11-25
    let outside = center + Vec3::new(cfg.bloater.cloud_radius + 0.5, 0.0, 0.0);
    assert_eq!(clouds.damage_at(outside), (0.0, 0.0));

    // Still active just before 6 s, expired at 6 s.
    // new: DECISIONS 2026-11-25
    clouds.update(5.9);
    assert_eq!(clouds.len(), 1);
    assert!(clouds.damage_at(center).0 > 0.0);

    // new: DECISIONS 2026-11-25
    clouds.update(0.1);
    assert!(clouds.is_empty(), "cloud should expire after 6 seconds");
    assert_eq!(clouds.damage_at(center), (0.0, 0.0));
}

#[test]
fn frozen_bite_carries_chill_eight() {
    let cfg = load_special_cfg();
    let frozen = SpecialState::new(SpecialKind::Frozen);
    let effect = frozen.on_attack_landed(&cfg);
    // data: special_zombies.json frozen.chillOnBite
    assert_eq!(effect.chill, Some(8.0));

    let husk = SpecialState::new(SpecialKind::Husk);
    assert_eq!(husk.on_attack_landed(&cfg).chill, None);
}

#[test]
fn husk_speed_exceeds_walker_and_lunges_with_husk_numbers() {
    let defs = ZombieDefs::load_default().expect("data/zombies.json should load");
    let walker = defs.get("walker").expect("walker def");
    let husk = defs.get("husk").expect("husk def");
    assert!(
        husk.chase_speed > walker.chase_speed,
        "husk chase_speed {} should exceed walker {}",
        husk.chase_speed,
        walker.chase_speed
    );
    assert!(
        husk.wander_speed > walker.wander_speed,
        "husk wander_speed {} should exceed walker {}",
        husk.wander_speed,
        walker.wander_speed
    );

    let cfg = load_special_cfg();
    let mut state = SpecialState::new(SpecialKind::Husk);
    let own = Vec3::ZERO;
    let far = Vec3::new(cfg.husk.lurk_range + 5.0, 0.0, 0.0);
    let d_far = state.tick(&cfg, own, far, true, false, DT, 0.0);
    assert_eq!(d_far.pose, SpecialPose::Normal);
    assert_eq!(d_far.velocity, None);

    // Within husk lungeTriggerRange: lunges and deals husk.lunge_damage.
    let mut pos = own;
    let player = Vec3::new(cfg.husk.lunge_trigger_range * 0.5, 0.0, 0.0);
    let mut lunged = false;
    let mut hit_damage = None;
    for _ in 0..150 {
        let d = state.tick(&cfg, pos, player, true, false, DT, 0.0);
        if d.pose == SpecialPose::Lunge {
            lunged = true;
        }
        if let Some(hit) = d.damage {
            hit_damage = Some(hit.amount);
        }
        if let Some(v) = d.velocity {
            pos += v * DT;
        }
    }
    assert!(lunged, "husk should lunge within lungeTriggerRange");
    let dmg = hit_damage.expect("husk lunge should connect");
    assert!((dmg - cfg.husk.lunge_damage).abs() <= FLOAT_EPS);
}
