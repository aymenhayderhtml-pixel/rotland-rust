//! Live special zombies (brute, screamer, crawler): runs the pure `SpecialState` each frame and applies
//! its directive to the zombie's brain output, body and the call / damage queues. R25-N7.

use game::zombie::senses::{CHEST_HEIGHT, EYE_HEIGHT};
use game::zombie::specials::SpecialPose;
use game::zombie::{AnimClip, BrainOutput, SpecialConfig, SpecialDirective};
use glam::{Vec2, Vec3};
use physics::PhysicsWorld;

use super::types::{Zombie, ZombieFrame};

/// Line-of-sight checks for specials run this often (4 times a second). // new: DECISIONS 2026-11-23
pub const SIGHT_EVERY: f32 = 0.25;
/// A special that moved less than this share of its wanted distance hit a wall. // new: DECISIONS 2026-11-23
pub const WALL_RATIO: f32 = 0.5;
/// Radius of the old, untuned screamer call. Only used when data/special_zombies.json fails to load.
/// // new: DECISIONS 2026-11-23 (old hardcoded 60.0, fallback only)
pub const LEGACY_CALL_RADIUS: f32 = 60.0;

/// A screamer call waiting for the end of the frame (all zombies have moved by then).
#[derive(Clone, Copy, Debug)]
pub struct PendingCall {
    /// Id of the zombie that called (never alerts itself).
    pub source: u64,
    pub origin: Vec3,
    pub target: Vec3,
    pub radius: f32,
    pub max: usize,
}

/// Runs a special zombie for one frame and applies its directive to `out`, `frame` and `calls`.
/// Only called for zombies with `special` set and a player to look at.
#[allow(clippy::too_many_arguments)]
pub fn step(
    z: &mut Zombie,
    cfg: &SpecialConfig,
    physics: &PhysicsWorld,
    player: Vec3,
    dt: f32,
    roll: f32,
    out: &mut BrainOutput,
    frame: &mut ZombieFrame,
    calls: &mut Vec<PendingCall>,
) {
    let gate = cfg
        .brute
        .charge_max_range
        .max(cfg.screamer.trigger_range)
        .max(cfg.crawler.lurk_range);
    z.special_sight_timer -= dt;
    if z.special_sight_timer <= 0.0 {
        z.special_sight_timer = SIGHT_EVERY;
        z.special_los = z.position.distance(player) <= gate && sight_clear(physics, z.position, player);
    }
    let Some(state) = z.special.as_mut() else {
        return;
    };
    let d = state.tick(cfg, z.position, player, z.special_los, z.special_hit_wall, dt, roll);
    z.special_vulnerable = d.vulnerable;
    apply(&d, z.id, z.position, out, frame, calls);
}

/// Copies one directive onto the zombie's frame output.
fn apply(
    d: &SpecialDirective,
    id: u64,
    position: Vec3,
    out: &mut BrainOutput,
    frame: &mut ZombieFrame,
    calls: &mut Vec<PendingCall>,
) {
    if let Some(v) = d.velocity {
        out.velocity = horizontal(v);
        out.attack_hit = None; // the special's own hit replaces the normal bite
    }
    if let Some(f) = d.face {
        // Godot convention: forward = (-sin yaw, -cos yaw)
        out.yaw = (-f.x).atan2(-f.z);
    }
    if let Some(anim) = pose_clip(d.pose) {
        out.anim = anim;
    }
    if let Some(hit) = d.damage {
        frame.special_hits.push(hit);
    }
    if let Some(call) = d.call {
        calls.push(PendingCall {
            source: id,
            origin: position,
            target: call.target,
            radius: call.radius,
            max: call.max_zombies as usize,
        });
    }
}

/// Maps a special pose to the nearest existing clip. `Normal` leaves the choice to the brain.
fn pose_clip(pose: SpecialPose) -> Option<AnimClip> {
    Some(match pose {
        SpecialPose::Normal => return None,
        SpecialPose::Windup | SpecialPose::Lurk => AnimClip::Idle,
        SpecialPose::Charge => AnimClip::Run,
        SpecialPose::Stunned | SpecialPose::Recover => AnimClip::Hit,
        SpecialPose::Lunge => AnimClip::Attack,
        SpecialPose::Scream => AnimClip::Scream,
        SpecialPose::Retreat => AnimClip::Walk,
    })
}

/// Remembers after the move whether a special with a wish to move got blocked (for the charge rule).
pub fn note_wall(z: &mut Zombie, walk: Vec3, new_position: Vec3, dt: f32) {
    if z.special.is_none() {
        return;
    }
    let want = horizontal(walk).length() * dt;
    let got = Vec2::new(new_position.x - z.position.x, new_position.z - z.position.z).length();
    z.special_hit_wall = want > 1e-3 && got < want * WALL_RATIO;
}

/// Alerts the living zombies within each call's radius, closest first, at most `max` of them.
pub fn apply_calls(list: &mut [Zombie], calls: &[PendingCall]) {
    for c in calls {
        let r2 = c.radius * c.radius;
        let mut near: Vec<(f32, usize)> = list
            .iter()
            .enumerate()
            .filter(|(_, z)| z.id != c.source && !z.health.is_dead())
            .filter_map(|(i, z)| {
                let d2 = Vec2::new(z.position.x - c.origin.x, z.position.z - c.origin.z).length_squared();
                (d2 <= r2).then_some((d2, i))
            })
            .collect();
        near.sort_by(|a, b| a.0.total_cmp(&b.0));
        for &(_, i) in near.iter().take(c.max) {
            list[i].brain.alert_chase(c.target);
        }
    }
}

/// Xorshift roll in [0, 1) for the crawler's lunge chance. Advances `state`.
pub fn next_roll(state: &mut u64) -> f32 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    (*state >> 40) as f32 / (1u64 << 24) as f32
}

/// Eye-to-chest ray against the world only (characters never block it).
fn sight_clear(physics: &PhysicsWorld, feet: Vec3, target: Vec3) -> bool {
    let eye = feet + Vec3::Y * EYE_HEIGHT;
    let offset = target + Vec3::Y * CHEST_HEIGHT - eye;
    let length = offset.length();
    if length < 1e-4 {
        return true;
    }
    physics.raycast(eye, offset / length, length).is_none()
}

fn horizontal(v: Vec3) -> Vec3 {
    Vec3::new(v.x, 0.0, v.z)
}
