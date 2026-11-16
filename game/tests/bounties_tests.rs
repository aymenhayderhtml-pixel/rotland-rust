use game::bounties::{
    BountyBoard, BountyClaimError, BountyConfig, BountyDef, BountyManager, BountyRoller,
};

fn assert_bounty(
    b: &BountyDef,
    id: &str,
    b_type: &str,
    text: &str,
    target: i32,
    dollars: i32,
    xp: i32,
) -> usize {
    assert_eq!(b.id, id);
    assert_eq!(b.bounty_type, b_type);
    assert_eq!(b.text, text);
    assert_eq!(b.target, target);
    assert_eq!(b.dollars, dollars);
    assert_eq!(b.xp, xp);
    6
}

#[test]
fn bounties_rolls_3_fixed_seeds_stable_and_valid() {
    let cfg = BountyConfig::from_repo().expect("data/bounties.json must load");
    let mut compared_count = 0;

    let seeds = [42, 100, 777];
    for &seed in &seeds {
        let daily1 = BountyRoller::roll_daily(seed, 0, &cfg);
        let daily2 = BountyRoller::roll_daily(seed, 0, &cfg);
        let weekly1 = BountyRoller::roll_weekly(seed, 0, &cfg);
        let weekly2 = BountyRoller::roll_weekly(seed, 0, &cfg);

        // Deterministic stability check
        assert_eq!(daily1, daily2);
        assert_eq!(weekly1, weekly2);
        compared_count += 2;

        assert_eq!(daily1.len(), cfg.daily_count as usize);
        compared_count += 1;

        for bounty in &daily1 {
            assert!(bounty.target >= 1);
            assert!(bounty.dollars >= 0);
            assert!(bounty.xp >= 0);
            assert!(!bounty.text.is_empty());
            assert_eq!(bounty.expires_on, 0);
            assert!(!bounty.weekly);
            compared_count += 6;
        }

        assert!(weekly1.target >= 1);
        assert!(weekly1.dollars >= 0);
        assert!(weekly1.xp >= 0);
        assert!(!weekly1.text.is_empty());
        assert_eq!(weekly1.expires_on, (0 + 1) * cfg.weekly_days - 1);
        assert!(weekly1.weekly);
        compared_count += 6;
    }

    // Exact golden value checks for Seed 42
    let d42 = BountyRoller::roll_daily(42, 0, &cfg);
    let w42 = BountyRoller::roll_weekly(42, 0, &cfg);
    compared_count += assert_bounty(&d42[0], "d0_0", "deliver_items", "Bring 3 raw meat", 3, 11, 43);
    assert_eq!(d42[0].item.as_deref(), Some("raw_meat"));
    compared_count += assert_bounty(&d42[1], "d0_1", "hunt_animals", "Hunt 5 rabbits", 5, 9, 40);
    assert_eq!(d42[1].animal.as_deref(), Some("rabbit"));
    compared_count += assert_bounty(&d42[2], "d0_2", "deliver_items", "Bring 3 hides", 3, 14, 36);
    assert_eq!(d42[2].item.as_deref(), Some("hide"));
    compared_count += assert_bounty(&w42, "w0", "hunt_animals", "Hunt 2 boar", 2, 59, 180);
    assert_eq!(w42.animal.as_deref(), Some("boar"));
    compared_count += 4;

    // Exact golden value checks for Seed 100
    let d100 = BountyRoller::roll_daily(100, 0, &cfg);
    let w100 = BountyRoller::roll_weekly(100, 0, &cfg);
    compared_count += assert_bounty(&d100[0], "d0_0", "kill_zombies", "Kill 4 runners", 4, 21, 60);
    assert_eq!(d100[0].zombie.as_deref(), Some("runner"));
    compared_count += assert_bounty(&d100[1], "d0_1", "hunt_animals", "Hunt 5 rabbits", 5, 10, 32);
    compared_count += assert_bounty(&d100[2], "d0_2", "loot_crates", "Search 2 loot crates", 2, 12, 34);
    compared_count += assert_bounty(&w100, "w0", "kill_zombies", "Kill 78 zombies", 78, 82, 295);
    compared_count += 1;

    // Exact golden value checks for Seed 777
    let d777 = BountyRoller::roll_daily(777, 0, &cfg);
    let w777 = BountyRoller::roll_weekly(777, 0, &cfg);
    compared_count += assert_bounty(&d777[0], "d0_0", "deliver_items", "Bring 5 cloth", 5, 13, 35);
    compared_count += assert_bounty(&d777[1], "d0_1", "deliver_items", "Bring 2 hides", 2, 14, 34);
    compared_count += assert_bounty(&d777[2], "d0_2", "hunt_animals", "Hunt 4 rabbits", 4, 8, 26);
    compared_count += assert_bounty(&w777, "w0", "deliver_items", "Bring 12 raw meat", 12, 80, 229);

    // Golden check for clear_poi building replacement on Week 2 for seed 42
    let w42_2 = BountyRoller::roll_weekly(42, 2, &cfg);
    compared_count += assert_bounty(
        &w42_2,
        "w2",
        "clear_poi",
        "Clear the shed of all its zombies",
        1,
        76,
        268,
    );
    assert_eq!(w42_2.building.as_deref(), Some("shed"));
    compared_count += 1;

    println!("bounties_rolls_3_fixed_seeds_stable_and_valid compared {compared_count} items");
    assert!(compared_count > 0, "must compare at least one item");
}

#[test]
fn bounties_accept_progress_complete_reward() {
    let mut mgr = BountyManager::from_repo(42).expect("data/bounties.json must load");
    mgr.refresh(0, 10.0);

    assert_eq!(mgr.offered().len(), 4);
    assert!(mgr.active().is_empty());

    // Accept "d0_1": "Hunt 5 rabbits"
    let accepted = mgr.accept("d0_1").expect("accept bounty");
    assert_eq!(accepted.id, "d0_1");
    assert_eq!(mgr.active().len(), 1);
    assert_eq!(mgr.offered().len(), 3);

    // Cannot claim unfinished bounty
    assert!(matches!(
        mgr.claim("d0_1"),
        Err(BountyClaimError::NotFinished)
    ));

    // Progress events
    assert_eq!(mgr.on_animal_killed("deer"), 0);
    assert_eq!(mgr.active()[0].progress, 0);

    assert_eq!(mgr.on_animal_killed("rabbit"), 1);
    assert_eq!(mgr.active()[0].progress, 1);

    for _ in 0..4 {
        mgr.on_animal_killed("rabbit");
    }
    assert_eq!(mgr.active()[0].progress, 5);
    assert!(mgr.active()[0].is_done());

    // Complete / claim reward
    let reward = mgr.complete("d0_1").expect("claim bounty reward");
    assert_eq!(reward.dollars, 9);
    assert_eq!(reward.xp, 40);
    assert_eq!(reward.skill, "hunting");
    assert!(mgr.active().is_empty());
    assert!(mgr.claimed().contains("d0_1"));

    // Accept and deliver item bounty: "d0_0": "Bring 3 raw meat"
    mgr.accept("d0_0").expect("accept d0_0");
    assert_eq!(mgr.active().len(), 1);

    // Progress with delivery
    mgr.on_item_delivered("raw_meat", 2);
    assert_eq!(mgr.active()[0].progress, 2);
    assert!(!mgr.active()[0].is_done());

    mgr.on_item_delivered("raw_meat", 1);
    assert_eq!(mgr.active()[0].progress, 3);
    assert!(mgr.active()[0].is_done());

    let reward2 = mgr.claim("d0_0").expect("claim d0_0");
    assert_eq!(reward2.dollars, 11);
    assert_eq!(reward2.xp, 43);
    assert_eq!(reward2.skill, "hunting");
    assert!(mgr.claimed().contains("d0_0"));
}

#[test]
fn bounties_expiry() {
    let mut mgr = BountyManager::from_repo(42).expect("data/bounties.json must load");
    mgr.refresh(0, 10.0);

    // Accept "d0_2": expires on day 0
    let bounty = mgr.accept("d0_2").expect("accept d0_2");
    assert_eq!(bounty.expires_on, 0);
    assert_eq!(mgr.active().len(), 1);

    // Advance to day 1, hour 10.0 (bounty day 1)
    mgr.refresh(1, 10.0);

    // Active day 0 bounty expired and removed
    assert!(mgr.active().is_empty());

    // Day 0 daily offered bounties expired, replaced by new day 1 bounties
    for offered in mgr.offered() {
        if !offered.weekly {
            assert_eq!(offered.expires_on, 1);
        }
    }
}

#[test]
fn bounties_board_refresh_per_day() {
    let mut mgr = BountyManager::from_repo(100).expect("data/bounties.json must load");

    // Initial refresh at day 0, 07:00
    mgr.refresh(0, 7.0);
    assert_eq!(mgr.daily_day(), 0);
    assert_eq!(mgr.offered().len(), 4);

    let initial_offered: Vec<String> = mgr.offered().iter().map(|b| b.id.clone()).collect();

    // Refresh later on the same day: no re-roll
    mgr.refresh(0, 18.0);
    let same_day_offered: Vec<String> = mgr.offered().iter().map(|b| b.id.clone()).collect();
    assert_eq!(initial_offered, same_day_offered);

    // Next morning at 04:00 (before reset hour 6:00): still day 0
    mgr.refresh(1, 4.0);
    assert_eq!(mgr.daily_day(), 0);

    // At 06:00: new bounty day 1 starts
    mgr.refresh(1, 6.0);
    assert_eq!(mgr.daily_day(), 1);

    let day1_daily_ids: Vec<String> = mgr
        .offered()
        .iter()
        .filter(|b| !b.weekly)
        .map(|b| b.id.clone())
        .collect();
    assert_eq!(day1_daily_ids, vec!["d1_0", "d1_1", "d1_2"]);

    // Board interaction logic
    let board = BountyBoard::new(glam::Vec3::new(5.0, 0.0, -10.0));
    assert_eq!(board.position, glam::Vec3::new(5.0, 0.0, -10.0));
    assert_eq!(BountyBoard::prompt(), "Read bounty board");
    assert!(board.can_interact());
    assert_eq!(board.offered(&mgr).len(), 4);
}

#[test]
fn bounties_save_round_trip() {
    let mut mgr = BountyManager::from_repo(42).expect("data/bounties.json must load");
    mgr.refresh(0, 10.0);

    // Accept d0_1 and advance progress
    mgr.accept("d0_1").expect("accept d0_1");
    mgr.on_animal_killed("rabbit");
    mgr.on_animal_killed("rabbit");

    // Check Godot save id
    assert_eq!(BountyManager::SAVE_ID, "bounties");

    let save_value = mgr.to_save();

    // Verify top-level Godot save keys
    let obj = save_value.as_object().expect("json object");
    assert!(obj.contains_key("offered"));
    assert!(obj.contains_key("active"));
    assert!(obj.contains_key("claimed"));
    assert!(obj.contains_key("dailyDay"));
    assert!(obj.contains_key("week"));

    // Check active bounty serialized keys
    let active_arr = obj.get("active").and_then(|v| v.as_array()).expect("active array");
    assert_eq!(active_arr.len(), 1);
    let active_obj = active_arr[0].as_object().expect("bounty object");
    assert_eq!(active_obj.get("id").and_then(|v| v.as_str()), Some("d0_1"));
    assert_eq!(active_obj.get("progress").and_then(|v| v.as_i64()), Some(2));
    assert_eq!(active_obj.get("target").and_then(|v| v.as_i64()), Some(5));

    // Restore into a fresh manager
    let mut restored = BountyManager::from_repo(42).expect("fresh manager");
    restored.from_save(&save_value).expect("from_save must succeed");

    assert_eq!(restored.seed(), mgr.seed());
    assert_eq!(restored.daily_day(), mgr.daily_day());
    assert_eq!(restored.week(), mgr.week());
    assert_eq!(restored.claimed(), mgr.claimed());
    assert_eq!(restored.active(), mgr.active());
    assert_eq!(restored.offered(), mgr.offered());

    // Round-trip re-serialization match
    assert_eq!(restored.to_save(), save_value);
}
