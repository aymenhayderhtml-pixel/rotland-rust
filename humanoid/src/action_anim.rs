//! Action cue animations (pickup, loot, eat, drink, bandage, build, fishing).

use std::sync::OnceLock;

use serde::Deserialize;

use crate::full_body::FullBodyOptions;
use crate::layers::{LayerId, LayerStack};
use crate::mask::MaskId;

const MIN_DT: f32 = 0.0; // new: DECISIONS 2026-11-24
const ZERO_WEIGHT: f32 = 0.0; // new: DECISIONS 2026-11-24
const FULL_WEIGHT: f32 = 1.0; // new: DECISIONS 2026-11-24

/// Player interaction or item-use animation cue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionCue {
    PickUp,
    OpenCrate,
    Eat,
    Drink,
    Bandage,
    Build,
    FishCast,
    FishReel,
    FishStop,
}

/// Per-cue clip and timing settings from `data/action_anim.json`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CueSpec {
    pub clip: String,
    pub full_body: bool,
    pub seconds: f32,
    pub speed: f32,
    pub fade_in: f32,
    pub fade_out: f32,
}

/// All 8 playable cues from `data/action_anim.json`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActionCues {
    pub pick_up: CueSpec,
    pub open_crate: CueSpec,
    pub eat: CueSpec,
    pub drink: CueSpec,
    pub bandage: CueSpec,
    pub build: CueSpec,
    pub fish_cast: CueSpec,
    pub fish_reel: CueSpec,
}

/// Shared tuning loaded from `data/action_anim.json`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActionAnimConfig {
    pub version: u32,
    pub cancel_speed: f32,
    pub cues: ActionCues,
}

impl ActionAnimConfig {
    /// Parses `data/action_anim.json`.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Loads and caches `data/action_anim.json`, falling back to defaults if unavailable.
    pub fn load_shared() -> &'static Self {
        static SHARED: OnceLock<ActionAnimConfig> = OnceLock::new();
        SHARED.get_or_init(|| {
            for path in ["data/action_anim.json", "../data/action_anim.json"] {
                if let Ok(text) = std::fs::read_to_string(path)
                    && let Ok(cfg) = Self::from_json(&text)
                {
                    return cfg;
                }
            }
            Self::default()
        })
    }

    /// Returns the cue specification for a playable cue (`FishStop` returns `None`).
    pub fn spec(&self, cue: ActionCue) -> Option<&CueSpec> {
        match cue {
            ActionCue::PickUp => Some(&self.cues.pick_up),
            ActionCue::OpenCrate => Some(&self.cues.open_crate),
            ActionCue::Eat => Some(&self.cues.eat),
            ActionCue::Drink => Some(&self.cues.drink),
            ActionCue::Bandage => Some(&self.cues.bandage),
            ActionCue::Build => Some(&self.cues.build),
            ActionCue::FishCast => Some(&self.cues.fish_cast),
            ActionCue::FishReel => Some(&self.cues.fish_reel),
            ActionCue::FishStop => None,
        }
    }
}

impl Default for ActionAnimConfig {
    fn default() -> Self {
        Self {
            version: 1, // data: action_anim.json version
            cancel_speed: 1.2, // data: action_anim.json cancelSpeed
            cues: ActionCues {
                pick_up: CueSpec {
                    clip: "pickup_low".to_owned(),
                    full_body: true,
                    seconds: 0.8, // data: action_anim.json seconds
                    speed: 1.0, // data: action_anim.json speed
                    fade_in: 0.12, // data: action_anim.json fadeIn
                    fade_out: 0.2, // data: action_anim.json fadeOut
                },
                open_crate: CueSpec {
                    clip: "loot_search".to_owned(),
                    full_body: true,
                    seconds: 1.2, // data: action_anim.json seconds
                    speed: 1.0, // data: action_anim.json speed
                    fade_in: 0.15, // data: action_anim.json fadeIn
                    fade_out: 0.25, // data: action_anim.json fadeOut
                },
                eat: CueSpec {
                    clip: "eat".to_owned(),
                    full_body: false,
                    seconds: 1.5, // data: action_anim.json seconds
                    speed: 1.0, // data: action_anim.json speed
                    fade_in: 0.12, // data: action_anim.json fadeIn
                    fade_out: 0.2, // data: action_anim.json fadeOut
                },
                drink: CueSpec {
                    clip: "drink".to_owned(),
                    full_body: false,
                    seconds: 1.5, // data: action_anim.json seconds
                    speed: 1.0, // data: action_anim.json speed
                    fade_in: 0.12, // data: action_anim.json fadeIn
                    fade_out: 0.2, // data: action_anim.json fadeOut
                },
                bandage: CueSpec {
                    clip: "bandage".to_owned(),
                    full_body: false,
                    seconds: 1.2, // data: action_anim.json seconds
                    speed: 1.0, // data: action_anim.json speed
                    fade_in: 0.12, // data: action_anim.json fadeIn
                    fade_out: 0.2, // data: action_anim.json fadeOut
                },
                build: CueSpec {
                    clip: "hammer_build".to_owned(),
                    full_body: false,
                    seconds: 0.7, // data: action_anim.json seconds
                    speed: 1.0, // data: action_anim.json speed
                    fade_in: 0.08, // data: action_anim.json fadeIn
                    fade_out: 0.15, // data: action_anim.json fadeOut
                },
                fish_cast: CueSpec {
                    clip: "fish_cast".to_owned(),
                    full_body: false,
                    seconds: 1.0, // data: action_anim.json seconds
                    speed: 1.0, // data: action_anim.json speed
                    fade_in: 0.1, // data: action_anim.json fadeIn
                    fade_out: 0.2, // data: action_anim.json fadeOut
                },
                fish_reel: CueSpec {
                    clip: "fish_reel".to_owned(),
                    full_body: false,
                    seconds: 0.0, // data: action_anim.json seconds
                    speed: 1.0, // data: action_anim.json speed
                    fade_in: 0.15, // data: action_anim.json fadeIn
                    fade_out: 0.3, // data: action_anim.json fadeOut
                },
            },
        }
    }
}

/// Command returned to `HumanoidAnimator` when full-body state needs updating.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActionAnimCommand {
    None,
    StartFullBody { cue: ActionCue, opts: FullBodyOptions },
    StopFullBody { fade_out: f32 },
}

#[derive(Clone, Copy, Debug)]
struct ActiveCue {
    cue: ActionCue,
    full_body: bool,
    elapsed: f32,
    seconds: f32,
    speed: f32,
    fade_in: f32,
    fade_out: f32,
    fading_out: bool,
    fade_out_elapsed: f32,
    fade_out_start_weight: f32,
}

impl ActiveCue {
    fn weight(&self) -> f32 {
        if self.fading_out {
            if self.fade_out <= MIN_DT {
                return ZERO_WEIGHT;
            }
            let frac = (FULL_WEIGHT - self.fade_out_elapsed / self.fade_out).clamp(ZERO_WEIGHT, FULL_WEIGHT);
            return self.fade_out_start_weight * frac;
        }
        if self.fade_in <= MIN_DT {
            FULL_WEIGHT
        } else {
            (self.elapsed / self.fade_in).clamp(ZERO_WEIGHT, FULL_WEIGHT)
        }
    }

    fn begin_fade_out(&mut self) {
        if !self.fading_out {
            self.fade_out_start_weight = self.weight();
            self.fading_out = true;
            self.fade_out_elapsed = MIN_DT;
        }
    }
}

/// Tracks the currently playing action cue without per-frame allocations.
pub struct ActionAnimState {
    config: &'static ActionAnimConfig,
    driven: bool,
    move_speed: f32,
    active: Option<ActiveCue>,
    upper_needs_release: bool,
}

impl Default for ActionAnimState {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionAnimState {
    /// Creates an idle action-cue state using the shared `data/action_anim.json`.
    pub fn new() -> Self {
        Self::with_config(ActionAnimConfig::load_shared())
    }

    /// Creates an idle action-cue state with an explicit config (for tests).
    pub fn with_config(config: &'static ActionAnimConfig) -> Self {
        Self {
            config,
            driven: false,
            move_speed: MIN_DT,
            active: None,
            upper_needs_release: false,
        }
    }

    pub fn is_driven(&self) -> bool { self.driven }
    pub fn active_cue(&self) -> Option<ActionCue> { self.active.map(|a| a.cue) }
    pub fn is_upper_active(&self) -> bool { self.active.is_some_and(|a| !a.full_body) }
    pub fn is_full_body_active(&self) -> bool { self.active.is_some_and(|a| a.full_body) }
    pub fn weight(&self) -> f32 { self.active.map_or(ZERO_WEIGHT, |a| a.weight()) }

    /// Records the latest horizontal movement speed in m/s.
    pub fn set_move_speed(&mut self, speed: f32) {
        if speed.is_finite() && speed >= MIN_DT {
            self.move_speed = speed;
        }
    }

    /// Starts or stops a cue. Returns any full-body command for the animator.
    pub fn play_cue(&mut self, cue: ActionCue, blocked: bool) -> ActionAnimCommand {
        self.driven = true;
        if cue == ActionCue::FishStop {
            if let Some(act) = &mut self.active
                && !act.full_body
            {
                act.begin_fade_out();
            }
            return ActionAnimCommand::None;
        }
        if blocked {
            return ActionAnimCommand::None;
        }
        let Some(spec) = self.config.spec(cue) else {
            return ActionAnimCommand::None;
        };
        let prev_full_body_fade = self.active.filter(|a| a.full_body).map(|a| a.fade_out);
        if self.active.is_some_and(|a| !a.full_body) && spec.full_body {
            self.upper_needs_release = true;
        }
        self.active = Some(ActiveCue {
            cue,
            full_body: spec.full_body,
            elapsed: MIN_DT,
            seconds: spec.seconds,
            speed: spec.speed,
            fade_in: spec.fade_in,
            fade_out: spec.fade_out,
            fading_out: false,
            fade_out_elapsed: MIN_DT,
            fade_out_start_weight: FULL_WEIGHT,
        });
        if spec.full_body {
            let opts = FullBodyOptions {
                speed: spec.speed,
                looping: true,
                hold_last: false,
                fade_in: spec.fade_in,
                fade_out: spec.fade_out,
            };
            ActionAnimCommand::StartFullBody { cue, opts }
        } else if let Some(fade_out) = prev_full_body_fade {
            ActionAnimCommand::StopFullBody { fade_out }
        } else {
            ActionAnimCommand::None
        }
    }

    /// Returns the clip name for a playable cue.
    pub fn clip_name(&self, cue: ActionCue) -> Option<&'static str> {
        self.config.spec(cue).map(|s| s.clip.as_str())
    }

    /// Advances active cue timing and checks movement/blocked cancellation.
    pub fn tick(&mut self, dt: f32, blocked: bool) -> ActionAnimCommand {
        if !self.driven || !dt.is_finite() || dt < MIN_DT {
            return ActionAnimCommand::None;
        }
        let Some(act) = &mut self.active else {
            return ActionAnimCommand::None;
        };
        act.elapsed += dt;
        if act.full_body {
            let speed_cancel = self.move_speed > self.config.cancel_speed;
            let stop_at = (act.seconds - act.fade_out).max(MIN_DT);
            let time_up = act.seconds > MIN_DT && act.elapsed >= stop_at;
            if blocked || speed_cancel || time_up {
                let fade_out = act.fade_out;
                self.active = None;
                return ActionAnimCommand::StopFullBody { fade_out };
            }
            return ActionAnimCommand::None;
        }
        if blocked {
            self.active = None;
            self.upper_needs_release = true;
            return ActionAnimCommand::None;
        }
        if !act.fading_out && act.seconds > MIN_DT {
            let stop_at = (act.seconds - act.fade_out).max(MIN_DT);
            if act.elapsed >= stop_at {
                act.begin_fade_out();
            }
        }
        if act.fading_out {
            act.fade_out_elapsed += dt;
            if act.fade_out <= MIN_DT || act.fade_out_elapsed >= act.fade_out {
                self.active = None;
                self.upper_needs_release = true;
            }
        }
        ActionAnimCommand::None
    }

    /// Writes the active upper-body cue into `LayerId::Action` (or releases it when done).
    pub fn apply_upper(&mut self, layers: &mut LayerStack) {
        if !self.driven {
            return;
        }
        if let Some(act) = self.active
            && !act.full_body
            && let Some(clip) = self.clip_name(act.cue)
        {
            self.upper_needs_release = false;
            if layers.get(LayerId::Action).clip.as_deref() != Some(clip) {
                layers.set_clip(LayerId::Action, Some(clip));
            }
            let w = act.weight();
            let layer = layers.get_mut(LayerId::Action);
            layer.mask = MaskId::Upper;
            layer.weight = w;
            layer.target = w;
            layer.time = act.elapsed * act.speed;
            layer.speed = act.speed;
            return;
        }
        if self.upper_needs_release {
            self.upper_needs_release = false;
            let layer = layers.get_mut(LayerId::Action);
            layer.weight = ZERO_WEIGHT;
            layer.target = ZERO_WEIGHT;
            layers.set_clip(LayerId::Action, None);
        }
    }
}
