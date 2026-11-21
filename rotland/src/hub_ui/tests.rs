//! Unit tests for HubUiState logic and stat formatting.

use std::collections::HashMap;

use super::screen::{HubScreen, HubTab};
use super::state::{
    HubUiState, LEVEL_UP_BANNER_DURATION, XP_TOAST_LIFETIME,
};
use super::view::{DogView, HubView};

#[test]
fn test_tab_order_and_wrap() {
    let mut state = HubUiState::new(HubScreen::Tab(HubTab::Bag));
    assert_eq!(state.tab(), HubTab::Bag);

    // Forward cycle (TabOrder: Profile, Bag, Skills, Bounties, Map, Settings)
    state.next_tab();
    assert_eq!(state.tab(), HubTab::Skills);
    state.next_tab();
    assert_eq!(state.tab(), HubTab::Bounties);
    state.next_tab();
    assert_eq!(state.tab(), HubTab::Map);
    state.next_tab();
    assert_eq!(state.tab(), HubTab::Settings);
    state.next_tab();
    assert_eq!(state.tab(), HubTab::Profile);
    state.next_tab();
    assert_eq!(state.tab(), HubTab::Bag);

    // Backward cycle
    state.prev_tab();
    assert_eq!(state.tab(), HubTab::Profile);
    state.prev_tab();
    assert_eq!(state.tab(), HubTab::Settings);
    state.prev_tab();
    assert_eq!(state.tab(), HubTab::Map);
    state.prev_tab();
    assert_eq!(state.tab(), HubTab::Bounties);
    state.prev_tab();
    assert_eq!(state.tab(), HubTab::Skills);
    state.prev_tab();
    assert_eq!(state.tab(), HubTab::Bag);
}

#[test]
fn test_switch_tab_resets_drag_and_status() {
    let mut state = HubUiState::new(HubScreen::Tab(HubTab::Bag));
    state.start_drag(4);
    state.set_status("Testing error message");
    assert!(state.is_dragging());
    assert_eq!(state.status(), "Testing error message");

    state.set_tab(HubTab::Skills);
    assert_eq!(state.tab(), HubTab::Skills);
    assert_eq!(state.screen(), HubScreen::Tab(HubTab::Skills));
    assert!(!state.is_dragging());
    assert!(state.status().is_empty());
}

#[test]
fn test_selected_slot_and_moves() {
    let mut state = HubUiState::new(HubScreen::Tab(HubTab::Bag));
    assert_eq!(state.selected_slot(), 0);

    state.select_slot(7);
    assert_eq!(state.selected_slot(), 7);

    // Moving the selected slot moves the selection to destination
    state.handle_slot_move(7, 15);
    assert_eq!(state.selected_slot(), 15);

    // Moving an unrelated slot does not alter current selection
    state.handle_slot_move(2, 3);
    assert_eq!(state.selected_slot(), 15);
}

#[test]
fn test_drag_and_drop_state() {
    let mut state = HubUiState::new(HubScreen::Tab(HubTab::Bag));
    assert!(!state.is_dragging());
    assert_eq!(state.dragged_slot(), None);

    state.start_drag(5);
    assert!(state.is_dragging());
    assert_eq!(state.dragged_slot(), Some(5));
    assert_eq!(state.selected_slot(), 5);

    // Dropping onto same slot cancels and returns None
    assert_eq!(state.drop_drag(5), None);
    assert!(!state.is_dragging());

    // Starting drag and cancelling
    state.start_drag(2);
    assert!(state.is_dragging());
    state.cancel_drag();
    assert!(!state.is_dragging());
    assert_eq!(state.dragged_slot(), None);

    // Dropping onto different slot returns (from, to) and updates selection
    state.start_drag(3);
    let drop = state.drop_drag(8);
    assert_eq!(drop, Some((3, 8)));
    assert_eq!(state.selected_slot(), 8);
    assert!(!state.is_dragging());
}

#[test]
fn test_split_count_clamps() {
    let mut state = HubUiState::new(HubScreen::Tab(HubTab::Bag));
    assert_eq!(state.split_count(), 1);

    state.clamp_split_count(10);
    assert_eq!(state.split_count(), 1);

    state.step_split_up(10);
    assert_eq!(state.split_count(), 2);
    state.step_split_up(10);
    assert_eq!(state.split_count(), 3);

    state.step_split_down();
    assert_eq!(state.split_count(), 2);
    state.step_split_down();
    assert_eq!(state.split_count(), 1);
    state.step_split_down();
    assert_eq!(state.split_count(), 1);

    // Clamping large count down to max
    state.set_split_count(20);
    assert_eq!(state.split_count(), 20);
    state.clamp_split_count(6);
    assert_eq!(state.split_count(), 6);

    state.clamp_split_count(0);
    assert_eq!(state.split_count(), 1);
}

#[test]
fn test_level_up_banner() {
    let mut state = HubUiState::new(HubScreen::Tab(HubTab::Bag));
    assert!(state.level_up_banner.is_none());

    state.trigger_level_up("Combat", 2);
    let banner = state.level_up_banner.as_ref().expect("banner must exist");
    assert_eq!(banner.text, "COMBAT LEVEL 2");
    assert!((banner.time_left - LEVEL_UP_BANNER_DURATION).abs() < 1e-4);

    state.update_overlays(1.0);
    assert!((state.level_up_banner.as_ref().unwrap().time_left - 1.5).abs() < 1e-4);

    // Second level-up replaces existing banner and resets duration
    state.trigger_level_up("Agility", 3);
    assert_eq!(state.pending_level_ups.len(), 2);
    let banner2 = state.level_up_banner.as_ref().expect("banner must exist");
    assert_eq!(banner2.text, "AGILITY LEVEL 3");
    assert!((banner2.time_left - LEVEL_UP_BANNER_DURATION).abs() < 1e-4);

    // Expiration after remaining time
    state.update_overlays(2.6);
    assert!(state.level_up_banner.is_none());
}

#[test]
fn test_xp_toast_stack_merging_and_expiration() {
    let mut state = HubUiState::new(HubScreen::Tab(HubTab::Bag));
    assert!(state.xp_toasts.is_empty());

    state.add_xp_toast("combat", "Combat", 25.0);
    assert_eq!(state.xp_toasts.len(), 1);
    assert_eq!(state.xp_toasts[0].skill_name, "Combat");
    assert_eq!(state.xp_toasts[0].amount, 25.0);
    assert_eq!(state.xp_toasts[0].age, 0.0);

    state.update_overlays(0.5);
    assert!((state.xp_toasts[0].age - 0.5).abs() < 1e-4);

    // Merges because age (0.5s) is within XP_TOAST_MERGE_WINDOW (1.0s)
    state.add_xp_toast("combat", "Combat", 15.0);
    assert_eq!(state.xp_toasts.len(), 1);
    assert_eq!(state.xp_toasts[0].amount, 40.0);
    assert_eq!(state.xp_toasts[0].age, 0.0);

    // Adding different skill creates new row
    state.add_xp_toast("survival", "Survival", 10.0);
    assert_eq!(state.xp_toasts.len(), 2);
    assert_eq!(state.xp_toasts[1].skill_name, "Survival");
    assert_eq!(state.xp_toasts[1].amount, 10.0);

    // Advancing past XP_TOAST_LIFETIME (1.5s) expires all
    state.update_overlays(XP_TOAST_LIFETIME + 0.1);
    assert!(state.xp_toasts.is_empty());
}

#[test]
fn test_format_play_time() {
    assert_eq!(HubView::format_play_time(0.0), "0h 00m");
    assert_eq!(HubView::format_play_time(59.0), "0h 00m");
    assert_eq!(HubView::format_play_time(60.0), "0h 01m");
    assert_eq!(HubView::format_play_time(3600.0), "1h 00m");
    assert_eq!(HubView::format_play_time(3665.0), "1h 01m");
    assert_eq!(HubView::format_play_time(7325.0), "2h 02m");
}

#[test]
fn test_format_distance() {
    assert_eq!(HubView::format_distance(0.0), "0 m");
    assert_eq!(HubView::format_distance(450.0), "450 m");
    assert_eq!(HubView::format_distance(999.0), "999 m");
    assert_eq!(HubView::format_distance(1000.0), "1.00 km");
    assert_eq!(HubView::format_distance(1250.0), "1.25 km");
}

#[test]
fn test_format_by_type() {
    let empty = HashMap::new();
    assert_eq!(HubView::format_by_type(&empty), "None yet");

    let mut kills = HashMap::new();
    kills.insert("walker".to_string(), 9);
    kills.insert("runner".to_string(), 3);
    assert_eq!(HubView::format_by_type(&kills), "Walker 9, Runner 3");

    let mut tied = HashMap::new();
    tied.insert("spitter".to_string(), 5);
    tied.insert("crawler".to_string(), 5);
    assert_eq!(HubView::format_by_type(&tied), "Crawler 5, Spitter 5");
}

#[test]
fn test_format_dog() {
    assert_eq!(HubView::format_dog(None), "No dog");

    let dog = DogView {
        name: "Rex".to_string(),
        mode: "Follow".to_string(),
        hunger: 0.0,
        health: 64.0,
        max_health: 80.0,
        age: 3,
        has_food_in_bag: false,
        pack_inventory: None,
        pack_max_weight: 10.0,
        pack_slots: 4,
    };
    assert_eq!(
        HubView::format_dog(Some(&dog)),
        "Rex (64/80 HP), together 3 days"
    );

    let puppy = DogView {
        name: "Buddy".to_string(),
        mode: "Follow".to_string(),
        hunger: 0.0,
        health: 80.0,
        max_health: 80.0,
        age: 1,
        has_food_in_bag: true,
        pack_inventory: None,
        pack_max_weight: 10.0,
        pack_slots: 4,
    };
    assert_eq!(
        HubView::format_dog(Some(&puppy)),
        "Buddy (80/80 HP), together 1 day"
    );
}

#[test]
fn test_dog_name_input() {
    let mut state = HubUiState::new(HubScreen::DogNameDialog);
    assert_eq!(state.dog_name_input(), "Rex");
    state.set_dog_name_input("Rover");
    assert_eq!(state.dog_name_input(), "Rover");
}

#[test]
fn test_screen_tab_helper() {
    assert_eq!(HubScreen::Tab(HubTab::Profile).tab(), Some(HubTab::Profile));
    assert_eq!(HubScreen::Tab(HubTab::Bag).tab(), Some(HubTab::Bag));
    assert_eq!(HubScreen::Trunk.tab(), None);
    assert_eq!(HubScreen::DogMenu.tab(), None);
    assert_eq!(HubScreen::DogPack.tab(), None);
    assert_eq!(HubScreen::DogNameDialog.tab(), None);
}
