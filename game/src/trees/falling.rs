//! Falling tree animation state.
//! Tracks tilt angle and fall direction away from player during felling.

use glam::Vec3;

/// Default duration for a felled tree to fall flat on the ground.
pub const TREE_FALL_DURATION: f32 = 1.2;

/// A tree currently in the process of falling down.
#[derive(Clone, Debug, PartialEq)]
pub struct FallingTree {
    pub id: String,
    pub position: Vec3,
    pub fall_dir: Vec3,
    /// Tilt angle in radians: 0.0 (standing upright) to FRAC_PI_2 (90 degrees, flat on ground).
    pub tilt_angle: f32,
    /// Progress from 0.0 to 1.0.
    pub progress: f32,
    pub elapsed: f32,
    pub duration: f32,
    /// True when the fall animation has completed (progress >= 1.0).
    pub finished: bool,
}

impl FallingTree {
    pub fn new(id: impl Into<String>, position: Vec3, fall_dir: Vec3, duration: f32) -> Self {
        let mut dir = Vec3::new(fall_dir.x, 0.0, fall_dir.z);
        if dir.length_squared() > 0.0001 {
            dir = dir.normalize();
        } else {
            dir = Vec3::Z;
        }
        Self {
            id: id.into(),
            position,
            fall_dir: dir,
            tilt_angle: 0.0,
            progress: 0.0,
            elapsed: 0.0,
            duration: if duration > 0.0 { duration } else { TREE_FALL_DURATION },
            finished: false,
        }
    }

    /// Advances the fall animation by dt seconds.
    pub fn update(&mut self, dt: f32) {
        if self.finished {
            return;
        }
        self.elapsed += dt;
        self.progress = (self.elapsed / self.duration).clamp(0.0, 1.0);
        self.tilt_angle = self.progress * std::f32::consts::FRAC_PI_2;
        if self.progress >= 1.0 {
            self.finished = true;
        }
    }

    /// Tilt angle in degrees (0.0 to 90.0).
    pub fn tilt_degrees(&self) -> f32 {
        self.tilt_angle.to_degrees()
    }
}
