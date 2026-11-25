//! Central draw dispatcher and layout for player hub and modal screens.

use engine::egui::{
    self, vec2, Align2, Color32, CornerRadius, Frame, Margin, Order, RichText, Sense, Stroke,
};

use super::action::HubAction;
use super::draw_bag::draw_bag;
use super::draw_bounties::draw_bounties;
use super::draw_dog::{draw_dog_menu, draw_dog_name_dialog, draw_dog_pack};
use super::draw_profile::draw_profile;
use super::draw_skills::draw_skills;
use super::draw_trunk::draw_trunk;
pub use super::draw_overlays::draw_overlays;
use super::screen::{HubScreen, HubTab};
use super::state::HubUiState;
use super::view::HubView;

pub const PANEL_WIDTH: f32 = 900.0;
pub const PANEL_HEIGHT: f32 = 620.0;

/// Parses a 6-hex-digit color string ("#RRGGBB" or "RRGGBB") into egui Color32.
pub fn parse_hex_color(hex: &str) -> Color32 {
    let s = hex.trim_start_matches('#');
    if s.len() == 6
        && let Ok(r) = u8::from_str_radix(&s[0..2], 16)
        && let Ok(g) = u8::from_str_radix(&s[2..4], 16)
        && let Ok(b) = u8::from_str_radix(&s[4..6], 16)
    {
        Color32::from_rgb(r, g, b)
    } else {
        Color32::from_rgb(120, 120, 120)
    }
}

/// Central drawing function for the player hub and related modal screens.
/// Returns an action if an interactive button was activated, or None otherwise.
pub fn draw(
    ctx: &egui::Context,
    state: &mut HubUiState,
    view: &HubView,
    icons: &mut crate::hud::IconCache,
) -> Option<HubAction> {
    // 1. Esc key closes any open hub screen
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        return Some(HubAction::Close);
    }

    // Modal screens dispatch directly
    match state.screen {
        HubScreen::Trunk => return draw_trunk(ctx, state, view),
        HubScreen::DogMenu => return draw_dog_menu(ctx, state, view),
        HubScreen::DogPack => return draw_dog_pack(ctx, state, view),
        HubScreen::DogNameDialog => return draw_dog_name_dialog(ctx, state, view),
        HubScreen::Tab(_) => {}
    }

    // 2. Q and E switch tabs while the main hub is open
    if ctx.input(|i| i.key_pressed(egui::Key::Q)) {
        state.prev_tab();
    }
    if ctx.input(|i| i.key_pressed(egui::Key::E)) {
        state.next_tab();
    }

    let mut action = None;

    // 3. Fullscreen dim background
    let screen_rect = ctx.content_rect();
    let bg_color = Color32::from_rgba_unmultiplied(0, 0, 0, 150);
    egui::Area::new(egui::Id::unique("hub_background"))
        .fixed_pos(screen_rect.min)
        .order(Order::Foreground)
        .show(ctx, |ui| {
            ui.painter().rect_filled(screen_rect, 0.0, bg_color);
            ui.allocate_rect(screen_rect, Sense::click_and_drag());
        });

    // 4. Centered panel frame
    egui::Area::new(egui::Id::unique("hub_content"))
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .order(Order::Foreground)
        .show(ctx, |ui| {
            Frame::NONE
                .fill(Color32::from_rgb(23, 26, 31))
                .inner_margin(Margin::same(14))
                .corner_radius(CornerRadius::same(8))
                .stroke(Stroke::new(2.0, Color32::from_rgb(89, 97, 107)))
                .show(ui, |ui| {
                    ui.set_width(PANEL_WIDTH);
                    ui.set_min_height(PANEL_HEIGHT - 40.0);

                    // --- Header: Tab Bar ---
                    ui.horizontal(|ui| {
                        for tab in HubTab::ALL {
                            let is_active = state.tab == tab;
                            let btn = egui::Button::new(
                                RichText::new(tab.tab_name())
                                    .size(14.0)
                                    .color(if is_active {
                                        Color32::from_rgb(143, 194, 74)
                                    } else {
                                        Color32::from_rgb(180, 185, 190)
                                    }),
                            );
                            if ui.add(btn).clicked() {
                                state.set_tab(tab);
                            }
                        }

                        // Player cash display on the right
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!("Your dollars: ${}", view.player_dollars()))
                                    .size(16.0)
                                    .strong()
                                    .color(Color32::from_rgb(255, 230, 102)),
                            );
                        });
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // --- Body Content ---
                    let body_action = match state.tab {
                        HubTab::Profile => draw_profile(ui, state, view),
                        HubTab::Bag => draw_bag(ui, state, view, icons),
                        HubTab::Skills => draw_skills(ui, state, view),
                        HubTab::Bounties => draw_bounties(ui, state, view),
                        HubTab::Map => match &view.map {
                            // The map fills the hub's content area (not more: inside an Area the space is unbounded)
                            Some(map) => match ui.allocate_ui(vec2(ui.available_width().min(1100.0), 560.0), |ui| crate::map_ui::draw_map(ui, &mut state.map_ui, map)).inner {
                                Some(crate::map_ui::MapAction::Close) => Some(HubAction::Close),
                                Some(a) => Some(HubAction::Map(a)),
                                None => None,
                            },
                            None => {
                                ui.label(RichText::new("No map yet").color(Color32::GRAY));
                                None
                            }
                        },
                        HubTab::Settings => {
                            ui.vertical_centered(|ui| {
                                ui.add_space(80.0);
                                ui.label(RichText::new("Settings").size(24.0).strong().color(Color32::WHITE));
                                ui.add_space(10.0);
                                ui.label(
                                    RichText::new("Game is paused while settings are open.")
                                        .size(14.0)
                                        .color(Color32::GRAY),
                                );
                            });
                            None
                        }
                    };

                    if let Some(act) = body_action {
                        action = Some(act);
                    }

                    ui.add_space(8.0);

                    // --- Status line ---
                    if !state.status().is_empty() {
                        ui.label(
                            RichText::new(state.status())
                                .size(14.0)
                                .color(Color32::from_rgb(217, 242, 217)),
                        );
                    } else {
                        ui.add_space(20.0);
                    }

                    ui.add_space(4.0);

                    // --- Footer: Hints and Close button ---
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(
                                "Q and E switch tabs. Tab or Esc closes. Bag: click to select, right-click to use, drag to move.",
                            )
                            .size(12.0)
                            .color(Color32::from_rgb(160, 165, 170)),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let close_btn = egui::Button::new(RichText::new("Close (Esc)").size(13.0))
                                .min_size(vec2(100.0, 30.0));
                            if ui.add(close_btn).clicked() {
                                action = Some(HubAction::Close);
                            }
                        });
                    });
                });
        });

    action
}
