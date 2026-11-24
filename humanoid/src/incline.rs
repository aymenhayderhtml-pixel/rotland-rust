//! Stairs and steep-slope lower-body animation state (A10).
//!
//! Estimates smoothed ground grade from frame-to-frame world position changes
//! and blends `stairs_up` or `stairs_down` onto the lower body using the idle
//! `Air` layer when grounded on steep inclines. Stays inert until `set_input`
//! is called so undriven callers remain unchanged.

use std::sync::OnceLock;

use glam::Vec3;
use serde::Deserialize;

use crate::layers::{LayerId, LayerStack};
use crate::mask::MaskId;

/// Lower-body looping clip played while walking up stairs or steep slopes.
pub const STAIRS_UP_CLIP: &str = "stairs_up";
/// Lower-body looping clip played while walking down stairs or steep slopes.
pub const STAIRS_DOWN_CLIP: &str = "stairs_down";

const JUMP_FALLBACK_CLIP: &str = "jump";
const WEIGHT_EPSILON: f32 = 0.001; // new: DECISIONS 2026-11-24
const HORIZ_EPSILON: f32 = 1e-4; // new: DECISIONS 2026-11-24
const MAX_RAW_GRADE: f32 = 1.2; // new: DECISIONS 2026-11-24
const MIN_TEMPO: f32 = 0.5; // new: DECISIONS 2026-11-24
const MAX_TEMPO: f32 = 1.8; // new: DECISIONS 2026-11-24
const DEFAULT_SPEED: f32 = 1.0; // new: DECISIONS 2026-11-24

/// Tuning values for stairs and steep-slope leg blends from `data/incline.json`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InclineConfig {
    pub version: u32,
    pub min_grade: f32,
    pub full_grade: f32,
    pub min_speed: f32,
    pub grade_smoothing: f32,
    pub weight_rate: f32,
    pub max_weight: f32,
    pub teleport_distance: f32,
    pub tempo_ref_speed: f32,
}

impl Default for InclineConfig {
    fn default() -> Self {
        Self {
            version: 1,              // data: incline.json version
            min_grade: 0.25,         // data: incline.json minGrade
            full_grade: 0.6,         // data: incline.json fullGrade
            min_speed: 0.5,          // data: incline.json minSpeed
            grade_smoothing: 8.0,    // data: incline.json gradeSmoothing
            weight_rate: 6.0,        // data: incline.json weightRate
            max_weight: 0.85,        // data: incline.json maxWeight
            teleport_distance: 1.5,  // data: incline.json teleportDistance
            tempo_ref_speed: 3.0,    // data: incline.json tempoRefSpeed
        }
    }
}

impl InclineConfig {
    /// Parses an `InclineConfig` from JSON text.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Loads and caches `data/incline.json`, falling back to defaults.
    pub fn load_shared() -> &'static InclineConfig {
        static CONFIG: OnceLock<InclineConfig> = OnceLock::new();
        CONFIG.get_or_init(|| {
            let path = paths::repo_path("data/incline.json");
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

/// Smoothed ground-grade and stairs blend state.
#[derive(Clone, Debug)]
pub struct InclineState {
    config: InclineConfig,
    driven: bool,
    position: Vec3,
    last_pos: Option<Vec3>,
    on_floor: bool,
    grade: f32,
    signed_weight: f32,
    horiz_speed: f32,
    owns_air: bool,
}

impl Default for InclineState {
    fn default() -> Self {
        Self::new()
    }
}

impl InclineState {
    /// Creates an inert state using the shared `data/incline.json` config.
    pub fn new() -> Self {
        Self::with_config(InclineConfig::load_shared().clone())
    }

    /// Creates an inert state with an explicit config (preferred for tests).
    pub fn with_config(config: InclineConfig) -> Self {
        Self {
            config,
            driven: false,
            position: Vec3::ZERO,
            last_pos: None,
            on_floor: false,
            grade: 0.0,
            signed_weight: 0.0,
            horiz_speed: 0.0,
            owns_air: false,
        }
    }

    /// The character's world position and whether it is on the ground this frame.
    pub fn set_input(&mut self, position: Vec3, on_floor: bool) {
        if position.is_finite() {
            self.position = position;
            if self.last_pos.is_none() {
                self.last_pos = Some(position);
            }
        }
        self.on_floor = on_floor;
        self.driven = true;
    }

    /// Advances smoothed grade and incline blend weights by `dt` seconds.
    pub fn tick(&mut self, dt: f32) {
        if !self.driven || !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let Some(prev) = self.last_pos else {
            self.last_pos = Some(self.position);
            return;
        };
        let delta = self.position - prev;
        self.last_pos = Some(self.position);

        let cfg = &self.config;
        if !delta.is_finite() || delta.length() > cfg.teleport_distance {
            self.grade = 0.0;
            self.signed_weight = 0.0;
            self.horiz_speed = 0.0;
            return;
        }

        let horiz = delta.x.hypot(delta.z);
        let speed = horiz / dt;
        let moving_on_floor = self.on_floor && speed > cfg.min_speed && horiz > HORIZ_EPSILON;
        self.horiz_speed = if self.on_floor { speed } else { 0.0 };

        let limit = cfg.max_weight.max(0.0);
        if moving_on_floor {
            let raw_grade = (delta.y / horiz).clamp(-MAX_RAW_GRADE, MAX_RAW_GRADE);
            self.grade = approach_signed(self.grade, raw_grade, cfg.grade_smoothing, dt)
                .clamp(-MAX_RAW_GRADE, MAX_RAW_GRADE);
            let mag = ramp(self.grade.abs(), cfg.min_grade, cfg.full_grade) * limit;
            let target = if self.grade >= 0.0 { mag } else { -mag };
            self.signed_weight =
                approach_signed(self.signed_weight, target, cfg.weight_rate, dt)
                    .clamp(-limit, limit);
        } else {
            self.grade = approach_signed(self.grade, 0.0, cfg.grade_smoothing, dt)
                .clamp(-MAX_RAW_GRADE, MAX_RAW_GRADE);
            self.signed_weight =
                approach_signed(self.signed_weight, 0.0, cfg.weight_rate, dt)
                    .clamp(-limit, limit);
        }
    }

    /// Smoothed signed ground grade (+ uphill, - downhill).
    pub fn grade(&self) -> f32 {
        self.grade
    }

    /// Uphill stairs blend weight in `0..=maxWeight` (mutually exclusive with `down_weight`).
    pub fn up_weight(&self) -> f32 {
        self.signed_weight.max(0.0)
    }

    /// Downhill stairs blend weight in `0..=maxWeight` (mutually exclusive with `up_weight`).
    pub fn down_weight(&self) -> f32 {
        (-self.signed_weight).max(0.0)
    }

    /// Playback speed for the stairs clip scaled to `speed` (0 when not driven).
    pub fn tempo(&self, speed: f32) -> f32 {
        if !self.driven {
            return 0.0;
        }
        if !speed.is_finite() || speed <= 0.0 || self.config.tempo_ref_speed <= 0.0 {
            return MIN_TEMPO;
        }
        (speed / self.config.tempo_ref_speed).clamp(MIN_TEMPO, MAX_TEMPO)
    }

    /// True while a non-negligible stairs blend weight is active.
    pub fn wants_incline(&self) -> bool {
        self.active_clip().is_some()
    }

    /// Blends `stairs_up` or `stairs_down` onto the lower body using the idle `Air` layer.
    pub fn apply_air(&mut self, layers: &mut LayerStack) {
        let Some((clip, weight)) = self.active_clip() else {
            self.release_air(layers);
            return;
        };
        let layer = layers.get_mut(LayerId::Air);
        layer.mask = MaskId::Lower;
        if layer.clip.as_deref() != Some(clip) {
            layer.clip = Some(clip.to_owned());
            layer.time = 0.0;
        }
        layer.weight = weight;
        layer.target = weight;
        layer.speed = self.tempo(self.horiz_speed);
        self.owns_air = true;
    }

    /// Restores the `Air` layer to full-body jump defaults when incline releases it.
    pub fn release_air(&mut self, layers: &mut LayerStack) {
        if !self.owns_air {
            return;
        }
        let layer = layers.get_mut(LayerId::Air);
        layer.mask = MaskId::Full;
        if layer.clip.as_deref() != Some(JUMP_FALLBACK_CLIP) {
            layer.clip = Some(JUMP_FALLBACK_CLIP.to_owned());
        }
        layer.time = 0.0;
        layer.weight = 0.0;
        layer.target = 0.0;
        layer.speed = DEFAULT_SPEED;
        self.owns_air = false;
    }

    fn active_clip(&self) -> Option<(&'static str, f32)> {
        if !self.driven {
            return None;
        }
        if self.signed_weight > WEIGHT_EPSILON {
            Some((STAIRS_UP_CLIP, self.signed_weight))
        } else if -self.signed_weight > WEIGHT_EPSILON {
            Some((STAIRS_DOWN_CLIP, -self.signed_weight))
        } else {
            None
        }
    }
}

/// Linear ramp from `start` (0) to `full` (1).
fn ramp(value: f32, start: f32, full: f32) -> f32 {
    let span = full - start;
    if span <= 0.0 {
        return if value >= full { 1.0 } else { 0.0 };
    }
    ((value - start) / span).clamp(0.0, 1.0)
}

/// Frame-rate independent exponential smoothing for signed values.
fn approach_signed(current: f32, target: f32, rate: f32, dt: f32) -> f32 {
    let cur = if current.is_finite() { current } else { 0.0 };
    let dst = if target.is_finite() { target } else { 0.0 };
    if !rate.is_finite() || rate <= 0.0 || dt <= 0.0 {
        return cur;
    }
    let k = 1.0 - (-rate * dt).exp();
    cur + (dst - cur) * k
}
