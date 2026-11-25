//! Top header tabs and bottom footer buttons for the Camp Board screen.

use engine::egui::{self, RichText, Vec2, vec2};

use crate::camp_ui::action::CampAction;
use crate::camp_ui::screen::CampScreen;
use crate::camp_ui::state::CampUiState;
use crate::camp_ui::view::CampView;
use crate::menus::theme::{self, ButtonTone};

pub const HEADER_H: f32 = 38.0; // moved: rotland/src/camp_ui/draw_upgrades.rs
pub const FOOTER_H: f32 = 36.0; // moved: rotland/src/camp_ui/draw_upgrades.rs
pub const TAB_WIDTH: f32 = 96.0; // moved: rotland/src/camp_ui/draw_upgrades.rs

pub fn draw_header(
    ui: &mut egui::Ui,
    state: &mut CampUiState,
    cash: i32,
    width: f32,
    scale: f32,
) {
    let tabs = [
        CampScreen::Shop,
        CampScreen::Bank,
        CampScreen::Crafting,
        CampScreen::Bounties,
        CampScreen::Upgrades,
    ];
    let tab_w = (TAB_WIDTH * scale).min((width * 0.46) / (tabs.len() as f32));

    ui.allocate_ui(vec2(width, HEADER_H * scale), |ui| {
        ui.set_min_height(HEADER_H * scale);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(CampScreen::Upgrades.title())
                    .size(theme::FONT_HEADING * scale)
                    .color(theme::TEXT_PRIMARY)
                    .strong(),
            );
            ui.add_space(14.0 * scale);

            for tab in tabs {
                let selected = state.screen == tab;
                if theme::tab_button(ui, tab.tab_name(), tab_w, scale, selected).clicked() {
                    state.switch_tab(tab);
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("${cash}"))
                        .size(theme::FONT_SUBHEADING * scale)
                        .color(theme::ACCENT_HOVER)
                        .strong(),
                );
                ui.label(
                    RichText::new("CASH")
                        .size(theme::FONT_HELP * scale)
                        .color(theme::TEXT_MUTED)
                        .strong(),
                );
            });
        });
    });
}

pub fn draw_footer(
    ui: &mut egui::Ui,
    view: &CampView,
    state: &mut CampUiState,
    width: f32,
    scale: f32,
) -> Option<CampAction> {
    let mut action = None;

    ui.allocate_ui(vec2(width, FOOTER_H * scale), |ui| {
        ui.set_min_height(FOOTER_H * scale);
        ui.horizontal(|ui| {
            let close_btn = theme::menu_button(
                ui,
                "Close (Esc)",
                vec2(144.0 * scale, FOOTER_H * scale),
                scale,
                ButtonTone::Default,
                true,
            );
            if close_btn.clicked() {
                action = Some(CampAction::Close);
            }

            if let Some(upgrades) = &view.upgrades {
                if upgrades.has_doctor {
                    ui.add_space(8.0 * scale);
                    let doc_btn = theme::menu_button(
                        ui,
                        "Doctor: Heal ($30)",
                        vec2(160.0 * scale, FOOTER_H * scale),
                        scale,
                        ButtonTone::Safe,
                        true,
                    );
                    if doc_btn.clicked() {
                        action = Some(CampAction::DoctorHeal);
                    }
                }
                if upgrades.has_mechanic {
                    ui.add_space(8.0 * scale);
                    let mech_btn = theme::menu_button(
                        ui,
                        "Mechanic: Repair ($50)",
                        vec2(180.0 * scale, FOOTER_H * scale),
                        scale,
                        ButtonTone::Primary,
                        true,
                    );
                    if mech_btn.clicked() {
                        action = Some(CampAction::MechanicRepair);
                    }
                }
            }

            if !state.status().is_empty() {
                ui.add_space(14.0 * scale);
                ui.label(
                    RichText::new(state.status())
                        .size(theme::FONT_BODY * scale)
                        .color(theme::SAFE_HOVER),
                );
            }
        });
    });

    action
}
