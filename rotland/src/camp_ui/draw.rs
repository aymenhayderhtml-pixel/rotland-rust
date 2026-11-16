//! Central draw dispatcher and layout for camp screens (egui 0.37).

use engine::egui::{self, vec2, Align2, Color32, CornerRadius, Frame, Id, Margin, Order, RichText, Sense, Stroke};

use super::action::CampAction;
use super::draw_bank::draw_bank;
use super::draw_bounties::draw_bounties;
use super::draw_crafting::draw_crafting;
use super::draw_shop::draw_shop;
use super::screen::CampScreen;
use super::state::CampUiState;
use super::view::CampView;

pub const PANEL_WIDTH: f32 = 760.0;
pub const PANEL_HEIGHT: f32 = 600.0;

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

/// Central drawing function for the camp screens.
/// Returns an action if a button was clicked, or None if no action was triggered this frame.
pub fn draw(
    ctx: &egui::Context,
    screen: CampScreen,
    state: &mut CampUiState,
    view: &CampView,
) -> Option<CampAction> {
    // 1. Esc key closes any open camp screen
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        return Some(CampAction::Close);
    }

    // Sync state screen when caller requests a different screen
    if state.requested_screen != Some(screen) {
        state.requested_screen = Some(screen);
        state.set_screen(screen);
    }

    let mut action = None;

    // 2. Fullscreen dim background
    let screen_rect = ctx.content_rect();
    let bg_color = Color32::from_rgba_unmultiplied(0, 0, 0, 140);
    egui::Area::new(Id::unique("camp_background"))
        .fixed_pos(screen_rect.min)
        .order(Order::Foreground)
        .show(ctx, |ui| {
            ui.painter().rect_filled(screen_rect, 0.0, bg_color);
            ui.allocate_rect(screen_rect, Sense::click_and_drag());
        });

    // 3. Centered panel contents
    egui::Area::new(Id::unique("camp_content"))
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

                    // --- Header ---
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(state.screen.title())
                                .size(22.0)
                                .color(Color32::WHITE)
                                .strong(),
                        );

                        ui.add_space(16.0);

                        // Tab buttons
                        let tabs = [
                            CampScreen::Shop,
                            CampScreen::Bank,
                            CampScreen::Crafting,
                            CampScreen::Bounties,
                        ];
                        for tab in tabs {
                            let is_active = state.screen == tab;
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
                                state.switch_tab(tab);
                            }
                        }

                        // Dollars label on the right
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!("Your dollars: ${}", view.player_dollars()))
                                    .size(18.0)
                                    .color(Color32::from_rgb(255, 230, 102))
                                    .strong(),
                            );
                        });
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // --- Body ---
                    let body_action = match state.screen {
                        CampScreen::Shop => draw_shop(ui, state, view),
                        CampScreen::Bank => draw_bank(ui, state, view),
                        CampScreen::Crafting => draw_crafting(ui, state, view),
                        CampScreen::Bounties => draw_bounties(ui, state, view),
                    };

                    if let Some(act) = body_action {
                        action = Some(act);
                    }

                    ui.add_space(8.0);

                    // --- Status line ---
                    if !state.status().is_empty() {
                        ui.label(
                            RichText::new(state.status())
                                .size(15.0)
                                .color(Color32::from_rgb(217, 242, 217)),
                        );
                    } else {
                        ui.add_space(22.0);
                    }

                    ui.add_space(6.0);

                    // --- Footer ---
                    ui.horizontal(|ui| {
                        let close_btn = egui::Button::new(RichText::new("Close (Esc)").size(14.0))
                            .min_size(vec2(100.0, 34.0));
                        if ui.add(close_btn).clicked() {
                            action = Some(CampAction::Close);
                        }
                    });
                });
        });

    action
}
