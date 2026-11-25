//! An animal's walking and motion using CharacterMover.
// Godot: AnimalMotion.cs

use glam::Vec3;
use physics::{CharacterMover, MoveResult, PhysicsWorld};

pub const STOP_DISTANCE: f32 = 0.3;
pub const GRAVITY: f32 = 9.8;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AnimalMotion {
    pub moving: bool,
}

impl AnimalMotion {
    pub fn new() -> Self {
        Self::default()
    }

    /// Turns the body toward a flat direction without moving it.
    // Godot: AnimalMotion.cs FaceDirection
    pub fn face_direction(dir: Vec3) -> f32 {
        (-dir.x).atan2(-dir.z)
    }

    /// Computes walk velocity and yaw toward target. Returns (walk_vel, yaw, still_moving).
    // Godot: AnimalMotion.cs Step
    pub fn step(
        &mut self,
        current_pos: Vec3,
        current_yaw: f32,
        target_pos: Vec3,
        speed: f32,
    ) -> (Vec3, f32, bool) {
        let offset = Self::flat(target_pos - current_pos);
        let distance = offset.length();
        if distance <= STOP_DISTANCE {
            self.moving = false;
            return (Vec3::ZERO, current_yaw, false);
        }
        let dir = offset / distance;
        let yaw = Self::face_direction(dir);
        self.moving = true;
        (dir * speed, yaw, true)
    }

    /// Flat horizontal vector (Y = 0).
    // Godot: AnimalMotion.cs Flat
    pub fn flat(v: Vec3) -> Vec3 {
        Vec3::new(v.x, 0.0, v.z)
    }

    /// Moves the animal with CharacterMover matching zombies.rs mover.move_and_slide.
    #[allow(clippy::too_many_arguments)]
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
