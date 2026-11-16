//! egui 0.37 rendering for SlotPicker screen and delete confirmation dialog.

use engine::egui::{self, vec2, Align2, Color32, CornerRadius, Frame, Id, Margin, Order, RichText, Stroke};

use super::{MenuAction, Menus, Screen, SlotMode, SlotState, SlotSummary};

/// Formats play time seconds as "h:mm".
pub fn format_play_time(seconds: f32) -> String {
    let total = seconds.max(0.0) as u32;
    let hours = total / 3600;
    let mins = (total % 3600) / 60;
    format!("{hours}h {mins:02}m")
}

/// Formats saved_at timestamp string cleanly.
pub fn format_saved_at(saved_at: &str) -> String {
    if saved_at.len() >= 16 {
        format!(" · saved {}", saved_at[..16].replace('T', " "))
    } else if !saved_at.is_empty() {
        format!(" · saved {saved_at}")
    } else {
        String::new()
    }
}

pub fn draw_slot_picker(
    menus: &mut Menus,
    ui: &mut egui::Ui,
    slots: &[SlotSummary; 3],
    _mode: SlotMode,
) -> Option<MenuAction> {
    let mut action = None;

    Frame::NONE
        .fill(Color32::from_rgb(22, 24, 22))
        .inner_margin(Margin::same(20))
        .corner_radius(CornerRadius::same(8))
        .stroke(Stroke::new(1.0, Color32::from_rgb(50, 55, 50)))
        .show(ui, |ui| {
            ui.set_width(560.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Choose a save slot").size(24.0).color(Color32::WHITE).strong());
            });
            ui.add_space(14.0);

            for i in 0..3 {
                let slot_num = (i + 1) as u8;
                let summary = &slots[i];

                Frame::NONE
                    .fill(Color32::from_rgb(30, 32, 30))
                    .inner_margin(Margin::same(12))
                    .corner_radius(CornerRadius::same(6))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(50, 55, 50)))
                    .show(ui, |ui| {
                        ui.label(RichText::new(format!("Slot {slot_num}")).size(18.0).strong());

                        let desc = match summary.state {
                            SlotState::Empty => "Empty slot. Start a new world here.".to_string(),
                            SlotState::Damaged => "This save file is damaged. Delete it to start over.".to_string(),
                            SlotState::Filled => {
                                let play_time = format_play_time(summary.play_time_s);
                                let saved_at = format_saved_at(&summary.saved_at);
                                let diff = if summary.difficulty.is_empty() {
                                    "Normal"
                                } else {
                                    &summary.difficulty
                                };
                                format!("Day {}  ·  {play_time} played\nDifficulty: {diff}{saved_at}", summary.day)
                            }
                        };
                        ui.label(RichText::new(desc).size(14.0).color(Color32::from_rgb(180, 185, 180)));
                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            let is_filled = summary.state == SlotState::Filled;
                            let is_empty = summary.state == SlotState::Empty;
                            let has_save = summary.state != SlotState::Empty;

                            if ui
                                .add_enabled(is_filled, egui::Button::new("Continue").min_size(vec2(140.0, 36.0)))
                                .clicked()
                            {
                                action = Some(MenuAction::Continue { slot: slot_num });
                            }
                            if ui
                                .add_enabled(is_empty, egui::Button::new("New game").min_size(vec2(140.0, 36.0)))
                                .clicked()
                            {
                                action = Some(MenuAction::NewGame { slot: slot_num });
                            }
                            if ui
                                .add_enabled(has_save, egui::Button::new("Delete").min_size(vec2(100.0, 36.0)))
                                .clicked()
                            {
                                menus.request_delete_slot(slot_num);
                            }
                        });
                    });
                ui.add_space(8.0);
            }

            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                if ui.add(egui::Button::new("Back").min_size(vec2(160.0, 40.0))).clicked() {
                    menus.screen = Screen::Main;
                }
            });
        });

    action
}

pub fn draw_delete_confirm(menus: &mut Menus, ctx: &egui::Context, slot: u8) -> Option<MenuAction> {
    let mut action = None;
    egui::Area::new(Id::unique("menu_delete_confirm"))
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .order(Order::Tooltip)
        .show(ctx, |ui| {
            Frame::NONE
                .fill(Color32::from_rgb(30, 20, 20))
                .inner_margin(Margin::same(20))
                .corner_radius(CornerRadius::same(8))
                .stroke(Stroke::new(2.0, Color32::from_rgb(180, 50, 50)))
                .show(ui, |ui| {
                    ui.set_width(320.0);
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("Delete save").size(20.0).strong().color(Color32::WHITE));
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!("Delete slot {slot}? Its world is lost for good."))
                                .size(14.0)
                                .color(Color32::from_rgb(220, 200, 200)),
                        );
                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Delete").color(Color32::from_rgb(255, 100, 100)),
                                    )
                                    .min_size(vec2(120.0, 36.0)),
                                )
                                .clicked()
                            {
                                action = menus.confirm_delete();
                            }
                            if ui.add(egui::Button::new("Keep").min_size(vec2(120.0, 36.0))).clicked() {
                                menus.cancel_delete();
                            }
                        });
                    });
                });
        });
    action
}
