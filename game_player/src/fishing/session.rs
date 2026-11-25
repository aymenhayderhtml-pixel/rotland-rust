//! High-level fishing session wrapping [`FishingMachine`] for simple game-loop wiring.

use super::config::FishingConfig;
use super::state::{FishingMachine, FishingPhase};

/// Per-frame player and environment input fed into [`FishingSession::tick`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FishingInput {
    pub has_rod: bool,
    pub can_fish_here: bool,
    pub click: bool,
    pub hurt: bool,
    pub random_01: f32,
}

/// Discrete gameplay events emitted by [`FishingSession::tick`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FishingEvent {
    Cast,
    Bite,
    Caught,
    Escaped,
    Cancelled,
}

/// Stateful fishing controller that handles casting, timing, cancelling, and click ownership.
#[derive(Clone, Debug, PartialEq)]
pub struct FishingSession {
    cfg: FishingConfig,
    machine: FishingMachine,
    ready_to_cast: bool,
}

impl FishingSession {
    pub fn new(cfg: FishingConfig) -> Self {
        let machine = FishingMachine::new(&cfg);
        Self {
            cfg,
            machine,
            ready_to_cast: false,
        }
    }

    /// Advances the fishing session by `dt` seconds and returns any transition event.
    ///
    /// [`FishingEvent::Caught`] is emitted at most once per catch.
    pub fn tick(&mut self, input: &FishingInput, dt: f32) -> Option<FishingEvent> {
        let dt = dt.max(0.0);
        let can_cast = input.has_rod && input.can_fish_here && !input.hurt;
        self.ready_to_cast = can_cast;

        match self.machine.phase {
            FishingPhase::Idle => {
                if can_cast
                    && input.click
                    && self
                        .machine
                        .start_cast(self.wait_duration_for(input.random_01))
                {
                    return Some(FishingEvent::Cast);
                }
                None
            }
            FishingPhase::Waiting => {
                if !can_cast {
                    self.machine.reset();
                    return Some(FishingEvent::Cancelled);
                }
                if input.click {
                    self.machine.pull_rod();
                    return Some(FishingEvent::Cancelled);
                }
                if let Some(FishingPhase::Bite) = self.machine.update(dt) {
                    return Some(FishingEvent::Bite);
                }
                None
            }
            FishingPhase::Bite => {
                if !can_cast {
                    self.machine.reset();
                    return Some(FishingEvent::Cancelled);
                }
                if input.click {
                    return if self.machine.pull_rod() {
                        Some(FishingEvent::Caught)
                    } else {
                        Some(FishingEvent::Escaped)
                    };
                }
                if let Some(FishingPhase::Escaped) = self.machine.update(dt) {
                    return Some(FishingEvent::Escaped);
                }
                None
            }
            FishingPhase::Caught | FishingPhase::Escaped => {
                if !can_cast {
                    self.machine.reset();
                    return None;
                }
                self.machine.update(dt);
                None
            }
        }
    }

    pub fn machine(&self) -> &FishingMachine {
        &self.machine
    }

    /// Returns `true` while clicks belong to fishing so they do not swing a weapon.
    pub fn wants_click(&self) -> bool {
        match self.machine.phase {
            FishingPhase::Idle => self.ready_to_cast,
            FishingPhase::Waiting
            | FishingPhase::Bite
            | FishingPhase::Caught
            | FishingPhase::Escaped => true,
        }
    }

    fn wait_duration_for(&self, random_01: f32) -> f32 {
        let t = if random_01.is_finite() {
            random_01.clamp(0.0, 1.0)
        } else {
            // new: DECISIONS 2026-11-23
            0.5
        };
        let min = self.cfg.cast_wait_min.min(self.cfg.cast_wait_max);
        let max = self.cfg.cast_wait_min.max(self.cfg.cast_wait_max);
        min + (max - min) * t
    }
}
