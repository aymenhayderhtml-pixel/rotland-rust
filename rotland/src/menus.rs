//! Pure menu state machine and transitions for Main, SlotPicker, Settings, and Paused screens.

use std::path::PathBuf;
use engine::egui;

use crate::settings::Settings;

pub mod menus_draw;
#[cfg(test)]
mod menus_tests;

/// State of a save slot file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SlotState {
    #[default]
    Empty,
    Damaged,
    Filled,
}

/// Metadata summary of one save slot shown in the SlotPicker cards.
#[derive(Debug, Clone, PartialEq)]
pub struct SlotSummary {
    pub state: SlotState,
    pub day: i32,
    pub play_time_s: f32,
    pub difficulty: String,
    pub saved_at: String,
}

impl Default for SlotSummary {
    fn default() -> Self {
        Self {
            state: SlotState::Empty,
            day: 0,
            play_time_s: 0.0,
            difficulty: String::new(),
            saved_at: String::new(),
        }
    }
}

/// How the slot picker was opened from the main menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotMode {
    Continue,
    NewGame,
}

/// The currently active menu screen.
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Main,
    SlotPicker { mode: SlotMode },
    Settings { from: Box<Screen> },
    Paused,
    None,
}

/// Actions chosen by the player in the menus.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    NewGame { slot: u8 },
    Continue { slot: u8 },
    DeleteSlot { slot: u8 },
    Resume,
    SaveAndQuitToMenu,
    Quit,
    SettingsChanged(Settings),
}

/// Menu state machine controlling current screen, settings, and slot selection.
pub struct Menus {
    pub screen: Screen,
    pub settings: Settings,
    pub pending_delete: Option<u8>,
    pub save_path: Option<PathBuf>,
}

impl Menus {
    /// Creates menus starting on the Main menu.
    pub fn new(settings: Settings) -> Self {
        Self {
            screen: Screen::Main,
            settings,
            pending_delete: None,
            save_path: default_save_path(),
        }
    }

    /// Creates menus for gameplay (starts closed: Screen::None).
    pub fn in_game(settings: Settings) -> Self {
        Self {
            screen: Screen::None,
            settings,
            pending_delete: None,
            save_path: default_save_path(),
        }
    }

    pub fn with_save_path(mut self, path: PathBuf) -> Self {
        self.save_path = Some(path);
        self
    }

    pub fn set_save_path(&mut self, path: PathBuf) {
        self.save_path = Some(path);
    }

    pub fn screen(&self) -> &Screen {
        &self.screen
    }

    pub fn set_screen(&mut self, screen: Screen) {
        self.screen = screen;
        self.pending_delete = None;
    }

    /// True while any menu is open (screen is not Screen::None).
    pub fn is_open(&self) -> bool {
        self.screen != Screen::None
    }

    /// True when the mouse cursor should be free and visible.
    pub fn wants_cursor(&self) -> bool {
        self.is_open()
    }

    /// Handles Escape key transitions:
    /// - If a delete confirmation is open, cancels the prompt.
    /// - In game (Screen::None) -> Paused
    /// - In Paused -> Resume (Screen::None)
    /// - In Settings -> Returns to the previous screen and saves settings
    /// - In SlotPicker -> Returns to Main menu
    /// - In Main menu -> Stays on Main menu
    pub fn on_escape(&mut self) {
        if self.pending_delete.is_some() {
            self.pending_delete = None;
            return;
        }

        match &self.screen {
            Screen::None => {
                self.screen = Screen::Paused;
            }
            Screen::Paused => {
                self.screen = Screen::None;
            }
            Screen::Settings { from } => {
                let prev = *from.clone();
                self.save_settings();
                self.screen = prev;
            }
            Screen::SlotPicker { .. } => {
                self.screen = Screen::Main;
            }
            Screen::Main => {}
        }
    }

    /// Returns true if at least one save slot is filled.
    pub fn any_slot_filled(slots: &[SlotSummary; 3]) -> bool {
        slots.iter().any(|s| s.state == SlotState::Filled)
    }

    /// Opens SlotPicker if any save slot is filled, returning true if transitioned.
    pub fn press_continue(&mut self, slots: &[SlotSummary; 3]) -> bool {
        if Self::any_slot_filled(slots) {
            self.screen = Screen::SlotPicker {
                mode: SlotMode::Continue,
            };
            true
        } else {
            false
        }
    }

    /// Opens SlotPicker in NewGame mode.
    pub fn press_new_game(&mut self) {
        self.screen = Screen::SlotPicker {
            mode: SlotMode::NewGame,
        };
    }

    /// Opens Settings, recording the current screen as the return destination.
    pub fn press_settings(&mut self) {
        let from = Box::new(self.screen.clone());
        self.screen = Screen::Settings { from };
    }

    /// Returns from Settings to the previous screen and persists settings to disk.
    pub fn press_back(&mut self) {
        if let Screen::Settings { from } = &self.screen {
            let prev = *from.clone();
            self.save_settings();
            self.screen = prev;
        }
    }

    /// Opens delete confirmation dialog for the given slot.
    pub fn request_delete_slot(&mut self, slot: u8) {
        self.pending_delete = Some(slot);
    }

    /// Confirms slot deletion, returning `MenuAction::DeleteSlot`.
    pub fn confirm_delete(&mut self) -> Option<MenuAction> {
        let slot = self.pending_delete.take()?;
        Some(MenuAction::DeleteSlot { slot })
    }

    /// Cancels slot deletion dialog.
    pub fn cancel_delete(&mut self) {
        self.pending_delete = None;
    }

    /// True while the delete confirmation dialog is visible.
    pub fn is_confirm_delete_open(&self) -> bool {
        self.pending_delete.is_some()
    }

    /// Saves settings to the configured save path (never panics).
    pub fn save_settings(&self) -> bool {
        if let Some(path) = &self.save_path {
            self.settings.save(path)
        } else {
            false
        }
    }

    /// Draws the current screen using egui and returns any action picked by the player.
    pub fn ui(&mut self, ctx: &egui::Context, slots: &[SlotSummary; 3]) -> Option<MenuAction> {
        if !self.is_open() {
            return None;
        }
        menus_draw::draw(self, ctx, slots)
    }
}

fn default_save_path() -> Option<PathBuf> {
    #[cfg(test)]
    {
        None
    }
    #[cfg(not(test))]
    {
        Some(crate::settings::settings_path())
    }
}
