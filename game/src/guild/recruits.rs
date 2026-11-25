//! Teammate recruitment, orders, state management, and daily wage payments.

use serde::{Deserialize, Serialize};

/// Combat and utility role for a recruited teammate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecruitRole {
    Fighter,
    Scavenger,
    Guard,
}

/// Dynamic activity state for a teammate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecruitState {
    IdleAtCamp,
    FollowingPlayer,
    GuardingCamp,
    Dead,
}

/// Standing order given to a teammate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecruitOrder {
    Stay,
    Follow,
    Guard,
}

/// Individual recruited teammate data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuildRecruit {
    pub id: String,
    pub name: String,
    pub role: RecruitRole,
    pub level: u32,
    pub daily_wage: u32,
    pub health: u32,
    pub max_health: u32,
    pub state: RecruitState,
    pub hire_cost: u32,
}

impl GuildRecruit {
    /// Creates a new recruit in the default idle state.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        role: RecruitRole,
        level: u32,
        daily_wage: u32,
        max_health: u32,
        hire_cost: u32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            role,
            level,
            daily_wage,
            health: max_health,
            max_health,
            state: RecruitState::IdleAtCamp,
            hire_cost,
        }
    }

    /// Whether this teammate is still alive.
    pub fn is_alive(&self) -> bool {
        self.state != RecruitState::Dead && self.health > 0
    }

    /// Updates this recruit's order and corresponding state if alive. Returns true if changed.
    pub fn set_order(&mut self, order: RecruitOrder) -> bool {
        if !self.is_alive() {
            return false;
        }
        self.state = match order {
            RecruitOrder::Stay => RecruitState::IdleAtCamp,
            RecruitOrder::Follow => RecruitState::FollowingPlayer,
            RecruitOrder::Guard => RecruitState::GuardingCamp,
        };
        true
    }

    /// Inflicts damage, transitioning to Dead if health reaches 0.
    pub fn take_damage(&mut self, damage: u32) {
        if damage >= self.health {
            self.health = 0;
            self.state = RecruitState::Dead;
        } else {
            self.health -= damage;
        }
    }

    /// Restores health up to max_health if alive.
    pub fn heal(&mut self, amount: u32) {
        if self.is_alive() {
            self.health = (self.health + amount).min(self.max_health);
        }
    }
}

/// Errors when hiring a recruit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecruitHireError {
    NotFound,
    AlreadyHired,
    SlotsFull(usize),
    InsufficientFunds { needed: u32, available: i32 },
}

impl std::fmt::Display for RecruitHireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Recruit candidate not found."),
            Self::AlreadyHired => write!(f, "Recruit is already hired."),
            Self::SlotsFull(slots) => write!(f, "Recruit slots full ({slots}). Increase guild rank."),
            Self::InsufficientFunds { needed, available } => {
                write!(f, "Cannot afford hire cost ({needed}$ needed, {available}$ available).")
            }
        }
    }
}

impl std::error::Error for RecruitHireError {}

/// Errors when dismissing a recruit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecruitDismissError {
    NotFound,
}

impl std::fmt::Display for RecruitDismissError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Hired recruit not found."),
        }
    }
}

impl std::error::Error for RecruitDismissError {}

/// Errors when assigning an order to a recruit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecruitOrderError {
    NotFound,
    RecruitDead,
}

impl std::fmt::Display for RecruitOrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Hired recruit not found."),
            Self::RecruitDead => write!(f, "Cannot give orders to a dead recruit."),
        }
    }
}

impl std::error::Error for RecruitOrderError {}

/// Summary report of morning wage payments.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct WageReport {
    pub total_wages_paid: u32,
    pub paid_recruits: Vec<String>,
    pub departed_recruits: Vec<String>,
}

/// Roster of available candidates and hired teammates.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct GuildRecruits {
    pub available: Vec<GuildRecruit>,
    pub hired: Vec<GuildRecruit>,
}

impl GuildRecruits {
    /// Creates a recruits manager with an initial pool of available candidates.
    pub fn new(available: Vec<GuildRecruit>) -> Self {
        Self {
            available,
            hired: Vec::new(),
        }
    }

    /// Available recruit candidates.
    pub fn available(&self) -> &[GuildRecruit] {
        &self.available
    }

    /// Recruits currently hired by the player.
    pub fn hired(&self) -> &[GuildRecruit] {
        &self.hired
    }

    /// Hires a recruit from the candidate pool, deducting hire cost from funds.
    pub fn hire(
        &mut self,
        id: &str,
        max_slots: usize,
        funds: &mut i32,
    ) -> Result<(), RecruitHireError> {
        if self.hired.iter().any(|r| r.id == id) {
            return Err(RecruitHireError::AlreadyHired);
        }
        if self.hired.len() >= max_slots {
            return Err(RecruitHireError::SlotsFull(max_slots));
        }
        let pos = self
            .available
            .iter()
            .position(|r| r.id == id)
            .ok_or(RecruitHireError::NotFound)?;
        let recruit = &self.available[pos];
        if *funds < recruit.hire_cost as i32 {
            return Err(RecruitHireError::InsufficientFunds {
                needed: recruit.hire_cost,
                available: *funds,
            });
        }
        *funds -= recruit.hire_cost as i32;
        let mut hired_recruit = self.available.remove(pos);
        hired_recruit.state = RecruitState::IdleAtCamp;
        self.hired.push(hired_recruit);
        Ok(())
    }

    /// Adds a rescued survivor to the hired roster for free with standing order Follow.
    pub fn add_rescued(
        &mut self,
        mut recruit: GuildRecruit,
        max_slots: usize,
    ) -> Result<(), RecruitHireError> {
        if self.hired.iter().any(|r| r.id == recruit.id) {
            return Err(RecruitHireError::AlreadyHired);
        }
        if self.hired.len() >= max_slots {
            return Err(RecruitHireError::SlotsFull(max_slots));
        }
        recruit.hire_cost = 0;
        recruit.set_order(RecruitOrder::Follow);
        self.hired.push(recruit);
        Ok(())
    }

    /// Dismisses a hired recruit back to the candidate pool.
    pub fn dismiss(&mut self, id: &str) -> Result<GuildRecruit, RecruitDismissError> {
        let pos = self
            .hired
            .iter()
            .position(|r| r.id == id)
            .ok_or(RecruitDismissError::NotFound)?;
        let mut recruit = self.hired.remove(pos);
        if recruit.is_alive() {
            recruit.state = RecruitState::IdleAtCamp;
            self.available.push(recruit.clone());
        }
        Ok(recruit)
    }

    /// Sets standing order for a hired recruit.
    pub fn set_order(&mut self, id: &str, order: RecruitOrder) -> Result<(), RecruitOrderError> {
        let recruit = self
            .hired
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or(RecruitOrderError::NotFound)?;
        if !recruit.is_alive() {
            return Err(RecruitOrderError::RecruitDead);
        }
        recruit.set_order(order);
        Ok(())
    }

    /// Pays daily wages to all hired teammates each morning. Unpaid teammates leave.
    pub fn pay_wages(&mut self, funds: &mut i32) -> WageReport {
        let mut report = WageReport::default();
        let mut retained = Vec::with_capacity(self.hired.len());

        for mut recruit in self.hired.drain(..) {
            if !recruit.is_alive() {
                retained.push(recruit);
                continue;
            }
            let wage = recruit.daily_wage;
            if *funds >= wage as i32 {
                *funds -= wage as i32;
                report.total_wages_paid += wage;
                report.paid_recruits.push(recruit.id.clone());
                retained.push(recruit);
            } else {
                // Cannot afford daily wage: teammate leaves
                report.departed_recruits.push(recruit.id.clone());
                recruit.state = RecruitState::IdleAtCamp;
                self.available.push(recruit);
            }
        }
        self.hired = retained;
        report
    }
}
