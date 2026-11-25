//! Guild notice board tracking offered and active tasks with accept limits.

use super::tasks::{GuildReward, GuildTask};

/// Error when accepting a task from the board.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuildAcceptError {
    NotFound,
    LimitReached(usize),
}

impl std::fmt::Display for GuildAcceptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Task not found on board."),
            Self::LimitReached(max) => write!(f, "Cannot accept more than {max} active tasks."),
        }
    }
}

impl std::error::Error for GuildAcceptError {}

/// Error when claiming a completed task's reward.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuildClaimError {
    NotFound,
    NotFinished,
}

impl std::fmt::Display for GuildClaimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "No active task found to claim."),
            Self::NotFinished => write!(f, "Task is not yet completed."),
        }
    }
}

impl std::error::Error for GuildClaimError {}

/// The guild board holding offered and active tasks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuildBoard {
    pub max_offered: usize,
    pub max_active: usize,
    pub offered: Vec<GuildTask>,
    pub active: Vec<GuildTask>,
    pub completed_ids: Vec<String>,
}

impl GuildBoard {
    /// Creates a new guild board with specified offered and active limits.
    pub fn new(max_offered: usize, max_active: usize) -> Self {
        Self {
            max_offered,
            max_active,
            offered: Vec::new(),
            active: Vec::new(),
            completed_ids: Vec::new(),
        }
    }

    /// Tasks currently open on the board for acceptance.
    pub fn offered(&self) -> &[GuildTask] {
        &self.offered
    }

    /// Tasks currently accepted and in progress.
    pub fn active(&self) -> &[GuildTask] {
        &self.active
    }

    /// IDs of tasks claimed and completed.
    pub fn completed_ids(&self) -> &[String] {
        &self.completed_ids
    }

    /// Sets the offered tasks, truncating to `max_offered`.
    pub fn set_offered(&mut self, mut tasks: Vec<GuildTask>) {
        if tasks.len() > self.max_offered {
            tasks.truncate(self.max_offered);
        }
        self.offered = tasks;
    }

    /// Accepts an offered task onto the active list up to `max_active`.
    pub fn accept(&mut self, id: &str) -> Result<GuildTask, GuildAcceptError> {
        if self.active.len() >= self.max_active {
            return Err(GuildAcceptError::LimitReached(self.max_active));
        }
        let pos = self
            .offered
            .iter()
            .position(|t| t.id == id)
            .ok_or(GuildAcceptError::NotFound)?;
        let task = self.offered.remove(pos);
        self.active.push(task.clone());
        Ok(task)
    }

    /// Abandons an active task without penalty. Returns true if removed.
    pub fn abandon(&mut self, id: &str) -> bool {
        let prev_len = self.active.len();
        self.active.retain(|t| t.id != id);
        self.active.len() < prev_len
    }

    /// Claims rewards for a finished active task.
    pub fn claim(&mut self, id: &str) -> Result<GuildReward, GuildClaimError> {
        let pos = self
            .active
            .iter()
            .position(|t| t.id == id)
            .ok_or(GuildClaimError::NotFound)?;
        if !self.active[pos].is_done() {
            return Err(GuildClaimError::NotFinished);
        }
        let task = self.active.remove(pos);
        self.completed_ids.push(task.id);
        Ok(task.reward)
    }

    /// Advances kill tasks when a zombie kill occurs. Returns count of advanced tasks.
    pub fn on_kill(&mut self, zombie_type: &str, in_city: bool) -> usize {
        let mut count = 0;
        for task in &mut self.active {
            if task.on_kill(zombie_type, in_city) {
                count += 1;
            }
        }
        count
    }

    /// Advances fetch tasks when items are delivered. Returns count of advanced tasks.
    pub fn on_item_delivered(&mut self, item_id: &str, item_count: u32) -> usize {
        let mut count = 0;
        for task in &mut self.active {
            if task.on_item_delivered(item_id, item_count) {
                count += 1;
            }
        }
        count
    }

    /// Advances explore tasks when a named place is reached. Returns count of advanced tasks.
    pub fn on_place_reached(&mut self, place_name: &str) -> usize {
        let mut count = 0;
        for task in &mut self.active {
            if task.on_place_reached(place_name) {
                count += 1;
            }
        }
        count
    }

    /// Advances clear tasks when a POI is cleared. Returns count of advanced tasks.
    pub fn on_poi_cleared(&mut self, poi_id: &str) -> usize {
        let mut count = 0;
        for task in &mut self.active {
            if task.on_poi_cleared(poi_id) {
                count += 1;
            }
        }
        count
    }
}
