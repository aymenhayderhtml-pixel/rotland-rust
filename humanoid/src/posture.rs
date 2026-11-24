//! Crouch walking, sprint style and the tired body (A07).
//!
//! Turns the player's speed, crouch, sprint and stamina into clip choices and layer
//! weights. It only drives the existing layers (`Crouch` for the sprint body, `Aim` for
//! crouch walk, sprint arms and catch breath, `Additive` for tired breathing), so the
//! animation pipeline itself is unchanged. When `set_input` is never called the state is
//! inert and the animator behaves exactly as before this job.

use std::sync::OnceLock;

use serde::Deserialize;

use crate::layers::{LayerId, LayerStack};
use crate::mask::MaskId;

/// Clip played on the `Aim` layer while the player crouch-walks (lower body only).
pub const CROUCH_WALK_CLIP: &str = "crouch_walk";
/// Full-body sprint cycle, played on the `Crouch` layer while sprinting.
pub const SPRINT_CLIP: &str = "sprint";
/// Upper-body clip holding a weapon low across the chest while sprinting.
pub const SPRINT_ARMS_WEAPON_CLIP: &str = "sprint_arms_weapon";
/// Small hunch and chest heave added on top of any pose by the `Additive` layer.
pub const TIRED_BREATH_CLIP: &str = "tired_breath";
/// One-shot bent-over breathing pose after a sprint that ran out of stamina.
pub const CATCH_BREATH_CLIP: &str = "catch_breath";
/// Clip the `Crouch` layer goes back to when the sprint body is released.
const CROUCH_CLIP: &str = "crouch";
/// Weights below this are treated as off so layers are not touched for nothing.
const WEIGHT_EPSILON: f32 = 0.001; // new: DECISIONS 2026-11-24

/// Tuning values for crouch walk, sprint and tired behaviour from `data/posture.json`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PostureConfig {
    pub version: u32,
    pub crouch_walk_start_speed: f32,
    pub crouch_walk_full_speed: f32,
    pub crouch_walk_blend_rate: f32,
    pub crouch_walk_natural_speed: f32,
    pub crouch_walk_min_playback: f32,
    pub crouch_walk_max_playback: f32,
    pub sprint_min_speed: f32,
    pub sprint_blend_rate: f32,
    pub tired_stamina_enter: f32,
    pub tired_stamina_exit: f32,
    pub tired_blend_rate: f32,
    pub tired_breath_weight: f32,
    pub catch_breath_stamina: f32,
    pub catch_breath_seconds: f32,
    pub catch_breath_fade_in: f32,
    pub catch_breath_fade_out: f32,
    pub catch_breath_blend_rate: f32,
    pub catch_breath_max_speed: f32,
}

impl Default for PostureConfig {
    fn default() -> Self {
        Self {
            version: 1,                                     // data: posture.json version
            crouch_walk_start_speed: 0.35,                  // data: posture.json crouchWalkStartSpeed
            crouch_walk_full_speed: 1.8,                    // data: posture.json crouchWalkFullSpeed
            crouch_walk_blend_rate: 9.0,                    // data: posture.json crouchWalkBlendRate
            crouch_walk_natural_speed: 1.6,                 // data: posture.json crouchWalkNaturalSpeed
            crouch_walk_min_playback: 0.5,                  // data: posture.json crouchWalkMinPlayback
            crouch_walk_max_playback: 1.6,                  // data: posture.json crouchWalkMaxPlayback
            sprint_min_speed: 6.5,                          // data: posture.json sprintMinSpeed
            sprint_blend_rate: 10.0,                        // data: posture.json sprintBlendRate
            tired_stamina_enter: 0.35,                      // data: posture.json tiredStaminaEnter
            tired_stamina_exit: 0.5,                        // data: posture.json tiredStaminaExit
            tired_blend_rate: 3.0,                          // data: posture.json tiredBlendRate
            tired_breath_weight: 0.85,                      // data: posture.json tiredBreathWeight
            catch_breath_stamina: 0.45,                     // data: posture.json catchBreathStamina
            catch_breath_seconds: 2.0,                      // data: posture.json catchBreathSeconds
            catch_breath_fade_in: 0.35,                     // data: posture.json catchBreathFadeIn
            catch_breath_fade_out: 0.6,                     // data: posture.json catchBreathFadeOut
            catch_breath_blend_rate: 8.0,                   // data: posture.json catchBreathBlendRate
            catch_breath_max_speed: 1.0,                    // data: posture.json catchBreathMaxSpeed
        }
    }
}

impl PostureConfig {
    /// Parses a `PostureConfig` from JSON text.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Loads and caches `data/posture.json`, falling back to defaults.
    pub fn load_shared() -> &'static PostureConfig {
        static CONFIG: OnceLock<PostureConfig> = OnceLock::new();
        CONFIG.get_or_init(|| {
            let path = paths::repo_path("data/posture.json");
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

    /// Playback speed of the crouch walk cycle so the feet follow `speed`.
    pub fn crouch_walk_playback(&self, speed: f32) -> f32 {
        if self.crouch_walk_natural_speed <= 0.0 {
            return 1.0;
        }
        (speed / self.crouch_walk_natural_speed)
            .clamp(self.crouch_walk_min_playback, self.crouch_walk_max_playback)
    }
}

/// Crouch walk, sprint, tired breathing and catch-breath state.
#[derive(Clone, Debug)]
pub struct PostureState {
    config: PostureConfig,
    driven: bool,
    speed: f32,
    crouching: bool,
    sprinting: bool,
    stamina: f32,
    was_sprinting: bool,
    tired_on: bool,
    crouch_walk: f32,
    sprint: f32,
    tired: f32,
    catch_active: bool,
    catch_time: f32,
    catch_weight: f32,
    owns_crouch: bool,
    owns_aim: bool,
    owns_breath: bool,
}

impl Default for PostureState {
    fn default() -> Self {
        Self::new()
    }
}

impl PostureState {
    /// Creates an inert state using the shared `data/posture.json` config.
    pub fn new() -> Self {
        Self::with_config(PostureConfig::load_shared().clone())
    }

    /// Creates an inert state with an explicit config (preferred for tests).
    pub fn with_config(config: PostureConfig) -> Self {
        Self {
            config,
            driven: false,
            speed: 0.0,
            crouching: false,
            sprinting: false,
            stamina: 1.0,
            was_sprinting: false,
            tired_on: false,
            crouch_walk: 0.0,
            sprint: 0.0,
            tired: 0.0,
            catch_active: false,
            catch_time: 0.0,
            catch_weight: 0.0,
            owns_crouch: false,
            owns_aim: false,
            owns_breath: false,
        }
    }

    /// Feeds this frame's movement. Until this is called the state is inert and the
    /// animator layers are left exactly as earlier jobs set them.
    pub fn set_input(&mut self, speed: f32, crouching: bool, sprinting: bool, stamina_frac: f32) {
        self.speed = if speed.is_finite() { speed.max(0.0) } else { 0.0 };
        self.crouching = crouching;
        self.sprinting = sprinting;
        self.stamina = if stamina_frac.is_finite() {
            stamina_frac.clamp(0.0, 1.0)
        } else {
            1.0
        };
        self.driven = true;
    }

    /// True once `set_input` has been called at least once.
    pub fn is_driven(&self) -> bool {
        self.driven
    }

    /// Advances all posture weights and catch-breath timers by `dt` seconds.
    pub fn tick(&mut self, dt: f32) {
        if !self.driven {
            return;
        }
        let dt = if dt.is_finite() && dt > 0.0 { dt } else { 0.0 };
        let cfg = &self.config;

        let crouch_target = if self.crouching {
            ramp(self.speed, cfg.crouch_walk_start_speed, cfg.crouch_walk_full_speed)
        } else {
            0.0
        };
        let sprint_target = if self.sprinting && !self.crouching && self.speed >= cfg.sprint_min_speed
        {
            1.0
        } else {
            0.0
        };

        // Hysteresis: start tired below `tiredStaminaEnter`, recover only above `tiredStaminaExit`.
        if self.stamina <= cfg.tired_stamina_enter {
            self.tired_on = true;
        } else if self.stamina >= cfg.tired_stamina_exit {
            self.tired_on = false;
        }
        let tired_target = if self.tired_on { 1.0 } else { 0.0 };

        if self.was_sprinting
            && !self.sprinting
            && self.stamina <= cfg.catch_breath_stamina
            && !self.catch_active
        {
            self.catch_active = true;
            self.catch_time = 0.0;
        }
        if self.catch_active {
            let moved = self.speed > cfg.catch_breath_max_speed;
            if self.sprinting || self.crouching || moved {
                self.catch_active = false;
            } else {
                self.catch_time += dt;
                if self.catch_time >= cfg.catch_breath_seconds {
                    self.catch_active = false;
                }
            }
        }
        let catch_target = if self.catch_active {
            catch_envelope(self.catch_time, cfg)
        } else {
            0.0
        };

        self.crouch_walk = approach(self.crouch_walk, crouch_target, cfg.crouch_walk_blend_rate, dt);
        self.sprint = approach(self.sprint, sprint_target, cfg.sprint_blend_rate, dt);
        self.tired = approach(self.tired, tired_target, cfg.tired_blend_rate, dt);
        self.catch_weight =
            approach(self.catch_weight, catch_target, cfg.catch_breath_blend_rate, dt);
        self.was_sprinting = self.sprinting;
    }

    /// Crouch walk blend (0 = crouch idle, 1 = full crouch walk) by speed.
    pub fn crouch_walk_weight(&self) -> f32 {
        self.crouch_walk
    }

    /// Sprint body blend, only above the sprint speed threshold.
    pub fn sprint_weight(&self) -> f32 {
        self.sprint
    }

    /// Tired fraction (hunched, heavy breathing) from the stamina hysteresis.
    pub fn tired_weight(&self) -> f32 {
        self.tired
    }

    /// Catch-breath blend (hands toward knees after a tired sprint).
    pub fn catch_breath_weight(&self) -> f32 {
        self.catch_weight
    }

    /// True while the catch-breath clip is playing its 2 s.
    pub fn catch_breath_active(&self) -> bool {
        self.catch_active
    }

    /// Writes this frame's clip choices and weights into the existing layers.
    pub fn apply(&mut self, layers: &mut LayerStack, armed: bool) {
        if !self.driven {
            return;
        }
        self.apply_sprint_body(layers);
        self.apply_upper_body(layers, armed);
        self.apply_breathing(layers);
    }

    /// `Crouch` layer: the full-body sprint cycle while sprinting, crouch idle otherwise.
    fn apply_sprint_body(&mut self, layers: &mut LayerStack) {
        let layer = layers.get_mut(LayerId::Crouch);
        if self.sprint > WEIGHT_EPSILON {
            if layer.clip.as_deref() != Some(SPRINT_CLIP) {
                layer.clip = Some(SPRINT_CLIP.to_owned());
                layer.time = 0.0;
            }
            layer.mask = MaskId::Full;
            layer.weight = self.sprint;
            layer.target = self.sprint;
            layer.speed = 1.0;
            self.owns_crouch = true;
        } else if self.owns_crouch {
            layer.clip = Some(CROUCH_CLIP.to_owned());
            layer.time = 0.0;
            layer.weight = 0.0;
            layer.target = if self.crouching { 1.0 } else { 0.0 };
            layer.speed = 1.0;
            self.owns_crouch = false;
        }
    }

    /// `Aim` layer: catch breath (whole body), crouch walk (legs), or sprint arms (upper body).
    fn apply_upper_body(&mut self, layers: &mut LayerStack, armed: bool) {
        let mut want: Option<(&'static str, MaskId, f32, f32)> = None;
        if self.catch_weight > WEIGHT_EPSILON {
            want = Some((CATCH_BREATH_CLIP, MaskId::Full, self.catch_weight, 1.0));
        } else if self.crouch_walk > WEIGHT_EPSILON {
            let speed = self.config.crouch_walk_playback(self.speed);
            want = Some((CROUCH_WALK_CLIP, MaskId::Lower, self.crouch_walk, speed));
        } else if armed && self.sprint > WEIGHT_EPSILON {
            want = Some((SPRINT_ARMS_WEAPON_CLIP, MaskId::Upper, self.sprint, 1.0));
        }
        let layer = layers.get_mut(LayerId::Aim);
        if let Some((clip, mask, weight, speed)) = want {
            if layer.clip.as_deref() != Some(clip) {
                layer.clip = Some(clip.to_owned());
                layer.time = 0.0;
            }
            layer.mask = mask;
            layer.weight = weight;
            layer.target = weight;
            layer.speed = speed;
            self.owns_aim = true;
        } else if self.owns_aim {
            layer.weight = 0.0;
            layer.target = 0.0;
            layer.speed = 1.0;
            self.owns_aim = false;
        }
    }

    /// `Additive` layer: the small tired hunch and chest heave, faded out during catch breath.
    fn apply_breathing(&mut self, layers: &mut LayerStack) {
        let breath = self.tired * self.config.tired_breath_weight * (1.0 - self.catch_weight);
        let layer = layers.get_mut(LayerId::Additive);
        if breath > WEIGHT_EPSILON {
            if layer.clip.as_deref() != Some(TIRED_BREATH_CLIP) {
                layer.clip = Some(TIRED_BREATH_CLIP.to_owned());
                layer.time = 0.0;
            }
            layer.mask = MaskId::Upper;
            layer.weight = breath;
            layer.target = breath;
            self.owns_breath = true;
        } else if self.owns_breath {
            layer.weight = 0.0;
            layer.target = 0.0;
            self.owns_breath = false;
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

/// Short fade in and out around the catch-breath hold.
fn catch_envelope(t: f32, cfg: &PostureConfig) -> f32 {
    let total = cfg.catch_breath_seconds;
    let rise = if cfg.catch_breath_fade_in > 0.0 {
        (t / cfg.catch_breath_fade_in).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let fall = if cfg.catch_breath_fade_out > 0.0 {
        ((total - t) / cfg.catch_breath_fade_out).clamp(0.0, 1.0)
    } else {
        1.0
    };
    rise.min(fall)
}

/// Frame-rate independent smoothing that stays put for a zero or broken step.
fn approach(current: f32, target: f32, rate: f32, dt: f32) -> f32 {
    let cur = if current.is_finite() { current.clamp(0.0, 1.0) } else { 0.0 };
    let target = if target.is_finite() { target.clamp(0.0, 1.0) } else { 0.0 };
    if !rate.is_finite() || rate <= 0.0 || dt <= 0.0 {
        return cur;
    }
    let k = 1.0 - (-rate * dt).exp();
    (cur + (target - cur) * k).clamp(0.0, 1.0)
}
