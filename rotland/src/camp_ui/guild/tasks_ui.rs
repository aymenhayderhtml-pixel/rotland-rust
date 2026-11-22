//! Task board section of the Guild screen (offered and active tasks).

use engine::egui::{self, Color32, CornerRadius, Frame, Margin, ProgressBar, RichText, ScrollArea, Stroke};
use game::guild::{GuildManager, GuildTask, GuildTaskKind};

use crate::camp_ui::action::GuildCampAction;
use crate::camp_ui::state::CampUiState;

/// Formats the goal description of a task.
pub fn format_task_goal(task: &GuildTask) -> String {
    match &task.kind {
        GuildTaskKind::Kill {
            zombie_type,
            in_city,
        } => {
            let z_name = zombie_type.as_deref().unwrap_or("any");
            let loc = if *in_city { " in Ashford" } else { "" };
            format!("Kill {} {} zombies{}", task.target, z_name, loc)
        }
        GuildTaskKind::Fetch { item_id } => {
            format!("Deliver {} {}", task.target, item_id.replace('_', " "))
        }
        GuildTaskKind::Explore { place_name } => {
            format!("Scout and reach {}", place_name)
        }
        GuildTaskKind::Clear { poi_id } => {
            format!("Clear {}", poi_id.replace('_', " "))
        }
    }
}

/// Formats the reward description of a task.
pub fn format_task_reward(task: &GuildTask) -> String {
    let mut parts = vec![
        format!("${}", task.reward.dollars),
        format!("+{} Rep", task.reward.reputation),
    ];
    if let Some((item, count)) = &task.reward.item {
        parts.push(format!("{} {}", count, item.replace('_', " ")));
    }
    parts.join(", ")
}

/// Draws the Task Board column with active and offered tasks.
pub fn draw_tasks_column(
    ui: &mut egui::Ui,
    guild: &GuildManager,
    _state: &mut CampUiState,
    max_height: f32,
) -> Option<GuildCampAction> {
    let mut action = None;
    let board = guild.board();
    let active = board.active();
    let offered = board.offered();
    let can_accept = active.len() < board.max_active;

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Task Board")
                    .size(16.0)
                    .color(Color32::from_rgb(204, 217, 230))
                    .strong(),
            );
            ui.label(
                RichText::new(format!("({}/{} Active)", active.len(), board.max_active))
                    .size(13.0)
                    .color(Color32::from_rgb(154, 152, 146)),
            );
        });

        ui.add_space(4.0);

        ScrollArea::vertical()
            .id_salt("guild_tasks_scroll")
            .max_height(max_height)
            .show(ui, |ui| {
                // --- Active Tasks ---
                if !active.is_empty() {
                    ui.label(
                        RichText::new("Active Tasks")
                            .size(14.0)
                            .color(Color32::from_rgb(143, 194, 74))
                            .strong(),
                    );
                    ui.add_space(2.0);

                    for task in active {
                        Frame::NONE
                            .fill(Color32::from_rgb(32, 35, 40))
                            .inner_margin(Margin::same(8))
                            .corner_radius(CornerRadius::same(6))
                            .stroke(Stroke::new(
                                1.0,
                                if task.is_done() {
                                    Color32::from_rgb(143, 194, 74)
                                } else {
                                    Color32::from_rgb(60, 65, 71)
                                },
                            ))
                            .show(ui, |ui| {
                                ui.label(RichText::new(&task.title).size(14.0).strong().color(Color32::WHITE));
                                ui.label(RichText::new(format_task_goal(task)).size(12.0).color(Color32::from_rgb(190, 190, 190)));

                                let ratio = task.progress_ratio();
                                let progress_text = format!("{}/{} ({:.0}%)", task.progress, task.target, ratio * 100.0);
                                ui.add(ProgressBar::new(ratio).text(progress_text).animate(false));

                                ui.label(
                                    RichText::new(format!("Reward: {}", format_task_reward(task)))
                                        .size(12.0)
                                        .color(Color32::from_rgb(255, 230, 102)),
                                );

                                ui.horizontal(|ui| {
                                    if task.is_done() {
                                        let claim_btn = egui::Button::new(
                                            RichText::new("Claim Reward")
                                                .color(Color32::BLACK)
                                                .strong(),
                                        )
                                        .fill(Color32::from_rgb(143, 194, 74));
                                        if ui.add(claim_btn).clicked() {
                                            action = Some(GuildCampAction::ClaimTask {
                                                task_id: task.id.clone(),
                                            });
                                        }
                                    } else {
                                        let abandon_btn = egui::Button::new(
                                            RichText::new("Abandon")
                                                .size(12.0)
                                                .color(Color32::from_rgb(214, 80, 70)),
                                        );
                                        if ui.add(abandon_btn).clicked() {
                                            action = Some(GuildCampAction::AbandonTask {
                                                task_id: task.id.clone(),
                                            });
                                        }
                                    }
                                });
                            });
                        ui.add_space(4.0);
                    }
                    ui.add_space(6.0);
                }

                // --- Offered Tasks ---
                ui.label(
                    RichText::new("Available Tasks")
                        .size(14.0)
                        .color(Color32::from_rgb(220, 220, 225))
                        .strong(),
                );
                ui.add_space(2.0);

                if offered.is_empty() {
                    ui.label(
                        RichText::new("No tasks on the board today. Check back tomorrow morning!")
                            .size(12.0)
                            .color(Color32::from_rgb(150, 150, 150)),
                    );
                }

                for task in offered {
                    Frame::NONE
                        .fill(Color32::from_rgb(26, 28, 32))
                        .inner_margin(Margin::same(8))
                        .corner_radius(CornerRadius::same(6))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(50, 54, 60)))
                        .show(ui, |ui| {
                            ui.label(RichText::new(&task.title).size(14.0).strong().color(Color32::WHITE));
                            ui.label(RichText::new(format_task_goal(task)).size(12.0).color(Color32::from_rgb(180, 180, 180)));
                            ui.label(
                                RichText::new(format!("Reward: {}", format_task_reward(task)))
                                    .size(12.0)
                                    .color(Color32::from_rgb(255, 230, 102)),
                            );

                            ui.horizontal(|ui| {
                                let accept_btn = egui::Button::new("Accept");
                                if ui.add_enabled(can_accept, accept_btn).clicked() {
                                    action = Some(GuildCampAction::AcceptTask {
                                        task_id: task.id.clone(),
                                    });
                                }
                                if !can_accept {
                                    ui.label(
                                        RichText::new("(Active limit reached)")
                                            .size(11.0)
                                            .color(Color32::from_rgb(150, 150, 150)),
                                    );
                                }
                            });
                        });
                    ui.add_space(4.0);
                }
            });
    });

    action
}
