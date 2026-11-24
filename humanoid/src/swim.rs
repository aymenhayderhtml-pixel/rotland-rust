//! Wading, swimming, treading water and climbing out animation state (A08).
//!
//! Converts lake water depth, deep-water state, horizontal speed and stamina into
//! clip choices, smoothed wading weight, swim playback tempo and a one-shot climb-out
//! trigger. Drives the existing `Air` layer (full-body while swimming, lower-body
//! while wading on the ground) and leaves the animator untouched until `set_input`
//! is called.

use std::sync::OnceLock;

use serde::Deserialize;

use crate::layers::{LayerId, LayerStack};
use crate::mask::MaskId;

/// Lower-body wading loop played while walking through shallow-to-chest water.
pub const WADE_WALK_CLIP: &str = "wade_walk";
/// Full-body upright treading loop played in deep water when slow or still.
pub const SWIM_TREAD_CLIP: &str = "swim_tread";
/// Full-body breaststroke loop played in deep water while moving.
pub const SWIM_FORWARD_CLIP: &str = "swim_forward";
/// Full-body struggling treading loop played in deep water when stamina is low.
pub const SWIM_TIRED_CLIP: &str = "swim_tired";
/// One-shot full-body climb-out clip played when leaving deep water.
pub const SWIM_CLIMB_OUT_CLIP: &str = "swim_climb_out";

const JUMP_FALLBACK_CLIP: &str = "jump";
const WEIGHT_EPSILON: f32 = 0.001; // new: DECISIONS 2026-11-24
const TIRED_HYSTERESIS: f32 = 0.05; // new: DECISIONS 2026-11-24
const SPEED_HYSTERESIS: f32 = 0.15; // new: DECISIONS 2026-11-24
const SWIM_MIN_TEMPO: f32 = 0.7; // new: DECISIONS 2026-11-24
const SWIM_MAX_TEMPO: f32 = 1.4; // new: DECISIONS 2026-11-24

/// Tuning values for wading and swimming animations from `data/swim_anim.json`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SwimAnimConfig {
    pub version: u32,
    pub wade_min_depth: f32,
    pub wade_full_depth: f32,
    pub swim_move_speed: f32,
    pub swim_full_speed: f32,
    pub tired_stamina: f32,
    pub blend_rate: f32,
    pub climb_out_seconds: f32,
}

impl Default for SwimAnimConfig {
    fn default() -> Self {
        Self {
            version: 1,             // data: swim_anim.json version
            wade_min_depth: 0.35,   // data: swim_anim.json wadeMinDepth
            wade_full_depth: 1.0,   // data: swim_anim.json wadeFullDepth
            swim_move_speed: 0.6,   // data: swim_anim.json swimMoveSpeed
            swim_full_speed: 2.5,   // data: swim_anim.json swimFullSpeed
            tired_stamina: 0.25,    // data: swim_anim.json tiredStamina
            blend_rate: 6.0,        // data: swim_anim.json blendRate
            climb_out_seconds: 1.1, // data: swim_anim.json climbOutSeconds
        }
    }
}

impl SwimAnimConfig {
    /// Parses a `SwimAnimConfig` from JSON text.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Loads and caches `data/swim_anim.json`, falling back to defaults.
    pub fn load_shared() -> &'static SwimAnimConfig {
        static CONFIG: OnceLock<SwimAnimConfig> = OnceLock::new();
        CONFIG.get_or_init(|| {
            let path = paths::repo_path("data/swim_anim.json");
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

/// High-level water animation phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaterPhase {
    Dry,
    Wading,
    Treading,
    Swimming,
    Tired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AirOwner {
    None,
    Swim,
    Wade,
}

/// Water animation state machine with hysteresis and smoothed wading weight.
#[derive(Clone, Debug)]
pub struct SwimState {
    config: SwimAnimConfig,
    driven: bool,
    depth: f32,
    swimming: bool,
    speed: f32,
    stamina: f32,
    was_swimming: bool,
    tired_on: bool,
    moving_on: bool,
    phase: WaterPhase,
    wade_weight: f32,
    air_owner: AirOwner,
}

impl Default for SwimState {
    fn default() -> Self {
        Self::new()
    }
}

impl SwimState {
    /// Creates an inert state using the shared `data/swim_anim.json` config.
    pub fn new() -> Self {
        Self::with_config(SwimAnimConfig::load_shared().clone())
    }

    /// Creates an inert state with an explicit config (preferred for tests).
    pub fn with_config(config: SwimAnimConfig) -> Self {
        Self {
            config,
            driven: false,
            depth: 0.0,
            swimming: false,
            speed: 0.0,
            stamina: 1.0, // new: DECISIONS 2026-11-24
            was_swimming: false,
            tired_on: false,
            moving_on: false,
            phase: WaterPhase::Dry,
            wade_weight: 0.0,
            air_owner: AirOwner::None,
        }
    }

    /// Feeds water depth above the feet (0 on land), deep-water flag, horizontal speed
    /// and stamina fraction. Until called, the state stays inert.
    pub fn set_input(&mut self, depth: f32, swimming: bool, speed: f32, stamina_frac: f32) {
        self.depth = if depth.is_finite() { depth.max(0.0) } else { 0.0 };
        self.swimming = swimming;
        self.speed = if speed.is_finite() { speed.max(0.0) } else { 0.0 };
        self.stamina = if stamina_frac.is_finite() {
            stamina_frac.clamp(0.0, 1.0)
        } else {
            1.0 // new: DECISIONS 2026-11-24
        };
        self.driven = true;
    }

    /// Advances phase and smoothed weights by `dt` seconds.
    /// Returns `true` exactly once when the player leaves deep water.
    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.driven {
            return false;
        }
        let dt = if dt.is_finite() && dt > 0.0 { dt } else { 0.0 };
        let cfg = &self.config;
        let climb_out = self.was_swimming && !self.swimming;

        if self.swimming {
            if self.stamina <= cfg.tired_stamina {
                self.tired_on = true;
            } else if self.stamina >= cfg.tired_stamina + TIRED_HYSTERESIS {
                self.tired_on = false;
            }

            let exit_speed = (cfg.swim_move_speed - SPEED_HYSTERESIS).max(0.0);
            if self.speed >= cfg.swim_move_speed {
                self.moving_on = true;
            } else if self.speed <= exit_speed {
                self.moving_on = false;
            }

            self.phase = if self.tired_on {
                WaterPhase::Tired
            } else if self.moving_on {
                WaterPhase::Swimming
            } else {
                WaterPhase::Treading
            };
            self.wade_weight = 0.0;
        } else {
            self.tired_on = false;
            self.moving_on = false;
            let wade_target = ramp(self.depth, cfg.wade_min_depth, cfg.wade_full_depth);
            self.wade_weight = approach(self.wade_weight, wade_target, cfg.blend_rate, dt);
            self.phase = if self.depth >= cfg.wade_min_depth {
                WaterPhase::Wading
            } else {
                WaterPhase::Dry
            };
        }

        self.was_swimming = self.swimming;
        climb_out
    }

    /// Current water phase.
    pub fn phase(&self) -> WaterPhase {
        self.phase
    }

    /// True while in deep water (`Treading`, `Swimming`, or `Tired`).
    pub fn is_swimming(&self) -> bool {
        matches!(self.phase, WaterPhase::Treading | WaterPhase::Swimming | WaterPhase::Tired)
    }

    /// Clip to play on the `Air` layer while swimming, or `None` when not swimming.
    pub fn swim_clip(&self) -> Option<&'static str> {
        match self.phase {
            WaterPhase::Treading => Some(SWIM_TREAD_CLIP),
            WaterPhase::Swimming => Some(SWIM_FORWARD_CLIP),
            WaterPhase::Tired => Some(SWIM_TIRED_CLIP),
            WaterPhase::Dry | WaterPhase::Wading => None,
        }
    }

    /// Playback speed for `swim_forward` scaled from horizontal speed (0 when not driven).
    pub fn swim_tempo(&self) -> f32 {
        if !self.driven {
            return 0.0;
        }
        let t = ramp(self.speed, self.config.swim_move_speed, self.config.swim_full_speed);
        SWIM_MIN_TEMPO + (SWIM_MAX_TEMPO - SWIM_MIN_TEMPO) * t
    }

    /// Smoothed wading blend in `0..=1` by water depth between `wadeMinDepth` and `wadeFullDepth`.
    pub fn wade_weight(&self) -> f32 {
        self.wade_weight
    }

    /// True while the smoothed wading weight is still visible on the legs.
    pub fn wants_wade(&self) -> bool {
        self.driven && !self.is_swimming() && self.wade_weight > WEIGHT_EPSILON
    }

    /// Short fade times for the climb-out full-body clip derived from `blendRate`.
    pub fn climb_out_fades(&self) -> (f32, f32) {
        let rate = self.config.blend_rate;
        let fade = if rate > 0.0 { 1.0 / rate } else { 0.0 };
        (fade, fade)
    }

    /// Playback speed for `swim_climb_out` to match `climbOutSeconds`.
    pub fn climb_out_speed(&self, clip_duration: f32) -> f32 {
        if clip_duration.is_finite()
            && clip_duration > 0.0
            && self.config.climb_out_seconds > 0.0
        {
            clip_duration / self.config.climb_out_seconds
        } else {
            1.0 // new: DECISIONS 2026-11-24
        }
    }

    /// Drives the full-body `Air` layer with the active deep-water swimming clip.
    pub fn apply_swim(&mut self, layers: &mut LayerStack) {
        let Some(clip) = self.swim_clip() else {
            return;
        };
        let layer = layers.get_mut(LayerId::Air);
        layer.mask = MaskId::Full;
        if layer.clip.as_deref() != Some(clip) {
            layer.clip = Some(clip.to_owned());
            layer.time = 0.0;
        }
        layer.target = 1.0; // new: DECISIONS 2026-11-24
        layer.speed = if clip == SWIM_FORWARD_CLIP {
            self.swim_tempo()
        } else {
            1.0 // new: DECISIONS 2026-11-24
        };
        self.air_owner = AirOwner::Swim;
    }

    /// Blends `wade_walk` onto the lower body using the idle `Air` layer while grounded.
    pub fn apply_wade(&mut self, layers: &mut LayerStack) {
        let layer = layers.get_mut(LayerId::Air);
        layer.mask = MaskId::Lower;
        if layer.clip.as_deref() != Some(WADE_WALK_CLIP) {
            layer.clip = Some(WADE_WALK_CLIP.to_owned());
            layer.time = 0.0;
        }
        layer.weight = self.wade_weight;
        layer.target = self.wade_weight;
        layer.speed = 1.0; // new: DECISIONS 2026-11-24
        self.air_owner = AirOwner::Wade;
    }

    /// Restores the `Air` layer to full-body jump defaults when water releases it.
    pub fn release_air(&mut self, layers: &mut LayerStack) {
        if self.air_owner == AirOwner::None {
            return;
        }
        let layer = layers.get_mut(LayerId::Air);
        layer.mask = MaskId::Full;
        layer.clip = Some(JUMP_FALLBACK_CLIP.to_owned());
        layer.time = 0.0;
        layer.weight = 0.0;
        layer.target = 0.0;
        layer.speed = 1.0; // new: DECISIONS 2026-11-24
        self.air_owner = AirOwner::None;
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

/// Frame-rate independent exponential smoothing that stays finite for any `dt`.
fn approach(current: f32, target: f32, rate: f32, dt: f32) -> f32 {
    let cur = if current.is_finite() { current.clamp(0.0, 1.0) } else { 0.0 };
    let target = if target.is_finite() { target.clamp(0.0, 1.0) } else { 0.0 };
    if !rate.is_finite() || rate <= 0.0 || dt <= 0.0 {
        return cur;
    }
    let k = 1.0 - (-rate * dt).exp();
    (cur + (target - cur) * k).clamp(0.0, 1.0)
}
