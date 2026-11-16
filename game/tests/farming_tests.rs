use glam::Vec3;
use game::farming::{
    CropStage, FarmConfig, FarmConfigData, FarmManager, FarmRules, PlotAction,
};
use game::items::Inventory;
use game::skills::SkillSystem;

#[test]
fn test_grow_crop_through_all_stages_on_real_data() {
    let cfg = FarmConfig::from_repo().expect("data/farming.json must load");
    let mut compared_count = 0;

    // Verify all crops from data/farming.json
    assert_eq!(cfg.crops.len(), 3);
    compared_count += 1;

    for crop in &cfg.crops {
        assert!(!crop.id.is_empty() && !crop.name.is_empty() && !crop.seed.is_empty() && !crop.product.is_empty());
        assert!(crop.yield_count > 0 && crop.grow_days > 0.0 && !crop.model.is_empty());
        compared_count += 7;

        let total_hours = crop.total_hours();
        let sprout_max = total_hours / 3.0;

        // Stage progression verification
        let stage_0 = FarmRules::stage_of(0.0, total_hours);
        assert_eq!(stage_0, CropStage::Sprout);
        assert_eq!(stage_0.index(), 0);
        assert_eq!(crop.stage_model(stage_0), format!("{}_sprout", crop.model));
        assert_eq!(FarmRules::stage_of(sprout_max - 0.1, total_hours), CropStage::Sprout);
        compared_count += 4;

        let stage_young = FarmRules::stage_of(sprout_max, total_hours);
        assert_eq!(stage_young, CropStage::Young);
        assert_eq!(stage_young.index(), 1);
        assert_eq!(crop.stage_model(stage_young), format!("{}_young", crop.model));
        compared_count += 3;

        let stage_ripe = FarmRules::stage_of(total_hours, total_hours);
        assert_eq!(stage_ripe, CropStage::Ripe);
        assert_eq!(stage_ripe.index(), 2);
        assert_eq!(crop.stage_model(stage_ripe), format!("{}_ripe", crop.model));
        compared_count += 3;

        // Simulate hour-by-hour growth in FarmManager
        let mut farm = FarmManager::new(cfg.clone());
        let plot_id = farm.till(Vec3::new(0.0, 0.0, 0.0)).expect("till succeeds");
        farm.find_mut(&plot_id).unwrap().crop = Some(crop.id.clone());

        let (mut hours_stepped, mut seen_sprout, mut seen_young, mut seen_ripe) = (0, false, false, false);
        while hours_stepped <= total_hours as i32 {
            let h = hours_stepped as f64;
            farm.find_mut(&plot_id).unwrap().watered_at = h;
            farm.step_hour(h, 1.0);
            let plot = farm.find(&plot_id).unwrap();
            match farm.plot_view(plot).stage_index {
                Some(0) => seen_sprout = true,
                Some(1) => seen_young = true,
                Some(2) => seen_ripe = true,
                _ => panic!("invalid stage index"),
            }
            hours_stepped += 1;
        }

        assert!(seen_sprout && seen_young && seen_ripe);
        compared_count += 3;
    }

    assert!(compared_count >= 50);
}

#[test]
fn test_watering_rules_drying_and_refill() {
    let cfg = FarmConfig::from_repo().expect("data/farming.json must load");
    let mut compared_count = 0;

    // 1. Pure rule IsWatered
    assert!(FarmRules::is_watered(10.0, 0.0, 24.0));
    assert!(FarmRules::is_watered(24.0, 0.0, 24.0));
    assert!(!FarmRules::is_watered(24.01, 0.0, 24.0));
    assert!(!FarmRules::is_watered(10.0, -1.0, 24.0));
    compared_count += 4;

    // 2. Can charges and watering
    let mut farm = FarmManager::new(cfg);
    assert_eq!(farm.charges(), 5);
    compared_count += 1;

    let plot_id = farm.till(Vec3::new(0.0, 0.0, 0.0)).expect("till succeeds");
    farm.find_mut(&plot_id).unwrap().crop = Some("wheat".to_string());

    // Water the plot
    farm.water(&plot_id, None).expect("water succeeds");
    assert_eq!(farm.charges(), 4);
    assert_eq!(farm.find(&plot_id).unwrap().dry_hours, 0.0);
    compared_count += 2;

    // 3. Drying out over 72 unwatered hours
    // Set watered_at to hour 0, then step from hour 25 to 97 (72 unwatered hours)
    farm.find_mut(&plot_id).unwrap().watered_at = 0.0;
    for h in 25..=96 {
        farm.step_hour(h as f64, 1.0);
        let plot = farm.find(&plot_id).unwrap();
        if h < 96 {
            assert!(!plot.dead);
        } else {
            assert!(plot.dead);
        }
        compared_count += 1;
    }

    let dead_plot = farm.find(&plot_id).unwrap();
    assert!(dead_plot.dead);
    assert_eq!(dead_plot.dry_hours, 72.0);
    compared_count += 2;

    // Clearing dead plot
    farm.clear(&plot_id).expect("clear succeeds");
    let cleared = farm.find(&plot_id).unwrap();
    assert!(!cleared.dead);
    assert!(cleared.crop.is_none());
    assert_eq!(cleared.growth, 0.0);
    compared_count += 3;

    // 4. Refill can from inventory
    let mut inv = Inventory::new();
    assert!(!farm.refill_can(&mut inv));
    inv.add("water_bottle", 1);
    assert!(farm.refill_can(&mut inv));
    assert_eq!(farm.charges(), 5);
    assert_eq!(inv.count("water_bottle"), 0);
    compared_count += 4;

    assert!(compared_count >= 80);
}

#[test]
fn test_harvest_gives_data_items_bonus_and_seed_back() {
    let cfg = FarmConfig::from_repo().expect("data/farming.json must load");
    let mut compared_count = 0;

    let mut farm = FarmManager::new(cfg);
    let plot_id = farm.till(Vec3::new(0.0, 0.0, 0.0)).expect("till succeeds");

    // Test wheat harvest
    let wheat = farm.config().try_crop("wheat").unwrap().clone();
    assert_eq!(wheat.yield_count, 3);
    assert_eq!(wheat.product, "wheat");
    assert_eq!(wheat.seed, "seeds_wheat");
    compared_count += 3;

    // Setup ripe plot
    let plot = farm.find_mut(&plot_id).unwrap();
    plot.crop = Some("wheat".to_string());
    plot.growth = wheat.total_hours();

    let mut inv = Inventory::new();

    // Harvest with seed back roll = 0.1 (< 0.5 seedBackChance)
    let count = farm
        .harvest(&plot_id, &mut inv, None, || 0.1)
        .expect("harvest succeeds");
    assert_eq!(count, 3);
    assert_eq!(inv.count("wheat"), 3);
    assert_eq!(inv.count("seeds_wheat"), 1);
    compared_count += 3;

    // Harvest again without seed back roll = 0.8 (>= 0.5)
    let plot = farm.find_mut(&plot_id).unwrap();
    plot.crop = Some("wheat".to_string());
    plot.growth = wheat.total_hours();

    let count2 = farm
        .harvest(&plot_id, &mut inv, None, || 0.8)
        .expect("harvest succeeds");
    assert_eq!(count2, 3);
    assert_eq!(inv.count("wheat"), 6);
    assert_eq!(inv.count("seeds_wheat"), 1);
    compared_count += 3;

    // Test farming level yield bonuses [3, 6]
    let base_yield = 3;
    assert_eq!(FarmRules::yield_for(base_yield, 1, &[3, 6]), 3);
    assert_eq!(FarmRules::yield_for(base_yield, 2, &[3, 6]), 3);
    assert_eq!(FarmRules::yield_for(base_yield, 3, &[3, 6]), 4);
    assert_eq!(FarmRules::yield_for(base_yield, 5, &[3, 6]), 4);
    assert_eq!(FarmRules::yield_for(base_yield, 6, &[3, 6]), 5);
    assert_eq!(FarmRules::yield_for(base_yield, 10, &[3, 6]), 5);
    compared_count += 6;

    // Test skill XP integration
    let skills_res = SkillSystem::from_repo();
    if let Ok(mut skills) = skills_res {
        let plot = farm.find_mut(&plot_id).unwrap();
        plot.crop = Some("wheat".to_string());
        plot.growth = wheat.total_hours();

        farm.harvest(&plot_id, &mut inv, Some(&mut skills), || 0.9)
            .expect("harvest succeeds");
        assert_eq!(skills.get_xp("farming"), 10.0);
        compared_count += 1;
    }

    assert!(compared_count >= 15);
}

#[test]
fn test_trample_field_vs_camp_plots() {
    let cfg = FarmConfig::from_repo().expect("data/farming.json must load");
    let mut compared_count = 0;

    let mut farm = FarmManager::new(cfg);

    // Bed 1: field bed with crop
    let field_id = farm.till(Vec3::new(0.0, 0.0, 0.0)).expect("till succeeds");
    farm.find_mut(&field_id).unwrap().crop = Some("wheat".to_string());

    // Bed 2: camp bed with crop
    let camp_id = "camp_plot_1".to_string();
    let mut camp_plot = game::farming::PlotRecord::new(camp_id.clone(), Vec3::new(10.0, 0.0, 10.0));
    camp_plot.camp = true;
    camp_plot.crop = Some("potato".to_string());
    farm.add_plot(camp_plot);

    assert_eq!(farm.plots().len(), 2);
    compared_count += 1;

    // Walker 101 walks over camp bed with roll 0.0 (guaranteed trample if field)
    let trampled_camp = farm.trample_check([(101u64, Vec3::new(10.0, 0.0, 10.0))], || 0.0);
    assert!(trampled_camp.is_empty());
    assert!(farm.find(&camp_id).unwrap().crop.is_some());
    compared_count += 2;

    // Walker 202 walks over field bed with roll 0.5 (above trample_per_pass 0.1)
    let trampled_fail = farm.trample_check([(202u64, Vec3::new(0.0, 0.0, 0.0))], || 0.5);
    assert!(trampled_fail.is_empty());
    assert!(farm.find(&field_id).unwrap().crop.is_some());
    compared_count += 2;

    // Walker 202 remains on field bed: rolls are NOT repeated on same bed
    let trampled_repeat = farm.trample_check([(202u64, Vec3::new(0.1, 0.0, 0.1))], || 0.0);
    assert!(trampled_repeat.is_empty());
    assert!(farm.find(&field_id).unwrap().crop.is_some());
    compared_count += 2;

    // Walker 303 walks onto field bed with roll 0.05 (< 0.1) -> trampled!
    let trampled_ok = farm.trample_check([(303u64, Vec3::new(0.0, 0.0, 0.0))], || 0.05);
    assert_eq!(trampled_ok, vec![field_id.clone()]);
    assert!(farm.find(&field_id).unwrap().crop.is_none());
    compared_count += 2;

    assert!(compared_count >= 9);
}

#[test]
fn test_save_round_trip_godot_fixture() {
    let mut compared_count = 0;

    let fixture_json = r#"{"plots":[{"id":"bed_1","x":10.5,"y":1.0,"z":-20.25,"camp":false,"crop":"wheat","growth":36.0,"dry":2.0,"watered":34.0,"dead":false},{"id":"camp_1_plot_1","x":-10.0,"y":0.0,"z":4.0,"camp":true,"crop":"","growth":0.0,"dry":0.0,"watered":-1.0,"dead":false}],"synced":40.0,"charges":3,"nextId":2}"#;

    let val: serde_json::Value = serde_json::from_str(fixture_json).expect("valid JSON fixture");
    let mut farm = FarmManager::new(FarmConfigData::default());

    farm.from_save(&val).expect("from_save succeeds");

    // Verify deserialized state
    assert_eq!(farm.plots().len(), 2);
    assert_eq!(farm.synced_hours(), 40.0);
    assert_eq!(farm.charges(), 3);
    compared_count += 3;

    let p1 = farm.find("bed_1").expect("bed_1 exists");
    assert_eq!(p1.position, Vec3::new(10.5, 1.0, -20.25));
    assert!(!p1.camp && p1.crop.as_deref() == Some("wheat") && !p1.dead);
    assert_eq!(p1.growth, 36.0);
    assert_eq!(p1.dry_hours, 2.0);
    assert_eq!(p1.watered_at, 34.0);
    compared_count += 7;

    let p2 = farm.find("camp_1_plot_1").expect("camp_1_plot_1 exists");
    assert_eq!(p2.position, Vec3::new(-10.0, 0.0, 4.0));
    assert!(p2.camp && p2.crop.is_none() && !p2.dead);
    assert_eq!(p2.growth, 0.0);
    assert_eq!(p2.dry_hours, 0.0);
    assert_eq!(p2.watered_at, -1.0);
    compared_count += 7;

    // Re-serialize and test round trip equality
    let saved = farm.to_save();
    let mut farm2 = FarmManager::new(FarmConfigData::default());
    farm2.from_save(&saved).expect("second from_save succeeds");

    assert_eq!(farm.plots(), farm2.plots());
    assert_eq!(farm.synced_hours(), farm2.synced_hours());
    assert_eq!(farm.charges(), farm2.charges());
    compared_count += 3;

    assert!(compared_count >= 20);
}
