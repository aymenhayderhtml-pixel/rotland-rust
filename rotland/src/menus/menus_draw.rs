//! egui 0.37 rendering for Main and Paused screens, and central draw dispatcher.

use engine::egui::{
    self, Align2, Color32, FontFamily, FontId, Id, Order, RichText, Sense, TextWrapMode, pos2, vec2,
};

use super::{MenuAction, Menus, Screen, SlotSummary};

mod menus_draw_settings;
mod menus_draw_slots;
#[path = "theme.rs"]
mod theme;

#[cfg(test)]
pub use menus_draw_slots::{format_play_time, format_saved_at};

/// Main drawing entry point.
pub fn draw(menus: &mut Menus, ctx: &egui::Context, slots: &[SlotSummary; 3]) -> Option<MenuAction> {
    if menus.screen == Screen::None {
        return None;
    }

    let screen_rect = ctx.content_rect();
    let scale = theme::ui_scale(screen_rect.height());

    theme::with_menu_style(ctx, scale, || {
        let mut action = None;

        // 1. Background overlay (keeps 3D world visible on Main via left-half gradient)
        egui::Area::new(Id::unique("menu_background"))
            .fixed_pos(screen_rect.min)
            .order(Order::Foreground)
            .show(ctx, |ui| {
                match &menus.screen {
                    Screen::Main => theme::paint_left_gradient(ui.painter(), screen_rect),
                    Screen::Paused => {
                        ui.painter().rect_filled(screen_rect, 0.0, theme::PAUSE_DIM);
                    }
                    Screen::SlotPicker { .. } | Screen::Settings { .. } => {
                        ui.painter().rect_filled(screen_rect, 0.0, theme::SETTINGS_DIM);
                    }
                    Screen::None => {}
                }
                ui.allocate_rect(screen_rect, Sense::click_and_drag());
            });

        // 2. Screen content (left-third column for Main, centered panel for other screens)
        let content_area = match &menus.screen {
            Screen::Main => {
                let left_offset = (screen_rect.width() * 0.065).clamp(48.0 * scale, 180.0 * scale); // new: DECISIONS 2026-11-21
                egui::Area::new(Id::unique("menu_content"))
                    .anchor(Align2::LEFT_CENTER, vec2(left_offset, 0.0))
                    .order(Order::Foreground)
            }
            _ => egui::Area::new(Id::unique("menu_content"))
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .order(Order::Foreground),
        };

        content_area.show(ctx, |ui| match &menus.screen {
            Screen::Main => {
                action = draw_main(menus, ui, slots, scale);
            }
            Screen::Paused => {
                action = draw_paused(menus, ui, scale);
            }
            Screen::SlotPicker { mode } => {
                let mode = *mode;
                action = menus_draw_slots::draw_slot_picker(menus, ui, slots, mode, scale);
            }
            Screen::Settings { .. } => {
                action = menus_draw_settings::draw_settings(menus, ui, screen_rect.height(), scale);
            }
            Screen::None => {}
        });

        // 3. Delete confirmation modal if pending
        if let Some(slot) = menus.pending_delete
            && let Some(act) = menus_draw_slots::draw_delete_confirm(menus, ctx, slot, scale)
        {
            action = Some(act);
        }

        action
    })
}

fn draw_main(
    menus: &mut Menus,
    ui: &mut egui::Ui,
    slots: &[SlotSummary; 3],
    scale: f32,
) -> Option<MenuAction> {
    let mut action = None;
    let can_continue = Menus::any_slot_filled(slots);
    let col_w = 310.0 * scale; // new: DECISIONS 2026-11-21
    let btn_size = vec2(col_w, theme::BTN_HEIGHT * scale);

    ui.set_width(col_w);
    ui.vertical(|ui| {
        // Road-sign rust-orange bar above the title
        let (bar_rect, _) = ui.allocate_exact_size(vec2(56.0 * scale, 4.0 * scale), Sense::hover()); // new: DECISIONS 2026-11-21
        ui.painter().rect_filled(bar_rect, 0.0, theme::ACCENT);
        ui.add_space(6.0 * scale); // new: DECISIONS 2026-11-21

        // Large ROTLAND title with crisp dark drop shadow over the 3D world
        let title_h = theme::FONT_TITLE * scale * 1.1; // new: DECISIONS 2026-11-21
        let (title_rect, _) = ui.allocate_exact_size(vec2(col_w, title_h), Sense::hover());
        let font = FontId::new(theme::FONT_TITLE * scale, FontFamily::Proportional);
        let shadow_offset = vec2(2.0 * scale, 2.0 * scale); // new: DECISIONS 2026-11-21
        let shadow = Color32::from_rgba_unmultiplied(0, 0, 0, 175); // new: DECISIONS 2026-11-21
        let text_pos = pos2(title_rect.left(), title_rect.center().y);
        ui.painter()
            .text(text_pos + shadow_offset, Align2::LEFT_CENTER, "ROTLAND", font.clone(), shadow);
        ui.painter()
            .text(text_pos, Align2::LEFT_CENTER, "ROTLAND", font, theme::TEXT_PRIMARY);

        ui.add_space(2.0 * scale);
        ui.add(
            egui::Label::new(
                RichText::new("SCAVENGE  ·  FORTIFY  ·  OUTLAST")
                    .size(13.0 * scale) // new: DECISIONS 2026-11-21
                    .color(theme::TEXT_MUTED),
            )
            .wrap_mode(TextWrapMode::Extend),
        );

        ui.add_space(14.0 * scale); // new: DECISIONS 2026-11-21
        theme::paint_divider(ui, col_w, scale);
        ui.add_space(20.0 * scale); // new: DECISIONS 2026-11-21

        let cont_tone = if can_continue {
            theme::ButtonTone::Safe
        } else {
            theme::ButtonTone::Default
        };
        if theme::menu_button(ui, "Continue", btn_size, scale, cont_tone, can_continue).clicked() {
            menus.press_continue(slots);
        }
        ui.add_space(8.0 * scale);

        let new_tone = if can_continue {
            theme::ButtonTone::Default
        } else {
            theme::ButtonTone::Primary
        };
        if theme::menu_button(ui, "New game", btn_size, scale, new_tone, true).clicked() {
            menus.press_new_game();
        }
        ui.add_space(8.0 * scale);

        if theme::menu_button(ui, "Settings", btn_size, scale, theme::ButtonTone::Default, true)
            .clicked()
        {
            menus.press_settings();
        }

        ui.add_space(16.0 * scale); // new: DECISIONS 2026-11-21
        if theme::menu_button(ui, "Quit", btn_size, scale, theme::ButtonTone::Danger, true)
            .clicked()
        {
            action = Some(MenuAction::Quit);
        }
    });

    action
}

fn draw_paused(menus: &mut Menus, ui: &mut egui::Ui, scale: f32) -> Option<MenuAction> {
    let mut action = None;
    let col_w = 312.0 * scale; // new: DECISIONS 2026-11-21
    let btn_size = vec2(col_w, theme::BTN_HEIGHT * scale);

    let frame_resp = theme::panel_frame(scale, theme::BORDER).show(ui, |ui| {
        ui.set_width(col_w);
        ui.vertical(|ui| {
            ui.add(
                egui::Label::new(
                    RichText::new("PAUSED")
                        .size(theme::FONT_HEADING * scale)
                        .color(theme::TEXT_PRIMARY)
                        .strong(),
                )
                .wrap_mode(TextWrapMode::Extend),
            );
            ui.add_space(10.0 * scale);
            theme::paint_divider(ui, col_w, scale);
            ui.add_space(16.0 * scale);

            if theme::menu_button(ui, "Resume", btn_size, scale, theme::ButtonTone::Safe, true)
                .clicked()
            {
                menus.screen = Screen::None;
                action = Some(MenuAction::Resume);
            }
            ui.add_space(8.0 * scale);

            if theme::menu_button(ui, "Settings", btn_size, scale, theme::ButtonTone::Default, true)
                .clicked()
            {
                menus.press_settings();
            }
            ui.add_space(8.0 * scale);

            if theme::menu_button(
                ui,
                "Save & quit to menu",
                btn_size,
                scale,
                theme::ButtonTone::Default,
                true,
            )
            .clicked()
            {
                menus.screen = Screen::Main;
                action = Some(MenuAction::SaveAndQuitToMenu);
            }

            ui.add_space(14.0 * scale);
            if theme::menu_button(
                ui,
                "Quit game",
                btn_size,
                scale,
                theme::ButtonTone::Danger,
                true,
            )
            .clicked()
            {
                action = Some(MenuAction::Quit);
            }

            ui.add_space(14.0 * scale);
            ui.add(
                egui::Label::new(
                    RichText::new("Press Esc to resume")
                        .size(theme::FONT_HELP * scale)
                        .color(theme::TEXT_MUTED),
                )
                .wrap_mode(TextWrapMode::Extend),
            );
        });
    });

    theme::paint_top_accent(ui, frame_resp.response.rect, theme::ACCENT, scale);
    action
}
