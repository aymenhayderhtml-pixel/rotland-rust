//! Obstacle avoidance steering feelers for dogs.
// Godot: DogSteering.cs

use glam::Vec3;
use physics::PhysicsWorld;

/// Walks the dog around trees, walls and steep drops.
/// Three feeler rays at knee height turn to the freer side when blocked.
// Godot: DogSteering.cs
#[derive(Clone, Debug)]
pub struct DogSteering {
    pub(super) memory_timer: f32,
    pub(super) memory_sign: f32,
    pub(super) rng_state: u64,
}

impl Default for DogSteering {
    fn default() -> Self {
        Self::new(12345)
    }
}

impl DogSteering {
    pub const FEELER_LENGTH: f32 = 2.0;
    pub const FEELER_ANGLE_DEG: f32 = 35.0;
    pub const KNEE_HEIGHT: f32 = 0.4;
    pub const MEMORY_SECONDS: f32 = 1.0;
    pub const BOXED_CLEARANCE: f32 = 0.3;
    /// Walkable ground is at most 45 degrees steep (cos 45 deg).
    pub const WALL_NORMAL_Y: f32 = 0.7071;
    pub const GROUND_PROBE_ABOVE: f32 = 2.0;
    pub const GROUND_PROBE_BELOW: f32 = 4.0;

    pub fn new(seed: u64) -> Self {
        Self {
            memory_timer: 0.0,
            memory_sign: 1.0,
            rng_state: seed.max(1),
        }
    }

    /// Turns the desired flat direction into a walking direction that avoids obstacles.
    // Godot: DogSteering.cs Choose
    pub fn choose(&mut self, body_pos: Vec3, desired: Vec3, dt: f32, physics: &PhysicsWorld) -> Vec3 {
        self.memory_timer = (self.memory_timer - dt).max(0.0);
        if self.memory_timer > 0.0 {
            let remembered = Self::rotate(desired, self.memory_sign * Self::FEELER_ANGLE_DEG);
            if Self::clearance(body_pos, remembered, physics) >= Self::FEELER_LENGTH {
                return remembered;
            }
        }

        if Self::clearance(body_pos, desired, physics) >= Self::FEELER_LENGTH {
            return desired;
        }

        let left = Self::clearance(body_pos, Self::rotate(desired, Self::FEELER_ANGLE_DEG), physics);
        let right = Self::clearance(body_pos, Self::rotate(desired, -Self::FEELER_ANGLE_DEG), physics);

        if left < Self::BOXED_CLEARANCE && right < Self::BOXED_CLEARANCE {
            let sign = self.random_sign();
            return Self::rotate(desired, sign * 90.0);
        }

        let sign = if (left - right).abs() < 1e-4 {
            self.random_sign()
        } else if left > right {
            1.0
        } else {
            -1.0
        };

        self.memory_sign = sign;
        self.memory_timer = Self::MEMORY_SECONDS;
        Self::rotate(desired, sign * Self::FEELER_ANGLE_DEG)
    }

    /// How far a feeler gets before something blocks it: the full length when clear,
    /// the hit distance for a wall or trunk, and zero when ground ahead drops away.
    // Godot: DogSteering.cs Clearance
    pub fn clearance(body_pos: Vec3, dir: Vec3, physics: &PhysicsWorld) -> f32 {
        let origin = body_pos + Vec3::new(0.0, Self::KNEE_HEIGHT, 0.0);
        if let Some(hit) = physics.raycast(origin, dir, Self::FEELER_LENGTH)
            && hit.normal.y < Self::WALL_NORMAL_Y {
                return hit.distance;
            }

        let end = origin + dir * Self::FEELER_LENGTH;
        if Self::is_walkable_ground(body_pos, end, physics) {
            Self::FEELER_LENGTH
        } else {
            0.0
        }
    }

    /// The ground under a point must be no steeper than walkable.
    // Godot: DogSteering.cs IsWalkableGround
    fn is_walkable_ground(body_pos: Vec3, point: Vec3, physics: &PhysicsWorld) -> bool {
        let top = body_pos.y + Self::GROUND_PROBE_ABOVE;
        let from = Vec3::new(point.x, top, point.z);
        let max_dist = Self::GROUND_PROBE_ABOVE + Self::GROUND_PROBE_BELOW;
        if let Some(hit) = physics.raycast(from, Vec3::NEG_Y, max_dist) {
            hit.normal.y >= Self::WALL_NORMAL_Y
        } else {
            false
        }
    }

    fn random_sign(&mut self) -> f32 {
        // Simple linear congruential pseudo-random generator
        self.rng_state = self.rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        if (self.rng_state >> 32) & 1 == 0 { -1.0 } else { 1.0 }
    }

    pub fn rotate(dir: Vec3, degrees: f32) -> Vec3 {
        let rad = degrees.to_radians();
        glam::Quat::from_rotation_y(rad) * dir
    }
}
