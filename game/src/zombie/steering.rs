//! Obstacle avoidance and stuck escape for zombies (port of ZombieSteering.cs).
//! Uses feeler rays against PhysicsWorld without a navmesh.

use std::f32::consts::{PI, TAU};

use glam::Vec3;
use physics::PhysicsWorld;

use super::rng::Rng;

pub const FEELER_LENGTH: f32 = 2.5;
pub const FEELER_ANGLE_DEG: f32 = 35.0;
pub const KNEE_HEIGHT: f32 = 0.5;
pub const SIDESTEP_MEMORY: f32 = 1.0;
pub const ARRIVE_DISTANCE: f32 = 1.0;
pub const GROUND_PROBE_ABOVE: f32 = 2.0;
pub const GROUND_PROBE_BELOW: f32 = 4.0;
pub const BOXED_CLEARANCE: f32 = 0.5;

/// Length of short rays used to pick an escape direction.
pub const ESCAPE_PROBE: f32 = 2.0;
pub const ESCAPE_DIRECTIONS: usize = 8;
pub const ESCAPE_BLOCKED_PENALTY: f32 = 1.5;
pub const ANKLE_HEIGHT: f32 = 0.15;

/// Step check lookahead distance and max ledge height the body cannot climb.
pub const STEP_PROBE: f32 = 0.6;
pub const STEP_RISE: f32 = 0.35;
pub const STEP_RAY_TOP: f32 = 1.0;
pub const STEP_RAY_BELOW: f32 = 1.0;

/// Directions within ~60 degrees of the one that just failed count as blocked (cos 60 = 0.5).
pub const FAILED_DIRECTION_COS: f32 = 0.5;
pub const STEER_INTERVAL: f32 = 0.05;

/// Feeler ray steering and stuck recovery for one zombie.
#[derive(Clone, Debug)]
pub struct ZombieSteering {
    rng: Rng,
    target: Vec3,
    last_dir: Vec3,
    memory_timer: f32,
    memory_sign: f32,
    escape_timer: f32,
    escape_tries: u32,
    escape_dir: Vec3,
    stuck_timer: f32,
    stuck_anchor: Vec3,
    steer_timer: f32,
    blocked_spots: Vec<Vec3>,
    teleport_pending: Option<Vec3>,
    // Tuning values from ZombieBehaviourDef
    stuck_seconds: f32,
    stuck_distance: f32,
    escape_seconds: f32,
    escape_tries_before_teleport: u32,
    teleport_metres: f32,
    blocked_memory: usize,
    blocked_radius: f32,
}

impl ZombieSteering {
    pub fn new(
        seed: u64,
        stuck_seconds: f32,
        stuck_distance: f32,
        escape_seconds: f32,
        escape_tries_before_teleport: u32,
        teleport_metres: f32,
        blocked_memory: usize,
        blocked_radius: f32,
    ) -> Self {
        let mut rng = Rng::new(seed);
        let steer_timer = rng.range(0.0, STEER_INTERVAL);
        Self {
            rng,
            target: Vec3::ZERO,
            last_dir: Vec3::ZERO,
            memory_timer: 0.0,
            memory_sign: 1.0,
            escape_timer: 0.0,
            escape_tries: 0,
            escape_dir: Vec3::ZERO,
            stuck_timer: 0.0,
            stuck_anchor: Vec3::ZERO,
            steer_timer,
            blocked_spots: Vec::with_capacity(blocked_memory.max(4)),
            teleport_pending: None,
            stuck_seconds,
            stuck_distance,
            escape_seconds,
            escape_tries_before_teleport,
            teleport_metres,
            blocked_memory,
            blocked_radius,
        }
    }

    pub fn set_target(&mut self, position: Vec3) {
        self.target = position;
    }

    pub fn reset(&mut self) {
        self.memory_timer = 0.0;
        self.escape_timer = 0.0;
        self.escape_tries = 0;
        self.stuck_timer = 0.0;
        self.steer_timer = self.rng.range(0.0, STEER_INTERVAL);
        self.stuck_anchor = Vec3::ZERO;
        self.blocked_spots.clear();
        self.teleport_pending = None;
    }

    /// True while actively following an escape direction.
    pub fn is_escaping(&self) -> bool {
        self.escape_timer > 0.0
    }

    /// Returns and clears any teleport position requested by the stuck recovery.
    pub fn take_teleport(&mut self) -> Option<Vec3> {
        self.teleport_pending.take()
    }

    /// Follow toward `self.target`. Returns the flat wish velocity and whether arrived.
    pub fn follow(
        &mut self,
        pos: Vec3,
        speed: f32,
        dt: f32,
        world: &PhysicsWorld,
    ) -> (Vec3, bool) {
        let to_target = Vec3::new(self.target.x - pos.x, 0.0, self.target.z - pos.z);
        let dist = to_target.length();
        if dist <= ARRIVE_DISTANCE {
            return (Vec3::ZERO, true);
        }

        let desired = to_target / dist;
        self.update_stuck(dt, pos, desired, world);

        // Close to target (< 4m) or during escape: update every frame; otherwise 20 Hz
        if dist <= 4.0 || self.escape_timer > 0.0 {
            self.last_dir = self.choose_direction(pos, desired, dt, world);
            self.steer_timer = 0.0;
        } else {
            self.steer_timer -= dt;
            if self.steer_timer <= 0.0 || self.last_dir == Vec3::ZERO {
                self.steer_timer = STEER_INTERVAL;
                self.last_dir = self.choose_direction(pos, desired, dt, world);
            }
        }

        (self.last_dir * speed, false)
    }

    fn choose_direction(
        &mut self,
        pos: Vec3,
        desired: Vec3,
        dt: f32,
        world: &PhysicsWorld,
    ) -> Vec3 {
        self.memory_timer = (self.memory_timer - dt).max(0.0);
        self.escape_timer = (self.escape_timer - dt).max(0.0);
        if self.escape_timer > 0.0 {
            return self.escape_dir;
        }

        let feeler_rad = FEELER_ANGLE_DEG.to_radians();
        if self.memory_timer > 0.0 {
            let remembered = rotate_y(desired, self.memory_sign * feeler_rad);
            if self.clearance(pos, remembered, FEELER_LENGTH, KNEE_HEIGHT, world) >= FEELER_LENGTH {
                return remembered;
            }
        }

        if self.clearance(pos, desired, FEELER_LENGTH, KNEE_HEIGHT, world) >= FEELER_LENGTH {
            return desired;
        }

        let left = self.clearance(pos, rotate_y(desired, feeler_rad), FEELER_LENGTH, KNEE_HEIGHT, world);
        let right = self.clearance(pos, rotate_y(desired, -feeler_rad), FEELER_LENGTH, KNEE_HEIGHT, world);
        if left < BOXED_CLEARANCE && right < BOXED_CLEARANCE {
            self.start_escape(pos, desired, world);
            return self.escape_dir;
        }

        let sign = if (left - right).abs() < 1e-4 {
            if self.rng.range(0.0, 1.0) < 0.5 { -1.0 } else { 1.0 }
        } else if left > right {
            1.0
        } else {
            -1.0
        };
        self.memory_sign = sign;
        self.memory_timer = SIDESTEP_MEMORY;
        rotate_y(desired, sign * feeler_rad)
    }

    fn clearance(
        &self,
        pos: Vec3,
        dir: Vec3,
        length: f32,
        height: f32,
        world: &PhysicsWorld,
    ) -> f32 {
        let wall_normal_y = (45.0_f32).to_radians().cos();
        let origin = pos + Vec3::Y * height;
        if let Some(hit) = world.raycast(origin, dir, length) {
            if hit.normal.y < wall_normal_y {
                return hit.distance;
            }
        }
        if self.step_ahead(pos, dir, world) {
            return 0.0;
        }
        if self.is_walkable_ground(pos + dir * length, pos.y, world) {
            length
        } else {
            0.0
        }
    }

    fn step_ahead(&self, pos: Vec3, dir: Vec3, world: &PhysicsWorld) -> bool {
        let ahead = pos + dir * STEP_PROBE;
        let from = Vec3::new(ahead.x, pos.y + STEP_RAY_TOP, ahead.z);
        let max_dist = STEP_RAY_TOP + STEP_RAY_BELOW;
        let Some(hit) = world.raycast(from, Vec3::NEG_Y, max_dist) else {
            return true;
        };
        hit.point.y - pos.y > STEP_RISE
    }

    fn is_walkable_ground(&self, point: Vec3, self_y: f32, world: &PhysicsWorld) -> bool {
        let wall_normal_y = (45.0_f32).to_radians().cos();
        let top = self_y + GROUND_PROBE_ABOVE;
        let from = Vec3::new(point.x, top, point.z);
        let max_dist = GROUND_PROBE_ABOVE + GROUND_PROBE_BELOW;
        let Some(hit) = world.raycast(from, Vec3::NEG_Y, max_dist) else {
            return false;
        };
        hit.normal.y >= wall_normal_y
    }

    fn update_stuck(
        &mut self,
        dt: f32,
        pos: Vec3,
        desired: Vec3,
        world: &PhysicsWorld,
    ) {
        self.stuck_timer += dt;
        if self.stuck_timer < self.stuck_seconds {
            return;
        }

        self.stuck_timer = 0.0;
        let dx = pos.x - self.stuck_anchor.x;
        let dz = pos.z - self.stuck_anchor.z;
        let moved = (dx * dx + dz * dz).sqrt();
        self.stuck_anchor = pos;
        if moved >= self.stuck_distance {
            self.escape_tries = 0;
            return;
        }
        if self.escape_timer > 0.0 {
            return;
        }

        self.escape_tries += 1;
        if self.escape_tries > self.escape_tries_before_teleport {
            self.escape_tries = 0;
            self.start_escape(pos, desired, world);
            self.teleport_aside(pos, world);
            return;
        }
        self.start_escape(pos, desired, world);
    }

    fn teleport_aside(&mut self, pos: Vec3, world: &PhysicsWorld) {
        let dest = pos + self.escape_dir * self.teleport_metres;
        if self.is_walkable_ground(dest, pos.y, world) {
            self.teleport_pending = Some(Vec3::new(dest.x, pos.y, dest.z));
        }
    }

    fn start_escape(&mut self, pos: Vec3, desired: Vec3, world: &PhysicsWorld) {
        self.escape_timer = self.escape_seconds;
        self.remember_blocked(pos);

        let mut best_score = f32::MIN;
        let sign = if self.rng.range(0.0, 1.0) < 0.5 { -1.0 } else { 1.0 };
        let mut best = rotate_y(desired, sign * (PI * 0.5));
        let step = TAU / ESCAPE_DIRECTIONS as f32;

        for k in 0..ESCAPE_DIRECTIONS {
            let dir = rotate_y(desired, k as f32 * step);
            let c_knee = self.clearance(pos, dir, ESCAPE_PROBE, KNEE_HEIGHT, world);
            let c_ankle = self.clearance(pos, dir, ESCAPE_PROBE, ANKLE_HEIGHT, world);
            let clear = c_knee.min(c_ankle) / ESCAPE_PROBE;
            let mut score = clear * 2.0 + dir.dot(desired).max(0.0);

            if self.last_dir != Vec3::ZERO && dir.dot(self.last_dir) > FAILED_DIRECTION_COS {
                score -= ESCAPE_BLOCKED_PENALTY;
            }
            if self.blocked_near(pos + dir * ESCAPE_PROBE) {
                score -= ESCAPE_BLOCKED_PENALTY;
            }
            if score > best_score {
                best_score = score;
                best = dir;
            }
        }
        self.escape_dir = best;
    }

    fn remember_blocked(&mut self, pos: Vec3) {
        self.blocked_spots.push(pos);
        while self.blocked_spots.len() > self.blocked_memory {
            self.blocked_spots.remove(0);
        }
    }

    fn blocked_near(&self, point: Vec3) -> bool {
        for &spot in &self.blocked_spots {
            let dx = spot.x - point.x;
            let dz = spot.z - point.z;
            if (dx * dx + dz * dz).sqrt() <= self.blocked_radius {
                return true;
            }
        }
        false
    }
}

/// Rotates a 3D horizontal direction around +Y by `radians`.
pub fn rotate_y(dir: Vec3, radians: f32) -> Vec3 {
    let cos = radians.cos();
    let sin = radians.sin();
    Vec3::new(
        dir.x * cos + dir.z * sin,
        dir.y,
        -dir.x * sin + dir.z * cos,
    )
}
