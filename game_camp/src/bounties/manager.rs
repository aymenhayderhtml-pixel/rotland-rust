use std::collections::HashSet;

use crate::bounties::config::{BountyConfig, BountyConfigData, BountyLoadError, CLEAR_POI};
use crate::bounties::def::{
    BountyAcceptError, BountyClaimError, BountyDef, BountyReward, BountySaveError,
};
use crate::bounties::roller::BountyRoller;

/// Manages offered bounties, accepted active bounties, claimed bounties, and progress tracking.
// Godot: BountyManager.cs
#[derive(Clone, Debug, PartialEq)]
pub struct BountyManager {
    seed: i32,
    config: BountyConfigData,
    offered: Vec<BountyDef>,
    pub(crate) active: Vec<BountyDef>,
    claimed: HashSet<String>,
    daily_day: i32,
    week: i32,
}

impl Default for BountyManager {
    fn default() -> Self {
        Self::new(0, BountyConfigData::default())
    }
}

impl BountyManager {
    // Godot: BountyManager.cs SaveId
    pub const SAVE_ID: &str = "bounties";

    pub fn new(seed: i32, config: BountyConfigData) -> Self {
        Self {
            seed,
            config,
            offered: Vec::new(),
            active: Vec::new(),
            claimed: HashSet::new(),
            daily_day: -1,
            week: -1,
        }
    }

    /// Loads the configuration from data/bounties.json and initializes the manager with `seed`.
    pub fn from_repo(seed: i32) -> Result<Self, BountyLoadError> {
        let config = BountyConfig::from_repo()?;
        Ok(Self::new(seed, config))
    }

    pub fn seed(&self) -> i32 {
        self.seed
    }

    pub fn set_seed(&mut self, seed: i32) {
        self.seed = seed;
    }

    pub fn config(&self) -> &BountyConfigData {
        &self.config
    }

    pub fn offered(&self) -> &[BountyDef] {
        &self.offered
    }

    pub fn active(&self) -> &[BountyDef] {
        &self.active
    }

    pub fn claimed(&self) -> &HashSet<String> {
        &self.claimed
    }

    pub fn daily_day(&self) -> i32 {
        self.daily_day
    }

    pub fn week(&self) -> i32 {
        self.week
    }

    // Godot: BountyManager.cs MaxActive
    pub fn max_active(&self) -> usize {
        self.config.max_active as usize
    }

    /// Re-rolls the daily and weekly lists when the day or week has changed, and drops expired bounties.
    // Godot: BountyManager.cs Refresh
    pub fn refresh(&mut self, day: i32, hour: f32) {
        let bounty_day = BountyRoller::bounty_day(day, hour, &self.config);
        self.refresh_bounty_day(bounty_day);
    }

    /// Refresh based directly on the current bounty day.
    pub fn refresh_bounty_day(&mut self, bounty_day: i32) {
        let week = BountyRoller::week_of(bounty_day, &self.config);

        if bounty_day != self.daily_day {
            self.offered.retain(|o| o.weekly);
            let rolled = BountyRoller::roll_daily(self.seed, bounty_day, &self.config);
            for b in rolled {
                if !self.claimed.contains(&b.id) {
                    self.offered.push(b);
                }
            }
            self.daily_day = bounty_day;
        }

        if week != self.week {
            self.offered.retain(|o| !o.weekly);
            let weekly = BountyRoller::roll_weekly(self.seed, week, &self.config);
            if !self.claimed.contains(&weekly.id) {
                self.offered.push(weekly);
            }
            self.week = week;
        }

        // A bounty not finished when its day or week ends is lost, with no penalty
        self.active.retain(|b| b.expires_on >= bounty_day);
        self.offered.retain(|o| o.expires_on >= bounty_day);
    }

    // Godot: BountyManager.cs OnHourChanged
    pub fn on_hour_changed(&mut self, day: i32, hour: f32) {
        self.refresh(day, hour);
    }

    /// Takes an offered bounty. Returns the accepted bounty or an error.
    // Godot: BountyManager.cs Take
    pub fn accept(&mut self, id: &str) -> Result<BountyDef, BountyAcceptError> {
        self.accept_with_poi(id, None, 0.0, 0.0)
    }

    /// Alias for `accept`.
    pub fn take(&mut self, id: &str) -> Result<BountyDef, BountyAcceptError> {
        self.accept(id)
    }

    /// Takes an offered bounty, optionally supplying POI details for `clear_poi`.
    pub fn accept_with_poi(
        &mut self,
        id: &str,
        poi_id: Option<String>,
        target_x: f32,
        target_z: f32,
    ) -> Result<BountyDef, BountyAcceptError> {
        let idx = self
            .offered
            .iter()
            .position(|o| o.id == id)
            .ok_or(BountyAcceptError::Gone)?;

        if self.active.len() >= self.max_active() {
            return Err(BountyAcceptError::LimitReached(self.max_active()));
        }

        let mut bounty = self.offered.remove(idx);
        if bounty.bounty_type == CLEAR_POI && poi_id.is_some() {
            bounty.poi_id = poi_id;
            bounty.target_x = target_x;
            bounty.target_z = target_z;
        }

        self.active.push(bounty.clone());
        Ok(bounty)
    }

    /// Drops an active bounty. No penalty. Returns true if removed.
    // Godot: BountyManager.cs Abandon
    pub fn abandon(&mut self, id: &str) -> bool {
        let initial_len = self.active.len();
        self.active.retain(|b| b.id != id);
        self.active.len() < initial_len
    }

    /// Pays a finished bounty: returns rewards (dollars, xp, skill).
    // Godot: BountyManager.cs Claim
    pub fn claim(&mut self, id: &str) -> Result<BountyReward, BountyClaimError> {
        let idx = self
            .active
            .iter()
            .position(|b| b.id == id)
            .ok_or(BountyClaimError::NotFound)?;

        if !self.active[idx].is_done() {
            return Err(BountyClaimError::NotFinished);
        }

        let bounty = self.active.remove(idx);
        self.claimed.insert(bounty.id.clone());

        Ok(BountyReward {
            dollars: bounty.dollars,
            xp: bounty.xp,
            skill: bounty.skill,
        })
    }

    /// Alias for `claim`.
    pub fn complete(&mut self, id: &str) -> Result<BountyReward, BountyClaimError> {
        self.claim(id)
    }

    // --- Test hooks ---

    // Godot: BountyManager.cs AddActiveForTest
    pub fn add_active_for_test(&mut self, bounty: BountyDef) {
        self.active.push(bounty);
    }

    // --- Save / Load ---

    pub(crate) fn set_save_state(
        &mut self,
        offered: Vec<BountyDef>,
        active: Vec<BountyDef>,
        claimed: HashSet<String>,
        daily_day: i32,
        week: i32,
    ) {
        self.offered = offered;
        self.active = active;
        self.claimed = claimed;
        self.daily_day = daily_day;
        self.week = week;
    }

    // Godot: BountyManager.cs Save
    pub fn to_save(&self) -> serde_json::Value {
        crate::bounties::save::to_save_manager(self)
    }

    // Godot: BountyManager.cs Load
    pub fn from_save(&mut self, data: &serde_json::Value) -> Result<(), BountySaveError> {
        crate::bounties::save::from_save_manager(self, data)
    }
}
