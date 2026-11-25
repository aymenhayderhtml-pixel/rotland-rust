//! In-progress upgrade bar and cancellation controls for camp upgrades.

use engine::egui::{self, RichText, Vec2, vec2};

use crate::camp_ui::action::CampAction;
use crate::camp_ui::state::CampUiState;
use crate::camp_ui::view::{CampUpgradesView, UpgradeInProgressView};
use crate::menus::theme::{self, ButtonTone};

use super::colors::{
    cancel_button_label, format_hours, resolve_in_progress_name, upgrade_progress_fraction,
};
use super::detail::draw_bar;

pub const CANCEL_CONFIRM_KEY: &str = "confirm_cancel_upgrade";

pub fn draw_in_progress_body(
    ui: &mut egui::Ui,
    prog: &UpgradeInProgressView,
    upgrades: &CampUpgradesView,
    state: &mut CampUiState,
    width: f32,
    scale: f32,
) -> Option<CampAction> {
    let mut action = None;
    let progress = upgrade_progress_fraction(prog.hours_left, prog.total_hours);
    let pct = (progress * 100.0).round() as u32; // new: DECISIONS 2026-11-22
    let name = resolve_in_progress_name(prog, upgrades);

    ui.add_space(2.0 * scale);
    ui.horizontal(|ui| {
        theme::status_badge(ui, "BUILDING", theme::ACCENT, scale);
        ui.label(
            RichText::new(format!("{name} ({pct}%)"))
                .size(theme::FONT_HELP * scale)
                .color(theme::TEXT_PRIMARY)
                .strong(),
        );
    });

    ui.add_space(2.0 * scale);
    draw_bar(ui, vec2(width, 8.0 * scale), progress, theme::ACCENT, scale);

    ui.label(
        RichText::new(format!("{} remaining", format_hours(prog.hours_left)))
            .size(theme::FONT_HELP * scale)
            .color(theme::TEXT_MUTED),
    );

    ui.add_space(4.0 * scale);

    let btn_h = 30.0 * scale; // new: DECISIONS 2026-11-22
    let confirming = state.selected_row() == Some(CANCEL_CONFIRM_KEY);
    if confirming {
        ui.horizontal(|ui| {
            if theme::menu_button(
                ui,
                "Keep",
                vec2(width * 0.35, btn_h),
                scale,
                ButtonTone::Default,
                true,
            )
            .clicked()
            {
                state.clear_selection();
            }
            if theme::menu_button(
                ui,
                "Confirm Cancel",
                vec2(width * 0.60, btn_h),
                scale,
                ButtonTone::Danger,
                true,
            )
            .clicked()
            {
                state.clear_selection();
                action = Some(CampAction::CancelUpgrade);
            }
        });
    } else {
        let label = cancel_button_label(upgrades.refund_ratio);
        if theme::menu_button(ui, &label, vec2(width, btn_h), scale, ButtonTone::Danger, true).clicked() {
            state.select_row(CANCEL_CONFIRM_KEY);
        }
    }

    action
}
