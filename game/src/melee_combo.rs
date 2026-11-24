//! Melee combo chains, input buffering, weapon weight, and clip-speed alignment.

use std::collections::HashMap;

use serde::Deserialize;

use crate::weapons::WeaponDef;

const FALLBACK_CLIP_SPEED: f32 = 1.0; // new: DECISIONS 2026-11-24

/// Tuning for melee combo classes and weapon-to-class mapping.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeleeMovesConfig {
    pub version: u32,
    pub buffer_seconds: f32,
    pub min_clip_speed: f32,
    pub max_clip_speed: f32,
    pub weapons: HashMap<String, String>,
    pub classes: HashMap<String, ComboClass>,
}

/// One weapon class's combo chain and recovery rules.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComboClass {
    pub combo_window: f32,
    pub miss_recover: f32,
    pub steps: Vec<ComboStep>,
}

/// One step in a melee combo chain.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComboStep {
    pub clip: String,
    pub impact: f32,
    pub windup_scale: f32,
    pub cooldown_scale: f32,
    pub damage_scale: f32,
    pub knockback_scale: f32,
    pub arc_scale: f32,
}

impl Default for MeleeMovesConfig {
    fn default() -> Self {
        Self {
            version: 1,            // data: melee_moves.json version
            buffer_seconds: 0.25,  // data: melee_moves.json bufferSeconds
            min_clip_speed: 0.6,   // data: melee_moves.json minClipSpeed
            max_clip_speed: 2.2,   // data: melee_moves.json maxClipSpeed
            weapons: HashMap::new(),
            classes: HashMap::new(),
        }
    }
}

impl MeleeMovesConfig {
    /// Parses melee combo configuration from a JSON string.
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("invalid melee_moves.json: {e}"))
    }

    /// Loads `data/melee_moves.json` from the repository root.
    pub fn from_repo() -> Result<Self, String> {
        let path = paths::repo_path("data/melee_moves.json");
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&raw)
    }

    /// Returns the combo class for `weapon_id`, if mapped and non-empty.
    pub fn class_for(&self, weapon_id: &str) -> Option<&ComboClass> {
        let class_name = self.weapons.get(weapon_id)?;
        let class = self.classes.get(class_name)?;
        (!class.steps.is_empty()).then_some(class)
    }
}

/// The swing to start now: a scaled copy of the weapon, the clip and its play speed.
#[derive(Clone, Debug)]
pub struct ComboMove {
    pub weapon: WeaponDef,
    pub clip: String,
    pub clip_speed: f32,
    pub step: usize,
}

/// Per-player melee combo chain, input buffer, and miss recovery state.
#[derive(Clone, Debug, Default)]
pub struct MeleeComboState {
    next_step: usize,
    window_left: f32,
    buffer_left: f32,
    miss_recover_left: f32,
    last_used_class: bool,
}

impl MeleeComboState {
    /// Advances combo window, input buffer, and miss recovery timers.
    pub fn tick(&mut self, dt: f32) {
        if dt <= 0.0 || !dt.is_finite() {
            return;
        }
        if self.window_left > 0.0 {
            self.window_left = (self.window_left - dt).max(0.0);
            if self.window_left == 0.0 {
                self.next_step = 0; // new: DECISIONS 2026-11-24
            }
        }
        self.buffer_left = (self.buffer_left - dt).max(0.0);
        self.miss_recover_left = (self.miss_recover_left - dt).max(0.0);
    }

    /// True when a swing should start this frame: `pressed` now (or a click buffered within
    /// `buffer_seconds`), `swing_idle` is true, and no miss recovery is running.
    /// A press that cannot start now is buffered.
    pub fn wants_swing(
        &mut self,
        pressed: bool,
        swing_idle: bool,
        cfg: &MeleeMovesConfig,
    ) -> bool {
        if pressed {
            self.buffer_left = cfg.buffer_seconds.max(0.0);
        }
        swing_idle && self.miss_recover_left <= 0.0 && (pressed || self.buffer_left > 0.0)
    }

    /// The move for the next step (the chain resets to step 0 if the window ran out).
    /// Falls back to the weapon's own clip at speed 1.0 for unknown weapons.
    pub fn next_move(&self, weapon: &WeaponDef, cfg: &MeleeMovesConfig) -> ComboMove {
        let Some(class) = cfg.class_for(&weapon.id) else {
            return ComboMove {
                weapon: weapon.clone(),
                clip: weapon.anim.attack.clone().unwrap_or_default(),
                clip_speed: FALLBACK_CLIP_SPEED,
                step: 0, // new: DECISIONS 2026-11-24
            };
        };

        let step_idx = self.next_step % class.steps.len();
        let step = &class.steps[step_idx];

        let mut scaled = weapon.clone();
        scaled.damage *= step.damage_scale;
        scaled.knockback *= step.knockback_scale;
        scaled.arc_deg *= step.arc_scale;
        scaled.windup *= step.windup_scale;
        scaled.cooldown *= step.cooldown_scale;

        let clip_speed = if scaled.windup > 0.0 {
            (step.impact / scaled.windup).clamp(cfg.min_clip_speed, cfg.max_clip_speed)
        } else {
            FALLBACK_CLIP_SPEED
        };

        ComboMove {
            weapon: scaled,
            clip: step.clip.clone(),
            clip_speed,
            step: step_idx,
        }
    }

    /// The swing really started (stamina paid): consume the buffer and advance the step
    /// (wrapping to 0 after the last step).
    pub fn on_started(&mut self, mv: &ComboMove, cfg: &MeleeMovesConfig) {
        self.buffer_left = 0.0; // new: DECISIONS 2026-11-24
        self.window_left = 0.0; // new: DECISIONS 2026-11-24
        if let Some(class) = cfg.class_for(&mv.weapon.id) {
            self.next_step = (mv.step + 1) % class.steps.len();
            self.last_used_class = true;
        } else {
            self.next_step = 0; // new: DECISIONS 2026-11-24
            self.last_used_class = false;
        }
    }

    /// The swing's hit moment passed: open the combo window; if it hit nothing, start missRecover.
    pub fn on_resolved(
        &mut self,
        hit_something: bool,
        weapon_id: &str,
        cfg: &MeleeMovesConfig,
    ) {
        if !self.last_used_class {
            return;
        }
        let Some(class) = cfg.class_for(weapon_id) else {
            return;
        };
        if class.combo_window > 0.0 {
            self.window_left = class.combo_window;
        } else {
            self.window_left = 0.0; // new: DECISIONS 2026-11-24
            self.next_step = 0; // new: DECISIONS 2026-11-24
        }
        if !hit_something && class.miss_recover > 0.0 {
            self.miss_recover_left = class.miss_recover;
        }
    }

    /// Clears combo progress, buffered input, and recovery (weapon switch, death).
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
