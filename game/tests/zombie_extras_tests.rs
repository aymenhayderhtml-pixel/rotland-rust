//! Tests for zombie extras (R3-09): scream, eat, bang, crawler, steering, and stuck escape.
//! Pure: runs on a PhysicsWorld with a flat chunk. Numbers come from data/zombies.json.

use std::sync::{Arc, Mutex};

use game::DamageInfo;
use game::zombie::{
    AnimClip, BrainInput, BrainOutput, HerdZombie, Noise,
    ZombieBehaviourDef, ZombieBrain, ZombieCorpses, ZombieDef, ZombieDefs,
    ZombieHorde, ZombieState,
};
use glam::{IVec2, Vec3};
use physics::{CHUNK_SAMPLES, PhysicsWorld};

const DT: f32 = 1.0 / 60.0;
const WALKER: &str = "walker";

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

fn flat_world() -> PhysicsWorld {
    let mut world = PhysicsWorld::new();
    let heights = vec![0.0f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
    world.add_ground_chunk(IVec2::new(0, 0), &heights);
    world
}

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

fn make_brain(r: &Rig, pos: Vec3, seed: u64) -> ZombieBrain {
    // Screams and eating are random in the game; tests switch them off unless a test turns them on
    // (as Godot's tests did with ScreamChanceForTest = 0)
    let mut brain = ZombieBrain::new(&r.def, &r.tuning, pos, seed);
    brain.set_scream_chance(0.0);
    brain.set_eat_chance_override(Some(0.0));
    brain
}

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

fn run_frame(brain: &mut ZombieBrain, body: &mut Body, ctx: BrainInput<'_>) -> BrainOutput {
    let out = brain.update(&ctx, DT);
    body.pos += out.velocity * DT;
    body.yaw = out.yaw;
    out
}

fn tick(
    brain: &mut ZombieBrain,
    body: &mut Body,
    world: &PhysicsWorld,
    target: Option<Vec3>,
) -> BrainOutput {
    let ctx = sense(*body, world, target);
    run_frame(brain, body, ctx)
}

fn make_box(hx: f32, hy: f32, hz: f32) -> Vec<[f32; 3]> {
    vec![
        [-hx, 0.0, -hz],
        [hx, 0.0, -hz],
        [hx, hy, -hz],
        [-hx, hy, -hz],
        [-hx, 0.0, hz],
        [hx, 0.0, hz],
        [hx, hy, hz],
        [-hx, hy, hz],
    ]
}

#[test]
fn a_first_sighting_can_trigger_a_scream_and_the_scream_alerts_a_nearby_idle_zombie() {
    let r = rig();
    let mut screamer_brain = make_brain(&r, start().pos, 1);
    screamer_brain.set_scream_chance(1.0);
    let mut screamer_body = start();

    let bystander_pos = Vec3::new(14.0, 0.0, 10.0);
    let mut bystander_brain = make_brain(&r, bystander_pos, 2);
    let mut bystander_body = Body {
        pos: bystander_pos,
        yaw: 0.0,
    };

    let player = Vec3::new(10.0, 0.0, 4.0); // 6 m ahead of screamer

    // Frame 1: screamer sees player and screams
    let out_scream = tick(&mut screamer_brain, &mut screamer_body, &r.world, Some(player));
    assert_eq!(out_scream.state, ZombieState::Scream); // assert 1
    assert!(out_scream.scream); // assert 2: scream event fired
    assert_eq!(out_scream.anim, AnimClip::Scream); // assert 3
    assert!(out_scream.velocity.length() < 0.1); // assert 4: stands still while screaming

    // The scream raises a sound at the player within scream_alert_radius (12 m)
    let scream_noise = [Noise {
        position: player,
        radius: r.tuning.scream_alert_radius,
    }];
    let bystander_ctx = BrainInput {
        heard: &scream_noise,
        ..sense(bystander_body, &r.world, None)
    };
    let out_bystander = run_frame(&mut bystander_brain, &mut bystander_body, bystander_ctx);
    assert_eq!(out_bystander.state, ZombieState::Investigate); // assert 5: alerted by scream
    assert_eq!(out_bystander.anim, AnimClip::Walk); // assert 6
    assert!(out_bystander.velocity.length() > 0.1); // assert 7: starts walking toward sound

    // Screamer stays in scream state through scream_seconds (1.4 s = ~84 frames)
    let scream_frames = (r.tuning.scream_seconds / DT) as usize;
    let mut out = out_scream;
    for _ in 0..scream_frames.saturating_sub(2) {
        out = tick(&mut screamer_brain, &mut screamer_body, &r.world, Some(player));
        assert_eq!(out.state, ZombieState::Scream); // assert 8: holds scream
    }

    // Once the scream ends, it enters chase
    for _ in 0..5 {
        out = tick(&mut screamer_brain, &mut screamer_body, &r.world, Some(player));
    }
    assert_eq!(out.state, ZombieState::Chase); // assert 9: chases after scream
    assert_eq!(out.anim, AnimClip::Run); // assert 10
    assert!(out.velocity.length() > 1.0); // assert 11: running speed

    // Also verify herd detection: idle bystander joins a chasing zombie nearby
    let chasing_info = [HerdZombie::new(
        screamer_body.pos,
        screamer_brain.state(),
        player,
    )];
    let mut idle_herd_brain = make_brain(&r, Vec3::new(12.0, 0.0, 10.0), 3);
    let mut idle_herd_body = Body {
        pos: Vec3::new(12.0, 0.0, 10.0),
        yaw: 0.0,
    };
    let herd_ctx = BrainInput {
        nearby_zombies: &chasing_info,
        ..sense(idle_herd_body, &r.world, None)
    };
    let herd_out = run_frame(&mut idle_herd_brain, &mut idle_herd_body, herd_ctx);
    assert_eq!(herd_out.state, ZombieState::Chase); // assert 12: joined chase via herd
}

#[test]
fn a_zombie_near_a_corpse_with_no_target_eats_it() {
    let r = rig();
    let mut brain = make_brain(&r, start().pos, 1);
    brain.set_eat_chance_override(Some(1.0)); // 100% chance to eat when pause ends
    let mut body = start();

    let corpse_pos = Vec3::new(11.0, 0.0, 10.0); // 1 m away, within 4 m corpse_spot_radius
    let corpses = [corpse_pos];

    // Tick until idle countdown ends and zombie decides to eat
    let mut entered_eat = false;
    for _ in 0..200 {
        let ctx = BrainInput {
            corpses: &corpses,
            ..sense(body, &r.world, None)
        };
        let out = run_frame(&mut brain, &mut body, ctx);
        if out.state == ZombieState::Eat {
            entered_eat = true;
            assert_eq!(out.anim, AnimClip::Eat); // assert 1
            assert_eq!(out.velocity, Vec3::ZERO); // assert 2: motionless eating
            break;
        }
    }
    assert!(entered_eat, "zombie near corpse should enter Eat"); // assert 3

    // Stays eating in place for multiple frames
    let eat_pos = body.pos;
    for _ in 0..60 {
        let ctx = BrainInput {
            corpses: &corpses,
            ..sense(body, &r.world, None)
        };
        let out = run_frame(&mut brain, &mut body, ctx);
        assert_eq!(out.state, ZombieState::Eat); // assert 4
        assert_eq!(out.anim, AnimClip::Eat); // assert 5
        assert_eq!(body.pos, eat_pos); // assert 6: doesn't move
    }

    // Spotting a player interrupts eating immediately
    brain.set_scream_chance(0.0); // go straight to chase
    let player = Vec3::new(10.0, 0.0, 5.0); // 5 m ahead
    let ctx = BrainInput {
        corpses: &corpses,
        ..sense(body, &r.world, Some(player))
    };
    let out = run_frame(&mut brain, &mut body, ctx);
    assert_eq!(out.state, ZombieState::Chase); // assert 7: interrupted eating to chase
    assert_ne!(out.anim, AnimClip::Eat); // assert 8

    // Test ZombieCorpses helper
    let mut recorder = ZombieCorpses::new();
    recorder.record(corpse_pos, 10.0);
    assert!(recorder.near_recent(body.pos, 4.0, 120.0, 15.0)); // assert 9: 5s old, within 4m
    assert!(!recorder.near_recent(body.pos, 0.5, 120.0, 15.0)); // assert 10: out of radius
    assert!(!recorder.near_recent(body.pos, 4.0, 2.0, 15.0)); // assert 11: expired
}

#[test]
fn a_player_standing_on_a_high_box_unreachable_gets_banged_for() {
    let mut r = rig();
    let box_pos = Vec3::new(10.0, 0.0, 4.0);
    let box_points = make_box(1.0, 3.0, 1.0); // 2x3x2 m box
    r.world.add_static(&[box_points], box_pos, 0.0, 1.0);

    let mut brain = make_brain(&r, start().pos, 1);
    let mut body = start();
    let player_on_box = Vec3::new(10.0, 3.0, 4.0);

    // Initial sight: starts chase
    tick(&mut brain, &mut body, &r.world, Some(player_on_box));
    assert_eq!(brain.state(), ZombieState::Chase); // assert 1

    // Player went out of reach on the box; zombie loses sight at the foot of the box
    brain.lose_sight_for_test(box_pos);
    let ctx = BrainInput {
        target_unreachable: true,
        ..sense(body, &r.world, Some(player_on_box))
    };
    let first = run_frame(&mut brain, &mut body, ctx);
    assert_eq!(first.state, ZombieState::Investigate); // assert 2: walks to last seen spot

    // Walks to the box foot; upon arriving, enters Bang instead of looking around
    let mut reached_bang = false;
    for _ in 0..300 {
        let ctx = BrainInput {
            target_unreachable: true,
            ..sense(body, &r.world, Some(player_on_box))
        };
        let out = run_frame(&mut brain, &mut body, ctx);
        if out.state == ZombieState::Bang {
            reached_bang = true;
            assert_eq!(out.anim, AnimClip::Bang); // assert 3
            assert_eq!(out.velocity, Vec3::ZERO); // assert 4
            let dist_to_foot = Vec3::new(body.pos.x - box_pos.x, 0.0, body.pos.z - box_pos.z).length();
            assert!(dist_to_foot <= 1.5, "banging close to box, got {dist_to_foot}"); // assert 5
            break;
        }
    }
    assert!(reached_bang, "should start banging at unreachable target"); // assert 6

    // Holds Bang state for bang_seconds (20 s)
    let bang_frames = (18.0 / DT) as usize;
    for _ in 0..bang_frames {
        let ctx = BrainInput {
            target_unreachable: true,
            ..sense(body, &r.world, Some(player_on_box))
        };
        let out = run_frame(&mut brain, &mut body, ctx);
        assert_eq!(out.state, ZombieState::Bang); // assert 7: still banging
        assert_eq!(out.anim, AnimClip::Bang); // assert 8
    }

    // After 20 seconds, gives up and wanders
    for _ in 0..(4.0 / DT) as usize {
        let ctx = BrainInput {
            target_unreachable: true,
            ..sense(body, &r.world, Some(player_on_box))
        };
        let out = run_frame(&mut brain, &mut body, ctx);
        if out.state == ZombieState::Wander {
            assert_eq!(out.anim, AnimClip::Walk); // assert 9
            break;
        }
    }
    assert_eq!(brain.state(), ZombieState::Wander); // assert 10
}

#[test]
fn a_leg_hit_with_enough_damage_makes_a_crawler_that_moves_slower() {
    let r = rig();
    let mut brain = make_brain(&r, start().pos, 1);
    let mut body = start();
    let player = Vec3::new(10.0, 0.0, -20.0);

    // Running normally first
    let mut out = tick(&mut brain, &mut body, &r.world, Some(player));
    for _ in 0..60 {
        out = tick(&mut brain, &mut body, &r.world, Some(player));
    }
    assert!(!brain.is_crawler()); // assert 1
    assert_eq!(out.state, ZombieState::Chase); // assert 2
    assert_eq!(out.anim, AnimClip::Run); // assert 3
    let normal_speed = out.velocity.length();
    assert!(normal_speed > 2.0, "normal speed was {normal_speed}"); // assert 4

    // Apply a leg hit with strong damage (>= 20.0)
    let leg_damage = DamageInfo {
        amount: 25.0,
        leg_hit: true,
        hit_point: body.pos + Vec3::new(0.0, 0.2, 0.0),
        ..DamageInfo::new(25.0)
    };
    let hit_ctx = BrainInput {
        took_damage: Some(leg_damage),
        ..sense(body, &r.world, Some(player))
    };
    let hit_out = run_frame(&mut brain, &mut body, hit_ctx);
    assert!(brain.is_crawler()); // assert 5: became crawler
    assert_eq!(hit_out.anim, AnimClip::Hit); // assert 6: staggered
    assert!(hit_out.staggered); // assert 7

    // Wait out the stagger
    let mut last = hit_out;
    while last.staggered {
        last = tick(&mut brain, &mut body, &r.world, Some(player));
    }
    assert!(!last.staggered); // assert 8

    // Now chasing as a crawler: slower speed and crawl clip
    for _ in 0..60 {
        last = tick(&mut brain, &mut body, &r.world, Some(player));
    }
    assert_eq!(last.state, ZombieState::Chase); // assert 9
    assert_eq!(last.anim, AnimClip::Crawl); // assert 10: crawl clip!
    let crawl_speed = last.velocity.length();
    let expected_speed = r.tuning.crawler_speed; // 0.8 m/s
    assert!(
        (crawl_speed - expected_speed).abs() < 0.1,
        "crawl speed {crawl_speed} should match crawler_speed {expected_speed}"
    ); // assert 11
    assert!(crawl_speed < normal_speed * 0.5); // assert 12: much slower
}

#[test]
fn a_zombie_walking_into_a_wall_steers_around_it_not_through_it() {
    let mut r = rig();
    // Wall blocking direct path from (10, 0, 10) to (10, 0, 0):
    // Wall center at (10, 0, 5), width 4m (X: 8..12), thickness 0.4m (Z: 4.8..5.2), height 3m
    let wall_pos = Vec3::new(10.0, 0.0, 5.0);
    let wall_box = make_box(2.0, 3.0, 0.2);
    r.world.add_static(&[wall_box], wall_pos, 0.0, 1.0);

    let mut brain = make_brain(&r, start().pos, 1);
    let mut body = start();
    let goal = Vec3::new(10.0, 0.0, 0.0);

    // Goal behind the wall: zombie chases/investigates toward goal
    brain.force_state_for_test(ZombieState::Investigate);
    brain.set_goal_for_test(goal);

    let mut steered_sideways = false;
    let mut passed_wall = false;
    let mut frames = 0;

    while frames < 600 {
        // Up to 10 seconds
        let ctx = BrainInput {
            target: Some(goal),
            ..sense(body, &r.world, Some(goal))
        };
        let out = run_frame(&mut brain, &mut body, ctx);
        frames += 1;

        // Verify it NEVER steps inside the wall volume (X in 8.1..11.9 and Z in 4.85..5.15)
        let inside_wall = body.pos.x > 8.1 && body.pos.x < 11.9 && body.pos.z > 4.85 && body.pos.z < 5.15;
        assert!(!inside_wall, "zombie walked through the wall at {:?}", body.pos); // assert 1 per frame

        if out.velocity.x.abs() > 0.2 {
            steered_sideways = true;
        }

        if body.pos.z < 4.0 {
            passed_wall = true;
            break;
        }
    }

    assert!(steered_sideways, "feeler rays should steer sideways around the wall"); // assert 2
    assert!(passed_wall, "zombie should reach the other side of the wall (z < 4.0) within 10s"); // assert 3
    assert!(body.pos.z < 4.0); // assert 4
}

#[test]
fn a_stuck_zombie_escapes() {
    let mut r = rig();
    // Build a 3-sided trap blocking Left, Front, Right:
    // Zombie starts at (10, 0, 10) facing -Z.
    // Front wall at z = 9.0 (blocking -Z)
    let front_pos = Vec3::new(10.0, 0.0, 9.0);
    r.world.add_static(&[make_box(1.5, 2.0, 0.2)], front_pos, 0.0, 1.0);
    // Left wall at x = 9.0 (blocking -X)
    let left_pos = Vec3::new(9.0, 0.0, 10.0);
    r.world.add_static(&[make_box(0.2, 2.0, 1.5)], left_pos, 0.0, 1.0);
    // Right wall at x = 11.0 (blocking +X)
    let right_pos = Vec3::new(11.0, 0.0, 10.0);
    r.world.add_static(&[make_box(0.2, 2.0, 1.5)], right_pos, 0.0, 1.0);

    let mut brain = make_brain(&r, start().pos, 1);
    let mut body = start();
    let goal = Vec3::new(10.0, 0.0, 0.0); // directly ahead beyond front wall

    // Zombie wants to walk toward goal
    brain.force_state_for_test(ZombieState::Investigate);
    brain.set_goal_for_test(goal);

    assert!(!brain.is_escaping()); // assert 1: not escaping initially

    // Tick into the trap: front and sides are blocked (< 0.5m boxed clearance)
    let mut saw_escape = false;
    let mut escaped_pos = body.pos;
    for _ in 0..120 {
        let ctx = BrainInput {
            target: Some(goal),
            ..sense(body, &r.world, Some(goal))
        };
        let out = run_frame(&mut brain, &mut body, ctx);
        if brain.is_escaping() {
            saw_escape = true;
            // The only open side is backward (+Z)
            assert!(out.velocity.length() > 0.1); // assert 2: moving during escape
            assert!(out.velocity.z > 0.0, "escape direction should head backward (+Z), got {:?}", out.velocity); // assert 3
            escaped_pos = body.pos;
            break;
        }
    }
    assert!(saw_escape, "stuck zombie should trigger escape"); // assert 4

    // Run through the escape duration (~1.2s = 72 frames)
    for _ in 0..80 {
        let ctx = BrainInput {
            target: Some(goal),
            ..sense(body, &r.world, Some(goal))
        };
        run_frame(&mut brain, &mut body, ctx);
    }
    // The zombie backed out of the trap (+Z)
    assert!(body.pos.z > 10.1, "zombie moved out of trap, pos is {:?}", body.pos); // assert 5

    // Also test ZombieHorde shared wandering
    let horde = Arc::new(Mutex::new(ZombieHorde::new()));
    let mut horde_zombie_1 = make_brain(&r, Vec3::new(10.0, 0.0, 10.0), 10);
    let mut horde_zombie_2 = make_brain(&r, Vec3::new(11.0, 0.0, 10.0), 20);
    horde_zombie_1.set_horde(Some(Arc::clone(&horde)));
    horde_zombie_2.set_horde(Some(Arc::clone(&horde)));

    horde_zombie_1.force_state_for_test(ZombieState::Wander);
    let shared = horde.lock().unwrap().wander_target;
    assert!(shared.is_some()); // assert 6: first member picked shared spot

    horde_zombie_2.force_state_for_test(ZombieState::Wander);
    let target_2 = horde_zombie_2.goal();
    let shared_pt = shared.unwrap();
    let diff = Vec3::new(target_2.x - shared_pt.x, 0.0, target_2.z - shared_pt.z).length();
    assert!(diff <= 2.5, "second zombie wanders near shared spot, diff is {diff}"); // assert 7
}
