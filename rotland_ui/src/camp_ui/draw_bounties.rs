//! egui 0.37 rendering for the Bounty Board screen and on-HUD Bounty Tracker.

use engine::egui::{self, pos2, vec2, Align2, Color32, CornerRadius, FontId, Frame, Margin, Pos2, RichText, ScrollArea, Stroke};
use game::CLEAR_POI;

use super::action::CampAction;
use super::state::CampUiState;
use super::view::CampView;

/// Draws the Bounty Board screen: Offered bounties on the left, Active bounties on the right.
pub fn draw_bounties(ui: &mut egui::Ui, _state: &mut CampUiState, view: &CampView) -> Option<CampAction> {
    let mut action = None;

    let manager = view.bounty_manager;
    let active_count = manager.map(|m| m.active().len()).unwrap_or(0);
    let max_active = manager.map(|m| m.max_active()).unwrap_or(2);
    let is_full = active_count >= max_active;

    ui.columns(2, |columns| {
        // --- Left column: Offered bounties ---
        let left = &mut columns[0];
        left.label(
            RichText::new("Offered (today and this week)")
                .size(16.0)
                .color(Color32::from_rgb(204, 217, 230))
                .strong(),
        );
        left.add_space(6.0);

        ScrollArea::vertical()
            .id_salt("bounties_offered_scroll")
            .max_height(380.0)
            .show(left, |ui| {
                if let Some(mgr) = manager {
                    let offered = mgr.offered();
                    if offered.is_empty() {
                        ui.label(RichText::new("No bounties offered today.").color(Color32::GRAY));
                    }
                    for bounty in offered {
                        let tag = if bounty.weekly { "Weekly" } else { "Today" };
                        let desc = format!("[{tag}] {}\n${} and {} XP", bounty.text, bounty.dollars, bounty.xp);

                        Frame::NONE
                            .fill(Color32::from_rgb(28, 30, 34))
                            .inner_margin(Margin::same(10))
                            .corner_radius(CornerRadius::same(6))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(50, 54, 60)))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(desc).size(14.0).color(Color32::WHITE));

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.add_enabled(!is_full, egui::Button::new("Take")).clicked() {
                                            action = Some(CampAction::Accept {
                                                bounty: bounty.id.clone(),
                                            });
                                        }
                                    });
                                });
                            });
                        ui.add_space(4.0);
                    }
                } else {
                    ui.label("Bounty board unavailable.");
                }
            });

        // --- Right column: Taken (active) bounties ---
        let right = &mut columns[1];
        right.label(
            RichText::new(format!("Taken ({active_count}/{max_active})"))
                .size(16.0)
                .color(Color32::from_rgb(204, 217, 230))
                .strong(),
        );
        right.add_space(6.0);

        ScrollArea::vertical()
            .id_salt("bounties_active_scroll")
            .max_height(380.0)
            .show(right, |ui| {
                if let Some(mgr) = manager {
                    let active = mgr.active();
                    if active.is_empty() {
                        ui.label(RichText::new("No active bounties.").color(Color32::GRAY));
                    }
                    for bounty in active {
                        let progress_text = if bounty.is_done() {
                            "Done. Claim it for your reward.".to_string()
                        } else if bounty.bounty_type == CLEAR_POI {
                            let dist = (view.player_pos.x - bounty.target_x)
                                .hypot(view.player_pos.z - bounty.target_z)
                                .round() as i32;
                            format!("Not cleared yet, {dist} m away")
                        } else {
                            format!("{}/{}", bounty.progress, bounty.target)
                        };

                        let desc = format!("{}\n{progress_text}", bounty.text);

                        Frame::NONE
                            .fill(Color32::from_rgb(28, 30, 34))
                            .inner_margin(Margin::same(10))
                            .corner_radius(CornerRadius::same(6))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(50, 54, 60)))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let text_color = if bounty.is_done() {
                                        Color32::from_rgb(143, 194, 74)
                                    } else {
                                        Color32::WHITE
                                    };
                                    ui.label(RichText::new(desc).size(14.0).color(text_color));

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("Abandon").clicked() {
                                            action = Some(CampAction::Abandon {
                                                bounty: bounty.id.clone(),
                                            });
                                        }
                                        if ui.add_enabled(bounty.is_done(), egui::Button::new("Claim")).clicked() {
                                            action = Some(CampAction::TurnIn {
                                                bounty: bounty.id.clone(),
                                            });
                                        }
                                    });
                                });
                            });
                        ui.add_space(4.0);
                    }
                } else {
                    ui.label("Bounty board unavailable.");
                }
            });
    });

    action
}

/// Draws the small on-HUD bounty tracker below the compass.
pub fn draw_bounty_tracker(ctx: &egui::Context, view: &CampView) {
    let Some(mgr) = view.bounty_manager else { return };
    let active = mgr.active();
    if active.is_empty() {
        return;
    }

    egui::Area::new(egui::Id::unique("bounty_tracker_hud"))
        .fixed_pos(pos2(0.0, 0.0))
        .interactable(false)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let painter = ui.painter();
            let screen = ctx.content_rect();
            let center_x = screen.center().x;
            let mut y = screen.min.y + 130.0;

            for bounty in active {
                let progress_suffix = if bounty.is_done() {
                    "(finished: return to the board)".to_string()
                } else if bounty.bounty_type == CLEAR_POI {
                    let dist = (view.player_pos.x - bounty.target_x)
                        .hypot(view.player_pos.z - bounty.target_z)
                        .round() as i32;
                    format!("({dist} m away)")
                } else {
                    format!("{}/{}", bounty.progress, bounty.target)
                };

                let line = format!("{}  {progress_suffix}", bounty.text);
                draw_outline_text(
                    painter,
                    pos2(center_x, y),
                    Align2::CENTER_TOP,
                    &line,
                    FontId::proportional(15.0),
                    Color32::from_rgb(255, 242, 191),
                );
                y += 20.0;
            }
        });
}

fn draw_outline_text(
    painter: &egui::Painter,
    pos: Pos2,
    anchor: Align2,
    text: &str,
    font_id: FontId,
    color: Color32,
) {
    let outline_color = Color32::from_rgba_unmultiplied(0, 0, 0, 230);
    let offsets = [
        vec2(-1.0, 0.0),
        vec2(1.0, 0.0),
        vec2(0.0, -1.0),
        vec2(0.0, 1.0),
        vec2(-1.0, -1.0),
        vec2(1.0, -1.0),
        vec2(-1.0, 1.0),
        vec2(1.0, 1.0),
    ];
    for offset in offsets {
        painter.text(pos + offset, anchor, text, font_id.clone(), outline_color);
    }
    painter.text(pos, anchor, text, font_id, color);
}
