use serde::{Deserialize, Serialize};
use super::config::FishingConfig;

/// Current phase of the fishing cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FishingPhase {
    Idle,
    Waiting,
    Bite,
    Caught,
    Escaped,
}

/// State machine driving the fishing loop.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FishingMachine {
    pub phase: FishingPhase,
    pub wait_timer: f32,
    pub wait_duration: f32,
    pub bite_timer: f32,
    pub bite_duration: f32,
    pub sweet_spot_min: f32,
    pub sweet_spot_max: f32,
    pub result_timer: f32,
}

impl FishingMachine {
    pub fn new(cfg: &FishingConfig) -> Self {
        Self {
            phase: FishingPhase::Idle,
            wait_timer: 0.0,
            wait_duration: cfg.cast_wait_min,
            bite_timer: 0.0,
            bite_duration: cfg.bite_duration,
            sweet_spot_min: cfg.sweet_spot_min,
            sweet_spot_max: cfg.sweet_spot_max,
            result_timer: 0.0,
        }
    }

    /// Starts casting if idle.
    pub fn start_cast(&mut self, wait_duration: f32) -> bool {
        if self.phase == FishingPhase::Idle {
            self.phase = FishingPhase::Waiting;
            self.wait_timer = 0.0;
            self.wait_duration = wait_duration;
            self.bite_timer = 0.0;
            self.result_timer = 0.0;
            true
        } else {
            false
        }
    }

    /// Progress through the bite window in [0.0, 1.0].
    pub fn progress(&self) -> f32 {
        if self.bite_duration > 0.0 {
            (self.bite_timer / self.bite_duration).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Whether the bite progress needle is currently inside the sweet spot.
    pub fn is_in_sweet_spot(&self) -> bool {
        let p = self.progress();
        p >= self.sweet_spot_min && p <= self.sweet_spot_max
    }

    /// Pulls the rod (attack press). Returns true if a fish was caught.
    pub fn pull_rod(&mut self) -> bool {
        match self.phase {
            FishingPhase::Bite => {
                if self.is_in_sweet_spot() {
                    self.phase = FishingPhase::Caught;
                    self.result_timer = 0.0;
                    true
                } else {
                    self.phase = FishingPhase::Escaped;
                    self.result_timer = 0.0;
                    false
                }
            }
            FishingPhase::Waiting => {
                // Pulled too early before a bite
                self.phase = FishingPhase::Idle;
                false
            }
            _ => false,
        }
    }

    /// Cancels or resets fishing back to Idle.
    pub fn reset(&mut self) {
        self.phase = FishingPhase::Idle;
        self.wait_timer = 0.0;
        self.bite_timer = 0.0;
        self.result_timer = 0.0;
    }

    /// Updates fishing timers. Returns Some(FishingPhase) on transitions into Caught or Escaped.
    pub fn update(&mut self, dt: f32) -> Option<FishingPhase> {
        match self.phase {
            FishingPhase::Idle => None,
            FishingPhase::Waiting => {
                self.wait_timer += dt;
                if self.wait_timer >= self.wait_duration {
                    self.phase = FishingPhase::Bite;
                    self.bite_timer = 0.0;
                    Some(FishingPhase::Bite)
                } else {
                    None
                }
            }
            FishingPhase::Bite => {
                self.bite_timer += dt;
                if self.bite_timer >= self.bite_duration {
                    // Time ran out, fish got away
                    self.phase = FishingPhase::Escaped;
                    self.result_timer = 0.0;
                    Some(FishingPhase::Escaped)
                } else {
                    None
                }
            }
            FishingPhase::Caught | FishingPhase::Escaped => {
                self.result_timer += dt;
                if self.result_timer >= 2.0 {
                    self.phase = FishingPhase::Idle;
                }
                None
            }
        }
    }
}
