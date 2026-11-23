//! Upper-body look-pitch aiming, damped-spring weapon recoil, and reload timing.

use std::collections::HashMap;
use std::sync::OnceLock;

use glam::Quat;
use serde::Deserialize;

use crate::pose_buffer::LocalTransform;
use crate::stance::StanceClass;

const DEFAULT_VERSION: u32 = 1; // data: aim.json version
const DEFAULT_MAX_PITCH_UP_DEG: f32 = 55.0; // data: aim.json maxPitchUpDeg
const DEFAULT_MAX_PITCH_DOWN_DEG: f32 = 50.0; // data: aim.json maxPitchDownDeg
const DEFAULT_PITCH_SPINE: f32 = 0.2; // data: aim.json pitchBones.spine
const DEFAULT_PITCH_CHEST: f32 = 0.35; // data: aim.json pitchBones.chest
const DEFAULT_PITCH_NECK: f32 = 0.2; // data: aim.json pitchBones.neck
const DEFAULT_PITCH_HEAD: f32 = 0.25; // data: aim.json pitchBones.head
const DEFAULT_CARRY_PITCH_FRACTION: f32 = 0.25; // data: aim.json carryPitchFraction
const DEFAULT_PITCH_RATE: f32 = 18.0; // data: aim.json pitchRate
const DEFAULT_MAX_KICK_DEG: f32 = 22.0; // data: aim.json maxKickDeg

/// Sign for positive pitch/recoil rotation around local X (+X right, +Y up, -Z forward).
/// Positive rotation around +X rotates forward (-Z) upward (+Y) and tilts upright (+Y) back (+Z).
const PITCH_SIGN: f32 = 1.0; // new: DECISIONS 2026-11-23
const RECOIL_SPINE_SHARE: f32 = 0.2; // new: DECISIONS 2026-11-23
const RECOIL_CHEST_SHARE: f32 = 0.5; // new: DECISIONS 2026-11-23
const RECOIL_ARM_SHARE: f32 = 0.3; // new: DECISIONS 2026-11-23
const MAX_BACK_MULTIPLIER: f32 = 2.0; // new: DECISIONS 2026-11-23
const SPRING_CRIT_EPS: f32 = 1e-4; // new: DECISIONS 2026-11-23
const SETTLE_EPS: f32 = 1e-4; // new: DECISIONS 2026-11-23
const APPLY_EPS: f32 = 1e-5; // new: DECISIONS 2026-11-23
const HALF: f32 = 0.5; // new: DECISIONS 2026-11-23
const TWO: f32 = 2.0; // new: DECISIONS 2026-11-23

/// Per-bone pitch distribution weights from `data/aim.json`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PitchBones {
    pub spine: f32,
    pub chest: f32,
    pub neck: f32,
    pub head: f32,
}

impl Default for PitchBones {
    fn default() -> Self {
        Self {
            spine: DEFAULT_PITCH_SPINE,
            chest: DEFAULT_PITCH_CHEST,
            neck: DEFAULT_PITCH_NECK,
            head: DEFAULT_PITCH_HEAD,
        }
    }
}

/// Per-weapon-class recoil spring tuning and optional reload clip name.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AimClassConfig {
    pub kick_deg: f32,
    pub kick_back: f32,
    pub stiffness: f32,
    pub damping: f32,
    #[serde(default)]
    pub reload_clip: Option<String>,
}

/// Parsed configuration from `data/aim.json`.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AimConfig {
    #[serde(default = "default_version")]
    pub version: u32,
    pub max_pitch_up_deg: f32,
    pub max_pitch_down_deg: f32,
    pub pitch_bones: PitchBones,
    pub carry_pitch_fraction: f32,
    pub pitch_rate: f32,
    pub classes: HashMap<String, AimClassConfig>,
    pub max_kick_deg: f32,
}

impl Default for AimConfig {
    fn default() -> Self {
        Self {
            version: DEFAULT_VERSION,
            max_pitch_up_deg: DEFAULT_MAX_PITCH_UP_DEG,
            max_pitch_down_deg: DEFAULT_MAX_PITCH_DOWN_DEG,
            pitch_bones: PitchBones::default(),
            carry_pitch_fraction: DEFAULT_CARRY_PITCH_FRACTION,
            pitch_rate: DEFAULT_PITCH_RATE,
            classes: HashMap::new(),
            max_kick_deg: DEFAULT_MAX_KICK_DEG,
        }
    }
}

impl AimConfig {
    /// Parses an `aim.json` document.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Cached with `OnceLock` from `paths::repo_path("data/aim.json")`.
    /// Logs once and falls back to defaults if the file is missing or invalid.
    pub fn load_shared() -> &'static AimConfig {
        static SHARED: OnceLock<AimConfig> = OnceLock::new();
        SHARED.get_or_init(|| {
            let path = paths::repo_path("data/aim.json");
            match std::fs::read_to_string(&path) {
                Ok(text) => match Self::from_json(&text) {
                    Ok(cfg) => cfg,
                    Err(err) => {
                        log::warn!(
                            "failed to parse {}: {err}; using default aim config",
                            path.display()
                        );
                        Self::default()
                    }
                },
                Err(err) => {
                    log::warn!(
                        "failed to read {}: {err}; using default aim config",
                        path.display()
                    );
                    Self::default()
                }
            }
        })
    }

    /// Looks up recoil and reload settings for a `StanceClass`.
    pub fn class_for(&self, class: StanceClass) -> Option<&AimClassConfig> {
        self.classes.get(stance_class_key(class))
    }
}

/// Cached skeleton joint indices modified by `AimController::apply`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AimJoints {
    pub spine: Option<usize>,
    pub chest: Option<usize>,
    pub neck: Option<usize>,
    pub head: Option<usize>,
    pub upper_arm_r: Option<usize>,
}

impl AimJoints {
    /// Resolves joint indices by canonical skeleton bone name.
    pub fn from_names(names: &[&str]) -> Self {
        let find = |target: &str| names.iter().position(|&n| n == target);
        Self {
            spine: find("spine"),
            chest: find("chest"),
            neck: find("neck"),
            head: find("head"),
            upper_arm_r: find("upper_arm_R"),
        }
    }
}

/// Per-character upper-body look pitch, recoil spring, and reload timer.
#[derive(Clone, Debug)]
pub struct AimController {
    config: AimConfig,
    class: Option<StanceClass>,
    target_pitch: f32,
    smoothed_pitch: f32,
    recoil_deg: f32,
    recoil_vel: f32,
    reload_remaining: f32,
}

impl Default for AimController {
    fn default() -> Self {
        Self::new()
    }
}

impl AimController {
    /// Creates a controller backed by the shared `data/aim.json` config.
    pub fn new() -> Self {
        Self::with_config(AimConfig::load_shared().clone())
    }

    /// Creates a controller with an explicit `AimConfig` (preferred for unit tests).
    pub fn with_config(config: AimConfig) -> Self {
        Self {
            config,
            class: None,
            target_pitch: 0.0,
            smoothed_pitch: 0.0,
            recoil_deg: 0.0,
            recoil_vel: 0.0,
            reload_remaining: 0.0,
        }
    }

    /// Updates the equipped weapon class; stops any active recoil and reload.
    pub fn set_class(&mut self, class: Option<StanceClass>) {
        self.class = class;
        self.recoil_deg = 0.0;
        self.recoil_vel = 0.0;
        self.reload_remaining = 0.0;
    }

    /// Sets camera look pitch in radians (positive looks upward), clamped to configured limits.
    pub fn set_look_pitch(&mut self, radians: f32) {
        if !radians.is_finite() {
            return;
        }
        self.target_pitch = self.clamp_pitch(radians);
    }

    /// Adds one shot's recoil kick for the active weapon class, capped at `maxKickDeg`.
    pub fn fire(&mut self) {
        let Some(cfg) = self.active_class_config() else {
            return;
        };
        let max_kick = self.config.max_kick_deg.max(0.0);
        let kick = cfg.kick_deg.max(0.0);
        self.recoil_deg = (self.recoil_deg + kick).min(max_kick);
        self.recoil_vel = self.recoil_vel.max(0.0);
    }

    /// Returns the reload clip name and playback speed that makes the clip last `seconds`.
    pub fn start_reload(
        &mut self,
        seconds: f32,
        clip_duration: impl Fn(&str) -> Option<f32>,
    ) -> Option<(&str, f32)> {
        if !seconds.is_finite() || seconds <= 0.0 {
            return None;
        }
        let class = self.class?;
        let duration = {
            let clip_name = self.config.class_for(class)?.reload_clip.as_deref()?;
            clip_duration(clip_name)?
        };
        if !duration.is_finite() || duration <= 0.0 {
            return None;
        }
        let speed = duration / seconds;
        self.reload_remaining = seconds;
        let clip_name = self.config.class_for(class)?.reload_clip.as_deref()?;
        Some((clip_name, speed))
    }

    /// Stops any active reload timer immediately.
    pub fn cancel_reload(&mut self) {
        self.reload_remaining = 0.0;
    }

    /// Returns `true` while a reload started via `start_reload` is still running.
    pub fn is_reloading(&self) -> bool {
        self.reload_remaining > 0.0
    }

    /// Advances look-pitch smoothing, closed-form recoil spring, and reload timer by `dt` seconds.
    pub fn tick(&mut self, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }

        let rate = self.config.pitch_rate.max(0.0);
        if rate > 0.0 {
            let alpha = 1.0 - (-rate * dt).exp();
            self.smoothed_pitch = self.clamp_pitch(
                self.smoothed_pitch + (self.target_pitch - self.smoothed_pitch) * alpha,
            );
        }

        if let Some(cfg) = self.active_class_config() {
            let (next_x, next_v) = step_spring_exact(
                self.recoil_deg,
                self.recoil_vel,
                cfg.stiffness,
                cfg.damping,
                dt,
            );
            let max_kick = self.config.max_kick_deg.max(0.0);
            self.recoil_deg = next_x.clamp(-max_kick, max_kick);
            self.recoil_vel = next_v;
            if self.recoil_deg.abs() < SETTLE_EPS && self.recoil_vel.abs() < SETTLE_EPS {
                self.recoil_deg = 0.0;
                self.recoil_vel = 0.0;
            }
        } else {
            self.recoil_deg = 0.0;
            self.recoil_vel = 0.0;
        }

        if self.reload_remaining > 0.0 {
            self.reload_remaining = (self.reload_remaining - dt).max(0.0);
        }
    }

    /// Applies look pitch and weapon recoil to the local pose. `aim_weight` is in `0..=1`.
    pub fn apply(&self, joints: &AimJoints, local: &mut [LocalTransform], aim_weight: f32) {
        let Some(cfg) = self.active_class_config() else {
            return;
        };
        let aim_w = if aim_weight.is_finite() {
            aim_weight.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let carry = self.config.carry_pitch_fraction.clamp(0.0, 1.0);
        let pitch_scale = carry + (1.0 - carry) * aim_w;
        let effective_pitch = self.smoothed_pitch * pitch_scale;
        let recoil_rad = self.recoil_deg.to_radians();

        if effective_pitch.abs() <= APPLY_EPS && recoil_rad.abs() <= APPLY_EPS {
            return;
        }

        let pb = self.config.pitch_bones;
        apply_pitch_x(local, joints.spine, effective_pitch * pb.spine + recoil_rad * RECOIL_SPINE_SHARE);
        apply_pitch_x(local, joints.chest, effective_pitch * pb.chest + recoil_rad * RECOIL_CHEST_SHARE);
        apply_pitch_x(local, joints.neck, effective_pitch * pb.neck);
        apply_pitch_x(local, joints.head, effective_pitch * pb.head);
        apply_pitch_x(local, joints.upper_arm_r, recoil_rad * RECOIL_ARM_SHARE);

        if let Some(chest_idx) = joints.chest
            && chest_idx < local.len()
            && cfg.kick_deg > 0.0
            && cfg.kick_back > 0.0
        {
            let ratio = (self.recoil_deg / cfg.kick_deg).clamp(0.0, MAX_BACK_MULTIPLIER);
            local[chest_idx].0.z += ratio * cfg.kick_back;
        }
    }

    /// Current recoil kick angle in degrees.
    pub fn kick_deg(&self) -> f32 {
        self.recoil_deg
    }

    /// Current smoothed camera pitch in radians.
    pub fn smoothed_pitch(&self) -> f32 {
        self.smoothed_pitch
    }

    fn active_class_config(&self) -> Option<&AimClassConfig> {
        self.class.and_then(|c| self.config.class_for(c))
    }

    fn clamp_pitch(&self, radians: f32) -> f32 {
        let max_up = self.config.max_pitch_up_deg.max(0.0).to_radians();
        let max_down = self.config.max_pitch_down_deg.max(0.0).to_radians();
        radians.clamp(-max_down, max_up)
    }
}

fn apply_pitch_x(local: &mut [LocalTransform], joint: Option<usize>, angle_rad: f32) {
    if angle_rad.abs() <= APPLY_EPS {
        return;
    }
    if let Some(idx) = joint
        && idx < local.len()
    {
        let rot = Quat::from_rotation_x(PITCH_SIGN * angle_rad);
        local[idx].1 = (rot * local[idx].1).normalize();
    }
}

/// Exact analytical step for a damped harmonic oscillator `x'' + c x' + k x = 0`.
/// Unconditionally stable and frame-rate independent for all `dt > 0`.
fn step_spring_exact(x0: f32, v0: f32, stiffness: f32, damping: f32, dt: f32) -> (f32, f32) {
    if !stiffness.is_finite() || stiffness <= 0.0 {
        return (0.0, 0.0);
    }
    let c = if damping.is_finite() { damping.max(0.0) } else { 0.0 };
    let alpha = c * HALF;
    let disc = alpha * alpha - stiffness;

    if disc < -SPRING_CRIT_EPS {
        let omega = (-disc).sqrt();
        let decay = (-alpha * dt).exp();
        let (s, co) = (omega * dt).sin_cos();
        let b = (v0 + alpha * x0) / omega;
        let x = decay * (x0 * co + b * s);
        let v = decay * ((b * omega - x0 * alpha) * co - (x0 * omega + b * alpha) * s);
        (x, v)
    } else if disc <= SPRING_CRIT_EPS {
        let decay = (-alpha * dt).exp();
        let b = v0 + alpha * x0;
        let x = decay * (x0 + b * dt);
        let v = decay * (b - alpha * (x0 + b * dt));
        (x, v)
    } else {
        let gamma = disc.sqrt();
        let r1 = -alpha + gamma;
        let r2 = -alpha - gamma;
        let denom = TWO * gamma;
        let c1 = (v0 - r2 * x0) / denom;
        let c2 = (r1 * x0 - v0) / denom;
        let e1 = (r1 * dt).exp();
        let e2 = (r2 * dt).exp();
        let x = c1 * e1 + c2 * e2;
        let v = c1 * r1 * e1 + c2 * r2 * e2;
        (x, v)
    }
}

const fn default_version() -> u32 {
    DEFAULT_VERSION
}

fn stance_class_key(class: StanceClass) -> &'static str {
    match class {
        StanceClass::Pistol => "pistol",
        StanceClass::Rifle => "rifle",
        StanceClass::Shotgun => "shotgun",
        StanceClass::Melee1H => "melee_1h",
        StanceClass::Melee2H => "melee_2h",
        StanceClass::Spear => "spear",
        StanceClass::Bow => "bow",
        StanceClass::Rod => "rod",
    }
}
