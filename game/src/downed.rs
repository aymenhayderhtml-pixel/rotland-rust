//! Knocked-down state machine before real death (bleed-out, teammate revive, self-bandage, re-down window).

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Backpack item ID consumed when self-reviving (`data/items.json`).
pub const BANDAGE_ITEM: &str = "bandage";

/// Tuning parameters loaded from `data/downed.json`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DownedConfig {
    pub bleed_out_seconds: f32,
    pub bite_seconds: f32,
    pub teammate_range: f32,
    pub revive_seconds: f32,
    pub self_revive_seconds: f32,
    pub revive_health: f32,
    pub crawl_speed_mul: f32,
    pub re_down_window_seconds: f32,
}

impl Default for DownedConfig {
    fn default() -> Self {
        Self {
            bleed_out_seconds: 30.0,      // data: downed.json bleedOutSeconds
            bite_seconds: 4.0,            // data: downed.json biteSeconds
            teammate_range: 12.0,         // data: downed.json teammateRange
            revive_seconds: 4.0,          // data: downed.json reviveSeconds
            self_revive_seconds: 6.0,     // data: downed.json selfReviveSeconds
            revive_health: 25.0,          // data: downed.json reviveHealth
            crawl_speed_mul: 0.35,        // data: downed.json crawlSpeedMul
            re_down_window_seconds: 60.0, // data: downed.json reDownWindowSeconds
        }
    }
}

impl DownedConfig {
    /// Parses a downed configuration from a JSON string.
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("invalid downed.json: {e}"))
    }

    /// Loads a downed configuration from disk.
    pub fn load(path: &Path) -> Result<Self, String> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&raw)
    }

    /// Loads `data/downed.json` from the repository root.
    pub fn from_repo() -> Result<Self, String> {
        Self::load(&paths::repo_path("data/downed.json"))
    }
}

/// Who is currently reviving the downed player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviveBy {
    None,
    Teammate,
    SelfBandage,
}

/// Current state of the downed state machine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DownedPhase {
    Up,
    Down {
        bleed: f32,
        revive_progress: f32,
        by: ReviveBy,
    },
    Dead,
}

/// One-frame outcome emitted by `Downed::tick`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DownedEvent {
    None,
    Revived,
    BledOut,
    BandageUsed,
}

impl DownedEvent {
    /// True when the player stood back up this frame (`Revived` or `BandageUsed`).
    pub fn is_revived(self) -> bool {
        matches!(self, Self::Revived | Self::BandageUsed)
    }
}

/// Knocked-down state machine tracking bleed-out, revives, and the re-down instant-death window.
#[derive(Clone, Debug)]
pub struct Downed {
    config: DownedConfig,
    phase: DownedPhase,
    last_revived_at: Option<f64>,
    downed_at: Option<f64>,
    down_elapsed: f64,
}

impl Default for Downed {
    fn default() -> Self {
        Self::new(DownedConfig::default())
    }
}

impl Downed {
    /// Creates a new `Up` state machine using `config`.
    pub fn new(config: DownedConfig) -> Self {
        Self {
            config,
            phase: DownedPhase::Up,
            last_revived_at: None,
            downed_at: None,
            down_elapsed: 0.0,
        }
    }

    /// Routes a lethal hit at world time `now` (seconds).
    ///
    /// Transitions `Up` -> `Down`, unless the player was revived less than
    /// `re_down_window_seconds` ago, in which case they die immediately (`Dead`).
    pub fn on_lethal_hit(&mut self, now: f64) -> DownedPhase {
        if !matches!(self.phase, DownedPhase::Up) {
            return self.phase;
        }
        if let Some(revived_at) = self.last_revived_at
            && now >= revived_at
            && (now - revived_at) < f64::from(self.config.re_down_window_seconds)
        {
            self.phase = DownedPhase::Dead;
            return self.phase;
        }
        self.downed_at = Some(now);
        self.down_elapsed = 0.0;
        self.phase = DownedPhase::Down {
            bleed: self.config.bleed_out_seconds,
            revive_progress: 0.0,
            by: ReviveBy::None,
        };
        self.phase
    }

    /// Applies one zombie bite while downed, shortening the bleed-out timer by `bite_seconds`.
    pub fn bite(&mut self) {
        if let DownedPhase::Down { bleed, .. } = &mut self.phase {
            *bleed = (*bleed - self.config.bite_seconds).max(0.0);
        }
    }

    /// Advances bleed-out and revive progress for one frame (`dt` seconds).
    pub fn tick(
        &mut self,
        dt: f32,
        reviving_teammate_in_range: bool,
        self_revive_held: bool,
        has_bandage: bool,
    ) -> DownedEvent {
        let dt = if dt > 0.0 && dt.is_finite() { dt } else { 0.0 };
        let DownedPhase::Down {
            mut bleed,
            mut revive_progress,
            mut by,
        } = self.phase
        else {
            return DownedEvent::None;
        };

        self.down_elapsed += f64::from(dt);
        if bleed <= 0.0 {
            self.phase = DownedPhase::Dead;
            return DownedEvent::BledOut;
        }

        let active_by = if reviving_teammate_in_range {
            ReviveBy::Teammate
        } else if self_revive_held && has_bandage {
            ReviveBy::SelfBandage
        } else {
            ReviveBy::None
        };

        match active_by {
            ReviveBy::None => {
                revive_progress = 0.0;
                by = ReviveBy::None;
            }
            ReviveBy::Teammate | ReviveBy::SelfBandage => {
                if by == active_by {
                    revive_progress += dt;
                } else {
                    by = active_by;
                    revive_progress = dt;
                }
                let target = self.revive_target(active_by);
                if target <= 0.0 || revive_progress >= target {
                    self.last_revived_at = self.downed_at.map(|t| t + self.down_elapsed);
                    self.downed_at = None;
                    self.down_elapsed = 0.0;
                    self.phase = DownedPhase::Up;
                    return match active_by {
                        ReviveBy::Teammate => DownedEvent::Revived,
                        ReviveBy::SelfBandage => DownedEvent::BandageUsed,
                        ReviveBy::None => DownedEvent::None,
                    };
                }
            }
        }

        bleed = (bleed - dt).max(0.0);
        if bleed <= 0.0 {
            self.phase = DownedPhase::Dead;
            return DownedEvent::BledOut;
        }

        self.phase = DownedPhase::Down {
            bleed,
            revive_progress,
            by,
        };
        DownedEvent::None
    }

    /// Current state machine phase.
    pub fn phase(&self) -> DownedPhase {
        self.phase
    }

    /// True while standing/normal (`DownedPhase::Up`).
    pub fn is_up(&self) -> bool {
        matches!(self.phase, DownedPhase::Up)
    }

    /// True while knocked down (`DownedPhase::Down { .. }`).
    pub fn is_down(&self) -> bool {
        matches!(self.phase, DownedPhase::Down { .. })
    }

    /// True after bleeding out or instant re-down death (`DownedPhase::Dead`).
    pub fn is_dead(&self) -> bool {
        matches!(self.phase, DownedPhase::Dead)
    }

    /// Seconds left until bleed-out when downed, or `0.0` otherwise.
    pub fn bleed_remaining(&self) -> f32 {
        match self.phase {
            DownedPhase::Down { bleed, .. } => bleed,
            DownedPhase::Up | DownedPhase::Dead => 0.0,
        }
    }

    /// Current revive progress in seconds when downed, or `0.0` otherwise.
    pub fn revive_progress(&self) -> f32 {
        match self.phase {
            DownedPhase::Down { revive_progress, .. } => revive_progress,
            DownedPhase::Up | DownedPhase::Dead => 0.0,
        }
    }

    /// Normalized `[0.0, 1.0]` revive completion fraction for the active reviver.
    pub fn revive_fraction(&self) -> f32 {
        match self.phase {
            DownedPhase::Down {
                revive_progress,
                by,
                ..
            } => {
                let target = self.revive_target(by);
                if target <= 0.0 {
                    0.0
                } else {
                    (revive_progress / target).clamp(0.0, 1.0)
                }
            }
            DownedPhase::Up | DownedPhase::Dead => 0.0,
        }
    }

    /// Who is currently reviving the player (`ReviveBy::None` when not downed or not reviving).
    pub fn revive_by(&self) -> ReviveBy {
        match self.phase {
            DownedPhase::Down { by, .. } => by,
            DownedPhase::Up | DownedPhase::Dead => ReviveBy::None,
        }
    }

    /// Active tuning configuration.
    pub fn config(&self) -> &DownedConfig {
        &self.config
    }

    /// Resets back to `Up` and clears re-down history (e.g. on respawn at camp).
    pub fn reset(&mut self) {
        self.phase = DownedPhase::Up;
        self.last_revived_at = None;
        self.downed_at = None;
        self.down_elapsed = 0.0;
    }

    fn revive_target(&self, by: ReviveBy) -> f32 {
        match by {
            ReviveBy::Teammate => self.config.revive_seconds,
            ReviveBy::SelfBandage => self.config.self_revive_seconds,
            ReviveBy::None => 0.0,
        }
    }
}
