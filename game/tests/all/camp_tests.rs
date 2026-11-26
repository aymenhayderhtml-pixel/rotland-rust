//! Integration tests for camp life and settlement NPCs (tasks/R5-04-camp-life.md).

use glam::Vec3;
use serde_json::json;
use world::{SettlementConfig, SettlementConfigData, SettlementSite};

use game::camp::{
    CampJob, CampLifeConfig, CampLifeDirector, CampState, ResidentOutput, SettlementGuard,
    SettlementNpc, SettlementWatcher,
};

fn default_test_site() -> SettlementSite {
    SettlementSite::new("camp_1", Vec3::ZERO, 0.0)
}

fn test_settlement_config() -> SettlementConfigData {
    let mut npc_models = std::collections::HashMap::new();
    npc_models.insert("npc_trader".to_string(), "npc_trader".to_string());
    npc_models.insert("npc_banker".to_string(), "npc_trader".to_string());
    npc_models.insert("npc_guard".to_string(), "npc_guard".to_string());
    npc_models.insert("npc_survivor".to_string(), "npc_survivor".to_string());

    SettlementConfigData {
        search_radius_chunks: 6,
        lot_size: 40.0,
        flat_radius: 24.0,
        blend_radius: 36.0,
        relief_limit: 8.0,
        cliff_check_radius: 40.0,
        max_slope_deg: 35.0,
        clear_radius: 32.0,
        safe_zone_half: 17.5,
        safe_zone_padding: 40.0,
        respawn_offset: [0.0, -2.6],
        guard_range: 40.0,
        guard_sight_interval: 0.5,
        guard_fire_interval: 1.2,
        guard_damage: 60.0,
        guard_miss_chance: 0.15,
        guard_eye_height: 1.6,
        npc_models,
    }
}

#[test]
fn camp_life_config_loads_and_validates_repo_file() {
    let mut compared = 0;
    let data = CampLifeConfig::load_default().expect("data/camp_life.json must load");

    assert_eq!(data.resident_count, 6);
    compared += 1;

    assert!((data.walk_speed - 1.2).abs() < 1e-4);
    assert!((data.idle_min_seconds - 3.0).abs() < 1e-4);
    assert!((data.idle_max_seconds - 8.0).abs() < 1e-4);
    assert!((data.chat_range - 15.0).abs() < 1e-4);
    assert!((data.greet_range - 4.0).abs() < 1e-4);
    assert!((data.bubble_range - 12.0).abs() < 1e-4);
    assert!((data.far_range - 60.0).abs() < 1e-4);
    assert!((data.fire_hour_from - 19.0).abs() < 1e-4);
    assert!((data.fire_hour_to - 6.0).abs() < 1e-4);
    assert_eq!(data.fire_seats, 4);
    assert_eq!(data.wander_points.len(), 13);
    assert_eq!(data.bed_points.len(), 3);
    assert_eq!(data.carry_spots.len(), 2);
    assert_eq!(data.chat_lines.len(), 30);
    compared += 14;

    assert!(data.validate().is_ok());
    compared += 1;

    let settlements_path = paths::repo_path("data/settlements.json");
    let settlements_cfg = SettlementConfig::load_from_file(&settlements_path)
        .expect("data/settlements.json must load");
    assert_eq!(settlements_cfg.guard_range, 40.0);
    assert_eq!(settlements_cfg.guard_damage, 60.0);
    assert_eq!(settlements_cfg.safe_zone_half, 17.5);
    compared += 3;

    println!("camp_life_config_loads_and_validates_repo_file compared {compared} items");
    assert!(compared > 0);
}

#[test]
fn settlement_guard_scans_targets_and_fires() {
    let mut compared = 0;
    let cfg = test_settlement_config();
    let mut guard = SettlementGuard::new("npc_guard_1", Vec3::new(-7.3, 2.7, -17.9), 0.0, &cfg);

    assert_eq!(guard.animation, SettlementGuard::READY_POSE);
    assert!(!guard.alert);
    assert_eq!(guard.shots_fired, 0);
    compared += 3;

    let mut rng = world::DotnetRandom::new(42);
    let zombie_in_range = Vec3::new(-7.3, 0.0, -10.0);

    // Sight interval is 0.5s; advance by 0.6s to acquire the target. The fire timer starts at 0, so the
    // guard shoots at once (Godot SettlementGuard.cs: _fireTimer = 0 until a target is seen)
    let _ = guard.update(0.6, &[zombie_in_range], &mut rng);
    assert!(guard.alert, "guard must become alert when zombie is in range");
    assert!(guard.target_pos.is_some());
    assert_eq!(guard.shots_fired, 1);
    compared += 3;

    // Fire interval is 1.2s; 1.3s later the second shot
    let hit = guard.update(1.3, &[zombie_in_range], &mut rng);
    assert_eq!(guard.shots_fired, 2);
    compared += 1;

    if let Some(dmg) = hit {
        assert_eq!(dmg.amount, 60.0);
        assert_eq!(dmg.damage_type, "bullet");
        compared += 2;
    }

    println!("settlement_guard_scans_targets_and_fires compared {compared} items");
    assert!(compared > 0);
}

#[test]
fn residents_planned_3_hours_follow_schedule_and_count_jobs() {
    let mut compared = 0;
    let data = CampLifeConfig::load_default().expect("data/camp_life.json must load");
    let cfg = test_settlement_config();
    let site = default_test_site();

    let mut camp = CampLifeDirector::new(site, data, &cfg);
    camp.manual_only = true;

    assert_eq!(camp.residents().len(), 6);
    compared += 1;

    assert_eq!(camp.residents()[0].job, CampJob::Woodcutter);
    assert_eq!(camp.residents()[1].job, CampJob::Carrier);
    assert_eq!(camp.residents()[2].job, CampJob::None);
    assert_eq!(camp.residents()[3].job, CampJob::None);
    assert_eq!(camp.residents()[4].job, CampJob::None);
    assert_eq!(camp.residents()[5].job, CampJob::None);
    compared += 6;

    let mut woodcutter_work_samples = 0;
    let mut carrier_work_samples = 0;
    let mut other_work_samples = 0;

    // Simulate 3 hours of game daytime (from hour 11.0 to 14.0):
    // 3 hours = 10,800 game seconds at step 0.5s = 21,600 ticks.
    let game_hour = 12.0;
    let step = 0.5;
    let total_seconds = 3.0 * 3600.0;
    let mut t = 0.0;
    while t < total_seconds {
        camp.tick(step, game_hour, None);

        let r0 = &camp.residents()[0];
        if r0.state == CampState::Work {
            woodcutter_work_samples += 1;
            assert_eq!(r0.job, CampJob::Woodcutter);
        }

        let r1 = &camp.residents()[1];
        if r1.state == CampState::Work {
            carrier_work_samples += 1;
            assert_eq!(r1.job, CampJob::Carrier);
        }

        for r in &camp.residents()[2..6] {
            if r.state == CampState::Work {
                other_work_samples += 1;
            }
        }

        t += step;
    }

    assert!(woodcutter_work_samples > 0, "woodcutter must have worked during 3 daytime hours");
    assert!(carrier_work_samples > 0, "carrier must have worked during 3 daytime hours");
    assert_eq!(other_work_samples, 0, "residents with CampJob::None must never do work");
    compared += 3;

    println!(
        "residents_planned_3_hours_follow_schedule compared {compared} items (woodcutter_samples={woodcutter_work_samples}, carrier_samples={carrier_work_samples})"
    );
    assert!(compared > 0);
}

#[test]
fn chat_picks_are_stable_for_fixed_seed() {
    let mut compared = 0;
    let data = CampLifeConfig::load_default().expect("data/camp_life.json must load");
    let cfg = test_settlement_config();

    let simulate_lines = |site_id: &str| -> Vec<String> {
        let site = SettlementSite::new(site_id, Vec3::ZERO, 0.0);
        let mut camp = CampLifeDirector::new(site, data.clone(), &cfg);
        camp.manual_only = true;

        let mut spoken = Vec::new();
        let step = 0.2;
        let total_seconds = 1200.0; // 20 minutes of camp life
        let mut t = 0.0;
        let mut prev_lines = 0;

        while t < total_seconds {
            camp.tick(step, 12.0, None);
            for chat in camp.chats() {
                if chat.started && chat.lines_spoken > prev_lines {
                    prev_lines = chat.lines_spoken;
                    if let Some(talker_idx) = chat.talker
                        && let Some(text) = camp.residents()[talker_idx].bubble.text()
                    {
                        spoken.push(text.to_string());
                    }
                }
            }
            t += step;
        }
        spoken
    };

    let run1 = simulate_lines("camp_fixed_seed_42");
    let run2 = simulate_lines("camp_fixed_seed_42");
    let run_diff = simulate_lines("camp_different_seed_999");

    assert!(!run1.is_empty(), "chat must pick lines during 20 minutes");
    assert_eq!(run1.len(), run2.len());
    assert_eq!(run1, run2, "identical seed must pick identical chat lines in order");
    compared += 3;

    // Different seed produces a different sequence
    assert_ne!(run1, run_diff, "different seed must vary chat lines or timing");
    compared += 1;

    for line in &run1 {
        assert!(data.chat_lines.contains(line));
        compared += 1;
    }

    println!(
        "chat_picks_are_stable_for_fixed_seed compared {compared} items (lines_spoken={})",
        run1.len()
    );
    assert!(compared > 0);
}

#[test]
fn interact_finds_trader_in_front_and_nothing_behind() {
    let mut compared = 0;
    let data = CampLifeConfig::load_default().expect("data/camp_life.json must load");
    let cfg = test_settlement_config();
    let site = SettlementSite::new("camp_1", Vec3::new(100.0, 10.0, 200.0), 0.0);

    let camp = CampLifeDirector::new(site, data, &cfg);

    // Trader is at local (-8.2, 0.0, -7.7), world (91.8, 10.0, 192.3)
    let trader_npc: &SettlementNpc = camp
        .fixed_npcs()
        .iter()
        .find(|n| n.role == "trader")
        .expect("trader must be in camp fixed npcs");
    assert_eq!(SettlementNpc::ROOT_LIFT, 1.1);
    compared += 1;
    assert_eq!(trader_npc.id, "npc_trader");
    assert_eq!(trader_npc.role, "trader");
    assert!(trader_npc.can_interact());
    assert_eq!(trader_npc.prompt(), "[E] Talk to trader");
    compared += 4;

    let trader_pos = trader_npc.position;
    assert!((trader_pos.x - 91.8).abs() < 1e-3);
    assert!((trader_pos.z - 192.3).abs() < 1e-3);
    compared += 2;

    // Player stands 1.5m south of trader, looking north (-Z) towards trader
    let player_in_front = trader_pos + Vec3::new(0.0, 0.0, 1.5);
    let facing_trader = Vec3::new(0.0, 0.0, -1.0);

    let hit = camp.nearest_interactable(player_in_front, facing_trader);
    assert!(hit.is_some(), "must find trader when standing 1.5m in front and facing trader");
    let (id, role, prompt) = hit.unwrap();
    assert_eq!(id, "npc_trader");
    assert_eq!(role, "trader");
    assert_eq!(prompt, "[E] Talk to trader");
    compared += 4;

    // Player turns around and faces south (+Z), away from trader
    let facing_away = Vec3::new(0.0, 0.0, 1.0);
    let hit_behind = camp.nearest_interactable(player_in_front, facing_away);
    assert!(hit_behind.is_none(), "must find nothing when facing away from trader");
    compared += 1;

    // Player faces east (+X), 90 degrees away from trader (exceeds 35° cone)
    let facing_side = Vec3::new(1.0, 0.0, 0.0);
    let hit_side = camp.nearest_interactable(player_in_front, facing_side);
    assert!(hit_side.is_none(), "must find nothing outside 35 degree cone");
    compared += 1;

    // Player stands 5m away (beyond 2.5m interact range)
    let player_far = trader_pos + Vec3::new(0.0, 0.0, 5.0);
    let hit_far = camp.nearest_interactable(player_far, facing_trader);
    assert!(hit_far.is_none(), "must find nothing when beyond 2.5m range");
    compared += 1;

    println!("interact_finds_trader_in_front_and_nothing_behind compared {compared} items");
    assert!(compared > 0);
}

#[test]
fn camp_has_gunsmith_npc_and_is_interactable() {
    let data = CampLifeConfig::load_default().expect("data/camp_life.json must load");
    let cfg = SettlementConfig::load_from_file(paths::repo_path("data/settlements.json"))
        .expect("data/settlements.json must load");
    assert_eq!(cfg.npc_models.get("npc_gunsmith").map(String::as_str), Some("npc_mechanic"));

    let site = SettlementSite::new("camp_1", Vec3::new(100.0, 10.0, 200.0), 0.0);
    let camp = CampLifeDirector::new(site, data, &cfg);
    let gunsmith = camp
        .fixed_npcs()
        .iter()
        .find(|n| n.id == "npc_gunsmith")
        .expect("npc_gunsmith must be in camp fixed_npcs");
    assert_eq!(gunsmith.role, "gunsmith");
    assert_eq!(gunsmith.model_id, "npc_mechanic");
    assert!(gunsmith.can_interact());
    assert_eq!(gunsmith.prompt(), "[E] Talk to gunsmith");

    let player_near = gunsmith.position + Vec3::new(0.0, 0.0, 1.5);
    let facing_north = Vec3::new(0.0, 0.0, -1.0);
    let hit = camp
        .nearest_interactable(player_near, facing_north)
        .expect("must find npc_gunsmith within 2.5m cone");
    assert_eq!(hit.0, "npc_gunsmith");
    assert_eq!(hit.1, "gunsmith");
    assert_eq!(hit.2, "[E] Talk to gunsmith");
}

#[test]
fn settlement_watcher_save_round_trip() {
    let mut compared = 0;
    let mut watcher = SettlementWatcher::new();

    assert_eq!(watcher.save_id(), "settlements");
    assert!(watcher.visited().is_empty());
    assert_eq!(watcher.respawn_id(), None);
    compared += 3;

    watcher.mark_visited("camp_1");
    watcher.mark_visited("camp_2");
    watcher.set_respawn(Some("camp_2".to_string()));

    assert_eq!(watcher.visited().len(), 2);
    assert!(watcher.is_visited("camp_1"));
    assert!(watcher.is_visited("camp_2"));
    assert_eq!(watcher.respawn_id(), Some("camp_2"));
    compared += 4;

    let saved = watcher.to_save();
    assert_eq!(saved["respawn"], "camp_2");
    let arr = saved["visited"].as_array().expect("visited must be array");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0], "camp_1");
    assert_eq!(arr[1], "camp_2");
    compared += 4;

    let mut second = SettlementWatcher::new();
    second.from_save(&saved);

    assert_eq!(second.visited().len(), 2);
    assert!(second.is_visited("camp_1"));
    assert!(second.is_visited("camp_2"));
    assert_eq!(second.respawn_id(), Some("camp_2"));
    compared += 4;

    // Empty save clears state
    let mut empty_watcher = SettlementWatcher::new();
    empty_watcher.from_save(&json!({}));
    assert!(empty_watcher.visited().is_empty());
    assert_eq!(empty_watcher.respawn_id(), None);
    compared += 2;

    println!("settlement_watcher_save_round_trip compared {compared} items");
    assert!(compared > 0);
}

#[test]
fn night_bedtime_and_fire_gathering() {
    let mut compared = 0;
    let data = CampLifeConfig::load_default().expect("data/camp_life.json must load");
    let cfg = test_settlement_config();
    let site = default_test_site();

    let mut camp = CampLifeDirector::new(site, data, &cfg);
    camp.manual_only = true;

    // Simulate 120s of night (hour 23.0)
    for _ in 0..1200 {
        camp.tick(0.1, 23.0, None);
    }

    // At night, bedtime residents (index < 3) eventually go to bed and hide
    let hidden = camp.count_in(CampState::Hidden);
    let sitting_fire = camp.count_in(CampState::Fire);

    assert!(hidden >= 1, "at night bedtime residents must retire to bed");
    assert!(sitting_fire >= 1, "at night non-bedtime residents must gather at fire");
    compared += 2;

    // Morning comes (hour 8.0): hidden residents wake up
    for _ in 0..50 {
        camp.tick(0.1, 8.0, None);
    }
    let hidden_morning = camp.count_in(CampState::Hidden);
    assert_eq!(hidden_morning, 0, "all residents must wake up in the morning");
    compared += 1;

    println!("night_bedtime_and_fire_gathering compared {compared} items");
    assert!(compared > 0);
}

#[test]
fn resident_greeting_and_per_frame_output() {
    let mut compared = 0;
    let data = CampLifeConfig::load_default().expect("data/camp_life.json must load");
    let cfg = test_settlement_config();
    let site = default_test_site();

    let mut camp = CampLifeDirector::new(site, data, &cfg);
    camp.manual_only = true;

    let r_pos = camp.residents()[0].position;
    let player_near = r_pos + Vec3::new(2.0, 0.0, 0.0);

    // manual_only: update() does not tick (like Godot's ManualOnly), so the test ticks by hand
    camp.tick(0.1, 12.0, Some(player_near));
    let outputs: Vec<ResidentOutput> = camp.update(0.0, 12.0, Some(player_near));
    assert_eq!(outputs.len(), 6);
    compared += 1;

    for out in &outputs {
        assert!(!out.id.is_empty());
        assert!(out.role == "villager" || out.role == "survivor");
        assert!(out.model_name == "npc_villager" || out.model_name == "npc_survivor");
        assert!(!out.animation.is_empty());
        compared += 4;
    }

    // Resident 0 greeted player
    let r0 = &camp.residents()[0];
    assert_eq!(r0.greets, 1);
    assert!(r0.is_greeting());
    compared += 2;

    // Output reflects greeting animation
    let outputs_greet: Vec<ResidentOutput> = camp.resident_outputs(Some(player_near));
    assert_eq!(outputs_greet[0].animation, "wave");
    compared += 1;

    // Moving player away and advancing 5 seconds
    let player_far = r_pos + Vec3::new(30.0, 0.0, 0.0);
    for _ in 0..50 {
        camp.tick(0.1, 12.0, Some(player_far));
    }

    // Moving player near again on the same day: no second wave
    camp.tick(0.1, 12.0, Some(player_near));
    assert_eq!(camp.residents()[0].greets, 1, "must only wave once per day");
    compared += 1;

    println!("resident_greeting_and_per_frame_output compared {compared} items");
    assert!(compared > 0);
}
