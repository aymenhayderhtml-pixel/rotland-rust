//! Two-bone foot placement IK, hip drop, and ground-slope foot alignment.
//!
//! Converts world-space ground hits to model space, smooths vertical offsets,
//! hip drop, and surface normals exponentially, drops the hips so the lower
//! foot can reach, and solves both leg chains with [`crate::ik::solve_two_bone_ik`].

use std::sync::OnceLock;

use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

use crate::ik::{forward_kinematics, solve_two_bone_ik};
use crate::pose_buffer::LocalTransform;

const FOOT_IK_JSON_PATH: &str = "data/foot_ik.json";
const EMBEDDED_FOOT_IK_JSON: &str = include_str!("../../data/foot_ik.json");

const TELEPORT_DIST_SQ: f32 = 4.0; // new: DECISIONS 2026-11-23
const MIN_RUN_SPAN: f32 = 2.0; // new: DECISIONS 2026-11-23
const WEIGHT_EPS: f32 = 1e-4; // new: DECISIONS 2026-11-23
const POS_EPS: f32 = 1e-5; // new: DECISIONS 2026-11-23
const POS_EPS_SQ: f32 = 1e-10; // new: DECISIONS 2026-11-23
const DIR_EPS_SQ: f32 = 1e-8; // new: DECISIONS 2026-11-23
const NORMAL_EPS: f32 = 1e-5; // new: DECISIONS 2026-11-23
const NORMAL_EPS_SQ: f32 = 1e-10; // new: DECISIONS 2026-11-23
const QUAT_EPS: f32 = 1e-5; // new: DECISIONS 2026-11-23
const DET_EPS: f32 = 1e-6; // new: DECISIONS 2026-11-23
const MIN_BONE_LEN: f32 = 1e-3; // new: DECISIONS 2026-11-23
const KNEE_POLE_DIR: Vec3 = Vec3::NEG_Z; // new: DECISIONS 2026-11-23

/// Tuning values for foot placement IK loaded from `data/foot_ik.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FootIkConfig {
    pub version: u32,
    pub enabled: bool,
    pub ray_up: f32,
    pub ray_down: f32,
    pub foot_height: f32,
    pub max_foot_lift: f32,
    pub max_hip_drop: f32,
    pub max_foot_tilt_deg: f32,
    pub foot_rate: f32,
    pub hip_rate: f32,
    pub weight_rate: f32,
    pub swing_lift_threshold: f32,
    pub max_speed_full_weight: f32,
    pub min_weight_at_run: f32,
}

impl Default for FootIkConfig {
    fn default() -> Self {
        Self {
            version: 1, // data: foot_ik.json version
            enabled: true, // data: foot_ik.json enabled
            ray_up: 0.6, // data: foot_ik.json rayUp
            ray_down: 0.9, // data: foot_ik.json rayDown
            foot_height: 0.07, // data: foot_ik.json footHeight
            max_foot_lift: 0.45, // data: foot_ik.json maxFootLift
            max_hip_drop: 0.32, // data: foot_ik.json maxHipDrop
            max_foot_tilt_deg: 28.0, // data: foot_ik.json maxFootTiltDeg
            foot_rate: 16.0, // data: foot_ik.json footRate
            hip_rate: 10.0, // data: foot_ik.json hipRate
            weight_rate: 8.0, // data: foot_ik.json weightRate
            swing_lift_threshold: 0.07, // data: foot_ik.json swingLiftThreshold
            max_speed_full_weight: 4.0, // data: foot_ik.json maxSpeedFullWeight
            min_weight_at_run: 0.0, // data: foot_ik.json minWeightAtRun
        }
    }
}

impl FootIkConfig {
    /// Parses a [`FootIkConfig`] from JSON text.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Loads and caches the shared tuning from `data/foot_ik.json`.
    pub fn load_shared() -> &'static FootIkConfig {
        static SHARED: OnceLock<FootIkConfig> = OnceLock::new();
        SHARED.get_or_init(|| {
            std::fs::read_to_string(FOOT_IK_JSON_PATH)
                .ok()
                .and_then(|text| Self::from_json(&text).ok())
                .or_else(|| Self::from_json(EMBEDDED_FOOT_IK_JSON).ok())
                .unwrap_or_default()
        })
    }
}

/// World-space surface hit beneath a foot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundHit {
    pub point: Vec3,
    pub normal: Vec3,
}

/// Joint indices for one leg chain (`thigh -> shin -> foot`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LegJoints {
    pub thigh: usize,
    pub shin: usize,
    pub foot: usize,
}

impl LegJoints {
    fn from_names(
        names: &[&str],
        thigh_names: &[&str],
        shin_names: &[&str],
        foot_names: &[&str],
    ) -> Option<Self> {
        Some(Self {
            thigh: find_any(names, thigh_names)?,
            shin: find_any(names, shin_names)?,
            foot: find_any(names, foot_names)?,
        })
    }
}

/// Skeleton joint indices used by foot IK.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FootJoints {
    pub hips: usize,
    pub left: Option<LegJoints>,
    pub right: Option<LegJoints>,
}

impl FootJoints {
    /// Resolves hip and leg joint indices by name, returning `None` if hips or both legs are absent.
    pub fn from_names(names: &[&str]) -> Option<Self> {
        let hips = find_any(names, &["hips", "pelvis"])?;
        let left = LegJoints::from_names(
            names,
            &["thigh_L", "upper_leg_L"],
            &["shin_L", "lower_leg_L", "calf_L"],
            &["foot_L", "ankle_L"],
        );
        let right = LegJoints::from_names(
            names,
            &["thigh_R", "upper_leg_R"],
            &["shin_R", "lower_leg_R", "calf_R"],
            &["foot_R", "ankle_R"],
        );
        if left.is_none() && right.is_none() {
            return None;
        }
        Some(Self { hips, left, right })
    }

    fn leg(self, side: usize) -> Option<LegJoints> {
        match side {
            0 => self.left,
            1 => self.right,
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct SmoothedFoot {
    offset_y: f32,
    normal: Vec3,
    hit_weight: f32,
}

impl Default for SmoothedFoot {
    fn default() -> Self {
        Self { offset_y: 0.0, normal: Vec3::Y, hit_weight: 0.0 }
    }
}

struct LegTarget {
    orig_ankle: Vec3,
    foot: SmoothedFoot,
    leg_w: f32,
}

/// Per-character smoothed foot placement IK state.
#[derive(Clone, Debug)]
pub struct FootIkState {
    config: FootIkConfig,
    active: bool,
    has_ground: bool,
    char_world: Mat4,
    prev_char_pos: Option<Vec3>,
    ground_model: [Option<GroundHit>; 2],
    feet: [SmoothedFoot; 2],
    hip_drop: f32,
    weight: f32,
    pending_dt: f32,
    last_probe_points: Option<[Vec3; 2]>,
}

impl Default for FootIkState {
    fn default() -> Self {
        Self::new()
    }
}

impl FootIkState {
    /// Creates a new state using the shared `data/foot_ik.json` config.
    pub fn new() -> Self {
        Self::with_config(FootIkConfig::load_shared().clone())
    }

    /// Creates a new state using an explicit [`FootIkConfig`].
    pub fn with_config(config: FootIkConfig) -> Self {
        Self {
            config,
            active: true,
            has_ground: false,
            char_world: Mat4::IDENTITY,
            prev_char_pos: None,
            ground_model: [None, None],
            feet: [SmoothedFoot::default(); 2],
            hip_drop: 0.0,
            weight: 0.0,
            pending_dt: 0.0,
            last_probe_points: None,
        }
    }

    /// World XZ points to probe next, from the feet of the last solved pose.
    /// Returns `None` when disabled or never solved.
    pub fn probe_points(&self) -> Option<[Vec3; 2]> {
        if !self.config.enabled {
            return None;
        }
        self.last_probe_points
    }

    /// Stores this frame's ground under each foot, plus the character transform.
    pub fn set_ground(
        &mut self,
        left: Option<GroundHit>,
        right: Option<GroundHit>,
        char_world: Mat4,
    ) {
        if !char_world.is_finite() || char_world.determinant().abs() <= DET_EPS {
            return;
        }
        let char_pos = char_world.w_axis.truncate();
        if let Some(prev) = self.prev_char_pos
            && prev.distance_squared(char_pos) > TELEPORT_DIST_SQ
        {
            self.reset_smoothing();
        }
        self.prev_char_pos = Some(char_pos);
        self.char_world = char_world;
        let inv = char_world.inverse();
        self.ground_model = [
            left.and_then(|h| hit_to_model(h, inv)),
            right.and_then(|h| hit_to_model(h, inv)),
        ];
        self.has_ground = true;
    }

    /// Enables or disables foot IK smoothing (false in air, on ladders, in cars, swimming, dead).
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Advances overall weight smoothing and records elapsed time for offset smoothing.
    pub fn tick(&mut self, dt: f32, speed: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let clean_speed = if speed.is_finite() { speed.max(0.0) } else { 0.0 };
        let target_w = if self.config.enabled && self.active {
            self.speed_weight(clean_speed)
        } else {
            0.0
        };
        let alpha = exp_alpha(self.config.weight_rate, dt);
        self.weight += (target_w - self.weight) * alpha;
        self.pending_dt += dt;
    }

    /// Returns the current overall foot IK weight in `[0, 1]`.
    pub fn weight(&self) -> f32 {
        self.weight
    }

    /// Moves the hips and solves both legs in `local`. Runs after all clip layers and leans.
    pub fn apply(
        &mut self,
        joints: &FootJoints,
        parents: &[Option<usize>],
        order: &[usize],
        local: &mut [LocalTransform],
        globals: &mut [Mat4],
    ) {
        if !self.config.enabled || !self.has_ground {
            return;
        }
        let n = local.len().min(globals.len()).min(parents.len());
        if joints.hips >= n {
            return;
        }

        let dt = self.pending_dt;
        self.pending_dt = 0.0;
        forward_kinematics(parents, order, local, globals);

        let mut orig_ankles = [Vec3::ZERO; 2];
        let mut target_offsets = [0.0f32; 2];
        let mut target_normals = [Vec3::Y; 2];
        let mut target_hit_w = [0.0f32; 2];
        let mut target_hip_drop = 0.0f32;
        let max_tilt_rad = self.config.max_foot_tilt_deg.to_radians();

        for side in [0usize, 1] {
            let Some(leg) = joints.leg(side) else { continue };
            if leg.thigh >= n || leg.shin >= n || leg.foot >= n {
                continue;
            }
            let ankle = globals[leg.foot].w_axis.truncate();
            orig_ankles[side] = ankle;
            if let Some(hit) = self.ground_model[side] {
                target_hit_w[side] = 1.0;
                let ground_y = hit.point.y;
                let is_swing = (ankle.y - self.config.foot_height) > self.config.swing_lift_threshold;
                let raw_target_y = (ankle.y + ground_y).max(ground_y + self.config.foot_height);
                let offset_y = (raw_target_y - ankle.y).min(self.config.max_foot_lift);
                target_offsets[side] = offset_y;
                if !is_swing {
                    target_normals[side] = clamp_normal_tilt(hit.normal, max_tilt_rad);
                    target_hip_drop = target_hip_drop.max((-offset_y).max(0.0));
                }
            }
        }
        target_hip_drop = target_hip_drop.clamp(0.0, self.config.max_hip_drop);

        if dt > 0.0 {
            let hip_alpha = exp_alpha(self.config.hip_rate, dt);
            let foot_alpha = exp_alpha(self.config.foot_rate, dt);
            self.hip_drop += (target_hip_drop - self.hip_drop) * hip_alpha;
            for (side, f) in self.feet.iter_mut().enumerate() {
                f.hit_weight += (target_hit_w[side] - f.hit_weight) * foot_alpha;
                f.offset_y += (target_offsets[side] - f.offset_y) * foot_alpha;
                f.normal = smooth_normal(f.normal, target_normals[side], foot_alpha);
            }
        }

        let mut modified = false;
        let applied_hip_drop = self.hip_drop * self.weight;
        if applied_hip_drop > POS_EPS {
            let model_drop = Vec3::new(0.0, -applied_hip_drop, 0.0);
            let local_drop = match parents[joints.hips] {
                Some(p) if p < n && globals[p].determinant().abs() > DET_EPS => {
                    globals[p].inverse().transform_vector3(model_drop)
                }
                _ => model_drop,
            };
            local[joints.hips].0 += local_drop;
            forward_kinematics(parents, order, local, globals);
            modified = true;
        }

        if self.weight > WEIGHT_EPS {
            for side in [0usize, 1] {
                let Some(leg) = joints.leg(side) else { continue };
                if leg.thigh >= n || leg.shin >= n || leg.foot >= n {
                    continue;
                }
                let leg_w = (self.weight * self.feet[side].hit_weight).clamp(0.0, 1.0);
                if leg_w <= WEIGHT_EPS {
                    continue;
                }
                let target = LegTarget { orig_ankle: orig_ankles[side], foot: self.feet[side], leg_w };
                if solve_leg(leg, parents, local, globals, target) {
                    modified = true;
                }
            }
        }

        if modified {
            forward_kinematics(parents, order, local, globals);
        }
        self.update_probe_points(joints, globals, n);
    }

    fn reset_smoothing(&mut self) {
        self.feet = [SmoothedFoot::default(); 2];
        self.hip_drop = 0.0;
        self.weight = 0.0;
        self.pending_dt = 0.0;
        self.last_probe_points = None;
    }

    fn speed_weight(&self, speed: f32) -> f32 {
        let max_full = self.config.max_speed_full_weight;
        if speed <= max_full {
            return 1.0;
        }
        let run_speed = crate::blend::RUN_BLEND.max(max_full + MIN_RUN_SPAN);
        let t = ((speed - max_full) / (run_speed - max_full)).clamp(0.0, 1.0);
        let min_w = self.config.min_weight_at_run.clamp(0.0, 1.0);
        1.0 + (min_w - 1.0) * t
    }

    fn update_probe_points(&mut self, joints: &FootJoints, globals: &[Mat4], n: usize) {
        let fallback = self.char_world.w_axis.truncate();
        let left = joints.left.filter(|l| l.foot < n).map_or(fallback, |l| {
            self.char_world.transform_point3(globals[l.foot].w_axis.truncate())
        });
        let right = joints.right.filter(|r| r.foot < n).map_or(fallback, |r| {
            self.char_world.transform_point3(globals[r.foot].w_axis.truncate())
        });
        self.last_probe_points = Some([left, right]);
    }
}

fn solve_leg(
    leg: LegJoints,
    parents: &[Option<usize>],
    local: &mut [LocalTransform],
    globals: &[Mat4],
    target: LegTarget,
) -> bool {
    let hip_pos = globals[leg.thigh].w_axis.truncate();
    let knee_pos = globals[leg.shin].w_axis.truncate();
    let cur_ankle = globals[leg.foot].w_axis.truncate();
    let old_v1 = knee_pos - hip_pos;
    let old_v2 = cur_ankle - knee_pos;
    let l1 = old_v1.length();
    let l2 = old_v2.length();
    if l1 < MIN_BONE_LEN || l2 < MIN_BONE_LEN {
        return false;
    }

    let full_target = Vec3::new(
        target.orig_ankle.x,
        target.orig_ankle.y + target.foot.offset_y,
        target.orig_ankle.z,
    );
    let target_pos = cur_ankle.lerp(full_target, target.leg_w);
    let move_leg = (target_pos - cur_ankle).length_squared() > POS_EPS_SQ;

    let full_tilt = Quat::from_rotation_arc(Vec3::Y, normalize_or_up(target.foot.normal));
    let tilt_quat = Quat::IDENTITY.slerp(full_tilt, target.leg_w).normalize();
    let tilt_foot = tilt_quat.w.abs() < 1.0 - QUAT_EPS;
    if !move_leg && !tilt_foot {
        return false;
    }

    let r_shin_old = matrix_rotation(globals[leg.shin]);
    let r_foot_old = matrix_rotation(globals[leg.foot]);
    let r_shin_new = if move_leg {
        let new_knee = solve_two_bone_ik(hip_pos, target_pos, l1, l2, KNEE_POLE_DIR);
        let r_thigh_old = matrix_rotation(globals[leg.thigh]);
        let r_parent = match parents.get(leg.thigh).copied().flatten() {
            Some(p) if p < globals.len() => matrix_rotation(globals[p]),
            _ => Quat::IDENTITY,
        };
        let new_v1 = new_knee - hip_pos;
        let dq1 = if new_v1.length_squared() > DIR_EPS_SQ {
            Quat::from_rotation_arc(old_v1 / l1, new_v1.normalize())
        } else {
            Quat::IDENTITY
        };
        let r_thigh_new = (dq1 * r_thigh_old).normalize();
        local[leg.thigh].1 = (r_parent.inverse() * r_thigh_new).normalize();

        let mid_v2 = (dq1 * (old_v2 / l2)).normalize();
        let new_v2 = target_pos - new_knee;
        let dq2 = if new_v2.length_squared() > DIR_EPS_SQ {
            Quat::from_rotation_arc(mid_v2, new_v2.normalize())
        } else {
            Quat::IDENTITY
        };
        let r_shin = (dq2 * (dq1 * r_shin_old).normalize()).normalize();
        local[leg.shin].1 = (r_thigh_new.inverse() * r_shin).normalize();
        r_shin
    } else {
        r_shin_old
    };

    let r_foot_new = (tilt_quat * r_foot_old).normalize();
    local[leg.foot].1 = (r_shin_new.inverse() * r_foot_new).normalize();
    true
}

fn find_any(names: &[&str], candidates: &[&str]) -> Option<usize> {
    candidates.iter().find_map(|c| names.iter().position(|n| n == c))
}

fn hit_to_model(hit: GroundHit, inv_char_world: Mat4) -> Option<GroundHit> {
    if !hit.point.is_finite() {
        return None;
    }
    let point = inv_char_world.transform_point3(hit.point);
    let normal = if hit.normal.is_finite() {
        normalize_or_up(inv_char_world.transform_vector3(hit.normal))
    } else {
        Vec3::Y
    };
    Some(GroundHit { point, normal })
}

fn exp_alpha(rate: f32, dt: f32) -> f32 {
    if !rate.is_finite() || rate <= 0.0 || dt <= 0.0 {
        return 0.0;
    }
    (1.0 - (-rate * dt).exp()).clamp(0.0, 1.0)
}

fn normalize_or_up(v: Vec3) -> Vec3 {
    if v.is_finite() && v.length_squared() > NORMAL_EPS_SQ {
        v.normalize()
    } else {
        Vec3::Y
    }
}

fn clamp_normal_tilt(normal: Vec3, max_tilt_rad: f32) -> Vec3 {
    let n = normalize_or_up(normal);
    if n.y <= 0.0 {
        return Vec3::Y;
    }
    let angle = n.y.clamp(-1.0, 1.0).acos();
    if angle <= max_tilt_rad || angle <= NORMAL_EPS {
        return n;
    }
    let axis = Vec3::Y.cross(n);
    if axis.length_squared() <= NORMAL_EPS_SQ {
        return Vec3::Y;
    }
    (Quat::from_axis_angle(axis.normalize(), max_tilt_rad) * Vec3::Y).normalize()
}

fn smooth_normal(current: Vec3, target: Vec3, alpha: f32) -> Vec3 {
    let a = normalize_or_up(current);
    let b = normalize_or_up(target);
    let delta = Quat::from_rotation_arc(a, b);
    let step = Quat::IDENTITY.slerp(delta, alpha);
    normalize_or_up(step * a)
}

fn matrix_rotation(m: Mat4) -> Quat {
    if m.is_finite() && m.determinant().abs() > DET_EPS {
        m.to_scale_rotation_translation().1.normalize()
    } else {
        Quat::IDENTITY
    }
}
