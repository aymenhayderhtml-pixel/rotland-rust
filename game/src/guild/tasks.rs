//! Guild task definition, rewards, and event-based progress tracking.

use serde::{Deserialize, Serialize};

/// The category and condition for completing a guild task.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuildTaskKind {
    Kill {
        zombie_type: Option<String>,
        in_city: bool,
    },
    Fetch {
        item_id: String,
    },
    Explore {
        place_name: String,
    },
    Clear {
        poi_id: String,
    },
}

/// Rewards granted upon completing and claiming a guild task.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GuildReward {
    pub dollars: i32,
    pub reputation: i32,
    #[serde(default)]
    pub item: Option<(String, u32)>,
}

/// One guild task on the board or accepted by the player.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuildTask {
    pub id: String,
    pub title: String,
    pub kind: GuildTaskKind,
    pub target: u32,
    pub progress: u32,
    pub reward: GuildReward,
}

impl GuildTask {
    /// Creates a new guild task.
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        kind: GuildTaskKind,
        target: u32,
        reward: GuildReward,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            kind,
            target,
            progress: 0,
            reward,
        }
    }

    /// Whether the task requirements have been completely fulfilled.
    pub fn is_done(&self) -> bool {
        self.progress >= self.target
    }

    /// Progress completion ratio clamped between 0.0 and 1.0.
    pub fn progress_ratio(&self) -> f32 {
        if self.target == 0 {
            1.0
        } else {
            (self.progress as f32 / self.target as f32).clamp(0.0, 1.0)
        }
    }

    /// Updates task progress when a zombie is killed. Returns true if progress changed.
    pub fn on_kill(&mut self, zombie_type: &str, in_city: bool) -> bool {
        if self.is_done() {
            return false;
        }
        match &self.kind {
            GuildTaskKind::Kill {
                zombie_type: task_zombie,
                in_city: req_city,
            } => {
                if *req_city && !in_city {
                    return false;
                }
                if let Some(expected) = task_zombie.as_deref()
                    && expected != zombie_type
                {
                    return false;
                }
                self.progress += 1;
                true
            }
            _ => false,
        }
    }

    /// Updates task progress when items are delivered. Returns true if progress changed.
    pub fn on_item_delivered(&mut self, item_id: &str, count: u32) -> bool {
        if self.is_done() || count == 0 {
            return false;
        }
        match &self.kind {
            GuildTaskKind::Fetch { item_id: expected } if expected == item_id => {
                self.progress = (self.progress + count).min(self.target);
                true
            }
            _ => false,
        }
    }

    /// Updates task progress when a named place is reached. Returns true if progress changed.
    pub fn on_place_reached(&mut self, place_name: &str) -> bool {
        if self.is_done() {
            return false;
        }
        match &self.kind {
            GuildTaskKind::Explore {
                place_name: expected,
            } if expected.eq_ignore_ascii_case(place_name) => {
                self.progress = self.target;
                true
            }
            _ => false,
        }
    }

    /// Updates task progress when a POI building is cleared. Returns true if progress changed.
    pub fn on_poi_cleared(&mut self, poi_id: &str) -> bool {
        if self.is_done() {
            return false;
        }
        match &self.kind {
            GuildTaskKind::Clear { poi_id: expected }
                if expected.eq_ignore_ascii_case(poi_id) =>
            {
                self.progress = self.target;
                true
            }
            _ => false,
        }
    }
}
