use glam::{Quat, Vec2, Vec3};
use physics::PhysicsWorld;

pub const CHASE_DISTANCE: f32 = 6.0;
pub const MIN_DISTANCE: f32 = 1.5;
pub const PIVOT_HEIGHT: f32 = 1.3;
pub const SMOOTHING: f32 = 10.0;
pub const ORBIT_MOUSE_RAD_PER_PIXEL: f32 = 0.004;
pub const ORBIT_KEY_RAD_PER_SECOND: f32 = 2.0;
pub const MIN_PITCH: f32 = -0.1;
pub const MAX_PITCH: f32 = 1.0;

/// Driver camera follow math (port of scripts/Vehicles/CarCamera.cs).
#[derive(Clone, Debug)]
pub struct CarCamera {
    hood: bool,
    active: bool,
    snap: bool,
    yaw_offset: f32,
    pitch: f32,
    current_eye: Vec3,
}

impl Default for CarCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl CarCamera {
    pub fn new() -> Self {
        Self {
            hood: false,
            active: false,
            snap: true,
            yaw_offset: 0.0,
            pitch: 0.25,
            current_eye: Vec3::ZERO,
        }
    }

    pub fn is_hood(&self) -> bool {
        self.hood
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.snap = true;
        self.yaw_offset = 0.0;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn toggle_hood(&mut self) {
        self.hood = !self.hood;
        self.snap = true;
    }

    pub fn orbit(&mut self, relative_pixels: Vec2) {
        self.yaw_offset -= relative_pixels.x * ORBIT_MOUSE_RAD_PER_PIXEL;
        self.pitch = (self.pitch + relative_pixels.y * ORBIT_MOUSE_RAD_PER_PIXEL)
            .clamp(MIN_PITCH, MAX_PITCH);
    }

    /// Advances the camera position, returning (eye_position, target_look_at).
    pub fn step(
        &mut self,
        dt: f32,
        free_input: bool,
        orbit_keys: f32,
        car_pos: Vec3,
        car_rot: Quat,
        physics: &PhysicsWorld,
    ) -> (Vec3, Vec3) {
        if self.hood {
            let seat_local = Vec3::new(-0.32, 0.50, -0.20);
            let eye_offset = seat_local + Vec3::new(0.0, 0.45, -0.1);
            let eye = car_pos + car_rot * eye_offset;
            let forward = car_rot * -Vec3::Z;
            let target = eye + forward;
            self.snap = false;
            return (eye, target);
        }

        if free_input {
            self.yaw_offset += orbit_keys * ORBIT_KEY_RAD_PER_SECOND * dt;
        }

        let pivot = car_pos + Vec3::Y * PIVOT_HEIGHT;
        let mut flat_back = car_rot * Vec3::Z;
        flat_back.y = 0.0;
        let flat_back = if flat_back.length_squared() < 1e-6 {
            Vec3::Z
        } else {
            flat_back.normalize()
        };

        let yawed = Quat::from_axis_angle(Vec3::Y, self.yaw_offset) * flat_back;
        let dir = yawed * self.pitch.cos() + Vec3::Y * self.pitch.sin();

        let mut distance = CHASE_DISTANCE;
        if let Some(hit) = physics.raycast(pivot, dir, CHASE_DISTANCE) {
            distance = (hit.distance - 0.3).max(MIN_DISTANCE);
        }

        let target_eye = pivot + dir * distance;
        let weight = if self.snap {
            1.0
        } else {
            1.0 - (-SMOOTHING * dt).exp()
        };
        self.current_eye = if self.snap {
            target_eye
        } else {
            self.current_eye.lerp(target_eye, weight)
        };
        self.snap = false;

        (self.current_eye, pivot)
    }
}
