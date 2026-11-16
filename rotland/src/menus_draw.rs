//! egui 0.37 rendering for Main and Paused screens, and central draw dispatcher.

use engine::egui::{self, vec2, Align2, Color32, CornerRadius, Frame, Id, Margin, Order, RichText, Sense, Stroke};

use super::{MenuAction, Menus, Screen, SlotSummary};

mod menus_draw_settings;
mod menus_draw_slots;

pub use menus_draw_slots::{format_play_time, format_saved_at};

/// Main drawing entry point.
pub fn draw(menus: &mut Menus, ctx: &egui::Context, slots: &[SlotSummary; 3]) -> Option<MenuAction> {
    let mut action = None;

    // 1. Fullscreen dark background dim / overlay
    let bg_color = match &menus.screen {
        Screen::Main | Screen::SlotPicker { .. } => Color32::from_rgb(13, 18, 13),
        Screen::Paused => Color32::from_rgba_unmultiplied(0, 0, 0, 153),
        Screen::Settings { .. } => Color32::from_rgba_unmultiplied(0, 0, 0, 191),
        Screen::None => return None,
    };

    let screen_rect = ctx.content_rect();
    egui::Area::new(Id::unique("menu_background"))
        .fixed_pos(screen_rect.min)
        .order(Order::Foreground)
        .show(ctx, |ui| {
            ui.painter().rect_filled(screen_rect, 0.0, bg_color);
            ui.allocate_rect(screen_rect, Sense::click_and_drag());
        });

    // 2. Centered panel contents
    egui::Area::new(Id::unique("menu_content"))
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .order(Order::Foreground)
        .show(ctx, |ui| match &menus.screen {
            Screen::Main => {
                action = draw_main(menus, ui, slots);
            }
            Screen::Paused => {
                action = draw_paused(menus, ui);
            }
            Screen::SlotPicker { mode } => {
                let mode = *mode;
                action = menus_draw_slots::draw_slot_picker(menus, ui, slots, mode);
            }
            Screen::Settings { .. } => {
                action = menus_draw_settings::draw_settings(menus, ui);
            }
            Screen::None => {}
        });

    // 3. Delete confirmation modal if pending
    if let Some(slot) = menus.pending_delete {
        if let Some(act) = menus_draw_slots::draw_delete_confirm(menus, ctx, slot) {
            action = Some(act);
        }
    }

    action
}

fn draw_main(menus: &mut Menus, ui: &mut egui::Ui, slots: &[SlotSummary; 3]) -> Option<MenuAction> {
    let mut action = None;
    let can_continue = Menus::any_slot_filled(slots);

    ui.set_width(320.0);
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new("ROTLAND")
                .size(52.0)
                .color(Color32::from_rgb(143, 194, 74))
                .strong(),
        );
        ui.add_space(28.0);

        let cont_btn = egui::Button::new(RichText::new("Continue").size(20.0)).min_size(vec2(320.0, 48.0));
        if ui.add_enabled(can_continue, cont_btn).clicked() {
            menus.press_continue(slots);
        }
        ui.add_space(8.0);

        if ui
            .add(egui::Button::new(RichText::new("New game").size(20.0)).min_size(vec2(320.0, 48.0)))
            .clicked()
        {
            menus.press_new_game();
        }
        ui.add_space(8.0);

        if ui
            .add(egui::Button::new(RichText::new("Settings").size(20.0)).min_size(vec2(320.0, 48.0)))
            .clicked()
        {
            menus.press_settings();
        }
        ui.add_space(8.0);

        if ui
            .add(egui::Button::new(RichText::new("Quit").size(20.0)).min_size(vec2(320.0, 48.0)))
            .clicked()
        {
            action = Some(MenuAction::Quit);
        }
    });

    action
}

fn draw_paused(menus: &mut Menus, ui: &mut egui::Ui) -> Option<MenuAction> {
    let mut action = None;

    Frame::NONE
        .fill(Color32::from_rgb(26, 26, 28))
        .inner_margin(Margin::same(20))
        .corner_radius(CornerRadius::same(8))
        .stroke(Stroke::new(1.0, Color32::from_rgb(60, 60, 65)))
        .show(ui, |ui| {
            ui.set_width(320.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Paused").size(28.0).color(Color32::WHITE).strong());
                ui.add_space(20.0);

                if ui
                    .add(egui::Button::new(RichText::new("Resume").size(18.0)).min_size(vec2(300.0, 44.0)))
                    .clicked()
                {
                    menus.screen = Screen::None;
                    action = Some(MenuAction::Resume);
                }
                ui.add_space(8.0);

                if ui
                    .add(egui::Button::new(RichText::new("Settings").size(18.0)).min_size(vec2(300.0, 44.0)))
                    .clicked()
                {
                    menus.press_settings();
                }
                ui.add_space(8.0);

                if ui
                    .add(
                        egui::Button::new(RichText::new("Save and quit to menu").size(18.0))
                            .min_size(vec2(300.0, 44.0)),
                    )
                    .clicked()
                {
                    menus.screen = Screen::Main;
                    action = Some(MenuAction::SaveAndQuitToMenu);
                }
                ui.add_space(8.0);

                if ui
                    .add(egui::Button::new(RichText::new("Quit game").size(18.0)).min_size(vec2(300.0, 44.0)))
                    .clicked()
                {
                    action = Some(MenuAction::Quit);
                }
            });
        });

    action
}
