//! Jump, fall and landing state machine driven by the character's velocity.
//!
//! Turns raw vertical speed into take-off, rise and fall phases and decides whether a
//! landing is a soft dip, a heavy crouch or a forward roll. Movement itself is untouched:
//! this module only reports which clip should play.

use std::sync::OnceLock;

use glam::Vec3;
use serde::Deserialize;

/// Tuning values for the airborne state machine from `data/airborne.json`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AirConfig {
    pub version: u32,
    pub min_air_seconds: f32,
    pub rise_speed: f32,
    pub fall_speed: f32,
    pub air_blend_rate: f32,
    pub soft_land_speed: f32,
    pub hard_land_speed: f32,
    pub roll_min_horizontal_speed: f32,
    pub land_cooldown: f32,
}

impl Default for AirConfig {
    fn default() -> Self {
        Self {
            version: 1,                                   // data: airborne.json version
            min_air_seconds: 0.12,                        // data: airborne.json minAirSeconds
            rise_speed: 0.6,                              // data: airborne.json riseSpeed
            fall_speed: -1.2,                             // data: airborne.json fallSpeed
            air_blend_rate: 12.0,                         // data: airborne.json airBlendRate
            soft_land_speed: 3.0,                         // data: airborne.json softLandSpeed
            hard_land_speed: 7.5,                         // data: airborne.json hardLandSpeed
            roll_min_horizontal_speed: 3.5,               // data: airborne.json rollMinHorizontalSpeed
            land_cooldown: 0.25,                          // data: airborne.json landCooldown
        }
    }
}

impl AirConfig {
    /// Parses an `AirConfig` from JSON text.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Loads and caches the shared `data/airborne.json` config, falling back to defaults.
    pub fn load_shared() -> &'static AirConfig {
        static CONFIG: OnceLock<AirConfig> = OnceLock::new();
        CONFIG.get_or_init(|| {
            for path in ["data/airborne.json", "../data/airborne.json"] {
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

/// Phase of the jump arc the character is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AirPhase {
    Grounded,
    Takeoff,
    Rise,
    Fall,
}

/// Landing animation chosen when the character touches the ground.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandingKind {
    Soft,
    Hard,
    Roll,
}

/// Airborne phase, timers and the fastest fall speed seen since leaving the ground.
#[derive(Clone, Debug)]
pub struct AirState {
    config: AirConfig,
    driven: bool,
    phase: AirPhase,
    air_time: f32,
    peak_fall_speed: f32,
    velocity: Vec3,
    cooldown: f32,
}

impl Default for AirState {
    fn default() -> Self {
        Self::new()
    }
}

impl AirState {
    /// Creates an inactive state using the shared `data/airborne.json` config.
    pub fn new() -> Self {
        Self::with_config(AirConfig::load_shared().clone())
    }

    /// Creates an inactive state with an explicit config.
    pub fn with_config(config: AirConfig) -> Self {
        Self {
            config,
            driven: false,
            phase: AirPhase::Grounded,
            air_time: 0.0,
            peak_fall_speed: 0.0,
            velocity: Vec3::ZERO,
            cooldown: 0.0,
        }
    }

    /// Feeds this frame's world velocity (metres per second). Until this is called the
    /// state stays inert so the old `jump` clip and weights are unchanged.
    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.velocity = if velocity.is_finite() { velocity } else { Vec3::ZERO };
        self.driven = true;
    }

    /// Advances one step. Returns a landing exactly once when one should play.
    pub fn tick(&mut self, dt: f32, on_floor: bool) -> Option<LandingKind> {
        let dt = if dt.is_finite() && dt > 0.0 { dt } else { 0.0 };
        if !self.driven {
            return None;
        }
        self.cooldown = (self.cooldown - dt).max(0.0);

        if on_floor {
            if self.phase == AirPhase::Grounded {
                self.reset_air();
                return None;
            }
            let landing = self.landing_kind();
            self.reset_air();
            if landing.is_some() {
                self.cooldown = self.config.land_cooldown;
            }
            return landing;
        }

        self.air_time += dt;
        let fall_speed = -self.velocity.y;
        if fall_speed > self.peak_fall_speed {
            self.peak_fall_speed = fall_speed;
        }
        self.advance_phase(self.velocity.y);
        None
    }

    /// Current phase of the jump arc.
    pub fn phase(&self) -> AirPhase {
        self.phase
    }

    /// Clip to play on the Air layer, or `None` while grounded.
    pub fn air_clip(&self) -> Option<&'static str> {
        match self.phase {
            AirPhase::Grounded => None,
            AirPhase::Takeoff => Some("air_takeoff"),
            AirPhase::Rise => Some("air_rise"),
            AirPhase::Fall => Some("air_fall"),
        }
    }

    /// Air layer weight target: 0 until the character has been airborne for
    /// `minAirSeconds`, so small bumps do nothing. A real jump push-off (Takeoff)
    /// is never a bump because it needs upward speed, so it shows immediately.
    pub fn air_target(&self) -> f32 {
        match self.phase {
            AirPhase::Grounded => 0.0,
            AirPhase::Takeoff => 1.0,
            AirPhase::Rise | AirPhase::Fall => {
                if self.air_time >= self.config.min_air_seconds { 1.0 } else { 0.0 }
            }
        }
    }

    /// Short fade times for a landing clip, derived from `airBlendRate`.
    pub fn landing_fades(&self) -> (f32, f32) {
        let rate = self.config.air_blend_rate;
        let fade = if rate > 0.0 { 1.0 / rate } else { 0.0 };
        (fade, fade)
    }

    fn reset_air(&mut self) {
        self.phase = AirPhase::Grounded;
        self.air_time = 0.0;
        self.peak_fall_speed = 0.0;
    }

    fn landing_kind(&self) -> Option<LandingKind> {
        let cfg = &self.config;
        if self.phase == AirPhase::Grounded || self.cooldown > 0.0 {
            return None;
        }
        let peak = self.peak_fall_speed;
        if peak >= cfg.hard_land_speed {
            let horiz = Vec3::new(self.velocity.x, 0.0, self.velocity.z).length();
            let kind = if horiz >= cfg.roll_min_horizontal_speed {
                LandingKind::Roll
            } else {
                LandingKind::Hard
            };
            return Some(kind);
        }
        if peak >= cfg.soft_land_speed || self.air_time >= cfg.min_air_seconds {
            return Some(LandingKind::Soft);
        }
        None
    }

    fn advance_phase(&mut self, v: f32) {
        let cfg = &self.config;
        if self.phase == AirPhase::Grounded {
            if v > cfg.rise_speed {
                self.phase = AirPhase::Takeoff;
            } else if self.air_time >= cfg.min_air_seconds {
                self.phase = AirPhase::Fall;
            }
        } else if v < cfg.fall_speed {
            self.phase = AirPhase::Fall;
        } else if self.phase == AirPhase::Takeoff && self.air_time >= cfg.min_air_seconds {
            self.phase = if v > 0.0 { AirPhase::Rise } else { AirPhase::Fall };
        } else if self.phase == AirPhase::Fall && v > 0.0 {
            self.phase = AirPhase::Rise;
        }
    }
}

/// Clip name for a landing kind.
pub fn landing_clip(kind: LandingKind) -> &'static str {
    match kind {
        LandingKind::Soft => "land_soft",
        LandingKind::Hard => "land_hard",
        LandingKind::Roll => "land_roll",
    }
}
