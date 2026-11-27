use glam::Vec3;
use physics::PhysicsWorld;
use world::MetroDistrict;

use crate::metro_spawn::{MetroZone, PlaceDanger, metro_profile, metro_profile_near_place};
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

#[test]
fn place_danger_block_loads_from_real_file() {
    let cfg = SpawnConfig::load_default().expect("data/spawning.json should load");
    let metro = cfg.metro.as_ref().expect("metro block should exist");
    let danger = metro
        .place_danger
        .as_ref()
        .expect("data/spawning.json metro.placeDanger must load");

    // data: spawning.json metro.placeDanger
    assert_eq!(danger.radius, 90.0);
    assert_eq!(danger.density_mult, 1.6);
    assert_eq!(danger.special_bonus, 0.08);
    assert!(danger.brute_at_army_line);
    assert!(metro.brute_at_army_line());
}

#[test]
fn near_place_is_denser_for_every_zone() {
    let cfg = SpawnConfig::load_default().expect("data/spawning.json should load");
    let metro = cfg.metro.as_ref().expect("metro block should exist");

    for zone in [MetroZone::Edge, MetroZone::Midtown, MetroZone::Core] {
        for is_night in [false, true] {
            let far = metro_profile_near_place(metro, zone, is_night, false);
            let near = metro_profile_near_place(metro, zone, is_night, true);

            assert!(
                near.density > far.density,
                "{zone:?} night={is_night}: near density {} must beat far {}",
                near.density,
                far.density
            );
            assert!(
                near.max_active > far.max_active,
                "{zone:?} night={is_night}: near cap {} must beat far {}",
                near.max_active,
                far.max_active
            );
            assert!(
                near.special_chance_bonus > far.special_chance_bonus,
                "{zone:?}: near special bonus {} must beat far {}",
                near.special_chance_bonus,
                far.special_chance_bonus
            );
            assert_eq!(near.types, far.types, "{zone:?}: types must not change");
        }
    }
}

#[test]
fn near_place_is_a_no_op_without_a_danger_block() {
    let cfg = SpawnConfig::load_default().expect("data/spawning.json should load");
    let loaded = cfg.metro.as_ref().expect("metro block should exist");

    // Older spawning.json without `metro.placeDanger`: near must still give the plain R41 numbers.
    let mut bare = loaded.clone();
    bare.place_danger = None;
    for zone in [MetroZone::Edge, MetroZone::Midtown, MetroZone::Core] {
        let far = metro_profile_near_place(&bare, zone, false, false);
        let near = metro_profile_near_place(&bare, zone, false, true);
        assert_eq!(near, far, "{zone:?}: no danger block means no bonus");
        assert!(!bare.brute_at_army_line());
    }

    // And a hand-written block drives the numbers, not just the shipped file.
    let mut tweaked = loaded.clone();
    tweaked.place_danger = Some(PlaceDanger {
        why: None,
        radius: 50.0,
        density_mult: 2.0,
        special_bonus: 0.2,
        brute_at_army_line: false,
    });
    let far = metro_profile_near_place(&tweaked, MetroZone::Edge, false, false);
    let near = metro_profile_near_place(&tweaked, MetroZone::Edge, false, true);
    assert!((near.density - far.density * 2.0).abs() < 1e-5);
    assert!((near.special_chance_bonus - (far.special_chance_bonus + 0.2)).abs() < 1e-5);
    assert!(!tweaked.brute_at_army_line());
}

#[test]
fn not_near_a_place_is_exactly_the_r41_numbers() {
    let cfg = SpawnConfig::load_default().expect("data/spawning.json should load");
    let metro = cfg.metro.as_ref().expect("metro block should exist");

    for zone in [MetroZone::Edge, MetroZone::Midtown, MetroZone::Core] {
        for is_night in [false, true] {
            let r41 = metro_profile(metro, zone, is_night);
            let far = metro_profile_near_place(metro, zone, is_night, false);
            // data: spawning.json metro.zones (R41 values, unchanged by the place bonus)
            assert_eq!(r41, far, "{zone:?} night={is_night}: far profile must equal R41");
        }
    }

    // Spot-check the raw R41 numbers so a data edit cannot slip through silently.
    let edge = metro_profile(metro, MetroZone::Edge, false);
    assert_eq!(edge.density, 2.0); // data: spawning.json metro.zones.edge.density
    assert_eq!(edge.max_active, 30); // data: spawning.json metro.zones.edge.maxActive
    assert_eq!(edge.special_chance_bonus, 0.0); // data: spawning.json metro.zones.edge.specialBonus

    let core_night = metro_profile(metro, MetroZone::Core, true);
    assert_eq!(core_night.density, 3.2); // data: spawning.json metro.zones.core.density
    // data: spawning.json metro.zones.core.maxActive(46) * metro.nightMaxActiveMultiplier(1.4) = 64
    assert_eq!(core_night.max_active, 64);
}

#[test]
fn near_place_uses_the_configured_multipliers_and_cap_still_applies() {
    let cfg = SpawnConfig::load_default().expect("data/spawning.json should load");
    let metro = cfg.metro.as_ref().expect("metro block should exist");

    let far = metro_profile_near_place(metro, MetroZone::Edge, false, false);
    let near = metro_profile_near_place(metro, MetroZone::Edge, false, true);
    // data: spawning.json metro.placeDanger densityMult(1.6), radius(90), specialBonus(0.08)
    assert!((near.density - far.density * 1.6).abs() < 1e-5);
    assert!((near.special_chance_bonus - (far.special_chance_bonus + 0.08)).abs() < 1e-5);

    // The raised metro cap must still be cut down by the spawner's `hardCap`.
    let spawner = Spawner::new(cfg.clone(), FIXED_SEED);
    let physics = PhysicsWorld::new();
    let mut far_ctx = SpawnContext::new(Vec3::ZERO, &physics, &[], &[]);
    far_ctx.metro = Some(far);
    let far_cap = spawner.calculate_cap(&far_ctx);

    let mut near_ctx = SpawnContext::new(Vec3::ZERO, &physics, &[], &[]);
    near_ctx.metro = Some(near);
    let near_cap = spawner.calculate_cap(&near_ctx);

    assert!(near_cap > far_cap, "near cap {near_cap} must beat far cap {far_cap}");
    assert!(
        near_cap <= cfg.hard_cap,
        "near cap {near_cap} must stay under hardCap {}",
        cfg.hard_cap
    );
}
