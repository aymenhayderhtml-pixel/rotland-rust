//! Data-driven animation timings and clip event definitions loaded from `data/animation.json`.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::blend::{ACTION_FADE_IN, ACTION_FADE_OUT, BLEND_RATE};

/// Schema version of `data/animation.json`.
pub const CONFIG_VERSION: u32 = 1; // data: animation.json version
/// Default fade-in duration when entering death pose (seconds).
pub const DEFAULT_DIE_FADE_IN: f32 = 0.15; // data: animation.json dieFadeIn
/// Default fade-in duration for full-body clips (seconds).
pub const DEFAULT_FULL_BODY_FADE_IN: f32 = 0.15; // data: animation.json fullBodyFadeIn
/// Default fade-out duration for full-body clips (seconds).
pub const DEFAULT_FULL_BODY_FADE_OUT: f32 = 0.2; // data: animation.json fullBodyFadeOut

static SHARED_CONFIG: OnceLock<AnimConfig> = OnceLock::new();

/// A single timed event marker on a clip in `data/animation.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipEventDef {
    pub name: String,
    pub time: f32,
}

/// Global animation tuning loaded from `data/animation.json` with safe built-in defaults.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AnimConfig {
    pub version: u32,
    pub blend_rate: f32,
    pub action_fade_in: f32,
    pub action_fade_out: f32,
    pub die_fade_in: f32,
    pub full_body_fade_in: f32,
    pub full_body_fade_out: f32,
    pub events: HashMap<String, Vec<ClipEventDef>>,
}

impl Default for AnimConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION, // data: animation.json version
            blend_rate: BLEND_RATE, // data: animation.json blendRate
            action_fade_in: ACTION_FADE_IN, // data: animation.json actionFadeIn
            action_fade_out: ACTION_FADE_OUT, // data: animation.json actionFadeOut
            die_fade_in: DEFAULT_DIE_FADE_IN, // data: animation.json dieFadeIn
            full_body_fade_in: DEFAULT_FULL_BODY_FADE_IN, // data: animation.json fullBodyFadeIn
            full_body_fade_out: DEFAULT_FULL_BODY_FADE_OUT, // data: animation.json fullBodyFadeOut
            events: HashMap::new(), // data: animation.json events
        }
    }
}

impl AnimConfig {
    /// Parses an `AnimConfig` from JSON text. Omitted keys fall back to defaults.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Loads `data/animation.json` once per process, falling back to built-in defaults on error.
    pub fn load_shared() -> &'static AnimConfig {
        SHARED_CONFIG.get_or_init(|| {
            let path = paths::repo_path("data/animation.json");
            match std::fs::read_to_string(&path) {
                Ok(text) => match Self::from_json(&text) {
                    Ok(cfg) => cfg,
                    Err(err) => {
                        log::warn!(
                            "Failed to parse {}: {err}; using built-in animation defaults",
                            path.display()
                        );
                        Self::default()
                    }
                },
                Err(err) => {
                    log::warn!(
                        "Failed to read {}: {err}; using built-in animation defaults",
                        path.display()
                    );
                    Self::default()
                }
            }
        })
    }
}
