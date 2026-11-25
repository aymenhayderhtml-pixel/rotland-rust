//! Comprehensive tests for the Guild camp board screen.

use engine::egui::{self, vec2};
use game::guild::{
    GuildConfig, GuildManager, GuildReward, GuildTask, GuildTaskKind, RecruitOrder, RecruitRole,
    RecruitState,
};

use crate::camp_ui::action::GuildCampAction;
use crate::camp_ui::draw::draw;
use crate::camp_ui::screen::CampScreen;
use crate::camp_ui::state::CampUiState;
use crate::camp_ui::view::CampView;

use super::draw_guild;
use super::recruits_ui::{role_badge, state_label};
use super::tasks_ui::{format_task_goal, format_task_reward};

#[test]
fn test_task_formatting() {
    let kill_task = GuildTask::new(
        "k1",
        "Kill Walkers",
        GuildTaskKind::Kill {
            zombie_type: Some("walker".into()),
            in_city: true,
        },
        10,
        GuildReward {
            dollars: 40,
            reputation: 20,
            item: Some(("bandage".into(), 2)),
        },
    );
    assert_eq!(format_task_goal(&kill_task), "Kill 10 walker zombies in Ashford");
    assert_eq!(format_task_reward(&kill_task), "$40, +20 Rep, 2 bandage");

    let fetch_task = GuildTask::new(
        "f1",
        "Fetch Wood",
        GuildTaskKind::Fetch {
            item_id: "wood_plank".into(),
        },
        5,
        GuildReward {
            dollars: 25,
            reputation: 15,
            item: None,
        },
    );
    assert_eq!(format_task_goal(&fetch_task), "Deliver 5 wood plank");
    assert_eq!(format_task_reward(&fetch_task), "$25, +15 Rep");

    let explore_task = GuildTask::new(
        "e1",
        "Scout Lake",
        GuildTaskKind::Explore {
            place_name: "the lake".into(),
        },
        1,
        GuildReward::default(),
    );
    assert_eq!(format_task_goal(&explore_task), "Scout and reach the lake");

    let clear_task = GuildTask::new(
        "c1",
        "Clear Gas Station",
        GuildTaskKind::Clear {
            poi_id: "gas_station".into(),
        },
        1,
        GuildReward::default(),
    );
    assert_eq!(format_task_goal(&clear_task), "Clear gas station");
}

#[test]
fn test_recruit_labels() {
    let (label, _) = role_badge(RecruitRole::Fighter);
    assert_eq!(label, "Fighter");
    let (label, _) = role_badge(RecruitRole::Scavenger);
    assert_eq!(label, "Scavenger");
    let (label, _) = role_badge(RecruitRole::Guard);
    assert_eq!(label, "Guard");

    assert_eq!(state_label(RecruitState::IdleAtCamp), "Staying at Camp");
    assert_eq!(state_label(RecruitState::FollowingPlayer), "Following");
    assert_eq!(state_label(RecruitState::GuardingCamp), "Guarding Camp");
    assert_eq!(state_label(RecruitState::Dead), "Fallen in Action");
}

#[test]
fn test_guild_actions_representation() {
    let a1 = GuildCampAction::AcceptTask { task_id: "t1".into() };
    let a2 = GuildCampAction::AbandonTask { task_id: "t1".into() };
    let a3 = GuildCampAction::ClaimTask { task_id: "t1".into() };
    let a4 = GuildCampAction::HireRecruit { recruit_id: "r1".into() };
    let a5 = GuildCampAction::DismissRecruit { recruit_id: "r1".into() };
    let a6 = GuildCampAction::SetRecruitOrder { recruit_id: "r1".into(), order: RecruitOrder::Follow };
    let a7 = GuildCampAction::BuyItem { item_id: "bandage".into(), count: 3 };

    assert_ne!(a1, a2);
    assert_ne!(a2, a3);
    assert_ne!(a4, a5);
    assert_ne!(a6, a7);
}

#[test]
fn test_draw_guild_screen_headless() {
    let ctx = egui::Context::default();
    let mut state = CampUiState::new(CampScreen::Guild);

    // 1. Without guild manager (fallback)
    let view_empty = CampView::default();
    let mut out1 = ctx.run_pass(Default::default(), |ctx| {
        egui::Area::new(egui::Id::unique("guild_test_empty")).show(ctx, |ui| {
            let act = draw_guild(ui, &mut state, &view_empty);
            assert_eq!(act, None);
        });
    });
    out1.textures_delta.clear();

    // 2. With populated guild manager
    let config = GuildConfig::from_repo().unwrap_or_default();
    let mut manager = GuildManager::new(42, config);
    let mut funds = 500;
    manager.refresh_morning(1, 7.0, Some(&mut funds));
    let view = CampView { guild: Some(&manager), ..CampView::default() };

    let mut out2 = ctx.run_pass(Default::default(), |ctx| {
        egui::Area::new(egui::Id::unique("guild_test_populated")).show(ctx, |ui| {
            ui.set_min_size(vec2(1280.0, 720.0));
            let act = draw_guild(ui, &mut state, &view);
            assert_eq!(act, None);
        });
    });
    out2.textures_delta.clear();

    // 3. Central draw dispatcher
    let mut out3 = ctx.run_pass(Default::default(), |ctx| {
        let act = draw(ctx, CampScreen::Guild, &mut state, &view);
        assert_eq!(act, None);
    });
    out3.textures_delta.clear();
}
