//! Zombie brain tests (R3-04). Pure, no window. Each test moves the zombie by the velocity the brain
//! returns (no collisions) on a flat chunk. Walker numbers come from data/zombies.json.

use game::DamageInfo;
use game::zombie::{
    AnimClip, BrainInput, BrainOutput, Noise, ZombieBehaviourDef, ZombieBrain, ZombieDef,
    ZombieDefs, ZombieState,
};
use glam::{IVec2, Vec3};
use physics::{CHUNK_SAMPLES, PhysicsWorld};

const DT: f32 = 1.0 / 60.0;
const WALKER: &str = "walker";

/// The data a test zombie needs, plus a flat world under it.
struct Rig {
    def: ZombieDef,
    tuning: ZombieBehaviourDef,
    world: PhysicsWorld,
}

fn rig() -> Rig {
    let defs = ZombieDefs::load_default().expect("data/zombies.json must load");
    Rig {
        def: defs.get(WALKER).expect("walker type").clone(),
        tuning: defs.behaviour.clone(),
        world: flat_world(),
    }
}

/// Flat ground over the origin chunk, so the sight ray and the ground both exist.
fn flat_world() -> PhysicsWorld {
    let mut world = PhysicsWorld::new();
    let heights = vec![0.0f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
    world.add_ground_chunk(IVec2::new(0, 0), &heights);
    world
}

/// Where the test zombie stands: (10, 0, 10), facing -Z.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Body {
    pos: Vec3,
    yaw: f32,
}

fn start() -> Body {
    Body {
        pos: Vec3::new(10.0, 0.0, 10.0),
        yaw: 0.0,
    }
}

fn make_brain(r: &Rig, seed: u64) -> ZombieBrain {
    ZombieBrain::new(&r.def, &r.tuning, start().pos, seed)
}

/// This frame's input for `body`: no sounds, no hit, not dead.
fn sense<'a>(body: Body, world: &'a PhysicsWorld, target: Option<Vec3>) -> BrainInput<'a> {
    BrainInput {
        position: body.pos,
        yaw: body.yaw,
        on_floor: true,
        target,
        heard: &[],
        flow_dir: None,
        took_damage: None,
        is_dead: false,
        world,
        nearby_zombies: &[],
        corpses: &[],
        target_unreachable: false,
        leg_hit: false,
    }
}

/// One frame: the brain thinks, then the body moves by the velocity it returns and turns to its yaw.
fn run_frame(brain: &mut ZombieBrain, body: &mut Body, ctx: BrainInput<'_>) -> BrainOutput {
    let out = brain.update(&ctx, DT);
    body.pos += out.velocity * DT;
    body.yaw = out.yaw;
    out
}

/// One ordinary frame with only a target (or none) to look at.
fn tick(
    brain: &mut ZombieBrain,
    body: &mut Body,
    world: &PhysicsWorld,
    target: Option<Vec3>,
) -> BrainOutput {
    let ctx = sense(*body, world, target);
    run_frame(brain, body, ctx)
}

#[test]
fn an_idle_zombie_with_the_player_in_view_chases_toward_them() {
    let r = rig();
    let mut brain = make_brain(&r, 1);
    let mut body = start();
    let player = Vec3::new(10.0, 0.0, 4.0); // 6 m straight ahead, in view

    let first = tick(&mut brain, &mut body, &r.world, Some(player));
    assert_eq!(first.state, ZombieState::Chase);

    let mut out = first;
    for _ in 0..60 {
        out = tick(&mut brain, &mut body, &r.world, Some(player));
    }
    assert_eq!(out.state, ZombieState::Chase);
    assert!(
        out.velocity.z < -1.0,
        "should walk toward the player (-Z), got {:?}",
        out.velocity
    );
    assert!(
        out.velocity.x.abs() < 1e-4,
        "no sideways drift, got {:?}",
        out.velocity
    );
    assert!(
        body.pos.z < start().pos.z - 1.0,
        "should have moved toward the player"
    );
}

#[test]
fn a_zombie_in_bite_range_bites_once_after_the_windup() {
    let r = rig();
    let mut brain = make_brain(&r, 1);
    let mut body = start();
    let player = Vec3::new(10.0, 0.0, 9.0); // 1 m away, inside the 1.4 m attack range

    tick(&mut brain, &mut body, &r.world, Some(player)); // sees the player: chase
    let first_attack = tick(&mut brain, &mut body, &r.world, Some(player)); // in range: attack starts
    assert_eq!(first_attack.state, ZombieState::Attack);

    let windup = r.def.attack_windup;
    let mut elapsed = 0.0_f32;
    let mut hits = Vec::new();
    while elapsed < windup + 0.5 {
        let out = tick(&mut brain, &mut body, &r.world, Some(player));
        elapsed += DT;
        assert_eq!(
            out.state,
            ZombieState::Attack,
            "stays in attack through the windup and cooldown"
        );
        if let Some(hit) = out.attack_hit {
            hits.push((elapsed, hit));
        }
    }
    assert_eq!(hits.len(), 1, "one bite in the window, got {hits:?}");
    let (when, hit) = hits[0];
    assert!(
        (when - windup).abs() <= 2.0 * DT,
        "bite at {when} s, windup is {windup} s"
    );
    assert_eq!(hit.damage, r.def.attack_damage);
}

#[test]
fn a_heard_noise_sends_an_idle_zombie_to_investigate_it() {
    let r = rig();
    let mut brain = make_brain(&r, 1);
    let mut body = start();
    let noise_at = Vec3::new(10.0, 0.0, -20.0); // 30 m away, straight ahead
    let heard = [Noise {
        position: noise_at,
        radius: 50.0,
    }]; // hearing reaches 50 m
    let ctx = BrainInput {
        heard: &heard,
        ..sense(body, &r.world, None)
    };
    let first = run_frame(&mut brain, &mut body, ctx);
    assert_eq!(first.state, ZombieState::Investigate);

    // It walks toward the noise, looks about at the spot, then wanders on
    let mut seen = vec![first.state];
    let mut walked_toward = false;
    let mut t = 0.0_f32;
    while t < 40.0 {
        let out = tick(&mut brain, &mut body, &r.world, None);
        t += DT;
        if out.velocity.z < -0.5 {
            walked_toward = true;
        }
        if seen.last() != Some(&out.state) {
            seen.push(out.state);
        }
        if out.state == ZombieState::Wander {
            break;
        }
    }
    assert!(walked_toward, "should walk toward -Z, where the noise was");
    assert_eq!(seen, vec![ZombieState::Investigate, ZombieState::Wander]);

    // A sound beyond its hearing changes nothing
    let mut deaf = make_brain(&r, 2);
    let mut deaf_body = start();
    let faint = [Noise {
        position: noise_at,
        radius: 0.5,
    }];
    let ctx = BrainInput {
        heard: &faint,
        ..sense(deaf_body, &r.world, None)
    };
    assert_eq!(
        run_frame(&mut deaf, &mut deaf_body, ctx).state,
        ZombieState::Idle
    );
}

#[test]
fn a_zombie_that_loses_the_player_goes_to_the_last_seen_spot_then_gives_up() {
    let r = rig();
    let mut brain = make_brain(&r, 1);
    let mut body = start();
    let player = Vec3::new(10.0, 0.0, 4.0); // 6 m ahead
    for _ in 0..10 {
        tick(&mut brain, &mut body, &r.world, Some(player));
    }
    assert_eq!(brain.state(), ZombieState::Chase);

    // Then the player sprints away (7 m/s, faster than the zombie) until far out of sight. A real
    // player cannot teleport: sight is re-checked every 0.2 s (as in the C#), so a jump would be "seen".
    let mut runner = player;
    let mut t = 0.0_f32;
    let mut seen: Vec<ZombieState> = Vec::new();
    let mut investigate_at = None;
    while t < 60.0 {
        if runner.z > -200.0 {
            runner.z -= 7.0 * DT;
        }
        let out = tick(&mut brain, &mut body, &r.world, Some(runner));
        t += DT;
        assert_ne!(
            out.state,
            ZombieState::Attack,
            "a player out of view is not bitten"
        );
        if seen.last() != Some(&out.state) {
            seen.push(out.state);
        }
        if out.state == ZombieState::Investigate && investigate_at.is_none() {
            investigate_at = Some(t);
        }
        if out.state == ZombieState::Wander {
            break;
        }
    }
    assert_eq!(
        seen,
        vec![
            ZombieState::Chase,
            ZombieState::Investigate,
            ZombieState::Wander
        ]
    );

    // It keeps chasing for the memory time (6 s) after losing sight. The runner stays in view for about
    // 3 s first (from 6 m apart to past the 18 m sight range at 7 vs 2.6 m/s), so allow up to 4.5 s more.
    let lose = r.def.lose_target_time;
    let at = investigate_at.expect("should give up the chase");
    assert!(
        (lose..=lose + 4.5).contains(&at),
        "gave up at {at} s, memory is {lose} s"
    );
}

#[test]
fn a_light_hit_flinches_the_zombie_and_alerts_it() {
    let r = rig();
    let mut brain = make_brain(&r, 1);
    let mut body = start();
    let attacker = Vec3::new(10.0, 0.0, -200.0); // out of view, but the hit says where it came from
    let light = DamageInfo::new(10.0);
    assert!(
        light.amount < r.tuning.strong_hit_damage,
        "test needs a light hit"
    );

    let ctx = BrainInput {
        took_damage: Some(light),
        ..sense(body, &r.world, Some(attacker))
    };
    let out = run_frame(&mut brain, &mut body, ctx);
    assert_eq!(out.anim, AnimClip::Hit);
    assert!(!out.staggered, "a light hit does not stagger");
    assert_eq!(out.state, ZombieState::Chase, "a hit alerts the zombie");

    let next = tick(&mut brain, &mut body, &r.world, Some(attacker));
    assert_ne!(next.anim, AnimClip::Hit, "the flinch is one frame");
}

#[test]
fn a_strong_hit_staggers_the_zombie_for_a_moment_then_it_walks_on() {
    let r = rig();
    let mut brain = make_brain(&r, 1);
    let mut body = start();
    let player = Vec3::new(10.0, 0.0, -5.0); // 15 m ahead: in view, out of bite range
    for _ in 0..60 {
        tick(&mut brain, &mut body, &r.world, Some(player));
    }
    let before = tick(&mut brain, &mut body, &r.world, Some(player));
    let speed_before = before.velocity.length();
    assert!(
        speed_before > 2.0,
        "should be chasing at speed, got {speed_before}"
    );

    let strong = DamageInfo::new(r.tuning.strong_hit_damage);
    let ctx = BrainInput {
        took_damage: Some(strong),
        ..sense(body, &r.world, Some(player))
    };
    let hit = run_frame(&mut brain, &mut body, ctx);
    assert!(hit.staggered, "a strong hit staggers");
    assert_eq!(hit.anim, AnimClip::Hit);
    assert!(
        hit.velocity.length() < speed_before * 0.9,
        "the walk eases to a stop, got {:?} from {speed_before}",
        hit.velocity
    );

    // Counts the frames until the stagger ends: about the stagger time from the data
    let mut staggered_frames = 0;
    loop {
        let out = tick(&mut brain, &mut body, &r.world, Some(player));
        if !out.staggered {
            break;
        }
        staggered_frames += 1;
        assert!(staggered_frames < 600, "the stagger never ended");
    }
    let secs = (staggered_frames + 1) as f32 * DT;
    let stagger = r.tuning.stagger_fallback_seconds;
    assert!(
        (secs - stagger).abs() <= 2.0 * DT,
        "stagger lasted {secs} s, data says {stagger} s"
    );

    let mut last = hit;
    for _ in 0..60 {
        last = tick(&mut brain, &mut body, &r.world, Some(player));
    }
    assert!(
        last.velocity.length() > 2.0,
        "walks on after the stagger, got {:?}",
        last.velocity
    );
}

#[test]
fn zero_health_is_dead_and_the_zombie_stops_for_good() {
    let r = rig();
    let mut brain = make_brain(&r, 1);
    let mut body = start();
    let player = Vec3::new(10.0, 0.0, 4.0);
    for _ in 0..60 {
        tick(&mut brain, &mut body, &r.world, Some(player));
    }

    let ctx = BrainInput {
        is_dead: true,
        ..sense(body, &r.world, Some(player))
    };
    let out = run_frame(&mut brain, &mut body, ctx);
    assert_eq!(out.state, ZombieState::Dead);
    assert_eq!(out.velocity, Vec3::ZERO);
    assert_eq!(out.anim, AnimClip::Die);
    assert!(out.attack_hit.is_none());

    let resting = body;
    for _ in 0..120 {
        let o = tick(&mut brain, &mut body, &r.world, Some(player));
        assert_eq!(o.state, ZombieState::Dead, "death is final");
        assert_eq!(o.velocity, Vec3::ZERO);
    }
    assert_eq!(body.pos, resting.pos, "a dead zombie does not move");
}

#[test]
fn the_same_seed_gives_the_same_wander_path() {
    let r = rig();
    let walk = |seed: u64| -> Vec<(Vec3, Vec3, f32)> {
        let mut brain = make_brain(&r, seed);
        let mut body = start();
        (0..60 * 40)
            .map(|_| {
                let out = tick(&mut brain, &mut body, &r.world, None);
                (body.pos, out.velocity, out.yaw)
            })
            .collect()
    };

    let a = walk(7);
    let b = walk(7);
    let c = walk(8);
    assert_eq!(a, b, "the same seed must give the same path");
    assert_ne!(a, c, "a different seed should wander differently");

    let home = start().pos;
    assert!(
        a.iter().any(|(p, _, _)| (*p - home).length() > 1.0),
        "the zombie should have wandered off"
    );
}

