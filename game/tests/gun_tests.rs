use game::guns::{Gun, ReloadDone};
use game::guns_fire::{fire_rays, resolve_hits, shot_noise, GunRng, GunTarget, Ray};
use game::{WeaponDef, WeaponRegistry};
use glam::{IVec2, Vec3};
use physics::{PhysicsWorld, CHUNK_SAMPLES};

fn weapon(id: &str) -> WeaponDef {
    let registry = WeaponRegistry::load(&paths::repo_path("data/weapons.json")).expect("data/weapons.json loads");
    registry.get(id).unwrap_or_else(|| panic!("weapon {id} in weapons.json")).clone()
}

/// Flat ground over the origin chunk (x and z in 0..64), so shots fired inside it have ground to hit.
fn flat_world() -> PhysicsWorld {
    let mut world = PhysicsWorld::new();
    let heights = vec![0.0f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
    world.add_ground_chunk(IVec2::new(0, 0), &heights);
    world
}

/// A 6 m wide, 3 m tall, 1 m thick wall whose bottom is at `at`.
fn add_wall(world: &mut PhysicsWorld, at: Vec3) {
    let (x, y, z) = (3.0, 3.0, 0.5);
    let mut points: Vec<[f32; 3]> = Vec::new();
    for px in [-x, x] {
        for py in [0.0, y] {
            for pz in [-z, z] {
                points.push([px, py, pz]);
            }
        }
    }
    world.add_static(&[points], at, 0.0, 1.0);
}

/// A zombie with its feet at z = 30, facing the shooter at z = 40.
fn zombie() -> GunTarget {
    GunTarget {
        id: 5,
        position: Vec3::new(10.0, 0.0, 30.0),
        body_radius: 0.4,
        head_center: Vec3::new(10.0, 1.6, 30.0),
        head_radius: 0.2,
    }
}

/// Shooter position for the target tests: 10 m in front of the zombie, looking down -Z.
fn shooter_at(height: f32) -> Vec3 {
    Vec3::new(10.0, height, 40.0)
}

#[test]
fn firing_uses_a_round_and_respects_the_fire_rate() {
    let pistol = weapon("pistol");
    assert_eq!(pistol.fire_interval, 0.25);
    let mut gun = Gun::new(12);

    assert!(gun.try_fire(&pistol, true, true));
    assert_eq!(gun.rounds(), 11);

    assert!(!gun.try_fire(&pistol, true, true), "still cooling down right after a shot");
    gun.update(0.1);
    assert!(!gun.try_fire(&pistol, true, true), "0.1 s is less than the 0.25 s interval");
    gun.update(0.2);
    assert!(gun.try_fire(&pistol, true, true), "0.3 s is past the interval");
    assert_eq!(gun.rounds(), 10);
}

#[test]
fn an_empty_magazine_cannot_fire() {
    let pistol = weapon("pistol");
    let mut gun = Gun::new(0);
    assert!(gun.is_empty());
    assert!(!gun.try_fire(&pistol, true, true));
    assert_eq!(gun.rounds(), 0);
    // The caller starts a reload when it sees the dry fire; the bag has ammo here.
    assert!(gun.start_reload(&pistol, 5));
}

#[test]
fn reload_takes_the_csharp_time_and_fills_from_the_bag() {
    let pistol = weapon("pistol");
    assert_eq!(pistol.reload_time, 1.6);
    let mut gun = Gun::new(3);

    assert!(gun.start_reload(&pistol, 20));
    assert!(gun.is_reloading());
    assert!(!gun.try_fire(&pistol, true, true), "no shots while reloading");
    assert_eq!(gun.update(1.0), None);
    assert_eq!(gun.update(0.5), None, "1.5 s is still short of 1.6 s");
    assert_eq!(gun.update(0.2), Some(ReloadDone { rounds_taken: 9 }), "magazine 12 - 3 in it = 9 from the bag");
    assert_eq!(gun.rounds(), 12);
    assert!(!gun.is_reloading());
}

#[test]
fn reload_is_partial_when_the_bag_is_short() {
    let pistol = weapon("pistol");
    let mut gun = Gun::new(3);
    assert!(gun.start_reload(&pistol, 4));
    assert_eq!(gun.update(2.0), Some(ReloadDone { rounds_taken: 4 }));
    assert_eq!(gun.rounds(), 7);
}

#[test]
fn reload_refuses_a_full_magazine_an_empty_bag_or_a_second_reload() {
    let pistol = weapon("pistol");
    assert!(!Gun::new(12).start_reload(&pistol, 50), "magazine already full");
    assert!(!Gun::new(3).start_reload(&pistol, 0), "no ammo in the bag");

    let mut gun = Gun::new(3);
    assert!(gun.start_reload(&pistol, 20));
    assert!(!gun.start_reload(&pistol, 20), "already reloading");
}

#[test]
fn cancelled_reload_moves_no_ammo() {
    let pistol = weapon("pistol");
    let mut gun = Gun::new(3);
    assert!(gun.start_reload(&pistol, 20));
    gun.cancel_reload();
    assert!(!gun.is_reloading());
    assert_eq!(gun.update(5.0), None);
    assert_eq!(gun.rounds(), 3);
}

#[test]
fn semi_automatic_fires_only_on_press_and_automatic_while_held() {
    let pistol = weapon("pistol");
    assert!(!pistol.auto);
    let mut semi = Gun::new(12);
    assert!(!semi.try_fire(&pistol, false, true), "holding does not fire a semi-auto gun");
    assert_eq!(semi.rounds(), 12);
    assert!(semi.try_fire(&pistol, true, true));
    semi.update(1.0);
    assert!(!semi.try_fire(&pistol, false, true), "held after the cooldown still does not fire");

    let ak = weapon("ak47");
    assert!(ak.auto);
    let mut auto = Gun::new(30);
    for _ in 0..3 {
        assert!(auto.try_fire(&ak, false, true), "an automatic gun fires while held");
        auto.update(0.2); // past the 0.1 s interval
    }
    assert_eq!(auto.rounds(), 27);
}

#[test]
fn shotgun_fires_one_ray_per_pellet() {
    let shotgun = weapon("shotgun");
    assert_eq!(shotgun.pellets, 8);
    let rays = fire_rays(&shotgun, shooter_at(1.6), Vec3::NEG_Z, false, &mut GunRng::new(7));
    assert_eq!(rays.len(), 8);

    let pistol = weapon("pistol");
    assert_eq!(fire_rays(&pistol, shooter_at(1.6), Vec3::NEG_Z, false, &mut GunRng::new(7)).len(), 1);
}

#[test]
fn ray_at_a_head_deals_headshot_damage() {
    let world = flat_world();
    let pistol = weapon("pistol");
    let ray = Ray { origin: shooter_at(1.6), dir: Vec3::NEG_Z };

    let hits = resolve_hits(&[ray], &pistol, &[zombie()], &world, pistol.range);
    assert_eq!(hits.len(), 1);
    let hit = &hits[0];
    assert_eq!(hit.target, Some(5));
    let damage = hit.damage.as_ref().expect("a zombie hit has damage");
    assert_eq!(damage.amount, 60.0, "pistol 30 x headshot 2.0");
    assert!(damage.headshot);
    assert_eq!(damage.damage_type, "bullet");
    assert!((hit.distance - 9.8).abs() < 1e-3, "head surface is 9.8 m away, got {}", hit.distance);
}

#[test]
fn ray_at_the_body_deals_normal_damage() {
    let world = flat_world();
    let pistol = weapon("pistol");
    let ray = Ray { origin: shooter_at(0.8), dir: Vec3::NEG_Z };

    let hits = resolve_hits(&[ray], &pistol, &[zombie()], &world, pistol.range);
    assert_eq!(hits.len(), 1);
    let damage = hits[0].damage.as_ref().expect("a zombie hit has damage");
    assert_eq!(damage.amount, 30.0);
    assert!(!damage.headshot);
    assert!((hits[0].distance - 9.6).abs() < 1e-3, "body surface is 9.6 m away, got {}", hits[0].distance);
}

#[test]
fn a_wall_in_front_stops_the_bullet() {
    let mut world = flat_world();
    add_wall(&mut world, Vec3::new(10.0, 0.0, 35.0));
    let pistol = weapon("pistol");
    let ray = Ray { origin: shooter_at(0.8), dir: Vec3::NEG_Z };

    let hits = resolve_hits(&[ray], &pistol, &[zombie()], &world, pistol.range);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].target, None, "the wall is hit, not the zombie behind it");
    assert!(hits[0].damage.is_none(), "walls take no damage");
    assert!((hits[0].distance - 4.5).abs() < 0.01, "wall face is 4.5 m away, got {}", hits[0].distance);
}

#[test]
fn a_ray_that_misses_everything_hits_nothing() {
    let world = flat_world();
    let pistol = weapon("pistol");
    let ray = Ray { origin: Vec3::new(30.0, 0.8, 40.0), dir: Vec3::NEG_Z };
    assert!(resolve_hits(&[ray], &pistol, &[zombie()], &world, pistol.range).is_empty());
}

#[test]
fn same_seed_gives_the_same_spread() {
    let shotgun = weapon("shotgun");
    let origin = shooter_at(1.6);
    let a = fire_rays(&shotgun, origin, Vec3::NEG_Z, false, &mut GunRng::new(42));
    let b = fire_rays(&shotgun, origin, Vec3::NEG_Z, false, &mut GunRng::new(42));
    let c = fire_rays(&shotgun, origin, Vec3::NEG_Z, false, &mut GunRng::new(43));
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn spread_stays_inside_the_cone_for_hip_and_aim() {
    let shotgun = weapon("shotgun");
    let origin = shooter_at(1.6);
    let angle_from_aim = |ray: &Ray| ray.dir.dot(Vec3::NEG_Z).clamp(-1.0, 1.0).acos().to_degrees();

    for ray in fire_rays(&shotgun, origin, Vec3::NEG_Z, false, &mut GunRng::new(9)) {
        assert!((ray.dir.length() - 1.0).abs() < 1e-4, "rays are unit length");
        assert!(angle_from_aim(&ray) <= shotgun.spread_hip_deg + 1e-3);
    }
    for ray in fire_rays(&shotgun, origin, Vec3::NEG_Z, true, &mut GunRng::new(9)) {
        assert!(angle_from_aim(&ray) <= shotgun.spread_aim_deg + 1e-3);
    }
}

#[test]
fn shot_noise_uses_the_weapon_radius_at_the_player() {
    let pistol = weapon("pistol");
    assert_eq!(pistol.noise_radius, 35.0);
    let player = Vec3::new(1.0, 0.0, 2.0);
    let noise = shot_noise(&pistol, player);
    assert_eq!(noise.position, player);
    assert_eq!(noise.radius, 35.0);
}

#[test]
fn zero_seed_still_produces_numbers() {
    let mut rng = GunRng::new(0);
    assert_ne!(rng.next_u64(), 0);
}
