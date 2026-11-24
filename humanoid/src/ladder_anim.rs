//! Distance-driven ladder climbing and enter/exit animation state (A09).
//!
//! Converts ladder active/on-rungs flags and player height into a cyclic rung
//! pose time for `ladder_climb` plus one-shot enter/exit transition events.
//! Drives the existing `Air` layer at weight 1 while on a ladder and leaves the
//! animator untouched until `set_input` is called.

use std::sync::OnceLock;

use serde::Deserialize;

use crate::layers::{LayerId, LayerStack};
use crate::mask::MaskId;

/// Looping rung-by-rung climb clip driven by vertical distance climbed.
pub const LADDER_CLIMB_CLIP: &str = "ladder_climb";
/// One-shot clip when stepping onto the bottom of a ladder.
pub const LADDER_ENTER_BOTTOM_CLIP: &str = "ladder_enter_bottom";
/// One-shot clip when stepping off the top onto the deck.
pub const LADDER_EXIT_TOP_CLIP: &str = "ladder_exit_top";
/// One-shot clip when stepping down onto the ladder from the deck.
pub const LADDER_ENTER_TOP_CLIP: &str = "ladder_enter_top";
/// One-shot clip when stepping off the bottom onto the ground.
pub const LADDER_EXIT_BOTTOM_CLIP: &str = "ladder_exit_bottom";

const JUMP_FALLBACK_CLIP: &str = "jump";
const DEFAULT_SPEED: f32 = 1.0; // new: DECISIONS 2026-11-24
const ZERO_SPEED: f32 = 0.0; // new: DECISIONS 2026-11-24
const FULL_WEIGHT: f32 = 1.0; // new: DECISIONS 2026-11-24
const ZERO_WEIGHT: f32 = 0.0; // new: DECISIONS 2026-11-24
const DIR_EPSILON: f32 = 1e-4; // new: DECISIONS 2026-11-24
const DIR_DOWN: i8 = -1; // new: DECISIONS 2026-11-24
const DIR_NONE: i8 = 0; // new: DECISIONS 2026-11-24
const DIR_UP: i8 = 1; // new: DECISIONS 2026-11-24

/// Tuning values for ladder animations from `data/ladder_anim.json`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LadderAnimConfig {
    pub version: u32,
    pub metres_per_cycle: f32,
    pub teleport_distance: f32,
    pub enter_bottom_seconds: f32,
    pub exit_top_seconds: f32,
    pub enter_top_seconds: f32,
    pub exit_bottom_seconds: f32,
    pub fade: f32,
}

impl Default for LadderAnimConfig {
    fn default() -> Self {
        Self {
            version: 1,                // data: ladder_anim.json version
            metres_per_cycle: 0.6,     // data: ladder_anim.json metresPerCycle
            teleport_distance: 1.5,    // data: ladder_anim.json teleportDistance
            enter_bottom_seconds: 0.5, // data: ladder_anim.json enterBottomSeconds
            exit_top_seconds: 0.45,    // data: ladder_anim.json exitTopSeconds
            enter_top_seconds: 0.45,   // data: ladder_anim.json enterTopSeconds
            exit_bottom_seconds: 0.4,  // data: ladder_anim.json exitBottomSeconds
            fade: 0.12,                // data: ladder_anim.json fade
        }
    }
}

impl LadderAnimConfig {
    /// Parses a `LadderAnimConfig` from JSON text.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Loads and caches `data/ladder_anim.json`, falling back to defaults.
    pub fn load_shared() -> &'static LadderAnimConfig {
        static CONFIG: OnceLock<LadderAnimConfig> = OnceLock::new();
        CONFIG.get_or_init(|| {
            let path = paths::repo_path("data/ladder_anim.json");
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

/// One-shot transition clip triggered when mounting or dismounting a ladder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LadderEvent {
    EnterBottom,
    ExitTop,
    EnterTop,
    ExitBottom,
}

/// Returns the clip name for a ladder mount/dismount event.
pub fn ladder_event_clip(event: LadderEvent) -> &'static str {
    match event {
        LadderEvent::EnterBottom => LADDER_ENTER_BOTTOM_CLIP,
        LadderEvent::ExitTop => LADDER_EXIT_TOP_CLIP,
        LadderEvent::EnterTop => LADDER_ENTER_TOP_CLIP,
        LadderEvent::ExitBottom => LADDER_EXIT_BOTTOM_CLIP,
    }
}

/// True if `name` is one of the ladder animation clips.
pub fn is_ladder_clip(name: &str) -> bool {
    matches!(
        name,
        LADDER_CLIMB_CLIP
            | LADDER_ENTER_BOTTOM_CLIP
            | LADDER_EXIT_TOP_CLIP
            | LADDER_ENTER_TOP_CLIP
            | LADDER_EXIT_BOTTOM_CLIP
    )
}

/// Per-character ladder animation state machine.
#[derive(Clone, Debug)]
pub struct LadderAnimState {
    config: LadderAnimConfig,
    driven: bool,
    active: bool,
    on_rungs: bool,
    y: f32,
    was_active: bool,
    was_on_rungs: bool,
    has_y: bool,
    last_y: f32,
    climbed_distance: f32,
    last_dir: i8,
    owns_air: bool,
}

impl Default for LadderAnimState {
    fn default() -> Self {
        Self::new()
    }
}

impl LadderAnimState {
    /// Creates an inert state using the shared `data/ladder_anim.json` config.
    pub fn new() -> Self {
        Self::with_config(LadderAnimConfig::load_shared().clone())
    }

    /// Creates an inert state with an explicit config (preferred for tests).
    pub fn with_config(config: LadderAnimConfig) -> Self {
        Self {
            config,
            driven: false,
            active: false,
            on_rungs: false,
            y: 0.0,
            was_active: false,
            was_on_rungs: false,
            has_y: false,
            last_y: 0.0,
            climbed_distance: 0.0,
            last_dir: DIR_NONE,
            owns_air: false,
        }
    }

    /// Feeds ladder state: `active` = game climb active, `on_rungs` = hands on
    /// rungs (not top step path), `y` = player height. Inert until first call.
    pub fn set_input(&mut self, active: bool, on_rungs: bool, y: f32) {
        self.active = active;
        self.on_rungs = on_rungs;
        if y.is_finite() {
            self.y = y;
        }
        self.driven = true;
    }

    /// Advances ladder state by `dt` seconds and returns at most one event per tick.
    pub fn tick(&mut self, dt: f32) -> Option<LadderEvent> {
        let _ = dt;
        if !self.driven {
            return None;
        }
        let mut event = None;

        if self.active && !self.was_active {
            self.climbed_distance = 0.0;
            self.last_dir = DIR_NONE;
            self.last_y = self.y;
            self.has_y = true;
            event = Some(if self.on_rungs {
                LadderEvent::EnterBottom
            } else {
                LadderEvent::EnterTop
            });
        } else if self.active {
            if self.on_rungs && self.was_on_rungs {
                self.accumulate_dy(true);
            } else if self.on_rungs && !self.was_on_rungs {
                self.last_y = self.y;
                self.has_y = true;
            } else if !self.on_rungs && self.was_on_rungs {
                self.accumulate_dy(false);
                if self.last_dir == DIR_UP {
                    event = Some(LadderEvent::ExitTop);
                }
            } else {
                self.last_y = self.y;
                self.has_y = true;
            }
        } else if self.was_active {
            if self.was_on_rungs {
                self.accumulate_dy(false);
            }
            if self.last_dir == DIR_DOWN {
                event = Some(LadderEvent::ExitBottom);
            }
            self.last_y = self.y;
            self.has_y = true;
        } else {
            self.last_y = self.y;
            self.has_y = true;
        }

        self.was_active = self.active;
        self.was_on_rungs = self.on_rungs;
        event
    }

    /// True while active on the ladder (including the top step-in/step-off paths).
    pub fn is_on_ladder(&self) -> bool {
        self.driven && self.active
    }

    /// Clip time for `ladder_climb` from the climbed distance (wraps; reverses when climbing down).
    pub fn climb_time(&self, clip_length: f32) -> f32 {
        let mpc = self.config.metres_per_cycle;
        if !self.driven
            || !clip_length.is_finite()
            || clip_length <= 0.0
            || !mpc.is_finite()
            || mpc <= 0.0
        {
            return 0.0;
        }
        let cycles = self.climbed_distance / mpc;
        let mut wrapped = cycles.rem_euclid(1.0); // new: DECISIONS 2026-11-24
        if !wrapped.is_finite() || wrapped >= 1.0 {
            wrapped = 0.0;
        }
        wrapped * clip_length
    }

    /// Target playback duration in seconds for a ladder transition event.
    pub fn event_seconds(&self, event: LadderEvent) -> f32 {
        match event {
            LadderEvent::EnterBottom => self.config.enter_bottom_seconds,
            LadderEvent::ExitTop => self.config.exit_top_seconds,
            LadderEvent::EnterTop => self.config.enter_top_seconds,
            LadderEvent::ExitBottom => self.config.exit_bottom_seconds,
        }
    }

    /// Playback speed for an event clip so it completes in its configured `*Seconds`.
    pub fn event_speed(&self, event: LadderEvent, clip_duration: f32) -> f32 {
        let target = self.event_seconds(event);
        if clip_duration.is_finite() && clip_duration > 0.0 && target.is_finite() && target > 0.0 {
            clip_duration / target
        } else {
            DEFAULT_SPEED
        }
    }

    /// Fade-in and fade-out duration for ladder transition clips.
    pub fn fade(&self) -> f32 {
        if self.config.fade.is_finite() && self.config.fade >= 0.0 {
            self.config.fade
        } else {
            0.0
        }
    }

    /// Drives the full-body `Air` layer with `ladder_climb` locked to `climb_time()`.
    pub fn apply_ladder(&mut self, layers: &mut LayerStack, clip_length: f32) {
        layers.set_clip(LayerId::Air, Some(LADDER_CLIMB_CLIP));
        let layer = layers.get_mut(LayerId::Air);
        layer.mask = MaskId::Full;
        layer.weight = FULL_WEIGHT;
        layer.target = FULL_WEIGHT;
        layer.speed = ZERO_SPEED;
        layer.time = self.climb_time(clip_length);
        self.owns_air = true;
    }

    /// Fades upper-body weapon stance, aim and posture additive layers to zero.
    pub fn suppress_upper(&self, layers: &mut LayerStack) {
        layers.get_mut(LayerId::Stance).target = ZERO_WEIGHT;
        layers.get_mut(LayerId::Aim).target = ZERO_WEIGHT;
        layers.get_mut(LayerId::Additive).target = ZERO_WEIGHT;
    }

    /// Restores the `Air` layer defaults when leaving the ladder.
    pub fn release_air(&mut self, layers: &mut LayerStack) {
        if !self.owns_air {
            return;
        }
        layers.set_clip(LayerId::Air, Some(JUMP_FALLBACK_CLIP));
        let layer = layers.get_mut(LayerId::Air);
        layer.mask = MaskId::Full;
        layer.time = 0.0;
        layer.weight = ZERO_WEIGHT;
        layer.target = ZERO_WEIGHT;
        layer.speed = DEFAULT_SPEED;
        self.owns_air = false;
    }

    fn accumulate_dy(&mut self, advance_distance: bool) {
        if !self.has_y {
            self.last_y = self.y;
            self.has_y = true;
            return;
        }
        let dy = self.y - self.last_y;
        self.last_y = self.y;
        if !dy.is_finite() || dy.abs() > self.config.teleport_distance {
            return;
        }
        if advance_distance {
            self.climbed_distance += dy;
        }
        if dy > DIR_EPSILON {
            self.last_dir = DIR_UP;
        } else if dy < -DIR_EPSILON {
            self.last_dir = DIR_DOWN;
        }
    }
}
