//! Play-side wiring for the New Halden rooftop radio (`data/radio.json`).

use game::radio::{RadioConfig, RadioState, radio_prompt, use_radio};
use glam::Vec3;

use crate::play::PlayState;

/// Result of checking/handling radio interaction for one frame.
pub struct RadioInteract {
    pub prompt: Option<String>,
    pub distance: f32,
    pub consumed_press: bool,
}

struct RadioPlayback {
    lines: Vec<String>,
    index: usize,
    timer: f32,
    line_seconds: f32,
}

/// Manages New Halden's rooftop radio state, timed transmission lines, and crate unlock.
#[derive(Default)]
pub struct MetroRadioRuntime {
    pub state: RadioState,
    config: Option<RadioConfig>,
    config_loaded: bool,
    playback: Option<RadioPlayback>,
}

impl MetroRadioRuntime {
    fn ensure_config(&mut self) -> Option<&RadioConfig> {
        if !self.config_loaded {
            self.config_loaded = true;
            match RadioConfig::from_repo() {
                Ok(cfg) => self.config = Some(cfg),
                Err(e) => log::warn!("data/radio.json not loaded, metro radio off: {e}"),
            }
        }
        self.config.as_ref()
    }

    /// Returns `true` when `crate_id` can be opened (the radio crate stays locked until used).
    pub fn can_open_crate(&self, crate_id: &str) -> bool {
        if self.state.used {
            return true;
        }
        let locked_id = self
            .config
            .as_ref()
            .map_or("metro_radio_crate", |c| c.crate_id.as_str()); // data: radio.json crateId
        crate_id != locked_id
    }

    /// Advances queued radio transmission lines and shows each in the HUD notice/toast area.
    pub fn advance_playback(&mut self, dt: f32, play_state: &mut PlayState) {
        let Some(pb) = self.playback.as_mut() else {
            return;
        };
        pb.timer += dt;
        if pb.timer < pb.line_seconds {
            return;
        }
        pb.timer -= pb.line_seconds;
        pb.index += 1;
        if let Some(line) = pb.lines.get(pb.index) {
            play_state.show_toast(line.clone());
        } else {
            self.playback = None;
        }
    }

    /// Shows the radio prompt near `console_pos` (New Halden only) and activates on `[E]`.
    pub fn interact_tick(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        console_pos: Option<Vec3>,
        play_state: &mut PlayState,
        pressed: bool,
    ) -> Option<RadioInteract> {
        self.advance_playback(dt, play_state);
        let console_pos = console_pos?;
        self.ensure_config()?;
        let cfg = self.config.as_ref()?;
        let prompt_text = radio_prompt(&self.state, player_pos, console_pos, cfg)?.to_string();
        let distance = console_pos.distance(player_pos);

        if !pressed {
            return Some(RadioInteract {
                prompt: Some(prompt_text),
                distance,
                consumed_press: false,
            });
        }

        if let Some(outcome) = use_radio(&mut self.state, player_pos, console_pos, cfg) {
            if let Some(first) = outcome.lines.first() {
                play_state.show_toast(first.clone());
                self.playback = Some(RadioPlayback {
                    lines: outcome.lines,
                    index: 0,
                    timer: 0.0,
                    line_seconds: cfg.line_seconds,
                });
            }
            Some(RadioInteract {
                prompt: None,
                distance,
                consumed_press: true,
            })
        } else {
            Some(RadioInteract {
                prompt: Some(cfg.used_prompt.clone()),
                distance,
                consumed_press: true,
            })
        }
    }

    /// Exports radio state for saving in `systems.extra["metro_radio"]`.
    pub fn to_save(&self) -> Option<serde_json::Value> {
        self.state.used.then(|| self.state.to_json())
    }

    /// Restores radio state from `systems.extra["metro_radio"]`.
    pub fn restore_save(&mut self, value: &serde_json::Value) {
        self.state = RadioState::from_json(value);
    }
}
