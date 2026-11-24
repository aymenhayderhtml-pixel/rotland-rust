//! Guild screen module for the camp board UI.
//!
//! Provides the three-section Guild tab: Task Board, Recruits, and Guild Shop.

pub mod layout;
pub mod recruits_ui;
pub mod shop_ui;
pub mod tasks_ui;

#[cfg(test)]
mod tests;

pub use layout::{calc_guild_layout, GuildLayout};
pub use recruits_ui::draw_recruits_column;
pub use shop_ui::draw_shop_column;
pub use tasks_ui::draw_tasks_column;

use engine::egui::{self, Color32, CornerRadius, Frame, Margin, ProgressBar, RichText, Stroke};

use crate::camp_ui::action::{CampAction, GuildCampAction};
use crate::camp_ui::state::CampUiState;
use crate::camp_ui::view::CampView;

/// Draws the complete Guild tab contents (rank banner + three columns).
/// Returns an action if an interaction occurs, and updates the status message in `state`.
pub fn draw_guild(
    ui: &mut egui::Ui,
    state: &mut CampUiState,
    view: &CampView,
) -> Option<CampAction> {
    let guild_opt = view.guild();
    let guild = match guild_opt {
        Some(g) => g,
        None => {
            ui.vertical_centered(|ui| {
                ui.add_space(60.0);
                ui.label(
                    RichText::new("Guild Board records are not available.")
                        .size(16.0)
                        .color(Color32::from_rgb(180, 180, 180)),
                );
            });
            return None;
        }
    };

    let rank = guild.current_rank();
    let rep = guild.reputation();
    let (curr_threshold, next_threshold) = guild.rank_progress();
    let player_dollars = view.player_dollars();

    // --- Top Rank and Reputation Banner ---
    Frame::NONE
        .fill(Color32::from_rgb(18, 20, 24))
        .inner_margin(Margin::same(10))
        .corner_radius(CornerRadius::same(6))
        .stroke(Stroke::new(1.0, Color32::from_rgb(60, 65, 71)))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("Rank: {}", rank.name))
                        .size(16.0)
                        .color(Color32::from_rgb(255, 204, 102))
                        .strong(),
                );

                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("({} Rep)", rep))
                        .size(14.0)
                        .color(Color32::from_rgb(220, 220, 220)),
                );

                ui.add_space(16.0);

                if let Some(next_rep) = next_threshold {
                    let range = (next_rep - curr_threshold).max(1) as f32;
                    let progress = (rep - curr_threshold).max(0) as f32;
                    let ratio = (progress / range).clamp(0.0, 1.0);
                    let bar_text = format!("{}/{} Rep to next rank", rep, next_rep);
                    ui.add(
                        ProgressBar::new(ratio)
                            .desired_width(220.0)
                            .text(bar_text)
                            .animate(false),
                    );
                } else {
                    ui.label(
                        RichText::new("Max Rank Achieved")
                            .size(13.0)
                            .color(Color32::from_rgb(143, 194, 74))
                            .strong(),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("Shop Tier: {} | Recruit Slots: {}", guild.shop_tier(), guild.max_recruits()))
                            .size(13.0)
                            .color(Color32::from_rgb(180, 185, 190)),
                    );
                });
            });
        });

    ui.add_space(8.0);

    // --- Three Columns: Task Board | Recruits | Guild Shop ---
    let avail = ui.available_size();
    let layout = calc_guild_layout(avail);
    let column_height = layout.body_height.min(avail.y - 10.0).max(220.0);

    let mut action = None;
    ui.columns(3, |columns| {
        // 1. Task Board
        if let Some(act) = draw_tasks_column(&mut columns[0], guild, view.inventory, state, column_height) {
            action = Some(act);
        }

        // 2. Recruits
        if let Some(act) = draw_recruits_column(&mut columns[1], guild, player_dollars, state, column_height) {
            action = Some(act);
        }

        // 3. Guild Shop
        if let Some(act) = draw_shop_column(&mut columns[2], guild, state, view, column_height) {
            action = Some(act);
        }
    });

    if let Some(ref act) = action {
        match act {
            GuildCampAction::AcceptTask { task_id } => state.set_status(format!("Accepted task: {task_id}")),
            GuildCampAction::AbandonTask { task_id } => state.set_status(format!("Abandoned task: {task_id}")),
            GuildCampAction::ClaimTask { task_id } => state.set_status(format!("Claimed task: {task_id}")),
            GuildCampAction::HandIn { task_id } => state.set_status(format!("Handing in items for: {task_id}")),
            GuildCampAction::HireRecruit { recruit_id } => state.set_status(format!("Hired recruit: {recruit_id}")),
            GuildCampAction::DismissRecruit { recruit_id } => state.set_status(format!("Dismissed recruit: {recruit_id}")),
            GuildCampAction::SetRecruitOrder { recruit_id, order } => state.set_status(format!("Order for {recruit_id}: {order:?}")),
            GuildCampAction::BuyItem { item_id, count } => state.set_status(format!("Bought {count} {item_id}")),
        }
    }

    action.map(CampAction::Guild)
}
