//! Top-level guild manager coordinating board, recruits, shop, and morning refresh.

use super::board::{GuildAcceptError, GuildBoard, GuildClaimError};
use super::config::{GuildConfig, GuildLoadError, GuildRankDef};
use super::rank::{current_rank, rank_progress};
use super::recruits::{
    GuildRecruit, GuildRecruits, RecruitDismissError, RecruitHireError, RecruitOrder,
    RecruitOrderError, RecruitRole, WageReport,
};
use super::roller::GuildTaskRoller;
use super::save::GuildState;
use super::shop::{GuildShop, GuildShopItem, ShopBuyError};
use super::tasks::{GuildReward, GuildTask};

/// Summary returned when the morning guild refresh occurs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MorningReport {
    pub day: i32,
    pub wages: WageReport,
    pub rolled_task_count: usize,
}

/// Central controller for all guild activities.
#[derive(Clone, Debug, PartialEq)]
pub struct GuildManager {
    seed: i32,
    config: GuildConfig,
    reputation: i32,
    last_refresh_day: i32,
    board: GuildBoard,
    recruits: GuildRecruits,
    shop: GuildShop,
}

impl Default for GuildManager {
    fn default() -> Self {
        Self::new(0, GuildConfig::default())
    }
}

impl GuildManager {
    /// Creates a new guild manager from seed and configuration.
    pub fn new(seed: i32, config: GuildConfig) -> Self {
        let board = GuildBoard::new(config.max_offered, config.max_active);

        let available_recruits = config
            .recruits
            .iter()
            .map(|r| {
                let role = match r.role.to_lowercase().as_str() {
                    "fighter" => RecruitRole::Fighter,
                    "scavenger" => RecruitRole::Scavenger,
                    "guard" => RecruitRole::Guard,
                    _ => RecruitRole::Fighter,
                };
                GuildRecruit::new(
                    &r.id,
                    &r.name,
                    role,
                    r.level,
                    r.daily_wage,
                    r.health,
                    r.hire_cost,
                )
            })
            .collect();
        let recruits = GuildRecruits::new(available_recruits);

        let shop_items = config
            .shop
            .iter()
            .map(|s| GuildShopItem::new(&s.item_id, s.tier, s.price, s.stock))
            .collect();
        let shop = GuildShop::new(shop_items);

        Self {
            seed,
            config,
            reputation: 0,
            last_refresh_day: -1,
            board,
            recruits,
            shop,
        }
    }

    /// Loads configuration from data/guild.json and initializes GuildManager with seed.
    pub fn from_repo(seed: i32) -> Result<Self, GuildLoadError> {
        let config = GuildConfig::from_repo()?;
        Ok(Self::new(seed, config))
    }

    pub fn seed(&self) -> i32 {
        self.seed
    }

    pub fn set_seed(&mut self, seed: i32) {
        self.seed = seed;
    }

    pub fn config(&self) -> &GuildConfig {
        &self.config
    }

    pub fn reputation(&self) -> i32 {
        self.reputation
    }

    pub fn add_reputation(&mut self, amount: i32) {
        if amount > 0 {
            self.reputation = self.reputation.saturating_add(amount);
        }
    }

    pub fn current_rank(&self) -> &GuildRankDef {
        current_rank(self.reputation, &self.config.ranks)
    }

    pub fn rank_progress(&self) -> (i32, Option<i32>) {
        rank_progress(self.reputation, &self.config.ranks)
    }

    pub fn max_recruits(&self) -> usize {
        self.current_rank().max_recruits
    }

    pub fn shop_tier(&self) -> u32 {
        self.current_rank().shop_tier
    }

    pub fn board(&self) -> &GuildBoard {
        &self.board
    }

    pub fn board_mut(&mut self) -> &mut GuildBoard {
        &mut self.board
    }

    pub fn recruits(&self) -> &GuildRecruits {
        &self.recruits
    }

    pub fn recruits_mut(&mut self) -> &mut GuildRecruits {
        &mut self.recruits
    }

    pub fn shop(&self) -> &GuildShop {
        &self.shop
    }

    pub fn shop_mut(&mut self) -> &mut GuildShop {
        &mut self.shop
    }

    pub fn last_refresh_day(&self) -> i32 {
        self.last_refresh_day
    }

    /// Refreshes the guild each morning at reset_hour: rolls new board tasks, pays wages, and restocks shop.
    pub fn refresh_morning(
        &mut self,
        day: i32,
        hour: f32,
        mut funds: Option<&mut i32>,
    ) -> Option<MorningReport> {
        if day != self.last_refresh_day && hour >= self.config.reset_hour {
            let rolled = GuildTaskRoller::roll_board(
                self.seed,
                day,
                self.config.max_offered,
                &self.config.tasks,
            );
            let rolled_count = rolled.len();
            self.board.set_offered(rolled);

            let wages = if let Some(ref mut wallet_funds) = funds {
                self.recruits.pay_wages(wallet_funds)
            } else {
                WageReport::default()
            };

            self.shop.restock();
            self.last_refresh_day = day;

            Some(MorningReport {
                day,
                wages,
                rolled_task_count: rolled_count,
            })
        } else {
            None
        }
    }

    /// Accepts an offered task onto the active list.
    pub fn accept_task(&mut self, id: &str) -> Result<GuildTask, GuildAcceptError> {
        self.board.accept(id)
    }

    /// Abandons an active task.
    pub fn abandon_task(&mut self, id: &str) -> bool {
        self.board.abandon(id)
    }

    /// Claims rewards for a finished active task, automatically adding reputation to the guild.
    pub fn claim_task(&mut self, id: &str) -> Result<GuildReward, GuildClaimError> {
        let reward = self.board.claim(id)?;
        self.add_reputation(reward.reputation);
        Ok(reward)
    }

    /// Forwards zombie kill events to active tasks. Returns count of advanced tasks.
    pub fn on_kill(&mut self, zombie_type: &str, in_city: bool) -> usize {
        self.board.on_kill(zombie_type, in_city)
    }

    /// Forwards item delivery events to active tasks. Returns count of advanced tasks.
    pub fn on_item_delivered(&mut self, item_id: &str, count: u32) -> usize {
        self.board.on_item_delivered(item_id, count)
    }

    /// Forwards place exploration events to active tasks. Returns count of advanced tasks.
    pub fn on_place_reached(&mut self, place_name: &str) -> usize {
        self.board.on_place_reached(place_name)
    }

    /// Forwards POI clearing events to active tasks. Returns count of advanced tasks.
    pub fn on_poi_cleared(&mut self, poi_id: &str) -> usize {
        self.board.on_poi_cleared(poi_id)
    }

    /// Hires a recruit using current rank's recruit slots and spending funds.
    pub fn hire_recruit(&mut self, id: &str, funds: &mut i32) -> Result<(), RecruitHireError> {
        let max_slots = self.max_recruits();
        self.recruits.hire(id, max_slots, funds)
    }

    /// Dismisses a hired recruit.
    pub fn dismiss_recruit(&mut self, id: &str) -> Result<GuildRecruit, RecruitDismissError> {
        self.recruits.dismiss(id)
    }

    /// Assigns standing order to a hired recruit.
    pub fn set_recruit_order(
        &mut self,
        id: &str,
        order: RecruitOrder,
    ) -> Result<(), RecruitOrderError> {
        self.recruits.set_order(id, order)
    }

    /// Manually pays recruit wages.
    pub fn pay_recruit_wages(&mut self, funds: &mut i32) -> WageReport {
        self.recruits.pay_wages(funds)
    }

    /// Purchases an item from the shop using the player's current guild shop tier.
    pub fn buy_shop_item(
        &mut self,
        item_id: &str,
        count: u32,
        funds: &mut i32,
    ) -> Result<u32, ShopBuyError> {
        let player_tier = self.shop_tier();
        self.shop.buy(item_id, count, player_tier, funds)
    }

    /// Serializes guild state for saving.
    pub fn to_state(&self) -> GuildState {
        GuildState {
            reputation: self.reputation,
            last_refresh_day: self.last_refresh_day,
            offered_tasks: self.board.offered.clone(),
            active_tasks: self.board.active.clone(),
            completed_task_ids: self.board.completed_ids.clone(),
            hired_recruits: self.recruits.hired.clone(),
            available_recruits: self.recruits.available.clone(),
            shop_items: self.shop.items.clone(),
        }
    }

    /// Restores guild state from a saved state.
    pub fn from_state(&mut self, state: GuildState) {
        self.reputation = state.reputation;
        self.last_refresh_day = state.last_refresh_day;
        self.board.offered = state.offered_tasks;
        self.board.active = state.active_tasks;
        self.board.completed_ids = state.completed_task_ids;
        self.recruits.hired = state.hired_recruits;
        self.recruits.available = state.available_recruits;
        self.shop.items = state.shop_items;
    }
}
