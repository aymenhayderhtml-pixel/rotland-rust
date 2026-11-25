//! Procedural first-person weapon motion driven by camera turns, locomotion,
//! jumps/landings, reloads, weapon switches, and breathing.

use std::f32::consts::{E, PI, TAU};

use glam::{Vec2, Vec3};
use serde::Deserialize;

use crate::player::SPRINT_SPEED;

const MAX_SUBSTEP: f32 = 1.0 / 240.0; // new: DECISIONS 2026-11-24
const MAX_FRAME_DT: f32 = 0.5; // new: DECISIONS 2026-11-24
const MIN_DT: f32 = 1e-6; // new: DECISIONS 2026-11-24
const BOB_ACTIVE_BLEND: f32 = 0.001; // new: DECISIONS 2026-11-24
const BOB_ACTIVE_SPEED: f32 = 0.01; // new: DECISIONS 2026-11-24
const MAX_SPEED_RATIO: f32 = 1.5; // new: DECISIONS 2026-11-24

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FpMotionConfig {
    pub look_stiffness: f32,
    pub look_damping: f32,
    pub look_lag_scale: f32,
    pub max_look_lag: f32,
    pub snap_angle: f32,
    pub look_tilt_per_lag: f32,
    pub look_rot_per_lag: f32,
    pub sprint_offset: [f32; 3],
    pub sprint_rot_deg: [f32; 3],
    pub sprint_bob_multiplier: f32,
    pub sprint_bob_amp: [f32; 2],
    pub sprint_bob_speed: f32,
    pub crouch_offset: [f32; 3],
    pub crouch_rot_deg: [f32; 3],
    pub air_rise_offset: f32,
    pub air_fall_offset: f32,
    pub air_ref_speed: f32,
    pub air_blend_rate: f32,
    pub landing_dip_per_mps: f32,
    pub landing_max_dip: f32,
    pub landing_stiffness: f32,
    pub landing_damping: f32,
    pub landing_pitch_per_m: f32,
    pub reload_offset: [f32; 3],
    pub reload_rot_deg: [f32; 3],
    pub reload_blend_rate: f32,
    pub switch_drop_distance: f32,
    pub switch_raise_time: f32,
    pub switch_pitch_deg: f32,
    pub breath_amplitude: [f32; 2],
    pub breath_period: f32,
    pub breath_tired_multiplier: f32,
    pub pose_blend_rate: f32,
    pub max_offset: [f32; 3],
}

impl FpMotionConfig {
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    pub fn from_repo() -> Option<Self> {
        let text = std::fs::read_to_string(paths::repo_path("data/fp_motion.json")).ok()?;
        Self::from_json(&text).ok()
    }
}

impl Default for FpMotionConfig {
    fn default() -> Self {
        Self {
            look_stiffness: 180.0,             // data: fp_motion.json lookStiffness
            look_damping: 22.0,                // data: fp_motion.json lookDamping
            look_lag_scale: 0.014,             // data: fp_motion.json lookLagScale
            max_look_lag: 0.045,               // data: fp_motion.json maxLookLag
            snap_angle: 1.0,                   // data: fp_motion.json snapAngle
            look_tilt_per_lag: 120.0,          // data: fp_motion.json lookTiltPerLag
            look_rot_per_lag: 140.0,           // data: fp_motion.json lookRotPerLag
            sprint_offset: [-0.03, -0.055, 0.02], // data: fp_motion.json sprintOffset
            sprint_rot_deg: [-10.0, 22.0, 12.0], // data: fp_motion.json sprintRotDeg
            sprint_bob_multiplier: 1.8,        // data: fp_motion.json sprintBobMultiplier
            sprint_bob_amp: [0.015, 0.012],    // data: fp_motion.json sprintBobAmp
            sprint_bob_speed: 11.0,            // data: fp_motion.json sprintBobSpeed
            crouch_offset: [-0.01, -0.035, 0.035], // data: fp_motion.json crouchOffset
            crouch_rot_deg: [0.0, 0.0, 4.0],   // data: fp_motion.json crouchRotDeg
            air_rise_offset: -0.035,           // data: fp_motion.json airRiseOffset
            air_fall_offset: 0.028,            // data: fp_motion.json airFallOffset
            air_ref_speed: 5.0,                // data: fp_motion.json airRefSpeed
            air_blend_rate: 10.0,              // data: fp_motion.json airBlendRate
            landing_dip_per_mps: 0.012,        // data: fp_motion.json landingDipPerMps
            landing_max_dip: 0.10,             // data: fp_motion.json landingMaxDip
            landing_stiffness: 140.0,          // data: fp_motion.json landingStiffness
            landing_damping: 18.0,             // data: fp_motion.json landingDamping
            landing_pitch_per_m: 140.0,        // data: fp_motion.json landingPitchPerM
            reload_offset: [-0.02, -0.09, 0.02], // data: fp_motion.json reloadOffset
            reload_rot_deg: [-14.0, 8.0, 22.0], // data: fp_motion.json reloadRotDeg
            reload_blend_rate: 10.0,           // data: fp_motion.json reloadBlendRate
            switch_drop_distance: 0.18,        // data: fp_motion.json switchDropDistance
            switch_raise_time: 0.28,           // data: fp_motion.json switchRaiseTime
            switch_pitch_deg: -14.0,           // data: fp_motion.json switchPitchDeg
            breath_amplitude: [0.0025, 0.004], // data: fp_motion.json breathAmplitude
            breath_period: 3.6,                // data: fp_motion.json breathPeriod
            breath_tired_multiplier: 2.5,      // data: fp_motion.json breathTiredMultiplier
            pose_blend_rate: 10.0,             // data: fp_motion.json poseBlendRate
            max_offset: [0.12, 0.24, 0.10],    // data: fp_motion.json maxOffset
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FpMotionInput {
    pub yaw: f32,
    pub pitch: f32,
    pub horizontal_speed: f32,
    pub vertical_velocity: f32,
    pub on_floor: bool,
    pub sprinting: bool,
    pub crouching: bool,
    pub reloading: bool,
    pub weapon_key: u64,
    pub stamina_frac: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FpMotion {
    initialized: bool,
    last_yaw: f32,
    last_pitch: f32,
    last_weapon_key: u64,
    was_on_floor: bool,
    last_air_vy: f32,
    look_pos: Vec2,
    look_vel: Vec2,
    sprint_blend: f32,
    sprint_bob_phase: f32,
    sprint_speed_blend: f32,
    crouch_blend: f32,
    reload_blend: f32,
    air_y: f32,
    landing_pos: f32,
    landing_vel: f32,
    switch_drop: f32,
    breath_phase: f32,
    cached_offset: Vec3,
    cached_rot_deg: Vec3,
}

impl FpMotion {
    pub fn update(&mut self, dt: f32, input: &FpMotionInput, cfg: &FpMotionConfig) {
        let stamina = input.stamina_frac.clamp(0.0, 1.0);
        if !self.initialized {
            self.initialized = true;
            self.last_yaw = input.yaw;
            self.last_pitch = input.pitch;
            self.last_weapon_key = input.weapon_key;
            self.was_on_floor = input.on_floor;
            self.last_air_vy = if input.on_floor { 0.0 } else { input.vertical_velocity };
        }
        if !dt.is_finite() || dt <= 0.0 {
            self.recompute_output(stamina, cfg);
            return;
        }
        let dt = dt.min(MAX_FRAME_DT);

        let dyaw = wrap_angle(input.yaw - self.last_yaw);
        let dpitch = input.pitch - self.last_pitch;
        self.last_yaw = input.yaw;
        self.last_pitch = input.pitch;

        let look_target = if dyaw.abs() > cfg.snap_angle
            || dpitch.abs() > cfg.snap_angle
            || dt < MIN_DT
        {
            Vec2::ZERO
        } else {
            let max_lag = cfg.max_look_lag.abs();
            Vec2::new(
                ((dyaw / dt) * cfg.look_lag_scale).clamp(-max_lag, max_lag),
                (-(dpitch / dt) * cfg.look_lag_scale).clamp(-max_lag, max_lag),
            )
        };

        if !self.was_on_floor && input.on_floor {
            let impact = (-self.last_air_vy).max(-input.vertical_velocity).max(0.0);
            if impact > 0.0 {
                let dip = (impact * cfg.landing_dip_per_mps).min(cfg.landing_max_dip);
                let omega = cfg.landing_stiffness.max(0.0).sqrt();
                self.landing_vel -= dip * E * omega;
            }
        }
        self.last_air_vy = if input.on_floor { 0.0 } else { input.vertical_velocity };
        self.was_on_floor = input.on_floor;

        if input.weapon_key != self.last_weapon_key {
            self.last_weapon_key = input.weapon_key;
            if input.weapon_key != 0 {
                self.switch_drop = 1.0;
            }
        }
        let raise_time = cfg.switch_raise_time.max(MIN_DT);
        self.switch_drop = (self.switch_drop - dt / raise_time).max(0.0);

        self.step_springs(dt, look_target, cfg);

        let pose_alpha = exp_blend(cfg.pose_blend_rate, dt);
        let reload_alpha = exp_blend(cfg.reload_blend_rate, dt);
        let air_alpha = exp_blend(cfg.air_blend_rate, dt);

        self.sprint_blend += (f32::from(input.sprinting) - self.sprint_blend) * pose_alpha;
        self.crouch_blend += (f32::from(input.crouching) - self.crouch_blend) * pose_alpha;
        self.reload_blend += (f32::from(input.reloading) - self.reload_blend) * reload_alpha;

        let speed_ratio = (input.horizontal_speed / SPRINT_SPEED).clamp(0.0, MAX_SPEED_RATIO);
        self.sprint_speed_blend += (speed_ratio - self.sprint_speed_blend) * pose_alpha;
        if self.sprint_blend > BOB_ACTIVE_BLEND && self.sprint_speed_blend > BOB_ACTIVE_SPEED {
            self.sprint_bob_phase = (self.sprint_bob_phase
                + dt * cfg.sprint_bob_speed * self.sprint_speed_blend)
                .rem_euclid(TAU);
        }

        let air_target = if input.on_floor {
            0.0
        } else {
            let frac = (input.vertical_velocity / cfg.air_ref_speed.max(MIN_DT)).clamp(-1.0, 1.0);
            if frac >= 0.0 {
                frac * cfg.air_rise_offset
            } else {
                (-frac) * cfg.air_fall_offset
            }
        };
        self.air_y += (air_target - self.air_y) * air_alpha;

        let breath_period = cfg.breath_period.max(MIN_DT);
        self.breath_phase = (self.breath_phase + dt * TAU / breath_period).rem_euclid(2.0 * TAU);

        self.recompute_output(stamina, cfg);
    }

    /// Camera-space offset (+X right, +Y up, +Z toward the camera), metres.
    pub fn offset(&self) -> Vec3 {
        self.cached_offset
    }

    /// Extra rotation in degrees (x pitch, y yaw, z roll), added to the hold rotation.
    pub fn rot_deg(&self) -> Vec3 {
        self.cached_rot_deg
    }

    fn step_springs(&mut self, dt: f32, look_target: Vec2, cfg: &FpMotionConfig) {
        let steps = (dt / MAX_SUBSTEP).ceil().max(1.0) as u32;
        let h = dt / steps as f32;
        let max_dip = cfg.landing_max_dip.abs();
        for _ in 0..steps {
            let look_acc = (look_target - self.look_pos) * cfg.look_stiffness
                - self.look_vel * cfg.look_damping;
            self.look_vel += look_acc * h;
            self.look_pos += self.look_vel * h;

            let land_acc = -self.landing_pos * cfg.landing_stiffness
                - self.landing_vel * cfg.landing_damping;
            self.landing_vel += land_acc * h;
            let next_land = self.landing_pos + self.landing_vel * h;
            if next_land < -max_dip {
                self.landing_pos = -max_dip;
                self.landing_vel = self.landing_vel.max(0.0);
            } else if next_land > max_dip {
                self.landing_pos = max_dip;
                self.landing_vel = self.landing_vel.min(0.0);
            } else {
                self.landing_pos = next_land;
            }
        }
    }

    fn recompute_output(&mut self, stamina: f32, cfg: &FpMotionConfig) {
        let look_off = Vec3::new(self.look_pos.x, self.look_pos.y, 0.0);
        let sprint_off = Vec3::from_array(cfg.sprint_offset) * self.sprint_blend;
        let extra_bob = (cfg.sprint_bob_multiplier - 1.0).max(0.0);
        let sprint_bob = Vec3::new(
            self.sprint_bob_phase.sin() * cfg.sprint_bob_amp[0],
            (self.sprint_bob_phase * 2.0).sin() * cfg.sprint_bob_amp[1],
            0.0,
        ) * (extra_bob * self.sprint_blend * self.sprint_speed_blend);
        let crouch_off = Vec3::from_array(cfg.crouch_offset) * self.crouch_blend;
        let reload_off = Vec3::from_array(cfg.reload_offset) * self.reload_blend;
        let switch_ease = self.switch_drop * self.switch_drop;
        let switch_off = Vec3::new(0.0, -cfg.switch_drop_distance * switch_ease, 0.0);
        let air_land_off = Vec3::new(0.0, self.air_y + self.landing_pos, 0.0);
        let tired_mult = 1.0 + (1.0 - stamina) * (cfg.breath_tired_multiplier - 1.0).max(0.0);
        let breath = Vec3::new(
            (self.breath_phase * 0.5).sin() * cfg.breath_amplitude[0] * tired_mult,
            self.breath_phase.sin() * cfg.breath_amplitude[1] * tired_mult,
            0.0,
        );

        let raw_offset = look_off
            + sprint_off
            + sprint_bob
            + crouch_off
            + reload_off
            + switch_off
            + air_land_off
            + breath;
        let max_off = Vec3::from_array(cfg.max_offset).abs();
        self.cached_offset = raw_offset.clamp(-max_off, max_off);

        let look_rot = Vec3::new(
            self.look_pos.y * cfg.look_rot_per_lag,
            -self.look_pos.x * cfg.look_rot_per_lag,
            self.look_pos.x * cfg.look_tilt_per_lag,
        );
        let sprint_rot = Vec3::from_array(cfg.sprint_rot_deg) * self.sprint_blend;
        let crouch_rot = Vec3::from_array(cfg.crouch_rot_deg) * self.crouch_blend;
        let reload_rot = Vec3::from_array(cfg.reload_rot_deg) * self.reload_blend;
        let switch_rot = Vec3::new(cfg.switch_pitch_deg * switch_ease, 0.0, 0.0);
        let land_rot = Vec3::new(self.landing_pos * cfg.landing_pitch_per_m, 0.0, 0.0);
        self.cached_rot_deg =
            look_rot + sprint_rot + crouch_rot + reload_rot + switch_rot + land_rot;
    }
}

fn wrap_angle(delta: f32) -> f32 {
    (delta + PI).rem_euclid(TAU) - PI
}

fn exp_blend(rate: f32, dt: f32) -> f32 {
    1.0 - (-rate.max(0.0) * dt).exp()
}
