//! egui 0.37 rendering for the Profile tab in the player hub.

use engine::egui::{
    self, pos2, vec2, Align2, Color32, CornerRadius, FontId, Grid, RichText,
    ScrollArea, Stroke,
};

use super::action::HubAction;
use super::state::HubUiState;
use super::view::HubView;

pub const PORTRAIT_WIDTH: f32 = 200.0;
pub const PORTRAIT_HEIGHT: f32 = 260.0;

/// Draws the Profile tab: portrait box on the left, lifetime stats and difficulty options on the right.
pub fn draw_profile(
    ui: &mut egui::Ui,
    _state: &mut HubUiState,
    view: &HubView,
) -> Option<HubAction> {
    let mut action = None;

    ui.horizontal(|ui| {
        // --- Left: Portrait Box ---
        ui.vertical(|ui| {
            ui.set_width(PORTRAIT_WIDTH);
            let (rect, _) = ui.allocate_exact_size(
                vec2(PORTRAIT_WIDTH, PORTRAIT_HEIGHT),
                egui::Sense::hover(),
            );

            let painter = ui.painter();
            // Outer frame
            painter.rect(
                rect,
                CornerRadius::same(6),
                Color32::from_rgb(26, 29, 36),
                Stroke::new(2.0, Color32::from_rgb(60, 65, 75)),
                engine::egui::StrokeKind::Inside,
            );

            // Silhouette placeholder box
            let head_center = pos2(rect.center().x, rect.min.y + 70.0);
            painter.circle_filled(head_center, 30.0, Color32::from_rgb(45, 50, 62));

            let shoulders_rect = engine::egui::Rect::from_min_max(
                pos2(rect.center().x - 55.0, rect.min.y + 110.0),
                pos2(rect.center().x + 55.0, rect.min.y + 200.0),
            );
            painter.rect_filled(
                shoulders_rect,
                CornerRadius::same(8),
                Color32::from_rgb(45, 50, 62),
            );

            painter.text(
                pos2(rect.center().x, rect.max.y - 20.0),
                Align2::CENTER_CENTER,
                "Survivor",
                FontId::proportional(14.0),
                Color32::from_rgb(180, 185, 195),
            );
        });

        ui.add_space(16.0);

        // --- Right: Lifetime Stats and Difficulty ---
        ui.vertical(|ui| {
            ui.set_width(500.0);
            ui.label(RichText::new("Profile").size(20.0).strong().color(Color32::WHITE));
            ui.add_space(6.0);

            ScrollArea::vertical()
                .id_salt("hub_profile_scroll")
                .max_height(480.0)
                .show(ui, |ui| {
                    Grid::new("profile_stats_grid")
                        .spacing(vec2(16.0, 6.0))
                        .striped(true)
                        .show(ui, |ui| {
                            add_stat_row(ui, "Days survived", &view.days_survived.to_string());
                            add_stat_row(
                                ui,
                                "Play time",
                                &HubView::format_play_time(view.play_time_seconds),
                            );
                            add_stat_row(ui, "Zombies killed", &view.kills.to_string());

                            let by_type_text = view
                                .kills_by_type
                                .map(HubView::format_by_type)
                                .unwrap_or_else(|| "None yet".to_string());
                            add_stat_row(ui, "By type", &by_type_text);

                            add_stat_row(ui, "Headshots", &view.headshots.to_string());
                            add_stat_row(ui, "Deaths", &view.deaths.to_string());
                            add_stat_row(
                                ui,
                                "Distance walked",
                                &HubView::format_distance(view.distance_metres),
                            );
                            add_stat_row(
                                ui,
                                "Dollars carried",
                                &format!("${}", view.player_dollars()),
                            );
                            add_stat_row(
                                ui,
                                "Bank balance",
                                &format!("${}", view.bank_dollars()),
                            );

                            let best_skill = match (view.skills, view.skill_registry) {
                                (Some(s), Some(r)) => HubView::format_best_skill(s, r),
                                _ => "-".to_string(),
                            };
                            add_stat_row(ui, "Highest skill", &best_skill);

                            let dog_text = HubView::format_dog(view.dog.as_ref());
                            add_stat_row(ui, "Dog", &dog_text);

                            let dogs_lost_text = view
                                .dogs_lost
                                .map(|n| n.to_string())
                                .unwrap_or_else(|| "-".to_string());
                            add_stat_row(ui, "Dogs lost", &dogs_lost_text);
                        });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // Difficulty controls
                    ui.label(
                        RichText::new("Difficulty settings")
                            .size(15.0)
                            .strong()
                            .color(Color32::from_rgb(204, 217, 230)),
                    );
                    ui.add_space(4.0);

                    if let (Some(settings), Some(cfg)) =
                        (view.difficulty_settings, view.difficulty_config)
                    {
                        Grid::new("profile_difficulty_grid")
                            .spacing(vec2(16.0, 6.0))
                            .show(ui, |ui| {
                                add_stat_row(ui, "Difficulty", &settings.label(cfg));
                                add_stat_row(
                                    ui,
                                    "Death rule (fixed)",
                                    cfg.death_rule_name(settings.death),
                                );

                                // Zombie amount stepper
                                ui.label(
                                    RichText::new("Zombie amount")
                                        .size(14.0)
                                        .color(Color32::from_rgb(180, 185, 190)),
                                );
                                ui.horizontal(|ui| {
                                    if ui.button("-").clicked() {
                                        let new_val = (settings.zombie_amount - cfg.step)
                                            .clamp(cfg.min_multiplier, cfg.max_multiplier);
                                        action = Some(HubAction::SetDifficulty {
                                            zombies: Some(new_val),
                                            loot: None,
                                        });
                                    }
                                    ui.label(format!("{:.2}x", settings.zombie_amount));
                                    if ui.button("+").clicked() {
                                        let new_val = (settings.zombie_amount + cfg.step)
                                            .clamp(cfg.min_multiplier, cfg.max_multiplier);
                                        action = Some(HubAction::SetDifficulty {
                                            zombies: Some(new_val),
                                            loot: None,
                                        });
                                    }
                                });
                                ui.end_row();

                                // Loot amount stepper
                                ui.label(
                                    RichText::new("Loot amount")
                                        .size(14.0)
                                        .color(Color32::from_rgb(180, 185, 190)),
                                );
                                ui.horizontal(|ui| {
                                    if ui.button("-").clicked() {
                                        let new_val = (settings.loot_amount - cfg.step)
                                            .clamp(cfg.min_multiplier, cfg.max_multiplier);
                                        action = Some(HubAction::SetDifficulty {
                                            zombies: None,
                                            loot: Some(new_val),
                                        });
                                    }
                                    ui.label(format!("{:.2}x", settings.loot_amount));
                                    if ui.button("+").clicked() {
                                        let new_val = (settings.loot_amount + cfg.step)
                                            .clamp(cfg.min_multiplier, cfg.max_multiplier);
                                        action = Some(HubAction::SetDifficulty {
                                            zombies: None,
                                            loot: Some(new_val),
                                        });
                                    }
                                });
                                ui.end_row();
                            });
                    }
                });
        });
    });

    action
}

fn add_stat_row(ui: &mut egui::Ui, name: &str, value: &str) {
    ui.label(RichText::new(name).size(14.0).color(Color32::from_rgb(180, 185, 190)));
    ui.label(RichText::new(value).size(14.0).strong().color(Color32::WHITE));
    ui.end_row();
}
