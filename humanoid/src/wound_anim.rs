//! Wounded body language animation state (A15).
//!
//! Below health thresholds the player limps (legs), holds a hand on the side
//! when not using the arms, and droops the spine at very low health. All
//! weights smooth over time and fade out when suppressed by sprint, swim,
//! ladder, car, full-body clips, melee swings/blocking, or dying.

use std::collections::HashMap;
use std::sync::OnceLock;

use engine::anim::Rig;
use pack::Clip;
use serde::Deserialize;

use crate::mask::{BoneMasks, MaskId};
use crate::pose_buffer::{LocalTransform, blend_pose_masked};

/// Looping limp walk clip (applied to `MaskId::Lower`).
pub const LIMP_WALK_CLIP: &str = "limp_walk_left";
/// Looping upper-body clip holding the left hand against the side (`MaskId::LeftArm`).
pub const HOLD_SIDE_CLIP: &str = "hold_side";
/// Looping hunched pose used for low-health spine droop (`MaskId::Spine`).
pub const DROOP_CLIP: &str = "wounded_idle";

const WEIGHT_EPSILON: f32 = 0.001; // new: DECISIONS 2026-11-24

/// Tuning values for wounded body language from `data/wound_anim.json`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WoundAnimConfig {
    pub version: u32,
    pub limp_threshold: f32,
    pub hold_side_threshold: f32,
    pub droop_threshold: f32,
    pub limp_weight: f32,
    pub hold_side_weight: f32,
    pub droop_weight: f32,
    pub limp_min_speed: f32,
    pub limp_full_speed: f32,
    pub blend_rate: f32,
    pub fade_out_rate: f32,
}

impl Default for WoundAnimConfig {
    fn default() -> Self {
        Self {
            version: 1,                // data: wound_anim.json version
            limp_threshold: 0.55,      // data: wound_anim.json limpThreshold
            hold_side_threshold: 0.45, // data: wound_anim.json holdSideThreshold
            droop_threshold: 0.25,     // data: wound_anim.json droopThreshold
            limp_weight: 0.85,         // data: wound_anim.json limpWeight
            hold_side_weight: 0.85,    // data: wound_anim.json holdSideWeight
            droop_weight: 0.75,        // data: wound_anim.json droopWeight
            limp_min_speed: 0.35,      // data: wound_anim.json limpMinSpeed
            limp_full_speed: 1.8,      // data: wound_anim.json limpFullSpeed
            blend_rate: 4.0,           // data: wound_anim.json blendRate
            fade_out_rate: 12.0,       // data: wound_anim.json fadeOutRate
        }
    }
}

impl WoundAnimConfig {
    /// Parses a `WoundAnimConfig` from JSON text.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Loads and caches `data/wound_anim.json`, falling back to defaults.
    pub fn load_shared() -> &'static Self {
        static CONFIG: OnceLock<WoundAnimConfig> = OnceLock::new();
        CONFIG.get_or_init(|| {
            let path = paths::repo_path("data/wound_anim.json");
            match std::fs::read_to_string(&path) {
                Ok(text) => match Self::from_json(&text) {
                    Ok(cfg) => cfg,
                    Err(err) => {
                        log::warn!("failed to parse {}: {err}; using defaults", path.display());
                        Self::default()
                    }
                },
                Err(err) => {
                    log::warn!("failed to read {}: {err}; using defaults", path.display());
                    Self::default()
                }
            }
        })
    }
}

/// Per-frame context used to suppress wounds during incompatible states.
#[derive(Clone, Copy, Debug, Default)]
pub struct WoundContext {
    pub speed: f32,
    pub crouching: bool,
    pub sprinting: bool,
    pub swimming: bool,
    pub on_ladder: bool,
    pub in_car: bool,
    pub full_body: bool,
    pub using_arms: bool,
    pub dying: bool,
}

/// Smoothed wound animation weights and clip clocks.
#[derive(Clone, Debug)]
pub struct WoundAnimState {
    config: WoundAnimConfig,
    driven: bool,
    health: f32,
    limp_weight: f32,
    hold_side_weight: f32,
    droop_weight: f32,
    limp_time: f32,
    hold_side_time: f32,
    droop_time: f32,
}

impl Default for WoundAnimState {
    fn default() -> Self {
        Self::new()
    }
}

impl WoundAnimState {
    /// Creates an inert state using the shared `data/wound_anim.json` config.
    pub fn new() -> Self {
        Self::with_config(WoundAnimConfig::load_shared().clone())
    }

    /// Creates an inert state with an explicit config (preferred for tests).
    pub fn with_config(config: WoundAnimConfig) -> Self {
        Self {
            config,
            driven: false,
            health: 1.0, // new: DECISIONS 2026-11-24
            limp_weight: 0.0,
            hold_side_weight: 0.0,
            droop_weight: 0.0,
            limp_time: 0.0,
            hold_side_time: 0.0,
            droop_time: 0.0,
        }
    }

    /// Feeds the player's current health fraction in `0.0..=1.0`.
    pub fn set_health(&mut self, frac: f32) {
        self.health = if frac.is_finite() { frac.clamp(0.0, 1.0) } else { 1.0 };
        self.driven = true;
    }

    /// True once `set_health` has been called at least once.
    pub fn is_driven(&self) -> bool {
        self.driven
    }

    pub fn limp_weight(&self) -> f32 {
        self.limp_weight
    }

    pub fn hold_side_weight(&self) -> f32 {
        self.hold_side_weight
    }

    pub fn droop_weight(&self) -> f32 {
        self.droop_weight
    }

    /// Advances wound weights and clip clocks by `dt` seconds.
    pub fn tick(&mut self, dt: f32, ctx: WoundContext) {
        if !self.driven {
            return;
        }
        let dt = if dt.is_finite() && dt > 0.0 { dt } else { 0.0 };
        let cfg = &self.config;

        let body_suppressed = ctx.dying
            || ctx.sprinting
            || ctx.crouching
            || ctx.swimming
            || ctx.on_ladder
            || ctx.in_car
            || ctx.full_body;

        let (limp_target, hold_target, droop_target) = if body_suppressed {
            (0.0, 0.0, 0.0)
        } else {
            let move_factor = ramp(ctx.speed, cfg.limp_min_speed, cfg.limp_full_speed);
            let limp = severity(self.health, cfg.limp_threshold) * move_factor * cfg.limp_weight;
            let hold = if ctx.using_arms {
                0.0
            } else {
                severity(self.health, cfg.hold_side_threshold) * cfg.hold_side_weight
            };
            let droop = severity(self.health, cfg.droop_threshold) * cfg.droop_weight;
            (limp, hold, droop)
        };

        let limp_rate = if limp_target < self.limp_weight && body_suppressed {
            cfg.fade_out_rate
        } else {
            cfg.blend_rate
        };
        let hold_rate = if hold_target < self.hold_side_weight && (body_suppressed || ctx.using_arms) {
            cfg.fade_out_rate
        } else {
            cfg.blend_rate
        };
        let droop_rate = if droop_target < self.droop_weight && body_suppressed {
            cfg.fade_out_rate
        } else {
            cfg.blend_rate
        };

        self.limp_weight = approach(self.limp_weight, limp_target, limp_rate, dt);
        self.hold_side_weight = approach(self.hold_side_weight, hold_target, hold_rate, dt);
        self.droop_weight = approach(self.droop_weight, droop_target, droop_rate, dt);

        if self.limp_weight > WEIGHT_EPSILON {
            self.limp_time += dt;
        } else {
            self.limp_time = 0.0;
        }
        if self.hold_side_weight > WEIGHT_EPSILON {
            self.hold_side_time += dt;
        } else {
            self.hold_side_time = 0.0;
        }
        if self.droop_weight > WEIGHT_EPSILON {
            self.droop_time += dt;
        } else {
            self.droop_time = 0.0;
        }
    }

    /// Blends active wound clips onto `base` using `scratch`.
    pub fn apply(
        &self,
        rig: &Rig,
        clips: &HashMap<String, Clip>,
        masks: &BoneMasks,
        base: &mut [LocalTransform],
        scratch: &mut [LocalTransform],
    ) {
        if !self.driven {
            return;
        }
        blend_clip(
            rig,
            clips.get(LIMP_WALK_CLIP),
            self.limp_time,
            self.limp_weight,
            masks.weights(MaskId::Lower),
            base,
            scratch,
        );
        blend_clip(
            rig,
            clips.get(DROOP_CLIP),
            self.droop_time,
            self.droop_weight,
            masks.weights(MaskId::Spine),
            base,
            scratch,
        );
        blend_clip(
            rig,
            clips.get(HOLD_SIDE_CLIP),
            self.hold_side_time,
            self.hold_side_weight,
            masks.weights(MaskId::LeftArm),
            base,
            scratch,
        );
    }
}

fn blend_clip(
    rig: &Rig,
    clip: Option<&Clip>,
    time: f32,
    weight: f32,
    mask: &[f32],
    base: &mut [LocalTransform],
    scratch: &mut [LocalTransform],
) {
    if weight <= WEIGHT_EPSILON {
        return;
    }
    let Some(c) = clip else {
        return;
    };
    rig.local_pose_into(c, time, scratch);
    blend_pose_masked(base, scratch, weight, mask);
}

fn severity(health: f32, threshold: f32) -> f32 {
    if threshold <= 0.0 || health >= threshold {
        0.0
    } else {
        ((threshold - health) / threshold).clamp(0.0, 1.0)
    }
}

fn ramp(value: f32, start: f32, full: f32) -> f32 {
    let span = full - start;
    if span <= 0.0 {
        return if value >= full { 1.0 } else { 0.0 };
    }
    ((value - start) / span).clamp(0.0, 1.0)
}

fn approach(current: f32, target: f32, rate: f32, dt: f32) -> f32 {
    let cur = if current.is_finite() { current.clamp(0.0, 1.0) } else { 0.0 };
    let target = if target.is_finite() { target.clamp(0.0, 1.0) } else { 0.0 };
    if !rate.is_finite() || rate <= 0.0 || dt <= 0.0 {
        return cur;
    }
    let k = 1.0 - (-rate * dt).exp();
    (cur + (target - cur) * k).clamp(0.0, 1.0)
}
