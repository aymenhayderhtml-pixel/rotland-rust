//! Integration tests for rescued survivors starting at their rescue spot instead of camp.

use glam::Vec3;

use game::guild::companion::{
    CompanionConfig, CompanionDraw, CompanionSquad, CompanionWorld,
};
use game::guild::{GuildRecruit, RecruitOrder, RecruitRole};

fn load_cfg() -> CompanionConfig {
    CompanionConfig::from_repo().expect("data/companions.json should load")
}

fn make_recruit(id: &str, name: &str, role: RecruitRole, order: RecruitOrder) -> GuildRecruit {
    let mut r = GuildRecruit::new(id, name, role, 1, 15, 100, 30); // new: DECISIONS 2026-11-25
    r.set_order(order);
    r
}

fn free_move(from: Vec3, step: Vec3) -> Vec3 {
    from + step
}

fn draw_for(squad: &CompanionSquad, id: &str) -> CompanionDraw {
    squad
        .draws()
        .find(|d| d.id == id)
        .expect("companion should exist")
}

#[test]
fn rescued_survivor_starts_at_rescue_spot_and_follows_player() {
    let cfg = load_cfg();
    let mut squad = CompanionSquad::new(cfg.clone());
    let camp = Vec3::ZERO;
    let rescue_pos = Vec3::new(350.0, 2.0, -120.0); // new: DECISIONS 2026-11-25

    let rescued = make_recruit("rescue_1", "Elena", RecruitRole::Fighter, RecruitOrder::Follow);
    squad.set_spawn_position(&rescued.id, rescue_pos);
    squad.sync(std::slice::from_ref(&rescued), camp);

    let initial = draw_for(&squad, &rescued.id);
    assert!(
        initial.position.distance(rescue_pos) < 1e-4, // new: DECISIONS 2026-11-25
        "expected rescued survivor at {rescue_pos:?}, got {:?}",
        initial.position
    );

    let player_pos = rescue_pos + Vec3::new(0.0, 0.0, -8.0); // new: DECISIONS 2026-11-25
    let world = CompanionWorld {
        player: player_pos,
        player_forward: Vec3::new(0.0, 0.0, -1.0),
        camp,
        zombies: &[],
    };
    let mut events = Vec::new();
    let dt = 0.05; // new: DECISIONS 2026-11-25
    for _ in 0..80 {
        squad.tick(&world, &mut free_move, dt, &mut events);
    }

    let after = draw_for(&squad, &rescued.id);
    assert!(
        after.position.distance(player_pos) < cfg.follow_distance + 1.5, // new: DECISIONS 2026-11-25
        "expected rescued survivor to follow player near {player_pos:?}, got {:?}",
        after.position
    );
}

#[test]
fn hired_at_camp_recruit_still_starts_at_camp() {
    let cfg = load_cfg();
    let mut squad = CompanionSquad::new(cfg);
    let camp = Vec3::new(10.0, 0.0, -20.0); // new: DECISIONS 2026-11-25
    let rescue_pos = Vec3::new(350.0, 0.0, 40.0); // new: DECISIONS 2026-11-25

    let camp_hire = make_recruit("camp_1", "Mark", RecruitRole::Guard, RecruitOrder::Guard);
    let rescued = make_recruit("rescue_2", "Sam", RecruitRole::Scavenger, RecruitOrder::Follow);
    squad.set_spawn_position(&rescued.id, rescue_pos);
    squad.sync(&[camp_hire.clone(), rescued.clone()], camp);

    let camp_draw = draw_for(&squad, &camp_hire.id);
    assert!(
        camp_draw.position.distance(camp) < 6.0, // new: DECISIONS 2026-11-25
        "expected camp hire near {camp:?}, got {:?}",
        camp_draw.position
    );

    let rescue_draw = draw_for(&squad, &rescued.id);
    assert!(
        rescue_draw.position.distance(rescue_pos) < 1e-4, // new: DECISIONS 2026-11-25
        "expected rescued survivor at {rescue_pos:?}, got {:?}",
        rescue_draw.position
    );
}

#[test]
fn spawn_override_is_consumed_once_and_does_not_reset_existing_member() {
    let cfg = load_cfg();
    let mut squad = CompanionSquad::new(cfg);
    let camp = Vec3::ZERO;
    let rescue_pos = Vec3::new(320.0, 0.0, 0.0); // new: DECISIONS 2026-11-25

    let rescued = make_recruit("rescue_3", "Nora", RecruitRole::Fighter, RecruitOrder::Follow);
    squad.set_spawn_position(&rescued.id, rescue_pos);
    squad.sync(std::slice::from_ref(&rescued), camp);

    let moved_player = rescue_pos + Vec3::new(10.0, 0.0, 0.0); // new: DECISIONS 2026-11-25
    let world = CompanionWorld {
        player: moved_player,
        player_forward: Vec3::new(0.0, 0.0, -1.0),
        camp,
        zombies: &[],
    };
    let mut events = Vec::new();
    let dt = 0.05; // new: DECISIONS 2026-11-25
    for _ in 0..40 {
        squad.tick(&world, &mut free_move, dt, &mut events);
    }
    let pos_before_resync = draw_for(&squad, &rescued.id).position;
    assert!(pos_before_resync.distance(rescue_pos) > 1.0);

    // Re-syncing an existing member must not reset its position back to rescue_pos or camp.
    squad.sync(std::slice::from_ref(&rescued), camp);
    let pos_after_resync = draw_for(&squad, &rescued.id).position;
    assert!(pos_after_resync.distance(pos_before_resync) < 1e-4); // new: DECISIONS 2026-11-25
}
