//! Unit tests for camp upgrades formatting, state helpers, and view defaults.

use game::camp_upgrades::UpgradeLine;

use crate::camp_ui::screen::CampScreen;
use crate::camp_ui::view::{
    CampUpgradesView, CampView, NextUpgradeView, UpgradeCostItem, UpgradeInProgressView,
    UpgradeLineView,
};
use crate::menus::theme;

use super::colors::*;

#[test]
fn format_hours_handles_whole_fractional_and_edge_cases() {
    assert_eq!(format_hours(0.0), "0 min");
    assert_eq!(format_hours(-2.5), "0 min");
    assert_eq!(format_hours(f32::NAN), "0 min");
    assert_eq!(format_hours(0.004), "< 1 min");
    assert_eq!(format_hours(0.5), "30 min");
    assert_eq!(format_hours(1.0), "1 h");
    assert_eq!(format_hours(1.5), "1 h 30 min");
    assert_eq!(format_hours(3.0 + 20.0 / 60.0), "3 h 20 min");
    assert_eq!(format_hours(59.9 / 60.0), "1 h");
}

#[test]
fn format_building_banner_matches_expected_style() {
    let text = format_building_banner("Metal Sheet Walls", 3.0 + 20.0 / 60.0);
    assert_eq!(text, "Building: Metal Sheet Walls — 3 h 20 min left");
}

#[test]
fn cost_row_states_and_colors_distinguish_enough_and_short() {
    assert_eq!(cash_cost_state(150, 150), CostRowState::Enough);
    assert_eq!(cash_cost_state(200, 150), CostRowState::Enough);
    assert_eq!(cash_cost_state(149, 150), CostRowState::Short);

    assert_eq!(item_cost_state(20, 20), CostRowState::Enough);
    assert_eq!(item_cost_state(25, 20), CostRowState::Enough);
    assert_eq!(item_cost_state(19, 20), CostRowState::Short);

    assert_eq!(cost_row_color(CostRowState::Enough), theme::SAFE_HOVER);
    assert_eq!(cost_row_color(CostRowState::Short), theme::DANGER_HOVER);
    assert_eq!(format_cash_cost(120, 150), "$120/$150");
    assert_eq!(format_item_cost(16, 20), "16/20");
}

#[test]
fn upgrade_progress_fraction_clamps_and_handles_zero_total() {
    assert!((upgrade_progress_fraction(4.0, 4.0) - 0.0).abs() < 1e-5);
    assert!((upgrade_progress_fraction(2.0, 4.0) - 0.5).abs() < 1e-5);
    assert!((upgrade_progress_fraction(0.0, 4.0) - 1.0).abs() < 1e-5);
    assert!((upgrade_progress_fraction(6.0, 4.0) - 0.0).abs() < 1e-5);
    assert!((upgrade_progress_fraction(-1.0, 4.0) - 1.0).abs() < 1e-5);
    assert!((upgrade_progress_fraction(0.0, 0.0) - 1.0).abs() < 1e-5);
    assert!((upgrade_progress_fraction(f32::NAN, 4.0) - 0.0).abs() < 1e-5);
}

#[test]
fn wall_hp_helpers_cover_none_healthy_damaged_and_breached() {
    assert!((wall_hp_fraction(0.0, 0.0) - 0.0).abs() < 1e-5);
    assert_eq!(format_wall_hp_label(0.0, 0.0), "No walls");

    let healthy = wall_hp_fraction(400.0, 500.0);
    assert!((healthy - 0.8).abs() < 1e-5);
    assert_eq!(wall_hp_color(healthy), theme::SAFE);
    assert_eq!(format_wall_hp_label(400.0, 500.0), "400 / 500");

    let damaged = wall_hp_fraction(200.0, 500.0);
    assert_eq!(wall_hp_color(damaged), theme::ACCENT);

    let critical = wall_hp_fraction(50.0, 500.0);
    assert_eq!(wall_hp_color(critical), theme::DANGER);

    assert_eq!(format_wall_hp_label(0.0, 500.0), "0 / 500 (Breached)");
}

#[test]
fn refund_percent_and_cancel_label_use_ratio() {
    assert_eq!(refund_percent(0.5), 50);
    assert_eq!(refund_percent(1.5), 100);
    assert_eq!(refund_percent(-0.2), 0);
    assert_eq!(refund_percent(f32::NAN), 0);
    assert_eq!(cancel_button_label(0.5), "Cancel (refund 50%)");
}

#[test]
fn resolve_in_progress_name_prefers_explicit_then_next_then_fallback() {
    let mut upgrades = CampUpgradesView::default();
    upgrades.lines[0].next = Some(NextUpgradeView {
        name: "Wood Palisade".to_string(),
        items: vec![UpgradeCostItem {
            name: "Wood".to_string(),
            have: 20,
            need: 20,
        }],
        ..Default::default()
    });

    let explicit = UpgradeInProgressView {
        line: UpgradeLine::Walls,
        target_level: 2,
        target_name: "Metal Sheet Walls".to_string(),
        hours_left: 3.0,
        total_hours: 4.0,
    };
    assert_eq!(resolve_in_progress_name(&explicit, &upgrades), "Metal Sheet Walls");

    let from_next = UpgradeInProgressView {
        line: UpgradeLine::Walls,
        target_level: 1,
        target_name: String::new(),
        hours_left: 1.5,
        total_hours: 2.0,
    };
    assert_eq!(resolve_in_progress_name(&from_next, &upgrades), "Wood Palisade");

    let fallback = UpgradeInProgressView {
        line: UpgradeLine::Guards,
        target_level: 2,
        target_name: String::new(),
        hours_left: 2.0,
        total_hours: 4.0,
    };
    assert_eq!(resolve_in_progress_name(&fallback, &upgrades), "Guards & Towers Lv.2");
}

#[test]
fn effective_cant_start_reason_respects_explicit_and_busy_states() {
    let ready = NextUpgradeView::default();
    assert!(ready.can_start());
    assert_eq!(effective_cant_start_reason(&ready, None, UpgradeLine::Walls), None);

    let busy_walls = UpgradeInProgressView {
        line: UpgradeLine::Walls,
        target_level: 1,
        target_name: "Wood Palisade".to_string(),
        hours_left: 1.0,
        total_hours: 2.0,
    };
    assert_eq!(
        effective_cant_start_reason(&ready, Some(&busy_walls), UpgradeLine::Walls),
        Some("Currently building")
    );
    assert_eq!(
        effective_cant_start_reason(&ready, Some(&busy_walls), UpgradeLine::Workshop),
        Some("Another upgrade is in progress")
    );

    let blocked = NextUpgradeView {
        cant_start_reason: Some("Requires Workshop Lv.1".to_string()),
        ..Default::default()
    };
    assert!(!blocked.can_start());
    assert_eq!(
        effective_cant_start_reason(&blocked, None, UpgradeLine::Traders),
        Some("Requires Workshop Lv.1")
    );
}

#[test]
fn camp_screen_upgrades_metadata_and_view_defaults() {
    assert_eq!(CampScreen::Upgrades.title(), "Camp Board");
    assert_eq!(CampScreen::Upgrades.tab_name(), "Upgrades");
    assert_eq!(CampScreen::Upgrades.as_str(), "upgrades");

    let upgrades = CampUpgradesView {
        player_cash: 325,
        ..Default::default()
    };
    assert_eq!(upgrades.line(UpgradeLine::Walls).map(|l| l.max_level), Some(3));
    assert_eq!(upgrades.line(UpgradeLine::Traders).map(|l| l.max_level), Some(4));
    assert!(upgrades.line(UpgradeLine::Walls).is_some_and(UpgradeLineView::is_max_level));

    let view = CampView {
        upgrades: Some(upgrades),
        ..Default::default()
    };
    assert_eq!(view.player_dollars(), 325);
}
