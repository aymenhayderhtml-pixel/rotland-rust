//! egui 0.37 rendering for the Camp Board (`CampScreen::Upgrades`) using the shared menu theme.

mod card;
mod chrome;
mod colors;
mod detail;
mod in_progress;
mod layout;

#[cfg(test)]
mod tests;

use engine::egui::{
    self, Align2, Frame, Id, Margin, Order, RichText, Sense, Stroke, Vec2, vec2,
};
use game::camp_upgrades::UpgradeLine;

use crate::camp_ui::action::CampAction;
use crate::camp_ui::state::CampUiState;
use crate::camp_ui::view::{CampUpgradesView, CampView, UpgradeLineView};
use crate::menus::theme;

pub use card::draw_upgrade_card;
pub use chrome::{draw_footer, draw_header, FOOTER_H, HEADER_H, TAB_WIDTH};
pub use colors::*;
pub use detail::*;
pub use in_progress::{draw_in_progress_body, CANCEL_CONFIRM_KEY};
pub use layout::{card_layout, panel_dimensions, CardLayout};

/// Renders the Camp Board screen using the shared menu theme and responsive layout.
pub fn draw_upgrades_screen(
    ctx: &egui::Context,
    state: &mut CampUiState,
    view: &CampView,
) -> Option<CampAction> {
    let screen_rect = ctx.content_rect();
    let scale = theme::ui_scale(screen_rect.height());

    theme::with_menu_style(ctx, scale, || {
        let mut action = None;

        egui::Area::new(Id::unique("camp_upgrades_bg"))
            .fixed_pos(screen_rect.min)
            .order(Order::Foreground)
            .show(ctx, |ui| {
                ui.painter().rect_filled(screen_rect, 0.0, theme::SETTINGS_DIM);
                ui.allocate_rect(screen_rect, Sense::click_and_drag());
            });

        let panel_size = panel_dimensions(screen_rect, scale);
        let pad = (12.0 * scale).round().clamp(6.0, 20.0) as i8; // new: DECISIONS 2026-11-22
        let inner_size = vec2(
            (panel_size.x - 2.0 * f32::from(pad)).max(240.0),
            (panel_size.y - 2.0 * f32::from(pad)).max(240.0),
        );

        egui::Area::new(Id::unique("camp_upgrades_panel"))
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .order(Order::Foreground)
            .show(ctx, |ui| {
                let frame = Frame::NONE
                    .fill(theme::PANEL_BG)
                    .inner_margin(Margin::same(pad))
                    .corner_radius(theme::radius(theme::CORNER_PANEL, scale))
                    .stroke(Stroke::new(1.0, theme::BORDER));

                let resp = frame.show(ui, |ui| {
                    ui.set_width(inner_size.x);
                    ui.set_min_height(inner_size.y);
                    ui.set_max_height(inner_size.y);
                    ui.spacing_mut().item_spacing = vec2(0.0, 6.0 * scale);

                    action = draw_board_contents(ui, state, view, inner_size, scale);
                });

                theme::paint_top_accent(ui, resp.response.rect, theme::ACCENT, scale);
            });

        action
    })
}

fn draw_board_contents(
    ui: &mut egui::Ui,
    state: &mut CampUiState,
    view: &CampView,
    size: Vec2,
    scale: f32,
) -> Option<CampAction> {
    let mut action = None;
    let cash = view
        .upgrades
        .as_ref()
        .map_or_else(|| view.player_dollars(), |u| u.player_cash);

    // 1. Header with tabs and cash
    draw_header(ui, state, cash, size.x, scale);
    ui.add_space(2.0 * scale);
    theme::paint_divider(ui, size.x, scale);
    ui.add_space(4.0 * scale);

    // 2. Main card area sized responsively
    let footer_space = (FOOTER_H + 10.0) * scale; // new: DECISIONS 2026-11-22
    let avail_for_cards = vec2(
        ui.available_width(),
        (ui.available_height() - footer_space).max(160.0),
    );

    if let Some(_upgrades) = &view.upgrades {
        let layout = card_layout(avail_for_cards);

        // Each card gets its own fixed rect and a top-down child ui: inside a left-to-right row the
        // card's contents would otherwise flow sideways (text wrapped one letter per line).
        let origin = ui.cursor().min;
        for (i, line_id) in UpgradeLine::ALL.into_iter().enumerate() {
            let (col, row) = (i % layout.columns, i / layout.columns);
            let min = origin
                + vec2(
                    col as f32 * (layout.card_width + layout.gap),
                    row as f32 * (layout.card_height + layout.gap),
                );
            let rect = egui::Rect::from_min_size(min, layout.card_size());
            let mut card_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            card_ui.set_clip_rect(rect.intersect(ui.clip_rect()));
            if let Some(act) = draw_line_card(&mut card_ui, line_id, view, state, &layout, scale) {
                action = Some(act);
            }
        }
        ui.allocate_space(vec2(layout.total_width(), layout.total_height()));
    } else {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new("Camp upgrade data unavailable.")
                    .size(theme::FONT_BODY * scale)
                    .color(theme::TEXT_MUTED),
            );
        });
    }

    ui.add_space(6.0 * scale);

    // 3. Footer with Close, Doctor, Mechanic, status
    if let Some(footer_act) = draw_footer(ui, view, state, size.x, scale) {
        action = Some(footer_act);
    }

    action
}

fn draw_line_card(
    ui: &mut egui::Ui,
    line_id: UpgradeLine,
    view: &CampView,
    state: &mut CampUiState,
    layout: &CardLayout,
    scale: f32,
) -> Option<CampAction> {
    let upgrades = view.upgrades.as_ref()?;
    if let Some(line_view) = upgrades.line(line_id) {
        draw_upgrade_card(ui, line_view, view, state, layout, scale)
    } else {
        let fallback = UpgradeLineView {
            line: line_id,
            current_level: 0,
            max_level: 0,
            current_summary: String::new(),
            next: None,
        };
        draw_upgrade_card(ui, &fallback, view, state, layout, scale)
    }
}
