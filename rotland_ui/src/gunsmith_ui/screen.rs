//! Tab definitions, persistent UI state, screen wrapper, and responsive layout math.

use engine::egui::{self, Key, Rect, Vec2};

use crate::hud::IconCache;
use crate::menus::theme;

use super::action::{GunsmithAction, GunsmithSlot};
use super::{GunsmithView, draw};

/// Active tab inside the Gunsmith screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GunsmithTab {
    #[default]
    Buy,
    Sell,
    Repair,
    Upgrade,
}

impl GunsmithTab {
    pub const ALL: [Self; 4] = [Self::Buy, Self::Sell, Self::Repair, Self::Upgrade];

    pub fn label(self) -> &'static str {
        match self {
            Self::Buy => "1  Buy",
            Self::Sell => "2  Sell",
            Self::Repair => "3  Repair",
            Self::Upgrade => "4  Upgrade",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
            Self::Repair => "repair",
            Self::Upgrade => "upgrade",
        }
    }

    /// Parses a CLI/flag tab name such as `"buy"`, `"sell"`, `"repair"`, `"upgrade"`.
    pub fn parse_name(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "buy" => Some(Self::Buy),
            "sell" => Some(Self::Sell),
            "repair" => Some(Self::Repair),
            "upgrade" => Some(Self::Upgrade),
            _ => None,
        }
    }

    /// Maps number keys 1-4 to their corresponding gunsmith tab.
    pub fn from_key(key: Key) -> Option<Self> {
        match key {
            Key::Num1 => Some(Self::Buy),
            Key::Num2 => Some(Self::Sell),
            Key::Num3 => Some(Self::Repair),
            Key::Num4 => Some(Self::Upgrade),
            _ => None,
        }
    }
}

/// Persistent UI state across frames for the Gunsmith screen.
#[derive(Clone, Debug, PartialEq)]
pub struct GunsmithUiState {
    pub tab: GunsmithTab,
    pub selected_buy: Option<String>,
    pub selected_sell_slot: Option<usize>,
    pub selected_upgrade_slot: Option<usize>,
    pub selected_attach_slot: GunsmithSlot,
    pub hovered_mod: Option<String>,
    pub buy_count: u32,
    pub sell_count: u32,
    pub status: String,
    pub last_panel_rect: Option<Rect>,
}

impl Default for GunsmithUiState {
    fn default() -> Self {
        Self::new(GunsmithTab::Buy)
    }
}

impl GunsmithUiState {
    pub fn new(initial_tab: GunsmithTab) -> Self {
        Self {
            tab: initial_tab,
            selected_buy: None,
            selected_sell_slot: None,
            selected_upgrade_slot: None,
            selected_attach_slot: GunsmithSlot::Optic,
            hovered_mod: None,
            buy_count: 1,
            sell_count: 1,
            status: String::new(),
            last_panel_rect: None,
        }
    }

    pub fn tab(&self) -> GunsmithTab {
        self.tab
    }

    /// Switches tabs, clearing transient selection state and status text if the tab changed.
    pub fn switch_tab(&mut self, tab: GunsmithTab) {
        if self.tab != tab {
            self.tab = tab;
            self.selected_buy = None;
            self.selected_sell_slot = None;
            self.hovered_mod = None;
            self.buy_count = 1;
            self.sell_count = 1;
            self.status.clear();
        }
    }

    pub fn select_buy(&mut self, item_id: impl Into<String>) {
        let id = item_id.into();
        if self.selected_buy.as_deref() != Some(&id) {
            self.selected_buy = Some(id);
            self.buy_count = 1;
        }
    }

    pub fn select_sell(&mut self, slot: usize) {
        if self.selected_sell_slot != Some(slot) {
            self.selected_sell_slot = Some(slot);
            self.sell_count = 1;
        }
    }

    pub fn select_upgrade_weapon(&mut self, slot: usize) {
        if self.selected_upgrade_slot != Some(slot) {
            self.selected_upgrade_slot = Some(slot);
            self.hovered_mod = None;
        }
    }

    pub fn select_attach_slot(&mut self, slot: GunsmithSlot) {
        if self.selected_attach_slot != slot {
            self.selected_attach_slot = slot;
            self.hovered_mod = None;
        }
    }

    pub fn step_up_buy(&mut self, max: u32) {
        self.buy_count = if max > 0 { self.buy_count.saturating_add(1).clamp(1, max) } else { 0 };
    }

    pub fn step_down_buy(&mut self) {
        if self.buy_count > 1 {
            self.buy_count -= 1;
        }
    }

    pub fn clamp_buy(&mut self, max: u32) {
        self.buy_count = if max == 0 { 0 } else { self.buy_count.clamp(1, max) };
    }

    pub fn step_up_sell(&mut self, max: u32) {
        self.sell_count = if max > 0 { self.sell_count.saturating_add(1).clamp(1, max) } else { 0 };
    }

    pub fn step_down_sell(&mut self) {
        if self.sell_count > 1 {
            self.sell_count -= 1;
        }
    }

    pub fn clamp_sell(&mut self, max: u32) {
        self.sell_count = if max == 0 { 0 } else { self.sell_count.clamp(1, max) };
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status = msg.into();
    }

    pub fn clear_status(&mut self) {
        self.status.clear();
    }

    pub fn last_panel_rect(&self) -> Option<Rect> {
        self.last_panel_rect
    }
}

/// Stateful handle for the Gunsmith screen.
#[derive(Clone, Debug, Default)]
pub struct GunsmithScreen {
    pub state: GunsmithUiState,
}

impl GunsmithScreen {
    pub fn new(initial_tab: GunsmithTab) -> Self {
        Self { state: GunsmithUiState::new(initial_tab) }
    }

    /// Processes keyboard shortcuts (`Esc` to close, `1`-`4` to switch tabs).
    pub fn handle_key(&mut self, key: Key) -> Option<GunsmithAction> {
        if key == Key::Escape {
            return Some(GunsmithAction::Close);
        }
        if let Some(tab) = GunsmithTab::from_key(key) {
            self.state.switch_tab(tab);
        }
        None
    }

    /// Draws the gunsmith screen and returns any triggered action.
    pub fn draw(
        &mut self,
        ctx: &egui::Context,
        view: &GunsmithView,
        icons: Option<&mut IconCache>,
    ) -> Option<GunsmithAction> {
        draw(ctx, &mut self.state, view, icons)
    }
}

/// Computed responsive layout dimensions for the Gunsmith screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GunsmithLayout {
    pub scale: f32,
    pub panel_width: f32,
    pub panel_height: f32,
    pub inner_width: f32,
    pub inner_height: f32,
    pub header_height: f32,
    pub footer_height: f32,
    pub body_height: f32,
    pub buy_columns: usize,
    pub buy_gap: f32,
    pub buy_card_width: f32,
    pub upgrade_left_width: f32,
    pub upgrade_right_width: f32,
    pub upgrade_gap: f32,
    pub upgrade_slots_height: f32,
    pub upgrade_stats_height: f32,
}

impl GunsmithLayout {
    /// Computes all panel and sub-region bounds from `screen_rect` so nothing overflows.
    pub fn compute(screen_rect: Rect) -> Self {
        let w = if screen_rect.width().is_finite() { screen_rect.width().max(320.0) } else { 1280.0 };
        let h = if screen_rect.height().is_finite() { screen_rect.height().max(240.0) } else { 720.0 };

        let scale = theme::ui_scale(h);
        let margin_x = (24.0 * scale).min(w * 0.04).max(8.0);
        let margin_y = (16.0 * scale).min(h * 0.04).max(8.0);

        let panel_width = (w - margin_x * 2.0).clamp(300.0, 1240.0).min(w - 16.0);
        let panel_height = (h - margin_y * 2.0).clamp(220.0, 820.0).min(h - 16.0);
        let inner_width = (panel_width - 28.0).max(240.0);
        let inner_height = (panel_height - 28.0).max(180.0);

        let header_height = (44.0 * scale).clamp(40.0, 62.0);
        let footer_height = (48.0 * scale).clamp(42.0, 68.0);
        let body_height = (inner_height - header_height - footer_height - 30.0).max(120.0);

        let buy_gap = 8.0;
        let buy_avail = (inner_width - 24.0).max(200.0);
        let buy_columns = if buy_avail >= 920.0 { 4 } else if buy_avail >= 600.0 { 3 } else { 2 };
        let col_f = buy_columns as f32;
        let buy_card_width = ((buy_avail - (col_f - 1.0) * buy_gap) / col_f).floor().max(120.0);

        let upgrade_gap = 12.0;
        let upgrade_left_width = (inner_width * 0.28).clamp(200.0, 320.0);
        let upgrade_right_width = (inner_width - upgrade_left_width - upgrade_gap).max(220.0);
        let upgrade_stats_height = (146.0 * scale).clamp(138.0, 184.0).min(body_height * 0.42);
        let upgrade_slots_height = (body_height - upgrade_stats_height - 12.0).max(120.0);

        Self {
            scale,
            panel_width,
            panel_height,
            inner_width,
            inner_height,
            header_height,
            footer_height,
            body_height,
            buy_columns,
            buy_gap,
            buy_card_width,
            upgrade_left_width,
            upgrade_right_width,
            upgrade_gap,
            upgrade_slots_height,
            upgrade_stats_height,
        }
    }

    /// Returns true if the panel fits strictly inside `screen` and sub-regions do not overflow.
    pub fn fits_in(&self, screen: Vec2) -> bool {
        let panel_fits = self.panel_width <= screen.x + 1e-3 && self.panel_height <= screen.y + 1e-3;
        let vertical_fits =
            self.header_height + self.body_height + self.footer_height <= self.inner_height + 1e-3;
        let buy_total_w = (self.buy_columns as f32) * self.buy_card_width
            + (self.buy_columns.saturating_sub(1) as f32) * self.buy_gap;
        let buy_fits = buy_total_w <= self.inner_width + 1e-3;
        let upgrade_w_fits =
            self.upgrade_left_width + self.upgrade_gap + self.upgrade_right_width
                <= self.inner_width + 1e-3;
        let upgrade_h_fits =
            self.upgrade_slots_height + self.upgrade_stats_height <= self.body_height + 1e-3;

        panel_fits && vertical_fits && buy_fits && upgrade_w_fits && upgrade_h_fits
    }
}
