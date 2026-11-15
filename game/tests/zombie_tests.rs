use game::zombie::{hears, AttackHit, Noise, Senses, ZombieAttack, ZombieDef, ZombieDefs};
use glam::{IVec2, Vec3};
use physics::{PhysicsWorld, CHUNK_SAMPLES};

const WALKER: &str = "walker";

fn load_defs() -> ZombieDefs {
    ZombieDefs::load_default().expect("data/zombies.json must load")
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

fn walker() -> ZombieDef {
    load_defs().get(WALKER).expect("walker type").clone()
}

/// Zombie at (10, 0, 10) facing -Z. Targets at z < 10 are in front of it.
fn zombie_feet() -> Vec3 {
    Vec3::new(10.0, 0.0, 10.0)
}

fn facing() -> Vec3 {
    Vec3::new(0.0, 0.0, -1.0)
}

#[test]
fn every_zombie_type_in_the_json_loads() {
    let text = std::fs::read_to_string(paths::repo_path("data/zombies.json")).unwrap();
    let raw: serde_json::Value = serde_json::from_str(&text).unwrap();
    let expected = raw["types"].as_object().expect("types object").len();
    assert!(expected > 0, "zombies.json has no types");

    let defs = load_defs();
    assert_eq!(defs.ids().count(), expected, "every JSON type must be loaded");
    assert_eq!(defs.get(WALKER).unwrap().max_health, 60.0);
    let runner = defs.get("runner").expect("runner type");
    assert_eq!(runner.run_clip.as_deref(), Some("zombie_runner_sprint"));
    assert_eq!(defs.get("walker").unwrap().run_clip, None);
    assert_eq!(defs.behaviour.horde_max, 5);
    assert_eq!(defs.behaviour.gaits.len(), 2);
    assert!(defs.get("nope").is_none());
}

#[test]
fn a_missing_key_is_an_error_not_a_silent_default() {
    let broken = r#"{"types": {"x": {"maxHealth": 1}}, "behaviour": {}}"#;
    assert!(ZombieDefs::from_json(broken).is_err());
}

#[test]
fn target_in_front_in_range_with_clear_view_is_seen() {
    let world = flat_world();
    let mut senses = Senses::new(&walker());
    let target = Vec3::new(10.0, 0.0, 2.0); // 8 m ahead
    senses.update(0.0, zombie_feet(), facing(), Some(target), &world);
    assert!(senses.can_see());
    assert_eq!(senses.last_seen(), target);
    assert_eq!(senses.time_since_seen(), 0.0);
}

#[test]
fn target_out_of_sight_range_is_not_seen() {
    let world = flat_world();
    let mut senses = Senses::new(&walker());
    let target = Vec3::new(10.0, 0.0, -9.5); // 19.5 m ahead, walker range is 18
    senses.update(0.0, zombie_feet(), facing(), Some(target), &world);
    assert!(!senses.can_see());
}

#[test]
fn target_behind_is_not_seen_unless_very_close() {
    let world = flat_world();
    let mut senses = Senses::new(&walker());
    let behind = Vec3::new(10.0, 0.0, 18.0); // 8 m behind, outside the 120 degree cone
    senses.update(0.0, zombie_feet(), facing(), Some(behind), &world);
    assert!(!senses.can_see());

    let mut close = Senses::new(&walker());
    let close_behind = Vec3::new(10.0, 0.0, 12.0); // 2 m behind: senses all around up close
    close.update(0.0, zombie_feet(), facing(), Some(close_behind), &world);
    assert!(close.can_see());
}

#[test]
fn a_wall_blocks_sight() {
    let mut world = flat_world();
    add_wall(&mut world, Vec3::new(10.0, 0.0, 5.0));
    let mut senses = Senses::new(&walker());
    let target = Vec3::new(10.0, 0.0, 2.0); // wall sits between them
    senses.update(0.0, zombie_feet(), facing(), Some(target), &world);
    assert!(!senses.can_see());
}

#[test]
fn sight_is_rechecked_only_every_check_interval() {
    let world = flat_world();
    let mut senses = Senses::new(&walker());
    let front = Vec3::new(10.0, 0.0, 2.0);
    let behind = Vec3::new(10.0, 0.0, 18.0);
    senses.update(0.0, zombie_feet(), facing(), Some(front), &world);
    assert!(senses.can_see());
    // 0.15 s later is still inside the 0.2 s interval, so the old answer holds
    senses.update(0.15, zombie_feet(), facing(), Some(behind), &world);
    assert!(senses.can_see());
    // another 0.1 s passes the interval, so the new check says no
    senses.update(0.1, zombie_feet(), facing(), Some(behind), &world);
    assert!(!senses.can_see());
}

#[test]
fn a_noise_inside_its_radius_is_heard_and_outside_is_not() {
    let feet = zombie_feet();
    let inside = Noise { position: feet + Vec3::X * 9.9, radius: 10.0 };
    let outside = Noise { position: feet + Vec3::X * 10.1, radius: 10.0 };
    assert!(hears(feet, &inside, 1.0));
    assert!(!hears(feet, &outside, 1.0));
    // A hearing multiplier of 0.5 halves the reach
    assert!(!hears(feet, &inside, 0.5));
}

#[test]
fn memory_lasts_lose_target_time_then_expires() {
    let world = flat_world();
    let mut senses = Senses::new(&walker()); // lose_target_time = 6 s
    let seen_at = Vec3::new(10.0, 0.0, 2.0);
    senses.update(0.0, zombie_feet(), facing(), Some(seen_at), &world);

    // Target gone: six 1 s steps is exactly 6 s, which is not yet past the limit
    let gone = Vec3::new(10.0, 0.0, 18.0);
    for _ in 0..6 {
        senses.update(1.0, zombie_feet(), facing(), Some(gone), &world);
    }
    assert!(!senses.memory_expired());
    assert_eq!(senses.last_seen(), seen_at);

    senses.update(0.5, zombie_feet(), facing(), Some(gone), &world);
    assert!(senses.memory_expired());
    assert_eq!(senses.last_seen(), seen_at, "last seen spot is kept until a new sighting");
}

#[test]
fn a_hit_refreshes_memory() {
    let mut senses = Senses::new(&walker());
    let attacker = Vec3::new(4.0, 0.0, 4.0);
    senses.notice_at(attacker);
    assert_eq!(senses.last_seen(), attacker);
    assert_eq!(senses.time_since_seen(), 0.0);
    assert!(!senses.memory_expired());
}

#[test]
fn bite_lands_only_after_windup_and_respects_cooldown() {
    let def = walker(); // windup 0.45 s, cooldown 1.2 s, reach 1.4 + 0.3
    let mut attack = ZombieAttack::new(&def);
    attack.start();
    assert!(attack.is_active());

    assert_eq!(attack.update(0.4, 1.0), None, "no bite before the windup ends");
    assert_eq!(attack.update(0.1, 1.0), Some(AttackHit { damage: 10.0 }), "bite at 0.5 s");
    assert!(attack.is_active(), "cooldown is running");

    // Still cooling down: a new bite cannot start, and nothing lands
    assert_eq!(attack.update(0.5, 1.0), None);
    assert!(attack.is_active());
    assert_eq!(attack.update(1.0, 1.0), None, "cooldown is over after 1.2 s");
    assert!(!attack.is_active());
}

#[test]
fn a_bite_out_of_reach_misses_but_still_uses_the_cooldown() {
    let def = walker();
    let mut attack = ZombieAttack::new(&def);
    attack.start();
    assert_eq!(attack.update(0.5, 2.0), None, "2 m is beyond 1.4 + 0.3 m");
    assert!(attack.is_active());
    attack.update(1.3, 0.0);
    assert!(!attack.is_active());
}

#[test]
fn bite_reach_includes_the_margin() {
    let def = walker();
    let mut attack = ZombieAttack::new(&def);
    attack.start();
    assert!(attack.update(0.5, 1.6).is_some(), "1.6 m is within 1.4 + 0.3");
}
