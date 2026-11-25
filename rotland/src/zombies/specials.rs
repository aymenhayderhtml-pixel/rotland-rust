//! Live special zombies (brute, screamer, crawler, frozen, husk, bloater): runs the pure `SpecialState`
//! each frame and applies its directive to the zombie's brain output, body and call / damage / burst queues.

use game::zombie::senses::{CHEST_HEIGHT, EYE_HEIGHT};
use game::zombie::specials::SpecialPose;
use game::zombie::{AnimClip, BrainOutput, SpecialConfig, SpecialDirective};
use glam::{Vec2, Vec3};
use physics::PhysicsWorld;

use super::types::{ToxicBurst, Zombie, ZombieFrame};

/// Line-of-sight checks for specials run this often (4 times a second). // new: DECISIONS 2026-11-23
pub const SIGHT_EVERY: f32 = 0.25;
/// A special that moved less than this share of its wanted distance hit a wall. // new: DECISIONS 2026-11-23
pub const WALL_RATIO: f32 = 0.5;
/// Radius of the old, untuned screamer call. Only used when data/special_zombies.json fails to load.
/// // new: DECISIONS 2026-11-23 (old hardcoded 60.0, fallback only)
pub const LEGACY_CALL_RADIUS: f32 = 60.0;

/// Default bloater toxic cloud radius in metres. // new: DECISIONS 2026-11-25
pub const BLOATER_RADIUS: f32 = 4.0;
/// Default bloater toxic cloud duration in seconds. // new: DECISIONS 2026-11-25
pub const BLOATER_DURATION: f32 = 6.0;
/// Default bloater toxic cloud damage per second to the player. // new: DECISIONS 2026-11-25
pub const BLOATER_PLAYER_DPS: f32 = 4.0;
/// Default bloater toxic cloud damage per second to other zombies. // new: DECISIONS 2026-11-25
pub const BLOATER_ZOMBIE_DPS: f32 = 2.0;

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

/// Builds a `ToxicBurst` at `position` with the standard bloater tuning values.
pub fn bloater_burst_at(position: Vec3) -> ToxicBurst {
    ToxicBurst {
        position,
        radius: BLOATER_RADIUS,
        duration: BLOATER_DURATION,
        player_dps: BLOATER_PLAYER_DPS,
        zombie_dps: BLOATER_ZOMBIE_DPS,
    }
}

/// Emits a `ToxicBurst` once when a bloater dies or is struck by fire.
pub fn maybe_bloater_burst(z: &mut Zombie, on_fire: bool, frame: &mut ZombieFrame) {
    if z.kind != "bloater" || z.bloater_burst_done {
        return;
    }
    if z.health.is_dead() || on_fire {
        z.bloater_burst_done = true;
        frame.toxic_bursts.push(bloater_burst_at(z.position));
    }
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
/// Both crawler and husk use `SpecialPose::Lunge` -> `AnimClip::Attack` (crawler overrides the clip via `clips`).
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

/// Xorshift roll in [0, 1) for the crawler and husk lunge chance. Advances `state`.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bloater_burst_matches_tuning() {
        let pos = Vec3::new(3.0, 1.0, -4.0);
        let burst = bloater_burst_at(pos);
        assert_eq!(burst.position, pos);
        assert!((burst.radius - 4.0).abs() < 1e-5);
        assert!((burst.duration - 6.0).abs() < 1e-5);
        assert!((burst.player_dps - 4.0).abs() < 1e-5);
        assert!((burst.zombie_dps - 2.0).abs() < 1e-5);
    }
}
