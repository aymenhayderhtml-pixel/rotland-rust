use glam::Vec3;

use game::projectiles::{bow_shot, spear_throw, Hit, Projectile, ProjectileEvent, ProjectileKind, ProjectileSim};
use game::weapons::{WeaponDef, WeaponKind};

fn test_bow_def() -> WeaponDef {
    WeaponDef {
        id: "bow".to_string(),
        name: "Hunting Bow".to_string(),
        kind: WeaponKind::Bow,
        damage: 50.0,
        headshot_multiplier: 2.5,
        draw_time: 1.0,
        arrow_speed: 45.0,
        gravity: 9.8,
        pickup_chance: 0.6,
        max_flight_seconds: 10.0,
        ..WeaponDef::default()
    }
}

fn test_spear_def() -> WeaponDef {
    WeaponDef {
        id: "spear".to_string(),
        name: "Spear".to_string(),
        kind: WeaponKind::Melee,
        damage: 30.0,
        throwable: true,
        throw_speed: 20.0,
        throw_damage: 45.0,
        gravity: 9.8,
        pickup_chance: 1.0,
        max_flight_seconds: 10.0,
        ..WeaponDef::default()
    }
}

#[test]
fn projectile_bow_arc_drops_by_gravity() {
    let def = test_bow_def();
    let shot = bow_shot(1.0, Vec3::new(0.0, 10.0, 0.0), Vec3::new(1.0, 0.0, 0.0), &def);
    assert_eq!(shot.kind, ProjectileKind::Arrow);
    assert_eq!(shot.pos, Vec3::new(0.0, 10.0, 0.0));
    assert_eq!(shot.vel, Vec3::new(45.0, 0.0, 0.0));

    let mut sim = ProjectileSim::new(42);
    sim.spawn(shot);
    assert_eq!(sim.len(), 1);

    // Step several times without any collisions
    let dt = 0.1;
    let mut last_y = 10.0;
    for _ in 0..5 {
        let events = sim.step(dt, |_from, _to| None);
        assert!(events.is_empty());
        let current = &sim.projectiles()[0];
        assert!(current.pos.y < last_y, "y should strictly decrease: {} < {}", current.pos.y, last_y);
        assert!(current.vel.y < 0.0, "vertical velocity should become negative due to gravity");
        last_y = current.pos.y;
    }

    // Verify arc drop matches gravitational acceleration:
    // After 0.5 s, drop should be approximately 0.5 * g * t^2 = 0.5 * 9.8 * 0.25 = 1.225 m
    let final_drop = 10.0 - last_y;
    assert!(final_drop > 1.0 && final_drop < 1.5, "drop was {} m", final_drop);
}

#[test]
fn projectile_longer_draw_goes_further() {
    let def = test_bow_def();
    let shot_half = bow_shot(0.3, Vec3::new(0.0, 10.0, 0.0), Vec3::new(1.0, 0.0, 0.0), &def);
    let shot_full = bow_shot(1.0, Vec3::new(0.0, 10.0, 0.0), Vec3::new(1.0, 0.0, 0.0), &def);

    assert_eq!(shot_half.vel.x, 45.0 * 0.3);
    assert_eq!(shot_full.vel.x, 45.0);
    assert_eq!(shot_half.damage, 50.0 * 0.3);
    assert_eq!(shot_full.damage, 50.0);

    let mut sim_half = ProjectileSim::new(1);
    let mut sim_full = ProjectileSim::new(1);
    sim_half.spawn(shot_half);
    sim_full.spawn(shot_full);

    // Step for 0.4 seconds
    sim_half.step(0.4, |_from, _to| None);
    sim_full.step(0.4, |_from, _to| None);

    let x_half = sim_half.projectiles()[0].pos.x;
    let x_full = sim_full.projectiles()[0].pos.x;

    assert!(x_full > x_half * 3.0, "full draw x ({}) must be > 3x half draw x ({})", x_full, x_half);
}

#[test]
fn projectile_ccd_no_tunnelling_at_high_speed() {
    let mut sim = ProjectileSim::new(10);
    // Projectile flying at 60 m/s along +X
    let proj = Projectile {
        pos: Vec3::new(0.0, 1.0, 0.0),
        vel: Vec3::new(60.0, 0.0, 0.0),
        kind: ProjectileKind::Arrow,
        damage: 40.0,
        owner: Some(7),
        headshot_multiplier: 2.0,
        gravity: 0.0, // pure horizontal check
        max_flight_seconds: 5.0,
        flight_time: 0.0,
        pickup_chance: 0.6,
    };
    sim.spawn(proj);

    // With dt = 1/60, the projectile moves from 0.0 to 1.0 in this single step.
    // Target is thin and located at x = 0.4 m (well before the end of the step).
    let dt = 1.0 / 60.0;
    let zombie_x = 0.4;
    let zombie_id = 99;

    let events = sim.step(dt, |from, to| {
        if from.x <= zombie_x && to.x >= zombie_x {
            Some(Hit::zombie(Vec3::new(zombie_x, 1.0, 0.0), zombie_id, false))
        } else {
            None
        }
    });

    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0],
        ProjectileEvent::HitZombie {
            id: zombie_id,
            damage: 40.0,
            headshot: false,
        }
    );
    // Projectile impacted, so simulation should now be empty
    assert!(sim.is_empty());
}

#[test]
fn projectile_headshot_multiplier() {
    let def = test_bow_def();
    let shot = bow_shot(1.0, Vec3::new(0.0, 1.0, 0.0), Vec3::new(1.0, 0.0, 0.0), &def);
    assert_eq!(shot.damage, 50.0);
    assert_eq!(shot.headshot_multiplier, 2.5);

    let mut sim = ProjectileSim::new(100);
    sim.spawn(shot);

    let events = sim.step(0.1, |_from, _to| {
        Some(Hit::Zombie {
            point: Vec3::new(1.0, 1.0, 0.0),
            id: 123,
            headshot: true,
        })
    });

    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0],
        ProjectileEvent::HitZombie {
            id: 123,
            damage: 125.0, // 50.0 * 2.5
            headshot: true,
        }
    );
}

#[test]
fn projectile_pickup_chance_roughly_60_percent() {
    let def = test_bow_def();
    let mut pickups = 0;
    let total_seeds = 1000;

    for seed in 1..=total_seeds {
        let shot = bow_shot(1.0, Vec3::ZERO, Vec3::X, &def);
        let mut sim = ProjectileSim::new(seed);
        sim.spawn(shot);

        let events = sim.step(0.1, |_from, _to| {
            Some(Hit::world(Vec3::new(1.0, 0.0, 0.0)))
        });

        assert_eq!(events.len(), 1);
        if let ProjectileEvent::HitWorld { pickup, .. } = &events[0] {
            if pickup.as_deref() == Some("arrow") {
                pickups += 1;
            }
        } else {
            panic!("expected HitWorld event");
        }
    }

    // Expected: 600 out of 1000. Accept [530, 670] (well within normal deviation for 1000 trials)
    println!("pickups over {} seeds: {}", total_seeds, pickups);
    assert!(
        (530..=670).contains(&pickups),
        "expected ~60% pickups over {} seeds, got {}",
        total_seeds,
        pickups
    );
}

#[test]
fn projectile_spear_throw_produces_pickup() {
    let def = test_spear_def();
    for seed in 1..=10 {
        let spear = spear_throw(Vec3::new(0.0, 1.5, 0.0), Vec3::new(1.0, 0.0, 0.0), &def);
        assert_eq!(spear.kind, ProjectileKind::ThrownSpear);
        assert_eq!(spear.damage, 45.0);
        assert_eq!(spear.vel.x, 20.0);

        let mut sim = ProjectileSim::new(seed);
        sim.spawn(spear);

        let events = sim.step(0.2, |_from, _to| {
            Some(Hit::World {
                point: Vec3::new(3.0, 0.0, 0.0),
            })
        });

        assert_eq!(events.len(), 1);
        match &events[0] {
            ProjectileEvent::HitWorld { point, pickup } => {
                assert_eq!(*point, Vec3::new(3.0, 0.0, 0.0));
                assert_eq!(pickup.as_deref(), Some("spear"));
                assert_eq!(events[0].pickup(), Some("spear"));
            }
            _ => panic!("expected HitWorld event"),
        }
    }
}

#[test]
fn projectile_expiration_after_max_flight_seconds() {
    let mut proj = Projectile::new(
        Vec3::ZERO,
        Vec3::new(10.0, 0.0, 0.0),
        ProjectileKind::Arrow,
        25.0,
        None,
    );
    proj.gravity = 0.0;
    proj.max_flight_seconds = 0.5;

    let mut sim = ProjectileSim::default();
    sim.spawn(proj);
    assert_eq!(sim.len(), 1);

    // Step 0.3 s (not yet expired)
    let ev1 = sim.step(0.3, |_from, _to| None);
    assert!(ev1.is_empty());
    assert_eq!(sim.len(), 1);

    // Step another 0.3 s (total 0.6 s >= 0.5 s max)
    let ev2 = sim.step(0.3, |_from, _to| None);
    assert_eq!(ev2, vec![ProjectileEvent::Expired]);
    assert!(sim.is_empty());
}

#[test]
fn projectile_api_coverage() {
    let default_proj = Projectile::default();
    assert_eq!(default_proj.kind, ProjectileKind::Arrow);
    assert_eq!(default_proj.damage, 0.0);

    let z_hit = Hit::zombie(Vec3::new(1.0, 2.0, 3.0), 10, true);
    assert_eq!(z_hit.point(), Vec3::new(1.0, 2.0, 3.0));

    let w_hit = Hit::world(Vec3::new(4.0, 5.0, 6.0));
    assert_eq!(w_hit.point(), Vec3::new(4.0, 5.0, 6.0));

    let no_pickup_ev = ProjectileEvent::HitWorld {
        point: Vec3::ZERO,
        pickup: None,
    };
    assert_eq!(no_pickup_ev.pickup(), None);

    let expired_ev = ProjectileEvent::Expired;
    assert_eq!(expired_ev.pickup(), None);
}
