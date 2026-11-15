use engine::render::Camera;
use glam::{Vec2, Vec3};
use physics::PhysicsWorld;

use crate::input::Actions;

// Mouse and zoom settings (from CameraRig.cs)
pub const MOUSE_SENSITIVITY: f32 = 0.003;
pub const MIN_ZOOM: f32 = 1.5;
pub const MAX_ZOOM: f32 = 7.0;
pub const DEFAULT_ZOOM: f32 = 4.0;
pub const ZOOM_STEP: f32 = 0.5;
pub const TRANSITION_DURATION: f32 = 0.2;

// Eye height settings (from CameraRig.cs)
pub const STANDING_EYE_HEIGHT: f32 = 1.65;
pub const CROUCH_EYE_HEIGHT: f32 = 1.1;
pub const EYE_HEIGHT_LERP_SPEED: f32 = 12.0;

// Aim settings (from CameraRig.cs)
pub const AIM_TRANSITION_TIME: f32 = 0.15;
pub const THIRD_PERSON_AIM_FOV: f32 = 55.0;
pub const FIRST_PERSON_AIM_FOV: f32 = 50.0;
pub const DEFAULT_FOV: f32 = 75.0;
pub const AIM_SPRING_LENGTH: f32 = 1.8;

// Over-the-shoulder offset (from CameraRig.cs)
pub const SHOULDER_OFFSET: f32 = 0.55;
pub const SHOULDER_RAISE: f32 = 0.2;
pub const SHOULDER_LERP_RATE: f32 = 12.0;
pub const SPRING_MARGIN: f32 = 0.1;

/// Orbiting third-person / first-person camera rig, ported from Godot's CameraRig.cs.
/// Handles yaw/pitch look, zooming, aim FOV blending, and spring-arm occlusion raycasts.
#[derive(Clone, Debug)]
pub struct CameraRig {
    pub yaw: f32,
    pub pitch: f32,
    pub is_first_person: bool,
    pub aiming: bool,
    pub aim_blend: f32,
    pub stored_zoom: f32,
    pub current_spring_length: f32,
    pub eye_height: f32,
    pub shoulder_offset: Vec2,
    pub fov: f32,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: -0.25, // looking slightly down in third person
            is_first_person: false,
            aiming: false,
            aim_blend: 0.0,
            stored_zoom: DEFAULT_ZOOM,
            current_spring_length: DEFAULT_ZOOM,
            eye_height: STANDING_EYE_HEIGHT,
            shoulder_offset: Vec2::new(SHOULDER_OFFSET, SHOULDER_RAISE),
            fov: DEFAULT_FOV,
        }
    }
}

impl CameraRig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates camera rotation, zoom, aiming blend, and transitions.
    pub fn update(&mut self, actions: &Actions, crouching: bool, dt: f32) {
        if dt <= 0.0 {
            return;
        }

        // Toggle first / third person mode
        if actions.toggle_camera {
            self.is_first_person = !self.is_first_person;
        }

        // Mouse look (in radians)
        self.yaw += -actions.look_delta.0 * MOUSE_SENSITIVITY;
        self.pitch += -actions.look_delta.1 * MOUSE_SENSITIVITY;

        // Pitch clamp (CameraRig.cs lines 275-277)
        let (min_pitch, max_pitch) = if self.is_first_person {
            (-89.0f32.to_radians(), 89.0f32.to_radians())
        } else {
            (-70.0f32.to_radians(), 55.0f32.to_radians())
        };
        self.pitch = self.pitch.clamp(min_pitch, max_pitch);

        // Zoom with wheel (in 3rd person only)
        if !self.is_first_person && actions.zoom.abs() > 0.001 {
            // Positive wheel zooms in (reduces arm length), negative zooms out
            self.stored_zoom =
                (self.stored_zoom - actions.zoom * ZOOM_STEP).clamp(MIN_ZOOM, MAX_ZOOM);
        }

        // Aim transition blend
        self.aiming = actions.aim;
        let target_blend = if self.aiming { 1.0 } else { 0.0 };
        let blend_step = dt / AIM_TRANSITION_TIME;
        if self.aim_blend < target_blend {
            self.aim_blend = (self.aim_blend + blend_step).min(target_blend);
        } else if self.aim_blend > target_blend {
            self.aim_blend = (self.aim_blend - blend_step).max(target_blend);
        }

        // Smoothly adjust eye height for crouch / standing (CameraRig.cs lines 140-143)
        let target_height = if crouching {
            CROUCH_EYE_HEIGHT
        } else {
            STANDING_EYE_HEIGHT
        };
        let height_weight = (dt * EYE_HEIGHT_LERP_SPEED).clamp(0.0, 1.0);
        self.eye_height += (target_height - self.eye_height) * height_weight;

        // Smoothly adjust shoulder offset (CameraRig.cs lines 309-314)
        let target_shoulder = if self.is_first_person {
            Vec2::ZERO
        } else {
            Vec2::new(SHOULDER_OFFSET, SHOULDER_RAISE)
        };
        let shoulder_weight = 1.0 - (-SHOULDER_LERP_RATE * dt).exp();
        self.shoulder_offset = self.shoulder_offset.lerp(target_shoulder, shoulder_weight);

        // Smoothly transition spring arm length (CameraRig.cs lines 184-196)
        let target_length = if self.is_first_person {
            0.0
        } else {
            self.stored_zoom + (AIM_SPRING_LENGTH - self.stored_zoom) * self.aim_blend
        };
        let transition_speed = self.stored_zoom.max(1.0) / TRANSITION_DURATION;
        let step = transition_speed * dt;
        if self.current_spring_length < target_length {
            self.current_spring_length = (self.current_spring_length + step).min(target_length);
        } else if self.current_spring_length > target_length {
            self.current_spring_length = (self.current_spring_length - step).max(target_length);
        }

        // Field of view (CameraRig.cs lines 300-305)
        let aim_fov = if self.is_first_person {
            FIRST_PERSON_AIM_FOV
        } else {
            THIRD_PERSON_AIM_FOV
        };
        self.fov = DEFAULT_FOV + (aim_fov - DEFAULT_FOV) * self.aim_blend;
    }

    /// Computes the camera transformation and handles spring-arm raycast pull-in against the physics world.
    pub fn camera(&self, player_position: &Vec3, physics: &PhysicsWorld) -> Camera {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let forward = Vec3::new(-sy * cp, sp, -cy * cp);
        let right = Vec3::new(cy, 0.0, -sy);
        let pitch_up = right.cross(forward).normalize_or_zero();

        let eye_pos = *player_position + Vec3::new(0.0, self.eye_height, 0.0);
        let pivot = eye_pos + right * self.shoulder_offset.x + pitch_up * self.shoulder_offset.y;

        let cam_pos = if self.is_first_person || self.current_spring_length <= 0.001 {
            pivot
        } else {
            let back_dir = -forward;
            let mut actual_len = self.current_spring_length;
            if let Some(hit) = physics.raycast(pivot, back_dir, self.current_spring_length) {
                let pulled = (hit.distance - SPRING_MARGIN).max(0.0);
                actual_len = actual_len.min(pulled);
            }
            pivot + back_dir * actual_len
        };

        Camera {
            position: cam_pos,
            yaw: self.yaw,
            pitch: self.pitch,
            fov_y_deg: self.fov,
            near: 0.05,
        }
    }
}
