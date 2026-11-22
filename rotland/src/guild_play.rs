//! Guild gameplay state manager for Rotland.
//!
//! Owns a [`GuildManager`] from `game::guild`, dispatches morning resets,
//! game event hooks (kills, exploration, item delivery), and handles save serialization.

use game::guild::{
    GuildConfig, GuildManager, GuildReward, GuildState, GuildTask, MorningReport, RecruitOrder,
    ShopBuyError,
};

/// High-level guild gameplay controller attached to the game session/camp.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub struct GuildPlay {
    manager: GuildManager,
}

impl Default for GuildPlay {
    fn default() -> Self {
        Self::new(0)
    }
}

#[allow(dead_code)]
impl GuildPlay {
    /// Creates a guild controller from the world seed, loading configuration from repo.
    /// Falls back to default config if repo files are unavailable.
    pub fn new(seed: i32) -> Self {
        let manager = GuildManager::from_repo(seed)
            .unwrap_or_else(|_| GuildManager::new(seed, GuildConfig::default()));
        Self { manager }
    }

    /// Read-only reference to the underlying guild manager.
    pub fn manager(&self) -> &GuildManager {
        &self.manager
    }

    /// Mutable reference to the underlying guild manager.
    pub fn manager_mut(&mut self) -> &mut GuildManager {
        &mut self.manager
    }

    /// Refreshes the guild each morning at reset hour (rolls tasks, pays wages, restocks shop).
    pub fn refresh_morning(
        &mut self,
        day: i32,
        hour: f32,
        funds: Option<&mut i32>,
    ) -> Option<MorningReport> {
        self.manager.refresh_morning(day, hour, funds)
    }

    /// Hook for zombie kill events. Returns count of advanced active tasks.
    pub fn on_kill(&mut self, kind: &str, in_city: bool) -> usize {
        self.manager.on_kill(kind, in_city)
    }

    /// Hook for reaching named places/regions. Returns count of advanced active tasks.
    pub fn on_place_reached(&mut self, name: &str) -> usize {
        self.manager.on_place_reached(name)
    }

    /// Hook for item deliveries. Returns count of advanced active tasks.
    pub fn on_item_delivered(&mut self, item_id: &str, count: u32) -> usize {
        self.manager.on_item_delivered(item_id, count)
    }

    /// Hook for cleared POIs. Returns count of advanced active tasks.
    pub fn on_poi_cleared(&mut self, poi_id: &str) -> usize {
        self.manager.on_poi_cleared(poi_id)
    }

    /// Accepts an offered task onto the active list.
    pub fn accept_task(&mut self, task_id: &str) -> Result<GuildTask, String> {
        self.manager
            .accept_task(task_id)
            .map_err(|e| e.to_string())
    }

    /// Abandons an active task without penalty.
    pub fn abandon_task(&mut self, task_id: &str) -> bool {
        self.manager.abandon_task(task_id)
    }

    /// Claims rewards for a completed task and increases reputation.
    pub fn claim_task(&mut self, task_id: &str) -> Result<GuildReward, String> {
        self.manager
            .claim_task(task_id)
            .map_err(|e| e.to_string())
    }

    /// Hires an available recruit teammate using player funds.
    pub fn hire_recruit(&mut self, recruit_id: &str, funds: &mut i32) -> Result<(), String> {
        self.manager
            .hire_recruit(recruit_id, funds)
            .map_err(|e| e.to_string())
    }

    /// Dismisses a hired recruit.
    pub fn dismiss_recruit(&mut self, recruit_id: &str) -> Result<(), String> {
        self.manager
            .dismiss_recruit(recruit_id)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Sets standing order for a teammate.
    pub fn set_recruit_order(&mut self, recruit_id: &str, order: RecruitOrder) -> Result<(), String> {
        self.manager
            .set_recruit_order(recruit_id, order)
            .map_err(|e| e.to_string())
    }

    /// Purchases an item from the guild shop.
    pub fn buy_shop_item(
        &mut self,
        item_id: &str,
        count: u32,
        funds: &mut i32,
    ) -> Result<u32, ShopBuyError> {
        self.manager.buy_shop_item(item_id, count, funds)
    }

    /// Serializes guild state for saving.
    pub fn to_state(&self) -> GuildState {
        self.manager.to_state()
    }

    /// Restores guild controller from a saved state.
    pub fn from_state(state: GuildState) -> Self {
        Self::from_seed_and_state(0, state)
    }

    /// Restores guild controller from a world seed and saved state.
    pub fn from_seed_and_state(seed: i32, state: GuildState) -> Self {
        let mut play = Self::new(seed);
        play.manager.from_state(state);
        play
    }

    /// Restores guild manager state in-place from a saved state.
    pub fn apply_state(&mut self, state: GuildState) {
        self.manager.from_state(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game::guild::{GuildTaskKind, RecruitOrder};

    #[test]
    fn test_guild_play_creation_and_morning_refresh() {
        let mut play = GuildPlay::new(42);
        assert_eq!(play.manager().reputation(), 0);
        assert_eq!(play.manager().last_refresh_day(), -1);

        let mut funds = 100;
        let report = play.refresh_morning(1, 6.0, Some(&mut funds));
        assert!(report.is_some());
        let rep = report.unwrap();
        assert_eq!(rep.day, 1);
        assert!(rep.rolled_task_count > 0);
        assert_eq!(play.manager().last_refresh_day(), 1);

        // Same day doesn't refresh twice
        let repeat = play.refresh_morning(1, 10.0, Some(&mut funds));
        assert!(repeat.is_none());
    }

    #[test]
    fn test_guild_play_event_hooks() {
        let mut play = GuildPlay::new(100);
        // Force refresh day 1
        let mut funds = 50;
        play.refresh_morning(1, 6.0, Some(&mut funds));

        let offered = play.manager().board().offered().to_vec();
        assert!(!offered.is_empty());
        let task_id = offered[0].id.clone();
        assert!(play.accept_task(&task_id).is_ok());

        // Test event dispatching without panics
        let _ = play.on_kill("walker", false);
        let _ = play.on_kill("runner", true);
        let _ = play.on_place_reached("Ashford core");
        let _ = play.on_item_delivered("wood", 2);
        let _ = play.on_poi_cleared("gas_station");
    }

    #[test]
    fn test_guild_play_save_round_trip() {
        let mut play = GuildPlay::new(777);
        let mut funds = 200;
        play.refresh_morning(1, 6.5, Some(&mut funds));
        play.manager_mut().add_reputation(150);

        let state = play.to_state();
        assert_eq!(state.reputation, 150);
        assert_eq!(state.last_refresh_day, 1);

        let restored = GuildPlay::from_seed_and_state(777, state.clone());
        assert_eq!(restored.manager().reputation(), 150);
        assert_eq!(restored.manager().last_refresh_day(), 1);

        let mut in_place = GuildPlay::new(999);
        in_place.apply_state(state);
        assert_eq!(in_place.manager().reputation(), 150);
        assert_eq!(in_place.manager().last_refresh_day(), 1);
    }

    #[test]
    fn test_guild_play_recruit_orders_and_dismiss() {
        let mut play = GuildPlay::new(123);
        let mut funds = 500;
        let available = play.manager().recruits().available().to_vec();
        assert!(!available.is_empty());
        let rec_id = available[0].id.clone();

        assert!(play.hire_recruit(&rec_id, &mut funds).is_ok());
        assert!(play.set_recruit_order(&rec_id, RecruitOrder::Follow).is_ok());
        assert!(play.set_recruit_order(&rec_id, RecruitOrder::Guard).is_ok());
        assert!(play.dismiss_recruit(&rec_id).is_ok());
    }

    #[test]
    fn test_guild_play_task_lifecycle() {
        let mut play = GuildPlay::new(456);
        let custom_task = GuildTask::new(
            "custom_kill",
            "Custom Walker Hunt",
            GuildTaskKind::Kill {
                zombie_type: Some("walker".into()),
                in_city: false,
            },
            2,
            GuildReward {
                dollars: 50,
                reputation: 25,
                item: None,
            },
        );
        play.manager_mut().board_mut().set_offered(vec![custom_task]);
        assert!(play.accept_task("custom_kill").is_ok());

        assert_eq!(play.on_kill("walker", false), 1);
        assert_eq!(play.on_kill("walker", false), 1);

        let claim_res = play.claim_task("custom_kill");
        assert!(claim_res.is_ok());
        let reward = claim_res.unwrap();
        assert_eq!(reward.dollars, 50);
        assert_eq!(reward.reputation, 25);
        assert_eq!(play.manager().reputation(), 25);

        // Test save state serialization and restoration
        let state = play.to_state();
        let restored = GuildPlay::from_state(state);
        assert_eq!(restored.manager().reputation(), 25);
    }
}
