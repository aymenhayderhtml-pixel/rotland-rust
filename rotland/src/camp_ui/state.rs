//! UI state for camp screens (tab switching, selected row, count steppers, status).

use super::screen::CampScreen;

/// UI state persisting across frames for camp screens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CampUiState {
    pub screen: CampScreen,
    pub requested_screen: Option<CampScreen>,
    pub selected_row: Option<String>,
    pub count: u32,
    pub status: String,
}

impl Default for CampUiState {
    fn default() -> Self {
        Self::new(CampScreen::Shop)
    }
}

impl CampUiState {
    /// Creates UI state initialized to the given screen.
    pub fn new(initial_screen: CampScreen) -> Self {
        Self {
            screen: initial_screen,
            requested_screen: Some(initial_screen),
            selected_row: None,
            count: 1,
            status: String::new(),
        }
    }

    // --- Tab switching ---

    pub fn screen(&self) -> CampScreen {
        self.screen
    }

    /// Sets the active screen, resetting row selection and count stepper.
    pub fn set_screen(&mut self, screen: CampScreen) {
        if self.screen != screen {
            self.screen = screen;
            self.selected_row = None;
            self.count = 1;
            self.status.clear();
        }
    }

    /// Alias for `set_screen`.
    pub fn switch_tab(&mut self, screen: CampScreen) {
        self.set_screen(screen);
    }

    // --- Selected row ---

    pub fn selected_row(&self) -> Option<&str> {
        self.selected_row.as_deref()
    }

    pub fn select_row(&mut self, id: impl Into<String>) {
        let s = id.into();
        if self.selected_row.as_deref() != Some(&s) {
            self.selected_row = Some(s);
            self.count = 1;
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_row = None;
        self.count = 1;
    }

    // --- Count steppers ---

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn set_count(&mut self, count: u32) {
        self.count = count;
    }

    pub fn step_up(&mut self, max: u32) {
        if max > 0 {
            self.count = (self.count.saturating_add(1)).clamp(1, max);
        } else {
            self.count = 0;
        }
    }

    pub fn step_down(&mut self) {
        if self.count > 1 {
            self.count -= 1;
        }
    }

    pub fn increment_count(&mut self, step: u32, max: u32) {
        if max > 0 {
            self.count = (self.count.saturating_add(step)).clamp(1, max);
        } else {
            self.count = 0;
        }
    }

    pub fn decrement_count(&mut self, step: u32) {
        self.count = self.count.saturating_sub(step).max(1);
    }

    pub fn clamp_count(&mut self, min: u32, max: u32) {
        if max < min {
            self.count = max;
        } else {
            self.count = self.count.clamp(min, max);
        }
    }

    /// Clamps count stepper to stock, money limit, and bag space limit.
    pub fn clamp_to_limits(&mut self, stock: u32, money_limit: u32, space_limit: u32) {
        let max = stock.min(money_limit).min(space_limit);
        if max == 0 {
            self.count = 0;
        } else {
            self.count = self.count.clamp(1, max);
        }
    }

    /// Clamps count for buying items: considers stock, dollars, price, and inventory room.
    pub fn clamp_buy(&mut self, stock: u32, dollars: i32, unit_price: i32, space: u32) {
        let money_limit = Self::max_affordable(dollars, unit_price);
        self.clamp_to_limits(stock, money_limit, space);
    }

    /// Clamps count for selling items: cannot sell more than the player has.
    pub fn clamp_sell(&mut self, have: u32) {
        if have == 0 {
            self.count = 0;
        } else {
            self.count = self.count.clamp(1, have);
        }
    }

    /// Clamps count for depositing items: cannot deposit more than in bag, nor more than bank can fit.
    pub fn clamp_deposit(&mut self, have_in_bag: u32, bank_space: u32) {
        let max = have_in_bag.min(bank_space);
        if max == 0 {
            self.count = 0;
        } else {
            self.count = self.count.clamp(1, max);
        }
    }

    /// Clamps count for withdrawing items: cannot withdraw more than in bank, nor more than bag can fit.
    pub fn clamp_withdraw(&mut self, have_in_bank: u32, bag_space: u32) {
        let max = have_in_bank.min(bag_space);
        if max == 0 {
            self.count = 0;
        } else {
            self.count = self.count.clamp(1, max);
        }
    }

    /// Maximum units affordable with dollars at unit_price.
    pub fn max_affordable(dollars: i32, unit_price: i32) -> u32 {
        if dollars <= 0 || unit_price <= 0 {
            0
        } else {
            (dollars as u32) / (unit_price as u32)
        }
    }

    // --- Status message ---

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status = msg.into();
    }

    pub fn clear_status(&mut self) {
        self.status.clear();
    }
}
