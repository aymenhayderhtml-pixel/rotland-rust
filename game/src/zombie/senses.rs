//! Sight, hearing and memory for one zombie (port of ZombieSenses.cs and the hearing rule in Zombie.cs).
//! The Godot version has no day/night difference, so none is modelled here.

use glam::{Vec2, Vec3};
use physics::PhysicsWorld;

use super::defs::ZombieDef;

/// Sight is checked this often, not every frame, to save ray casts.
pub const CHECK_INTERVAL: f32 = 0.2;
/// Within this flat distance the field-of-view cone is ignored: a zombie senses behind it up close.
pub const ALWAYS_SEE_RANGE: f32 = 2.5;
/// The sight ray starts at the zombie's eyes and ends at the target's chest.
pub const EYE_HEIGHT: f32 = 1.6;
pub const CHEST_HEIGHT: f32 = 1.2;

/// A sound. Zombies within `radius * hearing_multiplier` of `position` hear it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Noise {
    pub position: Vec3,
    pub radius: f32,
}

/// True when a zombie with feet at `feet` hears `noise` (3D distance, same as the Godot rule).
pub fn hears(feet: Vec3, noise: &Noise, hearing_multiplier: f32) -> bool {
    feet.distance(noise.position) <= noise.radius * hearing_multiplier
}

/// Sight and memory for one zombie. Call `update` every frame.
#[derive(Clone, Debug)]
pub struct Senses {
    sight_range: f32,
    sight_fov_deg: f32,
    lose_target_time: f32,
    /// Counts down to the next sight check.
    timer: f32,
    can_see: bool,
    last_seen: Vec3,
    /// Seconds since the target was last seen. Zero while seen.
    time_since_seen: f32,
}

impl Senses {
    pub fn new(def: &ZombieDef) -> Self {
        Self {
            sight_range: def.sight_range,
            sight_fov_deg: def.sight_fov_deg,
            lose_target_time: def.lose_target_time,
            timer: 0.0,
            can_see: false,
            last_seen: Vec3::ZERO,
            time_since_seen: 0.0,
        }
    }

    pub fn can_see(&self) -> bool {
        self.can_see
    }

    pub fn last_seen(&self) -> Vec3 {
        self.last_seen
    }

    pub fn time_since_seen(&self) -> f32 {
        self.time_since_seen
    }

    /// The zombie has lost sight for longer than `lose_target_time`. It then walks to `last_seen`
    /// instead of chasing. The memory itself is kept until a new sighting.
    pub fn memory_expired(&self) -> bool {
        self.time_since_seen > self.lose_target_time
    }

    /// A hit tells the zombie where its attacker is, even when sight fails.
    pub fn notice_at(&mut self, position: Vec3) {
        self.last_seen = position;
        self.time_since_seen = 0.0;
    }

    /// Forget everything. Used when a pooled zombie is handed out again.
    pub fn reset(&mut self) {
        self.timer = 0.0;
        self.can_see = false;
        self.last_seen = Vec3::ZERO;
        self.time_since_seen = 0.0;
    }

    /// `feet` is the zombie's position on the ground, `facing` the way it faces (height is ignored),
    /// `target` the target's feet position, or None when there is no living target.
    pub fn update(
        &mut self,
        dt: f32,
        feet: Vec3,
        facing: Vec3,
        target: Option<Vec3>,
        world: &PhysicsWorld,
    ) {
        let Some(target) = target else {
            self.can_see = false;
            self.time_since_seen += dt;
            return;
        };

        // Only re-checks sight every CHECK_INTERVAL; between checks the last answer holds
        self.timer -= dt;
        if self.timer <= 0.0 {
            self.timer = CHECK_INTERVAL;
            self.can_see = self.sees(feet, facing, target, world);
        }

        if self.can_see {
            self.last_seen = target;
            self.time_since_seen = 0.0;
        } else {
            self.time_since_seen += dt;
        }
    }

    fn sees(&self, feet: Vec3, facing: Vec3, target: Vec3, world: &PhysicsWorld) -> bool {
        let to_target = Vec2::new(target.x - feet.x, target.z - feet.z);
        let distance = to_target.length();
        if distance > self.sight_range {
            return false;
        }

        // Up close the zombie senses all around it, but walls still block (checked below)
        if distance > ALWAYS_SEE_RANGE {
            let forward = Vec2::new(facing.x, facing.z);
            let half_fov = (self.sight_fov_deg * 0.5).to_radians();
            if angle_between(forward, to_target) > half_fov {
                return false;
            }
        }

        let eye = feet + Vec3::Y * EYE_HEIGHT;
        let chest = target + Vec3::Y * CHEST_HEIGHT;
        let offset = chest - eye;
        let length = offset.length();
        // Any world hit on the way means a wall or prop blocks the view. Characters are not
        // in the world group, so they never block it.
        world.raycast(eye, offset / length, length).is_none()
    }
}

/// Unsigned angle between two horizontal directions, in radians. A zero vector gives 0 (the cone check
/// then passes), the same as Godot's AngleTo.
fn angle_between(a: Vec2, b: Vec2) -> f32 {
    let cross = a.x * b.y - a.y * b.x;
    cross.abs().atan2(a.dot(b))
}
