use glam::Vec3;
use physics::PhysicsWorld;
use world::MetroDistrict;

use crate::metro_spawn::{MetroZone, metro_profile};
use crate::spawner::{SpawnContext, Spawner};
use crate::spawner_config::SpawnConfig;

const SAMPLE_PICKS: usize = 2000; // new: DECISIONS 2026-11-27
const FIXED_SEED: u64 = 42; // new: DECISIONS 2026-11-27

#[test]
fn spawning_json_has_three_metro_zones() {
    let cfg = SpawnConfig::load_default().expect("data/spawning.json should load");
    let metro = cfg.metro.as_ref().expect("metro block should exist in spawning.json");

    let edge = metro.zone(MetroZone::from(MetroDistrict::Edge));
    let midtown = metro.zone(MetroZone::from(MetroDistrict::Midtown));
    let core = metro.zone(MetroZone::from(MetroDistrict::Core));

    assert!(edge.density < midtown.density && midtown.density < core.density);
    assert!(!edge.day.is_empty() && !edge.night.is_empty());
    assert!(!midtown.day.is_empty() && !midtown.night.is_empty());
    assert!(!core.day.is_empty() && !core.night.is_empty());
}

#[test]
fn metro_cap_rises_toward_core_day_and_night() {
    let cfg = SpawnConfig::load_default().expect("data/spawning.json should load");
    let metro = cfg.metro.as_ref().expect("metro block should exist");
    let spawner = Spawner::new(cfg.clone(), FIXED_SEED);
    let physics = PhysicsWorld::new();

    for is_night in [false, true] {
        let mut caps = [0usize; 3];
        for (idx, zone) in [MetroZone::Edge, MetroZone::Midtown, MetroZone::Core]
            .into_iter()
            .enumerate()
        {
            let mut ctx = SpawnContext::new(Vec3::ZERO, &physics, &[], &[]).with_night(is_night);
            ctx.metro = Some(metro_profile(metro, zone, is_night));
            caps[idx] = spawner.calculate_cap(&ctx);
        }
        assert!(
            caps[2] > caps[1] && caps[1] > caps[0],
            "expected core > midtown > edge (night={is_night}), got {caps:?}"
        );
    }
}

#[test]
fn runner_share_higher_at_night_and_core_night_dominant() {
    let cfg = SpawnConfig::load_default().expect("data/spawning.json should load");
    let metro = cfg.metro.as_ref().expect("metro block should exist");

    for zone in [MetroZone::Edge, MetroZone::Midtown, MetroZone::Core] {
        let day_prof = metro_profile(metro, zone, false);
        let night_prof = metro_profile(metro, zone, true);

        let mut day_spawner = Spawner::new(cfg.clone(), FIXED_SEED);
        let mut day_runners = 0usize;
        for _ in 0..SAMPLE_PICKS {
            if day_spawner.pick_spawn_type_id(None, false, false, false, 0.0, Some(&day_prof))
                == "runner"
            {
                day_runners += 1;
            }
        }

        let mut night_spawner = Spawner::new(cfg.clone(), FIXED_SEED);
        let mut night_runners = 0usize;
        let mut night_walkers = 0usize;
        for _ in 0..SAMPLE_PICKS {
            match night_spawner.pick_spawn_type_id(None, true, false, false, 0.0, Some(&night_prof))
            {
                "runner" => night_runners += 1,
                "walker" => night_walkers += 1,
                _ => {}
            }
        }

        assert!(
            night_runners > day_runners,
            "expected more runners at night in {zone:?}: day={day_runners}, night={night_runners}"
        );
        if zone == MetroZone::Core {
            let non_special_runner_share =
                night_runners as f32 / (night_runners + night_walkers) as f32;
            assert!(
                non_special_runner_share >= 0.75, // data: spawning.json metro.zones.core.night
                "core night non-special runner share {non_special_runner_share:.3} < 0.75"
            );

            // Also verify raw pick share >= 0.75 when special bonus is 0.
            let mut no_bonus = night_prof.clone();
            no_bonus.special_chance_bonus = 0.0; // new: DECISIONS 2026-11-27
            let mut pure_spawner = Spawner::new(cfg.clone(), FIXED_SEED);
            let mut pure_runners = 0usize;
            for _ in 0..SAMPLE_PICKS {
                if pure_spawner.pick_spawn_type_id(None, true, false, false, 0.0, Some(&no_bonus))
                    == "runner"
                {
                    pure_runners += 1;
                }
            }
            let pure_share = pure_runners as f32 / SAMPLE_PICKS as f32;
            assert!(
                pure_share >= 0.75, // data: spawning.json metro.zones.core.night
                "core night raw runner share {pure_share:.3} < 0.75"
            );
        }
    }
}

#[test]
fn non_metro_cap_and_picks_unchanged() {
    let cfg = SpawnConfig::load_default().expect("data/spawning.json should load");
    let physics = PhysicsWorld::new();

    // Biome case (swamp, day)
    let mut biome_spawner = Spawner::new(cfg.clone(), FIXED_SEED);
    let biome_ctx = SpawnContext::new(Vec3::ZERO, &physics, &[], &[]).with_biome("swamp");
    // data: spawning.json maxActive(20) * biomeDensity.swamp(1.3) = 26
    assert_eq!(biome_spawner.calculate_cap(&biome_ctx), 26);
    let first_biome_pick = biome_spawner
        .pick_spawn_type_id(Some("swamp"), false, false, false, 0.0, None)
        .to_string();
    assert!(!first_biome_pick.is_empty());

    // Ashford city case (in_city, in_city_core, night)
    let mut city_spawner = Spawner::new(cfg.clone(), FIXED_SEED);
    let city_ctx = SpawnContext::new(Vec3::ZERO, &physics, &[], &[])
        .with_night(true)
        .with_in_city(true)
        .with_in_city_core(true);
    // data: spawning.json city.maxActive(32) * nightMaxActiveMultiplier(1.6) = 51
    assert_eq!(city_spawner.calculate_cap(&city_ctx), 51);
    let first_city_pick = city_spawner
        .pick_spawn_type_id(None, true, true, true, 0.0, None)
        .to_string();
    assert!(!first_city_pick.is_empty());
}

#[test]
fn metro_never_picks_biome_specials() {
    let cfg = SpawnConfig::load_default().expect("data/spawning.json should load");
    let metro = cfg.metro.as_ref().expect("metro block should exist");
    let prof = metro_profile(metro, MetroZone::Core, false);
    let mut spawner = Spawner::new(cfg, FIXED_SEED);

    for _ in 0..SAMPLE_PICKS {
        let kind = spawner.pick_spawn_type_id(Some("swamp"), false, false, false, 0.0, Some(&prof));
        assert_ne!(kind, "bloater", "metro must not spawn biome special bloater");
    }
}
