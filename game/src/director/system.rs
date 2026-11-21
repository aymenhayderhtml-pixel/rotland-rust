//! Director state machine and pressure pacing logic.

use super::config::{DirectorConfig, DirectorStateConfig};
use super::state::{DirectorEvent, DirectorSaveState, DirectorState, HordeEvent, WaveKind};

/// Input per frame/tick to drive the director state machine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DirectorInput {
    pub dt: f32,
    pub is_night: bool,
    pub player_health: f32,
    pub recent_kills: u32,
    pub zombies_near: usize,
    pub in_city: bool,
    pub player_in_camp: bool,
}

impl DirectorInput {
    pub fn new(dt: f32) -> Self {
        Self {
            dt,
            is_night: false,
            player_health: 1.0,
            recent_kills: 0,
            zombies_near: 0,
            in_city: false,
            player_in_camp: false,
        }
    }
}

/// Output computed by the director each tick.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectorOutput {
    pub spawn_rate_multiplier: f32,
    pub max_active_multiplier: f32,
    pub special_chance: f32,
    pub events: Vec<DirectorEvent>,
    pub horde_events: Vec<HordeEvent>,
}

/// Paces zombie pressure across Calm, Building, Wave, and Relax states.
#[derive(Clone, Debug)]
pub struct Director {
    config: DirectorConfig,
    state: DirectorState,
    timer: f32,
    night_timer: f32,
    night_horde_triggered: bool,
    was_night: bool,
}

impl Director {
    pub fn new(config: DirectorConfig) -> Self {
        Self {
            config,
            state: DirectorState::Calm,
            timer: 0.0,
            night_timer: 0.0,
            night_horde_triggered: false,
            was_night: false,
        }
    }

    /// Current pacing state.
    pub fn state(&self) -> DirectorState {
        self.state
    }

    /// Access configuration.
    pub fn config(&self) -> &DirectorConfig {
        &self.config
    }

    /// Spawn rate multiplier for the current state.
    pub fn spawn_rate_multiplier(&self) -> f32 {
        self.state_config(self.state).spawn_rate_multiplier
    }

    /// Maximum active zombies multiplier for the current state.
    pub fn max_active_multiplier(&self) -> f32 {
        self.state_config(self.state).max_active_multiplier
    }

    /// Probability that a spawned zombie is a special variant.
    pub fn special_chance(&self) -> f32 {
        self.state_config(self.state).special_chance
    }

    /// Export current progress for save games.
    pub fn to_save_state(&self) -> DirectorSaveState {
        DirectorSaveState {
            state: self.state,
            timer: self.timer,
            night_timer: self.night_timer,
            night_horde_triggered: self.night_horde_triggered,
            was_night: self.was_night,
        }
    }

    /// Restore state from saved data.
    pub fn from_save_state(&mut self, save: &DirectorSaveState) {
        self.state = save.state;
        self.timer = save.timer;
        self.night_timer = save.night_timer;
        self.night_horde_triggered = save.night_horde_triggered;
        self.was_night = save.was_night;
    }

    /// Advance the director simulation by dt and evaluate events.
    pub fn update(&mut self, input: &DirectorInput) -> DirectorOutput {
        let mut events = Vec::new();
        let mut horde_events = Vec::new();

        // Track day-to-night transitions
        if input.is_night && !self.was_night {
            self.night_horde_triggered = false;
            self.night_timer = 0.0;
        }
        self.was_night = input.is_night;

        // Big night horde logic: about once per night, targets camp.
        // Triggers even if player is at low health or inside camp safe zone.
        if input.is_night {
            self.night_timer += input.dt;
            if !self.night_horde_triggered
                && self.night_timer >= self.config.rules.night_horde_delay
            {
                self.night_horde_triggered = true;
                events.push(DirectorEvent::CampAttack {
                    size: self.config.rules.night_horde_size,
                });
                horde_events.push(HordeEvent::CampAttack {
                    size: self.config.rules.night_horde_size,
                });
            }
        }

        self.timer += input.dt;
        let cur_cfg = self.state_config(self.state);
        let duration = self.duration_for(&cur_cfg, input.is_night, input.in_city);

        match self.state {
            DirectorState::Calm => {
                if self.timer >= duration {
                    self.state = DirectorState::Building;
                    self.timer = 0.0;
                }
            }
            DirectorState::Building => {
                // Rule: never a wave while player is in safe zone or health < 0.3 (unless night horde)
                let low_health = input.player_health < self.config.rules.low_health_threshold;
                if self.timer >= duration {
                    if input.player_in_camp || low_health {
                        // Hold in building state while condition persists
                        self.timer = duration;
                    } else {
                        self.state = DirectorState::Wave;
                        self.timer = 0.0;

                        let (kind, size) = if input.in_city {
                            (WaveKind::City, self.config.rules.wave_size_city)
                        } else if input.is_night {
                            (WaveKind::Night, self.config.rules.wave_size_night)
                        } else {
                            (WaveKind::Noise, self.config.rules.wave_size_day)
                        };
                        events.push(DirectorEvent::WaveStart { size, kind });
                        horde_events.push(HordeEvent::HordeIncoming { size });
                    }
                }
            }
            DirectorState::Wave => {
                if self.timer >= duration {
                    self.state = DirectorState::Relax;
                    self.timer = 0.0;
                }
            }
            DirectorState::Relax => {
                if self.timer >= duration {
                    self.state = DirectorState::Calm;
                    self.timer = 0.0;
                }
            }
        }

        let st_cfg = self.state_config(self.state);
        DirectorOutput {
            spawn_rate_multiplier: st_cfg.spawn_rate_multiplier,
            max_active_multiplier: st_cfg.max_active_multiplier,
            special_chance: st_cfg.special_chance,
            events,
            horde_events,
        }
    }

    fn state_config(&self, state: DirectorState) -> DirectorStateConfig {
        let key = match state {
            DirectorState::Calm => "calm",
            DirectorState::Building => "building",
            DirectorState::Wave => "wave",
            DirectorState::Relax => "relax",
        };
        self.config
            .states
            .get(key)
            .cloned()
            .unwrap_or(DirectorStateConfig {
                duration_day: 30.0, // data: director.json durationDay
                duration_night: 15.0, // data: director.json durationNight
                duration_city: 15.0, // data: director.json durationCity
                spawn_rate_multiplier: 1.0,
                max_active_multiplier: 1.0,
                special_chance: 0.0,
            })
    }

    fn duration_for(&self, cfg: &DirectorStateConfig, is_night: bool, in_city: bool) -> f32 {
        if in_city {
            cfg.duration_city
        } else if is_night {
            cfg.duration_night
        } else {
            cfg.duration_day
        }
    }
}
