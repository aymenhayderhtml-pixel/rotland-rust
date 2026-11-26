use game::camp_upgrades::{
    cancel, can_start, damage_walls, repair_walls, start, tick, walls_breached,
    CampEffects, CampUpgradeState, CampUpgradesConfig, CantStart, CommonLevelDef,
    GuardLevelDef, MaterialCost, TraderLevelDef, UpgradeCost, UpgradeInProgress,
    UpgradeLine, UpgradeRequirement, WallLevelDef, WorkshopLevelDef,
};

#[test]
fn camp_upgrades_config_loads_and_validates() {
    let cfg = CampUpgradesConfig::load_default().expect("failed to load data/camp_upgrades.json");
    assert!(cfg.validate().is_ok());

    assert_eq!(cfg.max_level(UpgradeLine::Walls), 3);
    assert_eq!(cfg.max_level(UpgradeLine::Guards), 3);
    assert_eq!(cfg.max_level(UpgradeLine::Workshop), 3);
    assert_eq!(cfg.max_level(UpgradeLine::Traders), 4);
    assert_eq!(cfg.max_level(UpgradeLine::Gunsmith), 3);

    let wall_def: &WallLevelDef = &cfg.walls[0];
    assert_eq!(wall_def.level, 0);

    let guard_def: &GuardLevelDef = &cfg.guards[0];
    assert_eq!(guard_def.level, 0);

    let ws_def: &WorkshopLevelDef = &cfg.workshop[0];
    assert_eq!(ws_def.level, 0);

    let tr_def: &TraderLevelDef = &cfg.traders[0];
    assert_eq!(tr_def.level, 0);

    assert_eq!(cfg.gunsmith.len(), 4);
    assert_eq!(cfg.gunsmith[0].level, 0);
    assert_eq!(cfg.gunsmith[0].gunsmith_tier, 0);
    assert_eq!(cfg.gunsmith[3].gunsmith_tier, 3);

    // Test load_from_file and from_json explicitly
    let path = paths::repo_path("data/camp_upgrades.json");
    let cfg2 = CampUpgradesConfig::load_from_file(&path).expect("load_from_file should succeed");
    assert_eq!(cfg.walls.len(), cfg2.walls.len());

    let json_text = std::fs::read_to_string(&path).expect("read camp_upgrades.json");
    let cfg3 = CampUpgradesConfig::from_json(&json_text).expect("from_json should succeed");
    assert_eq!(cfg.guards.len(), cfg3.guards.len());

    for line in UpgradeLine::ALL {
        assert!(!line.as_str().is_empty());
        let _ = format!("{line}");
    }
}

#[test]
fn camp_upgrades_requirements_chain() {
    let cfg = CampUpgradesConfig::load_default().expect("load config");
    let mut state = CampUpgradeState::default();

    // Traders level 3 requires workshop level 1
    // First, level up traders to level 2
    state.set_level(UpgradeLine::Traders, 2);
    assert_eq!(state.level(UpgradeLine::Traders), 2);
    assert_eq!(state.level(UpgradeLine::Workshop), 0);

    // Try to start traders level 3 without workshop level 1
    let result = can_start(&state, &cfg, UpgradeLine::Traders, 1000, |_id, _cnt| true);
    match result {
        Err(CantStart::MissingRequirement { line, required_level }) => {
            assert_eq!(line, UpgradeLine::Workshop);
            assert_eq!(required_level, 1);
            let msg = format!("{}", CantStart::MissingRequirement { line, required_level });
            assert!(msg.contains("workshop"));
        }
        other => panic!("expected MissingRequirement, got {other:?}"),
    }

    // Fulfill workshop requirement
    state.set_level(UpgradeLine::Workshop, 1);
    let result2 = state.can_start(&cfg, UpgradeLine::Traders, 1000, |_id, _cnt| true);
    assert!(result2.is_ok());

    // Check level_requirement helper
    let req = cfg.level_requirement(UpgradeLine::Traders, 3);
    assert!(req.is_some());
    let req_val: &UpgradeRequirement = req.unwrap();
    assert_eq!(req_val.line, UpgradeLine::Workshop);
    assert_eq!(req_val.level, 1);
}

#[test]
fn camp_upgrades_costs_reported() {
    let cfg = CampUpgradesConfig::load_default().expect("load config");
    let state = CampUpgradeState::default();

    // Not enough cash
    let res_no_cash = can_start(&state, &cfg, UpgradeLine::Walls, 10, |_id, _cnt| true);
    match res_no_cash {
        Err(CantStart::NotEnoughCash { required, actual }) => {
            assert_eq!(required, 50);
            assert_eq!(actual, 10);
            let msg = format!("{}", CantStart::NotEnoughCash { required, actual });
            assert!(msg.contains("$50"));
        }
        other => panic!("expected NotEnoughCash, got {other:?}"),
    }

    // Missing item
    let res_no_item = can_start(&state, &cfg, UpgradeLine::Walls, 100, |id, _cnt| id != "wood");
    match res_no_item {
        Err(CantStart::MissingItem { item_id, required }) => {
            assert_eq!(item_id, "wood");
            assert_eq!(required, 20);
            let msg = format!("{}", CantStart::MissingItem { item_id, required });
            assert!(msg.contains("wood"));
        }
        other => panic!("expected MissingItem, got {other:?}"),
    }

    // Success reports cost
    let cost = can_start(&state, &cfg, UpgradeLine::Walls, 100, |_id, _cnt| true)
        .expect("should can_start");
    assert_eq!(cost.cash, 50);
    assert_eq!(cost.build_hours, 2.0);
    assert_eq!(cost.materials, vec![MaterialCost::new("wood", 20)]);

    // Check level_cost and level_info helper
    let c = cfg.level_cost(UpgradeLine::Walls, 1).expect("level 1 cost");
    assert_eq!(c.cash, 50);
    let info: CommonLevelDef<'_> = cfg.level_info(UpgradeLine::Walls, 1).expect("info");
    assert_eq!(info.cost.cash, 50);
}

#[test]
fn camp_upgrades_cant_start_two_at_once() {
    let cfg = CampUpgradesConfig::load_default().expect("load config");
    let mut state = CampUpgradeState::default();

    let cost: UpgradeCost = start(&mut state, &cfg, UpgradeLine::Walls);
    assert_eq!(cost.cash, 50);
    assert!(state.in_progress().is_some());

    let res = can_start(&state, &cfg, UpgradeLine::Guards, 500, |_id, _cnt| true);
    assert_eq!(res, Err(CantStart::Busy));
    let msg = format!("{}", CantStart::Busy);
    assert!(msg.contains("already in progress"));
}

#[test]
fn camp_upgrades_tick_finishes_after_build_hours() {
    let cfg = CampUpgradesConfig::load_default().expect("load config");
    let mut state = CampUpgradeState::default();

    state.start(&cfg, UpgradeLine::Workshop);
    assert_eq!(state.workshop_level, 0);

    // Workshop level 1 requires 1.5 build hours
    let finished1 = state.tick(&cfg, 1.0);
    assert_eq!(finished1, None);
    assert_eq!(state.workshop_level, 0);

    let finished2 = state.tick(&cfg, 0.5);
    assert_eq!(finished2, Some((UpgradeLine::Workshop, 1)));
    assert_eq!(state.workshop_level, 1);
    assert!(state.in_progress.is_none());

    // Ticking when idle returns None
    assert_eq!(tick(&mut state, &cfg, 1.0), None);
}

#[test]
fn camp_upgrades_cancel_refunds_half() {
    let cfg = CampUpgradesConfig::load_default().expect("load config");
    let mut state = CampUpgradeState::default();

    // Walls level 1 costs 50 cash and 20 wood
    let _ = start(&mut state, &cfg, UpgradeLine::Walls);
    let refund = cancel(&mut state, &cfg).expect("refund should be present");

    assert_eq!(refund.cash, 25);
    assert_eq!(refund.materials, vec![MaterialCost::new("wood", 10)]);
    assert!(state.in_progress.is_none());

    // Cancel again returns None
    assert_eq!(state.cancel(&cfg), None);
}

#[test]
fn camp_upgrades_effects_grow_with_level() {
    let cfg = CampUpgradesConfig::load_default().expect("load config");
    let mut state = CampUpgradeState::default();

    let eff0: CampEffects = cfg.effects(&state);
    assert_eq!(eff0.wall_hp_max, 0.0);
    assert_eq!(eff0.safe_zone_half_bonus, 0.0);
    assert_eq!(eff0.gate_count, 0);
    assert_eq!(eff0.guard_count, 2);
    assert_eq!(eff0.guard_damage, 60.0);
    assert_eq!(eff0.guard_range, 40.0);
    assert_eq!(eff0.guard_fire_interval, 1.2);
    assert_eq!(eff0.tower_count, 0);
    assert_eq!(eff0.recipe_tier, 0);
    assert_eq!(eff0.trader_slots_bonus, 0);
    assert!(!eff0.has_doctor);
    assert!(!eff0.has_mechanic);
    assert_eq!(eff0.trader_stock_tier, 0);
    assert_eq!(eff0.gunsmith_tier, 0);

    // Max all levels
    state.set_level(UpgradeLine::Walls, 3);
    state.set_level(UpgradeLine::Guards, 3);
    state.set_level(UpgradeLine::Workshop, 3);
    state.set_level(UpgradeLine::Traders, 4);
    state.set_level(UpgradeLine::Gunsmith, 3);

    let eff_max = state.effects(&cfg);
    assert_eq!(eff_max.wall_hp_max, 2500.0);
    assert_eq!(eff_max.safe_zone_half_bonus, 7.5);
    assert_eq!(eff_max.gate_count, 2);
    assert_eq!(eff_max.guard_count, 4);
    assert_eq!(eff_max.guard_damage, 100.0);
    assert_eq!(eff_max.guard_range, 75.0);
    assert_eq!(eff_max.guard_fire_interval, 0.8);
    assert_eq!(eff_max.tower_count, 2);
    assert_eq!(eff_max.recipe_tier, 3);
    assert_eq!(eff_max.trader_slots_bonus, 8);
    assert!(eff_max.has_doctor);
    assert!(eff_max.has_mechanic);
    assert_eq!(eff_max.trader_stock_tier, 1);
    assert_eq!(eff_max.gunsmith_tier, 3);

    // Verify CantStart::MaxLevel when at max level
    let err = can_start(&state, &cfg, UpgradeLine::Walls, 10000, |_id, _cnt| true);
    assert_eq!(err, Err(CantStart::MaxLevel));
    let msg = format!("{}", CantStart::MaxLevel);
    assert!(msg.contains("maximum"));
}

#[test]
fn camp_upgrades_save_round_trip() {
    let state = CampUpgradeState {
        walls_level: 2,
        guards_level: 1,
        workshop_level: 3,
        traders_level: 2,
        gunsmith_level: 2,
        in_progress: Some(UpgradeInProgress {
            line: UpgradeLine::Guards,
            target_level: 2,
            hours_left: 3.5,
        }),
        wall_hp: 1100.0,
    };

    let json = serde_json::to_string(&state).expect("serialize");
    let loaded: CampUpgradeState = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(state, loaded);
}

#[test]
fn camp_upgrades_empty_json_loads_as_level_0() {
    let loaded: CampUpgradeState = serde_json::from_str("{}").expect("deserialize empty json");
    assert_eq!(loaded, CampUpgradeState::default());
    assert_eq!(loaded.walls_level, 0);
    assert_eq!(loaded.guards_level, 0);
    assert_eq!(loaded.workshop_level, 0);
    assert_eq!(loaded.traders_level, 0);
    assert_eq!(loaded.gunsmith_level, 0);
    assert!(loaded.in_progress.is_none());
    assert_eq!(loaded.wall_hp, 0.0);
    assert!(!loaded.has_walls());
    assert_eq!(loaded.wall_hp(), 0.0);
}

#[test]
fn camp_upgrades_wall_damage_repair_breached() {
    let cfg = CampUpgradesConfig::load_default().expect("load config");
    let mut state = CampUpgradeState::default();

    // At level 0, walls_breached is false (no walls)
    assert!(!walls_breached(&state));
    assert!(!state.walls_breached());

    // Build level 1 walls (500 HP)
    start(&mut state, &cfg, UpgradeLine::Walls);
    tick(&mut state, &cfg, 2.0);
    assert_eq!(state.walls_level, 1);
    assert_eq!(state.wall_hp, 500.0);
    assert!(state.has_walls());
    assert!(!walls_breached(&state));

    // Damage walls by 200
    damage_walls(&mut state, 200.0);
    assert_eq!(state.wall_hp, 300.0);
    assert!(!walls_breached(&state));

    // Damage remaining 300 -> breached!
    state.damage_walls(300.0);
    assert_eq!(state.wall_hp, 0.0);
    assert!(walls_breached(&state));

    // Repair with 4 wood (25 HP each = 100 HP)
    let restored = repair_walls(&mut state, &cfg, &[("wood", 4)]);
    assert_eq!(restored, 100.0);
    assert_eq!(state.wall_hp, 100.0);
    assert!(!walls_breached(&state));

    // Repair with 5 scrap (50 HP each = 250 HP)
    let restored2 = state.repair_walls(&cfg, &[("scrap_metal", 5)]);
    assert_eq!(restored2, 250.0);
    assert_eq!(state.wall_hp, 350.0);

    // Repair more than max HP -> clamps to max HP
    let restored3 = state.repair_walls(&cfg, &[("scrap_metal", 10)]);
    assert_eq!(restored3, 150.0); // only 150 missing (500 - 350)
    assert_eq!(state.wall_hp, 500.0);
}

#[test]
fn camp_upgrades_gunsmith_tiers_0_to_3() {
    let cfg = CampUpgradesConfig::load_default().expect("load config");
    let mut state = CampUpgradeState::default();
    assert_eq!(state.effects(&cfg).gunsmith_tier, 0);

    // Tier 0 -> 1
    let c1 = can_start(&state, &cfg, UpgradeLine::Gunsmith, 500, |_id, _cnt| true).expect("tier 1");
    assert_eq!(c1.cash, 100);
    start(&mut state, &cfg, UpgradeLine::Gunsmith);
    let done1 = tick(&mut state, &cfg, 2.0);
    assert_eq!(done1, Some((UpgradeLine::Gunsmith, 1)));
    assert_eq!(state.effects(&cfg).gunsmith_tier, 1);

    // Tier 2 requires workshop level 1
    let blocked = can_start(&state, &cfg, UpgradeLine::Gunsmith, 500, |_id, _cnt| true);
    assert_eq!(
        blocked,
        Err(CantStart::MissingRequirement {
            line: UpgradeLine::Workshop,
            required_level: 1,
        })
    );
    state.set_level(UpgradeLine::Workshop, 1);
    assert!(can_start(&state, &cfg, UpgradeLine::Gunsmith, 500, |_id, _cnt| true).is_ok());
    start(&mut state, &cfg, UpgradeLine::Gunsmith);
    tick(&mut state, &cfg, 4.0);
    assert_eq!(state.effects(&cfg).gunsmith_tier, 2);

    // Tier 3
    start(&mut state, &cfg, UpgradeLine::Gunsmith);
    tick(&mut state, &cfg, 6.0);
    assert_eq!(state.effects(&cfg).gunsmith_tier, 3);
    assert_eq!(
        can_start(&state, &cfg, UpgradeLine::Gunsmith, 1000, |_id, _cnt| true),
        Err(CantStart::MaxLevel)
    );
}
