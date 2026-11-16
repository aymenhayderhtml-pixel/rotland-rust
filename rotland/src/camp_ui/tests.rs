//! Pure logic tests for CampUiState (tab switching, selected row, count stepper clamping).

use super::screen::CampScreen;
use super::state::CampUiState;

#[test]
fn test_tab_switching() {
    let mut comparisons = 0;
    let mut state = CampUiState::new(CampScreen::Shop);

    assert_eq!(state.screen(), CampScreen::Shop);
    comparisons += 1;

    state.select_row("beans");
    state.set_count(5);
    state.set_status("Selected beans");

    assert_eq!(state.selected_row(), Some("beans"));
    assert_eq!(state.count(), 5);
    assert_eq!(state.status(), "Selected beans");
    comparisons += 3;

    // Switch to Bank: screen changes, selection, count and status reset
    state.switch_tab(CampScreen::Bank);
    assert_eq!(state.screen(), CampScreen::Bank);
    assert_eq!(state.selected_row(), None);
    assert_eq!(state.count(), 1);
    assert!(state.status().is_empty());
    comparisons += 4;

    // Switch to Crafting
    state.switch_tab(CampScreen::Crafting);
    assert_eq!(state.screen(), CampScreen::Crafting);
    comparisons += 1;

    // Switch to Bounties
    state.switch_tab(CampScreen::Bounties);
    assert_eq!(state.screen(), CampScreen::Bounties);
    comparisons += 1;

    // Switching to the same tab does not clear state
    state.select_row("bounty_1");
    state.set_count(3);
    state.switch_tab(CampScreen::Bounties);
    assert_eq!(state.selected_row(), Some("bounty_1"));
    assert_eq!(state.count(), 3);
    comparisons += 2;

    assert!(comparisons > 0, "No assertions were checked");
}

#[test]
fn test_selected_row() {
    let mut comparisons = 0;
    let mut state = CampUiState::new(CampScreen::Shop);

    assert_eq!(state.selected_row(), None);
    assert_eq!(state.count(), 1);
    comparisons += 2;

    // Select row
    state.select_row("axe");
    assert_eq!(state.selected_row(), Some("axe"));
    assert_eq!(state.count(), 1);
    comparisons += 2;

    // Changing count on current row
    state.set_count(4);
    assert_eq!(state.count(), 4);
    comparisons += 1;

    // Selecting same row again preserves count
    state.select_row("axe");
    assert_eq!(state.selected_row(), Some("axe"));
    assert_eq!(state.count(), 4);
    comparisons += 2;

    // Selecting a different row resets count to 1
    state.select_row("bandage");
    assert_eq!(state.selected_row(), Some("bandage"));
    assert_eq!(state.count(), 1);
    comparisons += 2;

    // Clear selection
    state.clear_selection();
    assert_eq!(state.selected_row(), None);
    assert_eq!(state.count(), 1);
    comparisons += 2;

    assert!(comparisons > 0, "No assertions were checked");
}

#[test]
fn test_count_stepper_basic() {
    let mut comparisons = 0;
    let mut state = CampUiState::new(CampScreen::Shop);

    assert_eq!(state.count(), 1);
    comparisons += 1;

    // Step up
    state.step_up(10);
    assert_eq!(state.count(), 2);
    comparisons += 1;

    // Step down
    state.step_down();
    assert_eq!(state.count(), 1);
    comparisons += 1;

    // Step down does not go below 1
    state.step_down();
    assert_eq!(state.count(), 1);
    comparisons += 1;

    // Increment with max limit
    state.increment_count(5, 10);
    assert_eq!(state.count(), 6);
    comparisons += 1;

    state.increment_count(10, 10);
    assert_eq!(state.count(), 10);
    comparisons += 1;

    // Decrement with min limit
    state.decrement_count(4);
    assert_eq!(state.count(), 6);
    comparisons += 1;

    state.decrement_count(100);
    assert_eq!(state.count(), 1);
    comparisons += 1;

    // Clamp count directly
    state.clamp_count(3, 8);
    assert_eq!(state.count(), 3);
    comparisons += 1;

    state.set_count(15);
    state.clamp_count(2, 7);
    assert_eq!(state.count(), 7);
    comparisons += 1;

    assert!(comparisons > 0, "No assertions were checked");
}

#[test]
fn test_count_stepper_clamp_to_stock() {
    let mut comparisons = 0;
    let mut state = CampUiState::new(CampScreen::Shop);

    // Stock = 5, space = 100, money limit = 100
    state.set_count(12);
    state.clamp_to_limits(5, 100, 100);
    assert_eq!(state.count(), 5);
    comparisons += 1;

    // Stock = 0 clamps count to 0
    state.set_count(5);
    state.clamp_to_limits(0, 100, 100);
    assert_eq!(state.count(), 0);
    comparisons += 1;

    // Stock = 1
    state.set_count(10);
    state.clamp_to_limits(1, 100, 100);
    assert_eq!(state.count(), 1);
    comparisons += 1;

    assert!(comparisons > 0, "No assertions were checked");
}

#[test]
fn test_count_stepper_clamp_to_money() {
    let mut comparisons = 0;
    let mut state = CampUiState::new(CampScreen::Shop);

    // max_affordable tests: 45 / 15 = 3
    assert_eq!(CampUiState::max_affordable(45, 15), 3);
    assert_eq!(CampUiState::max_affordable(50, 15), 3);
    assert_eq!(CampUiState::max_affordable(14, 15), 0);
    assert_eq!(CampUiState::max_affordable(0, 15), 0);
    assert_eq!(CampUiState::max_affordable(-20, 15), 0);
    assert_eq!(CampUiState::max_affordable(100, 0), 0);
    assert_eq!(CampUiState::max_affordable(100, -5), 0);
    comparisons += 7;

    // clamp_buy with $50 cash and price $15 (affords 3)
    state.set_count(10);
    state.clamp_buy(100, 50, 15, 100);
    assert_eq!(state.count(), 3);
    comparisons += 1;

    // clamp_buy with $0 cash
    state.set_count(5);
    state.clamp_buy(100, 0, 15, 100);
    assert_eq!(state.count(), 0);
    comparisons += 1;

    assert!(comparisons > 0, "No assertions were checked");
}

#[test]
fn test_count_stepper_clamp_to_space() {
    let mut comparisons = 0;
    let mut state = CampUiState::new(CampScreen::Shop);

    // Space limit is 4 units (even if player has $1000 and trader has 50 stock)
    state.set_count(10);
    state.clamp_buy(50, 1000, 10, 4);
    assert_eq!(state.count(), 4);
    comparisons += 1;

    // Space limit is 0
    state.set_count(5);
    state.clamp_buy(50, 1000, 10, 0);
    assert_eq!(state.count(), 0);
    comparisons += 1;

    assert!(comparisons > 0, "No assertions were checked");
}

#[test]
fn test_count_stepper_clamp_combined() {
    let mut comparisons = 0;
    let mut state = CampUiState::new(CampScreen::Shop);

    // Stock bottleneck (stock=3, money=10, space=8)
    state.set_count(20);
    state.clamp_to_limits(3, 10, 8);
    assert_eq!(state.count(), 3);
    comparisons += 1;

    // Money bottleneck (stock=10, money=2, space=8)
    state.set_count(20);
    state.clamp_to_limits(10, 2, 8);
    assert_eq!(state.count(), 2);
    comparisons += 1;

    // Space bottleneck (stock=10, money=10, space=1)
    state.set_count(20);
    state.clamp_to_limits(10, 10, 1);
    assert_eq!(state.count(), 1);
    comparisons += 1;

    // All zero
    state.set_count(5);
    state.clamp_to_limits(0, 0, 0);
    assert_eq!(state.count(), 0);
    comparisons += 1;

    assert!(comparisons > 0, "No assertions were checked");
}

#[test]
fn test_clamp_sell_and_bank() {
    let mut comparisons = 0;
    let mut state = CampUiState::new(CampScreen::Shop);

    // Selling: have 8 in bag
    state.set_count(25);
    state.clamp_sell(8);
    assert_eq!(state.count(), 8);
    comparisons += 1;

    state.set_count(3);
    state.clamp_sell(8);
    assert_eq!(state.count(), 3);
    comparisons += 1;

    state.clamp_sell(0);
    assert_eq!(state.count(), 0);
    comparisons += 1;

    // Bank deposit: have 12 in bag, bank space is 5
    state.set_count(20);
    state.clamp_deposit(12, 5);
    assert_eq!(state.count(), 5);
    comparisons += 1;

    // Bank withdraw: have 20 in vault, bag space is 7
    state.set_count(30);
    state.clamp_withdraw(20, 7);
    assert_eq!(state.count(), 7);
    comparisons += 1;

    // Status message
    state.set_status("Bought 3 Wood");
    assert_eq!(state.status(), "Bought 3 Wood");
    comparisons += 1;

    state.clear_status();
    assert!(state.status().is_empty());
    comparisons += 1;

    assert!(comparisons > 0, "No assertions were checked");
}
