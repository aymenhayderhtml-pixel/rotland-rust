//! Recruits section of the Guild screen (hired teammates and candidate hiring).

use engine::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke};
use game::guild::{GuildManager, GuildRecruit, RecruitOrder, RecruitRole, RecruitState};

use crate::camp_ui::action::GuildCampAction;
use crate::camp_ui::state::CampUiState;

/// Formats role badge label and color.
pub fn role_badge(role: RecruitRole) -> (&'static str, Color32) {
    match role {
        RecruitRole::Fighter => ("Fighter", Color32::from_rgb(220, 90, 80)),
        RecruitRole::Scavenger => ("Scavenger", Color32::from_rgb(90, 180, 220)),
        RecruitRole::Guard => ("Guard", Color32::from_rgb(140, 200, 90)),
    }
}

/// Formats state description for display.
pub fn state_label(state: RecruitState) -> &'static str {
    match state {
        RecruitState::IdleAtCamp => "Staying at Camp",
        RecruitState::FollowingPlayer => "Following",
        RecruitState::GuardingCamp => "Guarding Camp",
        RecruitState::Dead => "Fallen in Action",
    }
}

/// Draws the Recruits column with hired teammates and available candidates.
pub fn draw_recruits_column(
    ui: &mut egui::Ui,
    guild: &GuildManager,
    player_dollars: i32,
    _state: &mut CampUiState,
    max_height: f32,
) -> Option<GuildCampAction> {
    let mut action = None;
    let recruits = guild.recruits();
    let hired = recruits.hired();
    let available = recruits.available();
    let max_slots = guild.max_recruits();
    let can_hire_more = hired.len() < max_slots;

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Recruits")
                    .size(16.0)
                    .color(Color32::from_rgb(204, 217, 230))
                    .strong(),
            );
            ui.label(
                RichText::new(format!("({}/{} Slots)", hired.len(), max_slots))
                    .size(13.0)
                    .color(Color32::from_rgb(154, 152, 146)),
            );
        });

        ui.add_space(4.0);

        ScrollArea::vertical()
            .id_salt("guild_recruits_scroll")
            .max_height(max_height)
            .show(ui, |ui| {
                // --- Hired Teammates ---
                if !hired.is_empty() {
                    ui.label(
                        RichText::new("Hired Teammates")
                            .size(14.0)
                            .color(Color32::from_rgb(143, 194, 74))
                            .strong(),
                    );
                    ui.add_space(2.0);

                    for recruit in hired {
                        Frame::NONE
                            .fill(Color32::from_rgb(32, 35, 40))
                            .inner_margin(Margin::same(8))
                            .corner_radius(CornerRadius::same(6))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(60, 65, 71)))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&recruit.name).size(14.0).strong().color(Color32::WHITE));
                                    let (role_str, role_color) = role_badge(recruit.role);
                                    ui.label(RichText::new(role_str).size(12.0).color(role_color));
                                    ui.label(RichText::new(format!("Lv.{}", recruit.level)).size(12.0).color(Color32::from_rgb(180, 180, 180)));
                                });

                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("HP: {}/{}", recruit.health, recruit.max_health))
                                            .size(12.0)
                                            .color(if recruit.health < recruit.max_health / 3 {
                                                Color32::from_rgb(214, 80, 70)
                                            } else {
                                                Color32::from_rgb(180, 180, 180)
                                            }),
                                    );
                                    ui.label(
                                        RichText::new(format!("Wage: ${}/day", recruit.daily_wage))
                                            .size(12.0)
                                            .color(Color32::from_rgb(255, 230, 102)),
                                    );
                                });

                                ui.label(
                                    RichText::new(state_label(recruit.state))
                                        .size(11.0)
                                        .color(Color32::from_rgb(140, 190, 240)),
                                );

                                ui.add_space(2.0);

                                // Standing orders and Dismiss
                                ui.horizontal(|ui| {
                                    let is_stay = recruit.state == RecruitState::IdleAtCamp;
                                    let is_follow = recruit.state == RecruitState::FollowingPlayer;
                                    let is_guard = recruit.state == RecruitState::GuardingCamp;

                                    let stay_btn = egui::Button::new("Stay")
                                        .fill(if is_stay { Color32::from_rgb(50, 90, 50) } else { Color32::from_rgb(30, 32, 36) });
                                    if ui.add_enabled(recruit.is_alive(), stay_btn).clicked() {
                                        action = Some(GuildCampAction::SetRecruitOrder {
                                            recruit_id: recruit.id.clone(),
                                            order: RecruitOrder::Stay,
                                        });
                                    }

                                    let follow_btn = egui::Button::new("Follow")
                                        .fill(if is_follow { Color32::from_rgb(50, 90, 50) } else { Color32::from_rgb(30, 32, 36) });
                                    if ui.add_enabled(recruit.is_alive(), follow_btn).clicked() {
                                        action = Some(GuildCampAction::SetRecruitOrder {
                                            recruit_id: recruit.id.clone(),
                                            order: RecruitOrder::Follow,
                                        });
                                    }

                                    let guard_btn = egui::Button::new("Guard")
                                        .fill(if is_guard { Color32::from_rgb(50, 90, 50) } else { Color32::from_rgb(30, 32, 36) });
                                    if ui.add_enabled(recruit.is_alive(), guard_btn).clicked() {
                                        action = Some(GuildCampAction::SetRecruitOrder {
                                            recruit_id: recruit.id.clone(),
                                            order: RecruitOrder::Guard,
                                        });
                                    }

                                    let dismiss_btn = egui::Button::new(
                                        RichText::new("Dismiss")
                                            .size(12.0)
                                            .color(Color32::from_rgb(214, 80, 70)),
                                    );
                                    if ui.add(dismiss_btn).clicked() {
                                        action = Some(GuildCampAction::DismissRecruit {
                                            recruit_id: recruit.id.clone(),
                                        });
                                    }
                                });
                            });
                        ui.add_space(4.0);
                    }
                    ui.add_space(6.0);
                }

                // --- Available Candidates ---
                ui.label(
                    RichText::new("Available Recruits")
                        .size(14.0)
                        .color(Color32::from_rgb(220, 220, 225))
                        .strong(),
                );
                ui.add_space(2.0);

                if available.is_empty() {
                    ui.label(
                        RichText::new("No candidates looking for work today.")
                            .size(12.0)
                            .color(Color32::from_rgb(150, 150, 150)),
                    );
                }

                for candidate in available {
                    let can_afford = player_dollars >= candidate.hire_cost as i32;
                    let can_hire = can_hire_more && can_afford;

                    Frame::NONE
                        .fill(Color32::from_rgb(26, 28, 32))
                        .inner_margin(Margin::same(8))
                        .corner_radius(CornerRadius::same(6))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(50, 54, 60)))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&candidate.name).size(14.0).strong().color(Color32::WHITE));
                                let (role_str, role_color) = role_badge(candidate.role);
                                ui.label(RichText::new(role_str).size(12.0).color(role_color));
                                ui.label(RichText::new(format!("Lv.{}", candidate.level)).size(12.0).color(Color32::from_rgb(180, 180, 180)));
                            });

                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("HP: {}", candidate.max_health)).size(12.0).color(Color32::from_rgb(180, 180, 180)));
                                ui.label(RichText::new(format!("Wage: ${}/day", candidate.daily_wage)).size(12.0).color(Color32::from_rgb(255, 230, 102)));
                            });

                            ui.horizontal(|ui| {
                                let hire_btn = egui::Button::new(
                                    RichText::new(format!("Hire (${})", candidate.hire_cost))
                                        .color(if can_hire { Color32::WHITE } else { Color32::from_rgb(150, 150, 150) }),
                                );
                                if ui.add_enabled(can_hire, hire_btn).clicked() {
                                    action = Some(GuildCampAction::HireRecruit {
                                        recruit_id: candidate.id.clone(),
                                    });
                                }
                                if !can_hire_more {
                                    ui.label(RichText::new("(Slots full)").size(11.0).color(Color32::from_rgb(150, 150, 150)));
                                } else if !can_afford {
                                    ui.label(RichText::new("(Can't afford)").size(11.0).color(Color32::from_rgb(214, 80, 70)));
                                }
                            });
                        });
                    ui.add_space(4.0);
                }
            });
    });

    action
}
