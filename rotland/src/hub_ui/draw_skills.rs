//! egui 0.37 rendering for the Skills tab in the player hub.

use engine::egui::{self, Color32, CornerRadius, Frame, Margin, ProgressBar, RichText, ScrollArea, Stroke};

use super::action::HubAction;
use super::state::HubUiState;
use super::view::HubView;

/// Draws the Skills tab: list of skills, levels, progress bars, and XP text.
pub fn draw_skills(
    ui: &mut egui::Ui,
    _state: &mut HubUiState,
    view: &HubView,
) -> Option<HubAction> {
    ui.label(RichText::new("Skills").size(20.0).strong().color(Color32::WHITE));
    ui.add_space(8.0);

    ScrollArea::vertical()
        .id_salt("hub_skills_scroll")
        .max_height(480.0)
        .show(ui, |ui| {
            if let (Some(skills), Some(registry)) = (view.skills, view.skill_registry) {
                for def in registry.all() {
                    let level = skills.get_level(&def.id);
                    let xp = skills.get_xp(&def.id);
                    let to_next = skills.get_xp_to_next(&def.id);
                    let is_max = level >= def.max_level || to_next == 0;

                    Frame::NONE
                        .fill(Color32::from_rgb(26, 28, 32))
                        .inner_margin(Margin::same(10))
                        .corner_radius(CornerRadius::same(6))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(45, 48, 55)))
                        .show(ui, |ui| {
                            ui.set_width(700.0);

                            // Header: Skill name and Level
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(&def.name)
                                        .size(15.0)
                                        .strong()
                                        .color(Color32::WHITE),
                                );
                                ui.label(
                                    RichText::new(format!("Level {level}"))
                                        .size(15.0)
                                        .strong()
                                        .color(Color32::from_rgb(255, 230, 102)),
                                );

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let xp_label = if is_max {
                                        "MAX LEVEL".to_string()
                                    } else {
                                        format!("{:.0} / {to_next} XP", xp.floor())
                                    };
                                    ui.label(
                                        RichText::new(xp_label)
                                            .size(13.0)
                                            .color(Color32::from_rgb(180, 185, 190)),
                                    );
                                });
                            });

                            ui.add_space(4.0);

                            // Progress Bar
                            let progress = if is_max {
                                1.0
                            } else {
                                (xp / (to_next as f32).max(1.0)).clamp(0.0, 1.0)
                            };
                            let bar = ProgressBar::new(progress)
                                .desired_width(680.0)
                                .desired_height(12.0)
                                .fill(Color32::from_rgb(143, 194, 74));
                            ui.add(bar);
                        });

                    ui.add_space(6.0);
                }
            } else {
                ui.label(RichText::new("Skill definitions unavailable.").size(14.0).color(Color32::GRAY));
            }
        });

    None
}
