//! Unit and integration tests for the guild system: task board, recruits, ranks, shop, and state.

use game::guild::*;

#[test]
fn guild_config_load_from_repo() {
    let config = GuildConfig::from_repo().expect("failed to load data/guild.json");
    assert_eq!(config.reset_hour, 6.0);
    assert_eq!(config.max_offered, 5);
    assert_eq!(config.max_active, 3);
    assert_eq!(config.ranks.len(), 5);
    assert!(!config.tasks.is_empty());
    assert!(!config.recruits.is_empty());
    assert!(!config.shop.is_empty());

    let json_str = serde_json::to_string(&config).unwrap();
    let loaded_from_json = GuildConfig::from_json(&json_str).unwrap();
    assert_eq!(config, loaded_from_json);
}

#[test]
fn guild_board_refresh_deterministic() {
    let config = GuildConfig::from_repo().expect("failed to load data/guild.json");
    let seed = 424242;

    let roll_day1_a = GuildTaskRoller::roll_board(seed, 1, 5, &config.tasks);
    let roll_day1_b = GuildTaskRoller::roll_board(seed, 1, 5, &config.tasks);
    assert_eq!(roll_day1_a, roll_day1_b);
    assert_eq!(roll_day1_a.len(), 5);

    let roll_day2 = GuildTaskRoller::roll_board(seed, 2, 5, &config.tasks);
    assert_ne!(roll_day1_a, roll_day2);

    let roll_diff_seed = GuildTaskRoller::roll_board(99999, 1, 5, &config.tasks);
    assert_ne!(roll_day1_a, roll_diff_seed);
}

#[test]
fn guild_accept_limit() {
    let mut board = GuildBoard::new(5, 3);
    let task1 = GuildTask::new("t1", "Task 1", GuildTaskKind::Fetch { item_id: "wood".into() }, 5, GuildReward::default());
    let task2 = GuildTask::new("t2", "Task 2", GuildTaskKind::Fetch { item_id: "wood".into() }, 5, GuildReward::default());
    let task3 = GuildTask::new("t3", "Task 3", GuildTaskKind::Fetch { item_id: "wood".into() }, 5, GuildReward::default());
    let task4 = GuildTask::new("t4", "Task 4", GuildTaskKind::Fetch { item_id: "wood".into() }, 5, GuildReward::default());

    board.set_offered(vec![task1, task2, task3, task4]);
    assert_eq!(board.offered().len(), 4);
    assert_eq!(board.active().len(), 0);

    // Accept non-existent
    assert_eq!(board.accept("non_existent"), Err(GuildAcceptError::NotFound));

    // Accept up to 3
    assert!(board.accept("t1").is_ok());
    assert!(board.accept("t2").is_ok());
    assert!(board.accept("t3").is_ok());
    assert_eq!(board.active().len(), 3);
    assert_eq!(board.offered().len(), 1);

    // 4th acceptance fails due to limit
    assert_eq!(board.accept("t4"), Err(GuildAcceptError::LimitReached(3)));

    // Abandoning one frees a slot
    assert!(board.abandon("t2"));
    assert_eq!(board.active().len(), 2);
    assert!(board.accept("t4").is_ok());
    assert_eq!(board.active().len(), 3);

    // Test abandon through manager as well
    let mut mgr = GuildManager::default();
    let t = GuildTask::new("m_t1", "Title", GuildTaskKind::Fetch { item_id: "wood".into() }, 1, GuildReward::default());
    mgr.board_mut().set_offered(vec![t]);
    assert!(mgr.accept_task("m_t1").is_ok());
    assert!(mgr.abandon_task("m_t1"));
    assert_eq!(mgr.board().active().len(), 0);
}

#[test]
fn guild_kill_task_progress_and_complete() {
    let mut manager = GuildManager::from_repo(12345).unwrap();
    let task = GuildTask::new(
        "kill_test",
        "Kill runners in city",
        GuildTaskKind::Kill {
            zombie_type: Some("runner".into()),
            in_city: true,
        },
        2,
        GuildReward {
            dollars: 50,
            reputation: 25,
            item: Some(("bandage".into(), 2)),
        },
    );

    manager.board_mut().set_offered(vec![task]);
    assert_eq!(manager.board().offered().len(), 1);
    manager.accept_task("kill_test").unwrap();

    // Walker in city should not count
    assert_eq!(manager.on_kill("walker", true), 0);

    // Runner outside city should not count
    assert_eq!(manager.on_kill("runner", false), 0);

    // Runner in city should count
    assert_eq!(manager.on_kill("runner", true), 1);
    assert_eq!(manager.board().active()[0].progress, 1);
    assert!(!manager.board().active()[0].is_done());
    assert_eq!(manager.board().active()[0].progress_ratio(), 0.5);

    // Second kill completes task
    assert_eq!(manager.on_kill("runner", true), 1);
    assert!(manager.board().active()[0].is_done());
    assert_eq!(manager.board().active()[0].progress_ratio(), 1.0);

    // Claim reward
    let start_rep = manager.reputation();
    let reward = manager.claim_task("kill_test").unwrap();
    assert_eq!(reward.dollars, 50);
    assert_eq!(reward.reputation, 25);
    assert_eq!(reward.item, Some(("bandage".into(), 2)));
    assert_eq!(manager.reputation(), start_rep + 25);
    assert_eq!(manager.board().active().len(), 0);
    assert!(manager.board().completed_ids().contains(&"kill_test".to_string()));
}

#[test]
fn guild_fetch_task_progress_and_complete() {
    let mut board = GuildBoard::new(5, 3);
    let task = GuildTask::new(
        "fetch_test",
        "Bring wood",
        GuildTaskKind::Fetch { item_id: "wood".into() },
        10,
        GuildReward {
            dollars: 20,
            reputation: 15,
            item: None,
        },
    );

    board.set_offered(vec![task]);
    board.accept("fetch_test").unwrap();

    // Delivering other item does nothing
    assert_eq!(board.on_item_delivered("stone", 5), 0);

    // Delivering 4 wood
    assert_eq!(board.on_item_delivered("wood", 4), 1);
    assert_eq!(board.active()[0].progress, 4);

    // Delivering 10 more wood caps at target (10)
    assert_eq!(board.on_item_delivered("wood", 10), 1);
    assert_eq!(board.active()[0].progress, 10);
    assert!(board.active()[0].is_done());

    let reward = board.claim("fetch_test").unwrap();
    assert_eq!(reward.dollars, 20);
    assert_eq!(reward.reputation, 15);
}

#[test]
fn guild_explore_task_progress_and_complete() {
    let mut board = GuildBoard::new(5, 3);
    let task = GuildTask::new(
        "exp_test",
        "Scout Ashford Core",
        GuildTaskKind::Explore { place_name: "Ashford core".into() },
        1,
        GuildReward {
            dollars: 40,
            reputation: 30,
            item: None,
        },
    );

    board.set_offered(vec![task]);
    board.accept("exp_test").unwrap();

    // Non-matching place
    assert_eq!(board.on_place_reached("the lake"), 0);

    // Matching place (case insensitive)
    assert_eq!(board.on_place_reached("ashford CORE"), 1);
    assert!(board.active()[0].is_done());

    let reward = board.claim("exp_test").unwrap();
    assert_eq!(reward.dollars, 40);
    assert_eq!(reward.reputation, 30);
}

#[test]
fn guild_clear_task_progress_and_complete() {
    let mut board = GuildBoard::new(5, 3);
    let task = GuildTask::new(
        "clear_test",
        "Clear Gas Station",
        GuildTaskKind::Clear { poi_id: "gas_station".into() },
        1,
        GuildReward {
            dollars: 50,
            reputation: 35,
            item: Some(("pistol_ammo".into(), 16)),
        },
    );

    board.set_offered(vec![task]);
    board.accept("clear_test").unwrap();

    // Non-matching building
    assert_eq!(board.on_poi_cleared("house_small"), 0);

    // Matching building
    assert_eq!(board.on_poi_cleared("gas_station"), 1);
    assert!(board.active()[0].is_done());

    let reward = board.claim("clear_test").unwrap();
    assert_eq!(reward.dollars, 50);
    assert_eq!(reward.item, Some(("pistol_ammo".into(), 16)));
}

#[test]
fn guild_rank_thresholds() {
    let mut manager = GuildManager::from_repo(1).unwrap();

    assert_eq!(manager.current_rank().id, RANK_ROOKIE);
    assert_eq!(manager.max_recruits(), 1);
    assert_eq!(manager.shop_tier(), 1);
    assert_eq!(manager.rank_progress(), (0, Some(100)));

    // Progress to Veteran at 100 rep
    manager.add_reputation(100);
    assert_eq!(manager.current_rank().id, RANK_VETERAN);
    assert_eq!(manager.max_recruits(), 2);
    assert_eq!(manager.shop_tier(), 2);
    assert_eq!(manager.rank_progress(), (100, Some(250)));

    // Progress to Elite at 250 rep
    manager.add_reputation(150);
    assert_eq!(manager.current_rank().id, RANK_ELITE);
    assert_eq!(manager.max_recruits(), 3);
    assert_eq!(manager.shop_tier(), 3);
    assert_eq!(manager.rank_progress(), (250, Some(500)));

    // Progress to Master at 500 rep
    manager.add_reputation(250);
    assert_eq!(manager.current_rank().id, RANK_MASTER);
    assert_eq!(manager.max_recruits(), 4);
    assert_eq!(manager.shop_tier(), 4);
    assert_eq!(manager.rank_progress(), (500, Some(1000)));

    // Progress to Legend at 1000 rep
    manager.add_reputation(500);
    assert_eq!(manager.current_rank().id, RANK_LEGEND);
    assert_eq!(manager.max_recruits(), 5);
    assert_eq!(manager.shop_tier(), 5);
    assert_eq!(manager.rank_progress(), (1000, None));
}

#[test]
fn guild_recruit_hire_wage_leave() {
    let mut manager = GuildManager::from_repo(1).unwrap();
    let mut wallet = 100;

    // At rookie rank, max recruits is 1
    assert_eq!(manager.max_recruits(), 1);
    assert!(!manager.recruits().available().is_empty());

    let recruit_id = manager.recruits().available()[0].id.clone();
    let hire_cost = manager.recruits().available()[0].hire_cost;

    // Hire first recruit
    assert!(manager.hire_recruit(&recruit_id, &mut wallet).is_ok());
    assert_eq!(wallet, 100 - hire_cost as i32);
    assert_eq!(manager.recruits().hired().len(), 1);
    assert_eq!(manager.recruits().hired()[0].state, RecruitState::IdleAtCamp);

    // Trying to hire a second recruit exceeds slot limit (1 at Rookie)
    let second_id = manager.recruits().available()[0].id.clone();
    assert_eq!(
        manager.hire_recruit(&second_id, &mut wallet),
        Err(RecruitHireError::SlotsFull(1))
    );

    // Set order to follow and guard
    assert!(manager.set_recruit_order(&recruit_id, RecruitOrder::Follow).is_ok());
    assert_eq!(manager.recruits().hired()[0].state, RecruitState::FollowingPlayer);

    assert!(manager.set_recruit_order(&recruit_id, RecruitOrder::Guard).is_ok());
    assert_eq!(manager.recruits().hired()[0].state, RecruitState::GuardingCamp);

    assert!(manager.set_recruit_order(&recruit_id, RecruitOrder::Stay).is_ok());
    assert_eq!(manager.recruits().hired()[0].state, RecruitState::IdleAtCamp);

    // Damage recruit
    manager.recruits_mut().hired[0].take_damage(20);
    assert_eq!(manager.recruits().hired()[0].health, manager.recruits().hired()[0].max_health - 20);
    manager.recruits_mut().hired[0].heal(10);
    assert_eq!(manager.recruits().hired()[0].health, manager.recruits().hired()[0].max_health - 10);

    // Pay wages when funds sufficient
    let wage = manager.recruits().hired()[0].daily_wage;
    let pre_wage_wallet = wallet;
    let wage_report = manager.pay_recruit_wages(&mut wallet);
    assert_eq!(wage_report.total_wages_paid, wage);
    assert_eq!(wage_report.paid_recruits, vec![recruit_id.clone()]);
    assert!(wage_report.departed_recruits.is_empty());
    assert_eq!(wallet, pre_wage_wallet - wage as i32);
    assert_eq!(manager.recruits().hired().len(), 1);

    // Pay wages when funds insufficient: recruit leaves
    wallet = 0; // Empty funds
    let unpaid_report = manager.pay_recruit_wages(&mut wallet);
    assert_eq!(unpaid_report.total_wages_paid, 0);
    assert!(unpaid_report.paid_recruits.is_empty());
    assert_eq!(unpaid_report.departed_recruits, vec![recruit_id]);
    assert_eq!(manager.recruits().hired().len(), 0); // Teammate left!
}

#[test]
fn guild_recruit_death_and_dismiss() {
    let mut manager = GuildManager::from_repo(1).unwrap();
    let mut wallet = 500;

    let recruit_id = manager.recruits().available()[0].id.clone();
    manager.hire_recruit(&recruit_id, &mut wallet).unwrap();

    // Kill recruit
    manager.recruits_mut().hired[0].take_damage(999);
    assert!(!manager.recruits().hired()[0].is_alive());
    assert_eq!(manager.recruits().hired()[0].state, RecruitState::Dead);

    // Dead recruit cannot receive orders
    assert_eq!(
        manager.set_recruit_order(&recruit_id, RecruitOrder::Follow),
        Err(RecruitOrderError::RecruitDead)
    );

    // Dismiss recruit
    assert!(manager.dismiss_recruit(&recruit_id).is_ok());
    assert_eq!(manager.recruits().hired().len(), 0);
}

#[test]
fn guild_shop_stock_by_rank() {
    let mut manager = GuildManager::from_repo(1).unwrap();
    let mut wallet = 500;

    // At rookie rank (tier 1)
    assert_eq!(manager.shop_tier(), 1);
    let tier1_items = manager.shop().available_items(1);
    assert!(tier1_items.iter().all(|i| i.tier <= 1));

    // Bandage is tier 1
    assert!(manager.buy_shop_item("bandage", 2, &mut wallet).is_ok());
    assert_eq!(wallet, 500 - 12 * 2);

    // Medkit is tier 2: locked for rookie
    assert!(matches!(
        manager.buy_shop_item("medkit", 1, &mut wallet),
        Err(ShopBuyError::TierLocked { required_tier: 2, player_tier: 1 })
    ));

    // Advance rank to Veteran (tier 2)
    manager.add_reputation(100);
    assert_eq!(manager.shop_tier(), 2);

    // Now medkit can be purchased
    assert!(manager.buy_shop_item("medkit", 1, &mut wallet).is_ok());

    // Out of stock check
    assert!(matches!(
        manager.buy_shop_item("medkit", 999, &mut wallet),
        Err(ShopBuyError::OutOfStock { .. })
    ));

    // Insufficient funds check
    wallet = 1;
    assert!(matches!(
        manager.buy_shop_item("bandage", 1, &mut wallet),
        Err(ShopBuyError::InsufficientFunds { .. })
    ));

    // Restock
    manager.shop_mut().restock();
    let bandage = manager.shop().items().iter().find(|i| i.item_id == "bandage").unwrap();
    assert_eq!(bandage.stock, bandage.max_stock);
}

#[test]
fn guild_morning_refresh() {
    let mut manager = GuildManager::from_repo(555).unwrap();
    let mut wallet = 200;

    // Initial state before any refresh
    assert_eq!(manager.board().offered().len(), 0);
    assert_eq!(manager.last_refresh_day(), -1);

    // Hire a recruit
    let rec_id = manager.recruits().available()[0].id.clone();
    manager.hire_recruit(&rec_id, &mut wallet).unwrap();

    // Morning refresh on day 1 at 6:00
    let report = manager.refresh_morning(1, 6.0, Some(&mut wallet));
    assert!(report.is_some());
    let rep = report.unwrap();
    assert_eq!(rep.day, 1);
    assert_eq!(rep.rolled_task_count, 5);
    assert_eq!(manager.board().offered().len(), 5);
    assert_eq!(manager.last_refresh_day(), 1);

    // Calling again at hour 7 on day 1 does nothing
    assert!(manager.refresh_morning(1, 7.0, Some(&mut wallet)).is_none());

    // Next day at hour 6:00 refreshes board again
    let next_report = manager.refresh_morning(2, 6.0, Some(&mut wallet));
    assert!(next_report.is_some());
    assert_eq!(manager.last_refresh_day(), 2);
}

#[test]
fn guild_save_round_trip() {
    let mut manager = GuildManager::from_repo(777).unwrap();
    let mut wallet = 300;

    manager.add_reputation(150);
    manager.refresh_morning(3, 6.0, Some(&mut wallet));
    assert_eq!(manager.board().offered().len(), 5);

    let first_task_id = manager.board().offered()[0].id.clone();
    manager.accept_task(&first_task_id).unwrap();

    let rec_id = manager.recruits().available()[0].id.clone();
    manager.hire_recruit(&rec_id, &mut wallet).unwrap();

    // Serialize to GuildState
    let state = manager.to_state();
    assert_eq!(state.reputation, 150);
    assert_eq!(state.last_refresh_day, 3);
    assert_eq!(state.active_tasks.len(), 1);
    assert_eq!(state.hired_recruits.len(), 1);

    // JSON round-trip
    let json = serde_json::to_string(&state).unwrap();
    let loaded_state: GuildState = serde_json::from_str(&json).unwrap();
    assert_eq!(state, loaded_state);

    // Default JSON deserialization from "{}"
    let empty_state: GuildState = serde_json::from_str("{}").unwrap();
    assert_eq!(empty_state, GuildState::default());

    // Restore into a fresh manager
    let mut new_manager = GuildManager::from_repo(777).unwrap();
    new_manager.from_state(loaded_state);
    assert_eq!(new_manager.reputation(), 150);
    assert_eq!(new_manager.last_refresh_day(), 3);
    assert_eq!(new_manager.board().active().len(), 1);
    assert_eq!(new_manager.recruits().hired().len(), 1);
}

#[test]
fn guild_display_and_error_formatting() {
    let accept_err1 = GuildAcceptError::NotFound;
    let accept_err2 = GuildAcceptError::LimitReached(3);
    assert!(!accept_err1.to_string().is_empty());
    assert!(!accept_err2.to_string().is_empty());

    let claim_err1 = GuildClaimError::NotFound;
    let claim_err2 = GuildClaimError::NotFinished;
    assert!(!claim_err1.to_string().is_empty());
    assert!(!claim_err2.to_string().is_empty());

    let hire_err1 = RecruitHireError::NotFound;
    let hire_err2 = RecruitHireError::AlreadyHired;
    let hire_err3 = RecruitHireError::SlotsFull(2);
    let hire_err4 = RecruitHireError::InsufficientFunds { needed: 50, available: 10 };
    assert!(!hire_err1.to_string().is_empty());
    assert!(!hire_err2.to_string().is_empty());
    assert!(!hire_err3.to_string().is_empty());
    assert!(!hire_err4.to_string().is_empty());

    let dismiss_err = RecruitDismissError::NotFound;
    assert!(!dismiss_err.to_string().is_empty());

    let order_err1 = RecruitOrderError::NotFound;
    let order_err2 = RecruitOrderError::RecruitDead;
    assert!(!order_err1.to_string().is_empty());
    assert!(!order_err2.to_string().is_empty());

    let shop_err1 = ShopBuyError::ItemNotFound;
    let shop_err2 = ShopBuyError::TierLocked { required_tier: 3, player_tier: 1 };
    let shop_err3 = ShopBuyError::OutOfStock { requested: 5, available: 2 };
    let shop_err4 = ShopBuyError::InsufficientFunds { needed: 100, available: 20 };
    assert!(!shop_err1.to_string().is_empty());
    assert!(!shop_err2.to_string().is_empty());
    assert!(!shop_err3.to_string().is_empty());
    assert!(!shop_err4.to_string().is_empty());

    let load_err = GuildLoadError::NotFound("fake.json".into());
    assert!(!load_err.to_string().is_empty());
}
