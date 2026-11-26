use glam::{Quat, Vec2, Vec3};
use physics::PhysicsWorld;

pub const MIN_DISTANCE: f32 = 1.5;
pub const ORBIT_MOUSE_RAD_PER_PIXEL: f32 = 0.004;

/// Camera mode while driving: chase camera, tight trail view, or hood-mounted view.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CarCameraMode {
    #[default]
    Chase,
    Close,
    Hood,
}

impl CarCameraMode {
    pub fn next(self) -> Self {
        match self {
            Self::Chase => Self::Close,
            Self::Close => Self::Hood,
            Self::Hood => Self::Chase,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Chase => "CHASE",
            Self::Close => "CLOSE",
            Self::Hood => "HOOD",
        }
    }
}

/// Solved camera pose produced by `CarCamera::step_drive`.
#[derive(Clone, Copy, Debug)]
pub struct CarCameraOutput {
    pub eye: Vec3,
    pub target: Vec3,
    pub fov_y_deg: f32,
}

/// Driver camera follow math (port of scripts/Vehicles/CarCamera.cs).
#[derive(Clone, Debug)]
pub struct CarCamera {
    mode: CarCameraMode,
    active: bool,
    snap: bool,
    yaw_offset: f32,
    pitch: f32,
    idle_orbit_timer: f32,
    current_eye: Vec3,
    current_target: Vec3,
    current_fov: f32,
    shake: f32,
    shake_phase: f32,
}

impl Default for CarCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl CarCamera {
    pub fn new() -> Self {
        Self {
            mode: CarCameraMode::Chase,
            active: false,
            snap: true,
            yaw_offset: 0.0,
            pitch: 0.22,
            idle_orbit_timer: 0.0,
            current_eye: Vec3::ZERO,
            current_target: Vec3::ZERO,
            current_fov: 75.0,
            shake: 0.0,
            shake_phase: 0.0,
        }
    }

    pub fn mode(&self) -> CarCameraMode {
        self.mode
    }

    pub fn is_hood(&self) -> bool {
        self.mode == CarCameraMode::Hood
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.snap = true;
        self.yaw_offset = 0.0;
        self.pitch = 0.22;
        self.idle_orbit_timer = 0.0;
        self.current_fov = 75.0;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.shake = 0.0;
    }

    pub fn cycle_mode(&mut self) -> CarCameraMode {
        self.mode = self.mode.next();
        self.snap = true;
        self.yaw_offset = 0.0;
        self.mode
    }

    pub fn toggle_hood(&mut self) {
        self.mode = if self.mode == CarCameraMode::Hood {
            CarCameraMode::Chase
        } else {
            CarCameraMode::Hood
        };
        self.snap = true;
    }

    pub fn orbit(&mut self, relative_pixels: Vec2) {
        if relative_pixels.length_squared() <= 1e-4 {
            return;
        }
        self.yaw_offset -= relative_pixels.x * ORBIT_MOUSE_RAD_PER_PIXEL;
        self.idle_orbit_timer = 0.0;
        if self.is_hood() {
            self.pitch = (self.pitch + relative_pixels.y * ORBIT_MOUSE_RAD_PER_PIXEL).clamp(-0.25, 0.30);
            self.yaw_offset = self.yaw_offset.clamp(-0.95, 0.95);
        } else {
            self.pitch = (self.pitch + relative_pixels.y * ORBIT_MOUSE_RAD_PER_PIXEL).clamp(-0.10, 0.85);
        }
    }

    /// Turns the camera around the car by `yaw` radians (test screenshots, `--car-yaw`).
    pub fn set_orbit_yaw(&mut self, yaw: f32) {
        self.yaw_offset = yaw;
        self.idle_orbit_timer = 0.0;
        self.snap = true;
    }

    pub fn add_shake(&mut self, amount: f32) {
        self.shake = (self.shake + amount.max(0.0)).min(1.0);
    }

    /// Advances the camera position for this frame.
    #[allow(clippy::too_many_arguments)]
    pub fn step_drive(
        &mut self,
        dt: f32,
        car_pos: Vec3,
        car_rot: Quat,
        speed_kmh: f32,
        car_tag: u128,
        physics: &PhysicsWorld,
    ) -> CarCameraOutput {
        self.idle_orbit_timer += dt;
        self.shake = (self.shake - dt * 2.8).max(0.0);
        self.shake_phase += dt * 42.0;

        // Re-center behind the car once the driver has stopped orbiting and is moving
        if self.idle_orbit_timer > 1.2 && speed_kmh > 4.0 {
            self.yaw_offset *= (-2.8f32 * dt).exp();
        }

        let speed_frac = (speed_kmh / 90.0).clamp(0.0, 1.0);

        let (desired_eye, desired_look, target_fov) = match self.mode {
            CarCameraMode::Hood => {
                // The glass-tint windshield is opaque (ends z = -0.77): mount the
                // camera on the cowl at z = -0.86, looking over the hood and bullbar
                let eye_local = Vec3::new(-0.28, 1.24, -0.86);
                let desired_eye = car_pos + car_rot * eye_local;
                let look_local = Quat::from_rotation_y(self.yaw_offset.clamp(-0.95, 0.95))
                    * Quat::from_rotation_x(-self.pitch.clamp(-0.25, 0.30))
                    * Vec3::NEG_Z;
                let desired_look = desired_eye + (car_rot * look_local) * 10.0;
                let target_fov = 75.0 + speed_frac * 5.0;
                (desired_eye, desired_look, target_fov)
            }
            CarCameraMode::Chase | CarCameraMode::Close => {
                let (base_dist, pivot_h) = match self.mode {
                    CarCameraMode::Chase => (6.2 + speed_frac * 0.9, 1.50),
                    CarCameraMode::Close => (4.3 + speed_frac * 0.5, 1.38),
                    CarCameraMode::Hood => unreachable!(),
                };
                let pivot = car_pos + Vec3::Y * pivot_h;
                let mut flat_back = car_rot * Vec3::Z;
                flat_back.y = 0.0;
                let flat_back = if flat_back.length_squared() < 1e-6 {
                    Vec3::Z
                } else {
                    flat_back.normalize()
                };
                let yawed = Quat::from_axis_angle(Vec3::Y, self.yaw_offset) * flat_back;
                let dir = yawed * self.pitch.cos() + Vec3::Y * self.pitch.sin();

                // Wall avoidance: pull the eye in when something is behind the car
                let mut dist = base_dist;
                if let Some(hit) = physics.raycast_except(pivot, dir, base_dist, car_tag) {
                    dist = (hit.distance - 0.35).max(MIN_DISTANCE);
                }
                let mut desired_eye = pivot + dir * dist;
                // Ground clearance: stay at least 0.9 m above the hillside
                if let Some(h) = physics.ground_height(desired_eye.x, desired_eye.z) {
                    desired_eye.y = desired_eye.y.max(h + 0.9);
                }
                let desired_look = pivot + (car_rot * Vec3::NEG_Z) * (1.2 + speed_frac * 1.5);
                let target_fov = 75.0 + speed_frac * 9.0;
                (desired_eye, desired_look, target_fov)
            }
        };

        // Hood snaps immediately; everything else eases in smoothly
        if self.is_hood() || self.snap {
            self.current_eye = desired_eye;
            self.current_target = desired_look;
        } else {
            let eye_w = 1.0 - (-10.0 * dt).exp();
            let target_w = 1.0 - (-14.0 * dt).exp();
            self.current_eye = self.current_eye.lerp(desired_eye, eye_w);
            self.current_target = self.current_target.lerp(desired_look, target_w);
        }

        // FOV eases toward target (snap during re-activate/cycle)
        if self.snap {
            self.current_fov = target_fov;
        } else {
            let fov_w = 1.0 - (-6.0 * dt).exp();
            self.current_fov += (target_fov - self.current_fov) * fov_w;
        }
        self.snap = false;

        // Impact shake decaying with time (adds on top, never permanent)
        let mut eye = self.current_eye;
        let mut target = self.current_target;
        if self.shake > 0.001 {
            let mag = self.shake * self.shake * 0.22;
            let off = Vec3::new(
                self.shake_phase.sin(),
                (self.shake_phase * 1.37).cos() * 0.7,
                (self.shake_phase * 0.83).sin() * 0.5,
            ) * mag;
            eye += off;
            target += off * 0.4;
        }

        CarCameraOutput {
            eye,
            target,
            fov_y_deg: self.current_fov,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_world() -> PhysicsWorld {
        let mut world = PhysicsWorld::new();
        let heights = vec![0.0f32; physics::CHUNK_SAMPLES * physics::CHUNK_SAMPLES];
        for cz in -2..=2 {
            for cx in -2..=2 {
                world.add_ground_chunk(glam::IVec2::new(cx, cz), &heights);
            }
        }
        world
    }

    #[test]
    fn cycle_mode_loops_chase_close_hood() {
        let mut cam = CarCamera::new();
        assert_eq!(cam.mode(), CarCameraMode::Chase);
        assert_eq!(cam.cycle_mode(), CarCameraMode::Close);
        assert_eq!(cam.cycle_mode(), CarCameraMode::Hood);
        assert_eq!(cam.cycle_mode(), CarCameraMode::Chase);
        cam.toggle_hood();
        assert_eq!(cam.mode(), CarCameraMode::Hood);
        cam.toggle_hood();
        assert_eq!(cam.mode(), CarCameraMode::Chase);
        assert_eq!(cam.mode().label(), "CHASE");
        assert_eq!(CarCameraMode::Close.label(), "CLOSE");
        assert_eq!(CarCameraMode::Hood.label(), "HOOD");
    }

    #[test]
    fn chase_cam_increases_fov_with_speed_and_avoids_ground() {
        let world = flat_world();
        let mut slow = CarCamera::new();
        slow.activate();
        let eye_slow = slow.step_drive(0.016, Vec3::ZERO, Quat::IDENTITY, 0.0, 0, &world);
        let mut fast = CarCamera::new();
        fast.activate();
        let eye_fast = fast.step_drive(0.016, Vec3::ZERO, Quat::IDENTITY, 80.0, 0, &world);
        assert!(
            eye_fast.fov_y_deg > eye_slow.fov_y_deg,
            "FOV widens with speed: {:.1} vs {:.1}",
            eye_fast.fov_y_deg,
            eye_slow.fov_y_deg
        );
        assert!(
            eye_fast.eye.y >= 0.9,
            "eye stays at least 0.9 m above the ground, is {:.2}",
            eye_fast.eye.y
        );
    }

    #[test]
    fn shake_decays_to_zero() {
        let world = flat_world();
        let mut cam = CarCamera::new();
        cam.activate();
        cam.add_shake(1.0);
        for _ in 0..120 {
            cam.step_drive(1.0 / 60.0, Vec3::ZERO, Quat::IDENTITY, 0.0, 0, &world);
        }
        assert!(
            cam.shake <= 0.001,
            "shake should decay toward zero, is {:.4}",
            cam.shake
        );
    }
}
