//! UI state for player hub and HUD overlays.

use super::screen::{HubScreen, HubTab};

pub const LEVEL_UP_BANNER_DURATION: f32 = 2.5;
pub const XP_TOAST_LIFETIME: f32 = 1.5;
pub const XP_TOAST_MERGE_WINDOW: f32 = 1.0;

/// State for the big centered "LEVEL UP" announcement banner on HUD.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelUpBannerState {
    pub text: String,
    pub time_left: f32,
}

/// One line in the "+25 Combat XP" stack on the HUD.
#[derive(Clone, Debug, PartialEq)]
pub struct XpToast {
    pub skill_id: String,
    pub skill_name: String,
    pub amount: f32,
    pub age: f32,
}

/// UI state persisting across frames for the hub and modal screens.
#[derive(Clone, Debug, PartialEq)]
pub struct HubUiState {
    pub screen: HubScreen,
    pub tab: HubTab,
    pub selected_slot: usize,
    pub dragged_slot: Option<usize>,
    pub split_count: u32,
    pub dog_name_input: String,
    pub status: String,
    pub level_up_banner: Option<LevelUpBannerState>,
    pub xp_toasts: Vec<XpToast>,
}

impl Default for HubUiState {
    fn default() -> Self {
        Self::new(HubScreen::Tab(HubTab::Bag))
    }
}

impl HubUiState {
    /// Creates UI state initialized to the given screen.
    pub fn new(initial_screen: HubScreen) -> Self {
        let initial_tab = initial_screen.tab().unwrap_or(HubTab::Bag);
        Self {
            screen: initial_screen,
            tab: initial_tab,
            selected_slot: 0,
            dragged_slot: None,
            split_count: 1,
            dog_name_input: "Rex".to_string(),
            status: String::new(),
            level_up_banner: None,
            xp_toasts: Vec::new(),
        }
    }

    // --- Screen and Tab management ---

    pub fn screen(&self) -> HubScreen {
        self.screen
    }

    pub fn set_screen(&mut self, screen: HubScreen) {
        if self.screen != screen {
            self.screen = screen;
            if let Some(t) = screen.tab() {
                self.tab = t;
            }
            self.cancel_drag();
            self.status.clear();
        }
    }

    pub fn tab(&self) -> HubTab {
        self.tab
    }

    /// Switches tab, resetting row selection and drag state.
    pub fn set_tab(&mut self, tab: HubTab) {
        self.tab = tab;
        self.screen = HubScreen::Tab(tab);
        self.cancel_drag();
        self.status.clear();
    }

    /// Cycles forward to the next tab (wraps around).
    pub fn next_tab(&mut self) {
        self.set_tab(self.tab.next());
    }

    /// Cycles backward to the previous tab (wraps around).
    pub fn prev_tab(&mut self) {
        self.set_tab(self.tab.prev());
    }

    // --- Slot Selection and Moves ---

    pub fn selected_slot(&self) -> usize {
        self.selected_slot
    }

    pub fn select_slot(&mut self, slot: usize) {
        self.selected_slot = slot;
        self.split_count = 1;
    }

    /// Updates selected slot index when items are moved or swapped.
    pub fn handle_slot_move(&mut self, from: usize, to: usize) {
        if self.selected_slot == from {
            self.selected_slot = to;
        }
    }

    // --- Drag and Drop ---

    pub fn is_dragging(&self) -> bool {
        self.dragged_slot.is_some()
    }

    pub fn dragged_slot(&self) -> Option<usize> {
        self.dragged_slot
    }

    pub fn start_drag(&mut self, slot: usize) {
        self.dragged_slot = Some(slot);
        self.selected_slot = slot;
    }

    pub fn drop_drag(&mut self, target_slot: usize) -> Option<(usize, usize)> {
        if let Some(from) = self.dragged_slot.take() {
            if from != target_slot {
                self.handle_slot_move(from, target_slot);
                return Some((from, target_slot));
            }
        }
        None
    }

    pub fn cancel_drag(&mut self) {
        self.dragged_slot = None;
    }

    // --- Split Count Stepper ---

    pub fn split_count(&self) -> u32 {
        self.split_count
    }

    pub fn set_split_count(&mut self, count: u32) {
        self.split_count = count.max(1);
    }

    pub fn clamp_split_count(&mut self, max: u32) {
        if max <= 1 {
            self.split_count = 1;
        } else {
            self.split_count = self.split_count.clamp(1, max);
        }
    }

    pub fn step_split_up(&mut self, max: u32) {
        if max > 0 {
            self.split_count = (self.split_count.saturating_add(1)).clamp(1, max);
        }
    }

    pub fn step_split_down(&mut self) {
        if self.split_count > 1 {
            self.split_count -= 1;
        }
    }

    // --- Dog name dialog input ---

    pub fn dog_name_input(&self) -> &str {
        &self.dog_name_input
    }

    pub fn set_dog_name_input(&mut self, name: impl Into<String>) {
        self.dog_name_input = name.into();
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

    // --- Overlays: Level Up Banner and XP Toasts ---

    /// Triggers the level up banner for a skill. Replaces any existing banner and restarts timer.
    pub fn trigger_level_up(&mut self, skill_name: &str, level: u32) {
        self.level_up_banner = Some(LevelUpBannerState {
            text: format!("{} LEVEL {}", skill_name.to_ascii_uppercase(), level),
            time_left: LEVEL_UP_BANNER_DURATION,
        });
    }

    /// Adds an XP gain toast. Merges into recent toast for same skill within 1.0s.
    pub fn add_xp_toast(&mut self, skill_id: &str, skill_name: &str, amount: f32) {
        if amount <= 0.0 {
            return;
        }
        for toast in &mut self.xp_toasts {
            if toast.skill_id == skill_id && toast.age < XP_TOAST_MERGE_WINDOW {
                toast.amount += amount;
                toast.age = 0.0;
                return;
            }
        }
        self.xp_toasts.push(XpToast {
            skill_id: skill_id.to_string(),
            skill_name: skill_name.to_string(),
            amount,
            age: 0.0,
        });
    }

    /// Advances timers for active level up banner and toasts.
    pub fn update_overlays(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        if let Some(banner) = &mut self.level_up_banner {
            banner.time_left -= dt;
            if banner.time_left <= 0.0 {
                self.level_up_banner = None;
            }
        }
        for toast in &mut self.xp_toasts {
            toast.age += dt;
        }
        self.xp_toasts.retain(|t| t.age < XP_TOAST_LIFETIME);
    }
}
