//! Directional lower-body locomotion, turn-in-place stepping and procedural body lean.
//!
//! Blends backward, strafe and on-the-spot turn clips over the base locomotion lower body
//! while leaving straight-forward movement and upper-body weapon poses untouched.

use std::collections::HashMap;
use std::f32::consts::{E, PI, TAU};
use std::sync::OnceLock;

use engine::anim::Rig;
use glam::{Quat, Vec2, Vec3};
use pack::Clip;
use serde::Deserialize;

use crate::blend::WALK_BLEND;
use crate::pose_buffer::{LocalTransform, blend_pose_masked};

const WEIGHT_EPSILON: f32 = 1e-4; // new: DECISIONS 2026-11-23
const DIR_EPSILON: f32 = 1e-6; // new: DECISIONS 2026-11-23
const LEAN_EPSILON_RAD: f32 = 1e-4; // new: DECISIONS 2026-11-23
const MIN_TEMPO_SCALE: f32 = 0.1; // new: DECISIONS 2026-11-23
const TURN_FULL_RATE_MULT: f32 = 2.0; // new: DECISIONS 2026-11-23
const RECENT_SPEED_RATE: f32 = 3.0; // new: DECISIONS 2026-11-23
const HIPS_LEAN_SHARE: f32 = 0.20; // new: DECISIONS 2026-11-23
const SPINE_LEAN_SHARE: f32 = 0.40; // new: DECISIONS 2026-11-23
const CHEST_LEAN_SHARE: f32 = 0.40; // new: DECISIONS 2026-11-23

/// Tuning values for directional movement, leaning and turn-in-place from `data/motion.json`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MotionConfig {
    pub version: u32,
    pub direction_rate: f32,
    pub strafe_speed_scale: f32,
    pub back_speed_scale: f32,
    pub min_move_speed: f32,
    pub lean_max_deg: f32,
    pub lean_per_rad_per_sec: f32,
    pub lean_rate: f32,
    pub accel_lean_deg: f32,
    pub accel_lean_per_mps2: f32,
    pub stop_settle_deg: f32,
    pub turn_in_place_deg_per_sec: f32,
    pub turn_step_rate: f32,
    pub max_yaw_rate_rad: f32,
}

impl Default for MotionConfig {
    fn default() -> Self {
        Self {
            version: 1,                     // data: motion.json version
            direction_rate: 10.0,           // data: motion.json directionRate
            strafe_speed_scale: 0.85,       // data: motion.json strafeSpeedScale
            back_speed_scale: 0.8,          // data: motion.json backSpeedScale
            min_move_speed: 0.25,           // data: motion.json minMoveSpeed
            lean_max_deg: 9.0,              // data: motion.json leanMaxDeg
            lean_per_rad_per_sec: 3.5,      // data: motion.json leanPerRadPerSec
            lean_rate: 8.0,                 // data: motion.json leanRate
            accel_lean_deg: 6.0,            // data: motion.json accelLeanDeg
            accel_lean_per_mps2: 0.35,      // data: motion.json accelLeanPerMps2
            stop_settle_deg: 3.0,           // data: motion.json stopSettleDeg
            turn_in_place_deg_per_sec: 45.0, // data: motion.json turnInPlaceDegPerSec
            turn_step_rate: 8.0,            // data: motion.json turnStepRate
            max_yaw_rate_rad: 8.0,          // data: motion.json maxYawRateRad
        }
    }
}

impl MotionConfig {
    /// Parses a `MotionConfig` from JSON text.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Loads and caches the shared `data/motion.json` config, falling back to defaults.
    pub fn load_shared() -> &'static MotionConfig {
        static CONFIG: OnceLock<MotionConfig> = OnceLock::new();
        CONFIG.get_or_init(|| {
            for path in ["data/motion.json", "../data/motion.json"] {
                if let Ok(text) = std::fs::read_to_string(path)
                    && let Ok(cfg) = Self::from_json(&text)
                {
                    return cfg;
                }
            }
            Self::default()
        })
    }
}

/// Normalized 4-way movement weights in body-local space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DirectionWeights {
    pub forward: f32,
    pub back: f32,
    pub left: f32,
    pub right: f32,
}

/// Converts a body-local movement vector (`x` = right, `y` = forward) to 4-way weights summing to 1.
pub fn direction_weights(local_move: Vec2) -> DirectionWeights {
    if !local_move.is_finite() {
        return DirectionWeights::default();
    }
    let forward = local_move.y.max(0.0);
    let back = (-local_move.y).max(0.0);
    let right = local_move.x.max(0.0);
    let left = (-local_move.x).max(0.0);
    let sum = forward + back + left + right;
    if sum <= DIR_EPSILON {
        return DirectionWeights::default();
    }
    let inv = 1.0 / sum;
    DirectionWeights {
        forward: forward * inv,
        back: back * inv,
        left: left * inv,
        right: right * inv,
    }
}

/// Converts world velocity to body-local velocity (`x` = right, `y` = forward).
///
/// Uses Godot yaw convention: forward = `(-sin yaw, 0, -cos yaw)`, right = `(cos yaw, 0, -sin yaw)`.
pub fn local_velocity(velocity: Vec3, yaw: f32) -> Vec2 {
    if !velocity.is_finite() || !yaw.is_finite() {
        return Vec2::ZERO;
    }
    let (s, c) = yaw.sin_cos();
    let right = velocity.x * c - velocity.z * s;
    let forward = -velocity.x * s - velocity.z * c;
    Vec2::new(right, forward)
}

/// Skeleton joint indices used by procedural body lean.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MotionJoints {
    pub hips: Option<usize>,
    pub spine: Option<usize>,
    pub chest: Option<usize>,
}

impl MotionJoints {
    /// Resolves `hips`, `spine` and `chest` indices from skeleton joint names.
    pub fn from_names(names: &[&str]) -> Self {
        let find = |target: &str| names.iter().position(|&n| n == target);
        Self {
            hips: find("hips"),
            spine: find("spine"),
            chest: find("chest"),
        }
    }
}

/// Borrowed references to the 8 directional movement and turn-in-place clips.
#[derive(Clone, Copy, Default)]
pub struct MotionClips<'a> {
    pub walk_back: Option<&'a Clip>,
    pub run_back: Option<&'a Clip>,
    pub strafe_l: Option<&'a Clip>,
    pub strafe_r: Option<&'a Clip>,
    pub run_strafe_l: Option<&'a Clip>,
    pub run_strafe_r: Option<&'a Clip>,
    pub turn_l: Option<&'a Clip>,
    pub turn_r: Option<&'a Clip>,
}

impl<'a> MotionClips<'a> {
    /// Looks up the 8 directional and turn clips from an animator's clip map.
    pub fn from_map(clips: &'a HashMap<String, Clip>) -> Self {
        Self {
            walk_back: clips.get("move_walk_back"),
            run_back: clips.get("move_run_back"),
            strafe_l: clips.get("move_strafe_l"),
            strafe_r: clips.get("move_strafe_r"),
            run_strafe_l: clips.get("move_run_strafe_l"),
            run_strafe_r: clips.get("move_run_strafe_r"),
            turn_l: clips.get("move_turn_l"),
            turn_r: clips.get("move_turn_r"),
        }
    }

    fn as_slots(&self) -> ClipSlots<&'a Clip> {
        ClipSlots {
            walk_back: self.walk_back,
            run_back: self.run_back,
            strafe_l: self.strafe_l,
            strafe_r: self.strafe_r,
            run_strafe_l: self.run_strafe_l,
            run_strafe_r: self.run_strafe_r,
            turn_l: self.turn_l,
            turn_r: self.turn_r,
        }
    }
}

/// Generic clip handles matching `MotionClips`, used for non-allocating sampling and unit tests.
#[derive(Clone, Copy, Default)]
pub(crate) struct ClipSlots<T> {
    pub walk_back: Option<T>,
    pub run_back: Option<T>,
    pub strafe_l: Option<T>,
    pub strafe_r: Option<T>,
    pub run_strafe_l: Option<T>,
    pub run_strafe_r: Option<T>,
    pub turn_l: Option<T>,
    pub turn_r: Option<T>,
}

/// Sampling parameters for lower-body directional and turn clip blending.
#[derive(Clone, Copy)]
pub(crate) struct LegSampleParams<'a> {
    pub time_loco: f32,
    pub run_fraction: f32,
    pub lower_mask: &'a [f32],
}

/// Smoothed directional movement, turn-in-place and lean controller.
#[derive(Clone, Debug)]
pub struct MotionState {
    config: MotionConfig,
    has_input: bool,
    initialized: bool,
    input_velocity: Vec3,
    input_yaw: f32,
    prev_yaw: f32,
    prev_local_vel: Vec2,
    recent_speed: f32,
    weights: DirectionWeights,
    turn_amount: f32,
    lean_roll_deg: f32,
    lean_pitch_deg: f32,
    stop_settle: f32,
}

impl Default for MotionState {
    fn default() -> Self {
        Self::new()
    }
}

impl MotionState {
    /// Creates an inactive motion state using the shared `data/motion.json` config.
    pub fn new() -> Self {
        Self::with_config(MotionConfig::load_shared().clone())
    }

    /// Creates an inactive motion state with an explicit config.
    pub fn with_config(config: MotionConfig) -> Self {
        Self {
            config,
            has_input: false,
            initialized: false,
            input_velocity: Vec3::ZERO,
            input_yaw: 0.0,
            prev_yaw: 0.0,
            prev_local_vel: Vec2::ZERO,
            recent_speed: 0.0,
            weights: DirectionWeights::default(),
            turn_amount: 0.0,
            lean_roll_deg: 0.0,
            lean_pitch_deg: 0.0,
            stop_settle: 0.0,
        }
    }

    /// Call once per rendered frame. Until this is called the state has no effect.
    pub fn set_input(&mut self, velocity: Vec3, yaw: f32) {
        self.input_velocity = if velocity.is_finite() { velocity } else { Vec3::ZERO };
        self.input_yaw = if yaw.is_finite() { yaw } else { 0.0 };
        self.has_input = true;
    }

    /// Advances smoothed direction weights, leaning, stop settle and turn-in-place state.
    pub fn tick(&mut self, dt: f32, on_floor: bool) {
        if !self.has_input || !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let local_vel = local_velocity(self.input_velocity, self.input_yaw);
        let speed = local_vel.length();

        let (yaw_rate, forward_accel, snapped) = if self.initialized {
            let dyaw = wrap_angle(self.input_yaw - self.prev_yaw);
            if dyaw.abs() > self.config.max_yaw_rate_rad * dt {
                (0.0, 0.0, true)
            } else {
                let yr = dyaw / dt;
                let accel = (local_vel.y - self.prev_local_vel.y) / dt;
                (yr, accel, false)
            }
        } else {
            self.initialized = true;
            self.recent_speed = speed;
            (0.0, 0.0, false)
        };

        let prev_speed = self.prev_local_vel.length();
        let prev_forward = self.prev_local_vel.y;
        self.prev_yaw = self.input_yaw;
        self.prev_local_vel = local_vel;

        // 1. Smoothed 4-way movement weights.
        let target_weights = if speed >= self.config.min_move_speed {
            direction_weights(local_vel)
        } else {
            DirectionWeights::default()
        };
        let k_dir = exp_alpha(self.config.direction_rate, dt);
        self.weights.forward += (target_weights.forward - self.weights.forward) * k_dir;
        self.weights.back += (target_weights.back - self.weights.back) * k_dir;
        self.weights.left += (target_weights.left - self.weights.left) * k_dir;
        self.weights.right += (target_weights.right - self.weights.right) * k_dir;

        // 2. Turn in place: positive = turning left, negative = turning right.
        let turn_thresh = self.config.turn_in_place_deg_per_sec.to_radians();
        let target_turn = if on_floor
            && !snapped
            && speed < self.config.min_move_speed
            && turn_thresh > 0.0
            && yaw_rate.abs() >= turn_thresh
        {
            (yaw_rate / (turn_thresh * TURN_FULL_RATE_MULT)).clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let k_turn = exp_alpha(self.config.turn_step_rate, dt);
        self.turn_amount += (target_turn - self.turn_amount) * k_turn;

        // 3. Turn bank roll: positive = leaning left (into left turn), negative = leaning right.
        let move_factor = if speed >= self.config.min_move_speed {
            (speed / WALK_BLEND).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let target_roll = if on_floor && !snapped {
            (yaw_rate * move_factor * self.config.lean_per_rad_per_sec)
                .clamp(-self.config.lean_max_deg, self.config.lean_max_deg)
        } else {
            0.0
        };
        let k_lean = exp_alpha(self.config.lean_rate, dt);
        self.lean_roll_deg += (target_roll - self.lean_roll_deg) * k_lean;

        // 4. Acceleration pitch + stop settle overshoot (positive = forward, negative = back).
        let just_stopped = on_floor
            && !snapped
            && prev_speed >= self.config.min_move_speed
            && speed < self.config.min_move_speed;
        if just_stopped && prev_speed > DIR_EPSILON {
            let forward_ratio = (prev_forward / prev_speed).clamp(-1.0, 1.0);
            let stop_speed = self.recent_speed.max(prev_speed);
            let speed_ratio = (stop_speed / WALK_BLEND).clamp(0.0, 1.0);
            let max_settle = self.config.stop_settle_deg * E;
            self.stop_settle = (self.stop_settle + forward_ratio * speed_ratio * max_settle)
                .clamp(-max_settle, max_settle);
            self.recent_speed = 0.0;
        } else if speed >= self.config.min_move_speed {
            let k_recent = exp_alpha(RECENT_SPEED_RATE, dt);
            self.recent_speed += (speed - self.recent_speed) * k_recent;
        } else {
            self.recent_speed = 0.0;
        }
        if !on_floor {
            self.stop_settle = 0.0;
        }

        let target_pitch = if on_floor && !snapped {
            (forward_accel * self.config.accel_lean_per_mps2)
                .clamp(-self.config.accel_lean_deg, self.config.accel_lean_deg)
        } else {
            0.0
        };
        let decay = (-self.config.lean_rate * dt).exp();
        let rt = self.config.lean_rate * dt;
        self.lean_pitch_deg = target_pitch
            + (self.lean_pitch_deg - target_pitch) * decay
            + self.stop_settle * rt * decay;
        self.stop_settle *= decay;
    }

    /// Returns the smoothed 4-way movement weights.
    pub fn weights(&self) -> DirectionWeights {
        self.weights
    }

    /// Returns non-forward directional blend amount (0 when moving straight forward or idle).
    pub fn directional_amount(&self) -> f32 {
        (self.weights.back + self.weights.left + self.weights.right).clamp(0.0, 1.0)
    }

    /// Returns turn-in-place amount in `-1..=1` (positive = left, negative = right).
    pub fn turn_amount(&self) -> f32 {
        self.turn_amount
    }

    /// Returns turn bank lean in degrees (positive leans left, negative leans right).
    pub fn lean_roll_deg(&self) -> f32 {
        self.lean_roll_deg
    }

    /// Returns forward/back lean in degrees (positive leans forward, negative leans back).
    pub fn lean_pitch_deg(&self) -> f32 {
        self.lean_pitch_deg
    }

    /// Returns stride tempo correction for backward and sideways movement (1.0 for forward/idle).
    pub fn tempo_scale(&self) -> f32 {
        if !self.has_input {
            return 1.0;
        }
        let side = self.weights.left + self.weights.right;
        let scale = 1.0
            + (self.config.strafe_speed_scale - 1.0) * side
            + (self.config.back_speed_scale - 1.0) * self.weights.back;
        scale.max(MIN_TEMPO_SCALE)
    }

    /// Blends the directional and turn clips over the lower body in `local`.
    #[allow(clippy::too_many_arguments)] // one call site; the inputs are unrelated
    pub fn apply_legs(
        &self,
        rig: &Rig,
        clips: &MotionClips<'_>,
        time_loco: f32,
        run_fraction: f32,
        lower_mask: &[f32],
        local: &mut [LocalTransform],
        scratch: &mut [LocalTransform],
    ) {
        let params = LegSampleParams {
            time_loco,
            run_fraction,
            lower_mask,
        };
        self.apply_legs_sampled(clips.as_slots(), params, local, scratch, |clip, t, out| {
            rig.local_pose_into(clip, t, out);
        });
    }

    pub(crate) fn apply_legs_sampled<T, F>(
        &self,
        clips: ClipSlots<T>,
        params: LegSampleParams<'_>,
        local: &mut [LocalTransform],
        scratch: &mut [LocalTransform],
        mut sample: F,
    ) where
        T: Copy,
        F: FnMut(T, f32, &mut [LocalTransform]),
    {
        if !self.has_input
            || local.is_empty()
            || scratch.is_empty()
            || params.lower_mask.is_empty()
        {
            return;
        }
        let dir_amt = self.directional_amount();
        let turn_amt = self.turn_amount.abs();
        if dir_amt <= WEIGHT_EPSILON && turn_amt <= WEIGHT_EPSILON {
            return;
        }

        let rf = params.run_fraction.clamp(0.0, 1.0);
        let mut list = ClipBlendList::new();
        list.push_dir(clips.walk_back, clips.run_back, self.weights.back.max(0.0), rf);
        list.push_dir(clips.strafe_l, clips.run_strafe_l, self.weights.left.max(0.0), rf);
        list.push_dir(clips.strafe_r, clips.run_strafe_r, self.weights.right.max(0.0), rf);

        let present_sum = list.total_weight();
        if present_sum > WEIGHT_EPSILON {
            let mut w_acc = (1.0 - present_sum).max(0.0);
            for (clip, w) in list.iter() {
                w_acc += w;
                let alpha = (w / w_acc).clamp(0.0, 1.0);
                sample(clip, params.time_loco, scratch);
                blend_pose_masked(local, scratch, alpha, params.lower_mask);
            }
        }

        if turn_amt > WEIGHT_EPSILON {
            let turn_clip = if self.turn_amount > 0.0 {
                clips.turn_l
            } else {
                clips.turn_r
            };
            if let Some(clip) = turn_clip {
                sample(clip, params.time_loco, scratch);
                blend_pose_masked(local, scratch, turn_amt, params.lower_mask);
            }
        }
    }

    /// Adds the procedural turn and acceleration lean on hips, spine and chest.
    pub fn apply_lean(&self, joints: &MotionJoints, local: &mut [LocalTransform]) {
        if !self.has_input {
            return;
        }
        let roll_rad = self.lean_roll_deg.to_radians();
        let pitch_rad = self.lean_pitch_deg.to_radians();
        if roll_rad.abs() <= LEAN_EPSILON_RAD && pitch_rad.abs() <= LEAN_EPSILON_RAD {
            return;
        }
        let shares = [
            (joints.hips, HIPS_LEAN_SHARE),
            (joints.spine, SPINE_LEAN_SHARE),
            (joints.chest, CHEST_LEAN_SHARE),
        ];
        for (joint, share) in shares {
            if let Some(idx) = joint
                && idx < local.len()
            {
                let dq = Quat::from_rotation_z(roll_rad * share)
                    * Quat::from_rotation_x(-pitch_rad * share);
                local[idx].1 = (dq * local[idx].1.normalize()).normalize();
            }
        }
    }
}

struct ClipBlendList<T> {
    items: [Option<(T, f32)>; 6], // new: DECISIONS 2026-11-23
    len: usize,
}

impl<T: Copy> ClipBlendList<T> {
    fn new() -> Self {
        Self {
            items: [None; 6], // new: DECISIONS 2026-11-23
            len: 0,
        }
    }

    fn push(&mut self, clip: T, weight: f32) {
        if weight > WEIGHT_EPSILON && self.len < self.items.len() {
            self.items[self.len] = Some((clip, weight));
            self.len += 1;
        }
    }

    fn push_dir(&mut self, walk: Option<T>, run: Option<T>, dir_w: f32, run_frac: f32) {
        if dir_w <= WEIGHT_EPSILON {
            return;
        }
        match (walk, run) {
            (Some(w), Some(r)) => {
                self.push(w, dir_w * (1.0 - run_frac));
                self.push(r, dir_w * run_frac);
            }
            (Some(w), None) => self.push(w, dir_w),
            (None, Some(r)) => self.push(r, dir_w),
            (None, None) => {}
        }
    }

    fn total_weight(&self) -> f32 {
        self.items[..self.len].iter().flatten().map(|(_, w)| *w).sum()
    }

    fn iter(&self) -> impl Iterator<Item = (T, f32)> + '_ {
        self.items[..self.len].iter().flatten().copied()
    }
}

fn exp_alpha(rate: f32, dt: f32) -> f32 {
    if rate <= 0.0 || dt <= 0.0 {
        return 0.0;
    }
    1.0 - (-rate * dt).exp()
}

fn wrap_angle(angle: f32) -> f32 {
    if !angle.is_finite() {
        return 0.0;
    }
    (angle + PI).rem_euclid(TAU) - PI
}
