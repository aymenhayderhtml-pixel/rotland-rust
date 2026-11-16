//! egui 0.37 rendering for the Bounties tab in the player hub.

use engine::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke};
use game::CLEAR_POI;

use super::action::HubAction;
use super::state::HubUiState;
use super::view::HubView;

/// Draws the Bounties tab: read-only view of active and offered bounties with progress.
pub fn draw_bounties(
    ui: &mut egui::Ui,
    _state: &mut HubUiState,
    view: &HubView,
) -> Option<HubAction> {
    ui.label(RichText::new("Bounties").size(20.0).strong().color(Color32::WHITE));
    ui.label(
        RichText::new("Take and claim bounties at the bounty board. Here you see your progress.")
            .size(13.0)
            .color(Color32::from_rgb(180, 185, 190)),
    );
    ui.add_space(8.0);

    ScrollArea::vertical()
        .id_salt("hub_bounties_scroll")
        .max_height(480.0)
        .show(ui, |ui| {
            if let Some(manager) = view.bounty_manager {
                let active = manager.active();
                let offered = manager.offered();
                let max_active = manager.max_active();

                // Section 1: Taken Bounties
                ui.label(
                    RichText::new(format!("Taken ({}/{max_active})", active.len()))
                        .size(16.0)
                        .strong()
                        .color(Color32::from_rgb(204, 217, 230)),
                );
                ui.add_space(4.0);

                Frame::NONE
                    .fill(Color32::from_rgb(26, 28, 32))
                    .inner_margin(Margin::same(10))
                    .corner_radius(CornerRadius::same(6))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(45, 48, 55)))
                    .show(ui, |ui| {
                        ui.set_width(700.0);
                        if active.is_empty() {
                            ui.label(
                                RichText::new("No bounties taken yet.")
                                    .size(13.0)
                                    .color(Color32::GRAY),
                            );
                        } else {
                            for bounty in active {
                                ui.label(
                                    RichText::new(format!("- {}", bounty.text))
                                        .size(14.0)
                                        .color(Color32::WHITE),
                                );
                                let progress_text = if bounty.is_done() {
                                    "Done. Claim it at the board.".to_string()
                                } else if bounty.bounty_type == CLEAR_POI {
                                    "Not cleared yet".to_string()
                                } else {
                                    format!("Progress {}/{}", bounty.progress, bounty.target)
                                };
                                ui.label(
                                    RichText::new(format!("    {progress_text}"))
                                        .size(13.0)
                                        .color(if bounty.is_done() {
                                            Color32::from_rgb(143, 194, 74)
                                        } else {
                                            Color32::from_rgb(180, 200, 220)
                                        }),
                                );
                                ui.add_space(4.0);
                            }
                        }
                    });

                ui.add_space(14.0);

                // Section 2: Offered Bounties
                ui.label(
                    RichText::new("Offered (today and this week)")
                        .size(16.0)
                        .strong()
                        .color(Color32::from_rgb(204, 217, 230)),
                );
                ui.add_space(4.0);

                Frame::NONE
                    .fill(Color32::from_rgb(26, 28, 32))
                    .inner_margin(Margin::same(10))
                    .corner_radius(CornerRadius::same(6))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(45, 48, 55)))
                    .show(ui, |ui| {
                        ui.set_width(700.0);
                        if offered.is_empty() {
                            ui.label(
                                RichText::new("Nothing on the board right now.")
                                    .size(13.0)
                                    .color(Color32::GRAY),
                            );
                        } else {
                            for bounty in offered {
                                let tag = if bounty.weekly { "Weekly" } else { "Today" };
                                ui.label(
                                    RichText::new(format!("- [{tag}] {}", bounty.text))
                                        .size(14.0)
                                        .color(Color32::WHITE),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "    ${} and {} XP",
                                        bounty.dollars, bounty.xp
                                    ))
                                    .size(13.0)
                                    .color(Color32::from_rgb(255, 230, 102)),
                                );
                                ui.add_space(4.0);
                            }
                        }
                    });
            } else {
                ui.label(RichText::new("Bounty board unavailable.").size(14.0).color(Color32::GRAY));
            }
        });

    None
}
