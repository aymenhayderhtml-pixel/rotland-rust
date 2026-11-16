//! Dog walking and movement using CharacterMover.
// Godot: DogMotion.cs

use glam::Vec3;
use physics::{CharacterMover, MoveResult, PhysicsWorld};

use super::config::DogConfigData;
use super::steering::DogSteering;

pub const GRAVITY: f32 = 9.8;

/// Manages dog movement, idle timing, and physics integration.
// Godot: DogMotion.cs
#[derive(Clone, Debug)]
pub struct DogMotion {
    pub(super) idle_seconds: f32,
    pub(super) moving: bool,
    pub(super) speed: f32,
}

impl Default for DogMotion {
    fn default() -> Self {
        Self::new()
    }
}

impl DogMotion {
    pub fn new() -> Self {
        Self {
            idle_seconds: 0.0,
            moving: false,
            speed: 0.0,
        }
    }

    /// Seconds since the dog last walked. Drives sit (after 5s) and lie (after 20s).
    // Godot: DogMotion.cs IdleSeconds
    pub fn idle_seconds(&self) -> f32 {
        self.idle_seconds
    }

    /// True when the dog moved during the last step.
    // Godot: DogMotion.cs Moving
    pub fn is_moving(&self) -> bool {
        self.moving
    }

    /// Current movement speed (m/s).
    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// Turns the dog yaw to face the given horizontal direction.
    /// The model faces -Z, so yaw turns -Z onto heading.
    // Godot: DogMotion.cs Step
    pub fn face_direction(dir: Vec3) -> f32 {
        (-dir.x).atan2(-dir.z)
    }

    /// Horizontal component of a 3D vector (Y = 0).
    // Godot: DogMotion.cs Flat
    pub fn flat(v: Vec3) -> Vec3 {
        Vec3::new(v.x, 0.0, v.z)
    }

    /// Steps toward target position. Returns (walk_velocity, yaw, moved).
    // Godot: DogMotion.cs Step
    pub fn step(
        &mut self,
        current_pos: Vec3,
        current_yaw: f32,
        target_pos: Vec3,
        speed: f32,
        dt: f32,
        cfg: &DogConfigData,
        steering: &mut DogSteering,
        physics: &PhysicsWorld,
    ) -> (Vec3, f32, bool) {
        let offset = Self::flat(target_pos - current_pos);
        let distance = offset.length();
        if distance <= cfg.stop_distance {
            self.idle(dt);
            return (Vec3::ZERO, current_yaw, false);
        }

        let dir = offset / distance;
        let heading = steering.choose(current_pos, dir, dt, physics);
        let yaw = Self::face_direction(heading);
        self.moving = true;
        self.idle_seconds = 0.0;
        self.speed = speed;
        (heading * speed, yaw, true)
    }

    /// Stands still for this frame.
    // Godot: DogMotion.cs Idle
    pub fn idle(&mut self, dt: f32) {
        self.moving = false;
        self.idle_seconds += dt;
        self.speed = 0.0;
    }

    /// Puts the dog at a point directly (teleport, spawn, load).
    // Godot: DogMotion.cs PlaceAt
    pub fn place_at(&mut self) {
        self.idle_seconds = 0.0;
        self.moving = false;
        self.speed = 0.0;
    }

    /// Moves the character through physics using CharacterMover.
    pub fn move_and_slide(
        mover: &CharacterMover,
        physics: &PhysicsWorld,
        current_pos: Vec3,
        walk_vel: Vec3,
        knock: Vec3,
        current_vel_y: f32,
        on_floor: bool,
        dt: f32,
    ) -> MoveResult {
        let fall = if on_floor { -1.0 } else { current_vel_y - GRAVITY * dt };
        let wanted = Vec3::new(walk_vel.x, 0.0, walk_vel.z) + knock + Vec3::new(0.0, fall, 0.0);
        mover.move_and_slide(physics, current_pos, wanted, dt)
    }
}
