use game::melee::{gun_bash, Attacker, MeleeSwing, MeleeTarget, SwingState};
use game::{Stamina, WeaponDef, WeaponRegistry};
use glam::{IVec2, Vec3};
use physics::{PhysicsWorld, CHUNK_SAMPLES};

const BAT: &str = "bat";

fn weapons() -> WeaponRegistry {
    let path = paths::repo_path("data/weapons.json");
    WeaponRegistry::load(&path).expect("data/weapons.json must load")
}

fn weapon(id: &str) -> WeaponDef {
    weapons().get(id).unwrap_or_else(|| panic!("{id} in weapons.json")).clone()
}

/// Flat ground over the origin chunk, so the sight ray has something under the feet.
fn flat_world() -> PhysicsWorld {
    let mut world = PhysicsWorld::new();
    let heights = vec![0.0f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
    world.add_ground_chunk(IVec2::new(0, 0), &heights);
    world
}

/// A 6 m wide, 3 m tall, 1 m thick wall centred at `at`, as convex points.
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

/// Attacker at (10, 0, 10) facing -Z, with its sight at head height.
fn attacker() -> Attacker {
    Attacker {
        id: 1,
        position: Vec3::new(10.0, 0.0, 10.0),
        facing: Vec3::NEG_Z,
        eye: Vec3::new(10.0, 1.7, 10.0),
    }
}

/// A target standing `forward` metres in front of the attacker (-Z) and `side` metres to the side.
fn target_at(id: u64, forward: f32, side: f32) -> MeleeTarget {
    MeleeTarget {
        id,
        position: Vec3::new(10.0 + side, 0.0, 10.0 - forward),
        radius: 0.4,
    }
}

fn assert_close(actual: Vec3, expected: Vec3) {
    assert!(
        actual.distance(expected) < 1e-4,
        "{actual:?} != {expected:?}"
    );
}

#[test]
fn every_melee_weapon_in_the_json_is_a_melee_swing() {
    for id in ["bat", "knife", "axe", "machete", "crowbar", "katana"] {
        let w = weapon(id);
        assert!(!w.is_gun(), "{id} must be melee");
        assert!(w.range > 0.0, "{id} needs a range");
        assert!(w.windup > 0.0 || w.cooldown > 0.0, "{id} needs timing");
    }
}

#[test]
fn target_in_front_is_hit_once_at_the_impact_time() {
    let world = flat_world();
    let w = weapon(BAT);
    let mut stamina = Stamina::default();
    let mut swing = MeleeSwing::new();
    let targets = [target_at(7, 1.2, 0.0)];

    assert!(swing.try_start(&w, &mut stamina));
    // Just before the windup ends: no hit yet.
    assert!(swing.update(w.windup - 0.01, attacker(), &targets, &world).is_empty());
    assert_eq!(swing.state(), SwingState::WindingUp);

    // Crossing the windup: exactly one hit, with the bat's numbers.
    let hits = swing.update(0.02, attacker(), &targets, &world);
    assert_eq!(hits.len(), 1);
    let hit = &hits[0];
    assert_eq!(hit.target, 7);
    assert_eq!(hit.damage.amount, w.damage);
    assert_eq!(hit.damage.damage_type, "melee");
    assert_eq!(hit.damage.source, Some(1));
    assert!(!hit.damage.headshot);
    assert_close(hit.damage.direction, Vec3::NEG_Z);
    // Hit point is the target's chest (feet + 0.9 m).
    assert_close(hit.damage.hit_point, Vec3::new(10.0, 0.9, 8.8));
    // Knockback pushes away from the attacker (-Z here), scaled by the bat's knockback.
    assert_close(hit.knockback, Vec3::new(0.0, 0.0, -w.knockback));
    assert_eq!(hit.stagger, w.stagger);

    // The same swing never hits again.
    assert!(swing.update(1.0, attacker(), &targets, &world).is_empty());
}

#[test]
fn target_outside_the_arc_or_out_of_range_is_not_hit() {
    let world = flat_world();
    let w = weapon(BAT); // arc 90 degrees (45 either side), range 2.0 m
    let mut stamina = Stamina::default();
    let mut swing = MeleeSwing::new();
    let targets = [
        target_at(1, 1.0, 0.0),  // in front: control, must be hit
        target_at(2, 1.0, 1.5),  // about 56 degrees off the facing: outside the arc
        target_at(3, 3.0, 0.0),  // 2.6 m from the body edge: out of range
    ];

    assert!(swing.try_start(&w, &mut stamina));
    let hits = swing.update(w.windup + 0.01, attacker(), &targets, &world);
    let ids: Vec<u64> = hits.iter().map(|h| h.target).collect();
    assert_eq!(ids, vec![1]);
}

#[test]
fn max_targets_limits_the_hits_and_keeps_the_nearest() {
    let world = flat_world();
    let w = weapon(BAT); // maxTargets 2
    assert_eq!(w.max_targets, 2);
    let mut stamina = Stamina::default();
    let mut swing = MeleeSwing::new();
    // Three targets in a line: distances 1.5 (id 1), 0.8 (id 2), 1.2 (id 3).
    let targets = [target_at(1, 1.5, 0.0), target_at(2, 0.8, 0.0), target_at(3, 1.2, 0.0)];

    assert!(swing.try_start(&w, &mut stamina));
    let hits = swing.update(w.windup + 0.01, attacker(), &targets, &world);
    let ids: Vec<u64> = hits.iter().map(|h| h.target).collect();
    assert_eq!(ids, vec![2, 3], "the two nearest, nearest first");
}

#[test]
fn a_wall_between_blocks_the_hit() {
    let w = weapon(BAT);
    let targets = [target_at(5, 1.5, 0.0)];

    // Control: with no wall, the target is hit.
    let open = flat_world();
    let mut stamina = Stamina::default();
    let mut swing = MeleeSwing::new();
    assert!(swing.try_start(&w, &mut stamina));
    assert_eq!(swing.update(w.windup + 0.01, attacker(), &targets, &open).len(), 1);

    // With a wall between the attacker (z = 10) and the target (z = 8.5), the hit is blocked.
    let mut walled = flat_world();
    add_wall(&mut walled, Vec3::new(10.0, 0.0, 9.2));
    let mut stamina = Stamina::default();
    let mut swing = MeleeSwing::new();
    assert!(swing.try_start(&w, &mut stamina));
    assert!(swing.update(w.windup + 0.01, attacker(), &targets, &walled).is_empty());
}

#[test]
fn cooldown_blocks_a_second_swing_until_it_ends() {
    let world = flat_world();
    let w = weapon(BAT); // windup 0.15 s, cooldown 0.6 s
    let mut stamina = Stamina::default();
    let mut swing = MeleeSwing::new();

    assert!(swing.try_start(&w, &mut stamina));
    assert!(!swing.try_start(&w, &mut stamina), "no second swing while winding up");

    // Windup ends, cooldown runs on: 0.4 s left.
    assert!(swing.update(0.2, attacker(), &[], &world).is_empty());
    assert_eq!(swing.state(), SwingState::CoolingDown);
    assert!(!swing.try_start(&w, &mut stamina), "still cooling down");

    // 0.5 s into the 0.6 s cooldown: 0.1 s is left, so still blocked.
    swing.update(0.3, attacker(), &[], &world);
    assert!(!swing.try_start(&w, &mut stamina), "0.1 s of cooldown left");

    // Past the full cooldown: idle, and a new swing starts.
    swing.update(0.2, attacker(), &[], &world);
    assert_eq!(swing.state(), SwingState::Idle);
    assert!(swing.try_start(&w, &mut stamina));
}

#[test]
fn not_enough_stamina_blocks_the_swing() {
    let w = weapon(BAT); // stamina cost 12
    let mut swing = MeleeSwing::new();

    let mut short = Stamina { current: 5.0, ..Default::default() };
    assert!(!swing.try_start(&w, &mut short));
    assert_eq!(short.current, 5.0, "a refused swing spends nothing");
    assert_eq!(swing.state(), SwingState::Idle, "a refused swing starts no cooldown");

    // Exactly the cost is enough, and it resets the regen timer as the C# TryUse does.
    let mut exact = Stamina { current: 12.0, time_since_use: 0.5, ..Default::default() };
    assert!(swing.try_start(&w, &mut exact));
    assert_eq!(exact.current, 0.0);
    assert_eq!(exact.time_since_use, 0.0);
}

#[test]
fn turning_during_the_windup_does_not_bend_the_swing() {
    let world = flat_world();
    let w = weapon(BAT);
    let mut stamina = Stamina::default();
    let mut swing = MeleeSwing::new();
    let targets = [target_at(9, 1.2, 0.0)]; // in front (-Z) when the swing starts

    assert!(swing.try_start(&w, &mut stamina));
    // The first update captures the swing direction (-Z).
    assert!(swing.update(0.0, attacker(), &targets, &world).is_empty());
    // The player then turns around (+Z); the swing still goes the way it started.
    let turned = Attacker { facing: Vec3::Z, ..attacker() };
    let hits = swing.update(w.windup, turned, &targets, &world);
    assert_eq!(hits.len(), 1);
    assert_close(hits[0].damage.direction, Vec3::NEG_Z);
}

#[test]
fn a_missed_swing_still_makes_its_noise_once() {
    let world = flat_world();
    let w = weapon(BAT);
    let mut stamina = Stamina::default();
    let mut swing = MeleeSwing::new();

    assert!(swing.try_start(&w, &mut stamina));
    assert!(swing.update(w.windup + 0.01, attacker(), &[], &world).is_empty());
    let noise = swing.take_noise().expect("the swing makes noise even when it misses");
    assert_eq!(noise.position, attacker().position);
    assert_eq!(noise.radius, w.noise_radius);
    assert!(swing.take_noise().is_none(), "noise is taken once");
}

#[test]
fn cancelled_swing_does_not_hit_or_make_noise() {
    let world = flat_world();
    let w = weapon(BAT);
    let mut stamina = Stamina::default();
    let mut swing = MeleeSwing::new();
    let targets = [target_at(4, 1.2, 0.0)];

    assert!(swing.try_start(&w, &mut stamina));
    swing.cancel();
    assert_eq!(swing.state(), SwingState::CoolingDown, "cooldown keeps running");
    assert!(swing.update(w.windup + 0.01, attacker(), &targets, &world).is_empty());
    assert!(swing.take_noise().is_none());
}

#[test]
fn gun_bash_takes_its_numbers_from_the_gun() {
    let world = flat_world();
    let gun = weapon("pistol");
    let bash = gun_bash(&gun);

    assert!(!bash.is_gun());
    assert_eq!(bash.id, "pistol");
    assert_eq!(bash.damage, gun.bash_damage);
    assert_eq!(bash.windup, gun.bash_impact);
    assert_eq!(bash.anim.attack.as_deref(), Some("gun_bash"));
    assert_eq!(bash.anim.impact, gun.bash_impact);
    // Fixed GunBash.cs numbers.
    assert_eq!(bash.range, 1.9);
    assert_eq!(bash.arc_deg, 80.0);
    assert_eq!(bash.cooldown, 0.7);
    assert_eq!(bash.stamina_cost, 10.0);
    assert_eq!(bash.max_targets, 1);
    assert_eq!(bash.noise_radius, 6.0);

    // The bash hits at the gun's bash impact time, for the gun's bash damage.
    let targets = [target_at(6, 1.0, 0.0)];
    let mut stamina = Stamina::default();
    let mut swing = MeleeSwing::new();
    assert!(swing.try_start(&bash, &mut stamina));
    assert!(swing.update(gun.bash_impact - 0.01, attacker(), &targets, &world).is_empty());
    let hits = swing.update(0.02, attacker(), &targets, &world);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].damage.amount, gun.bash_damage);
}
