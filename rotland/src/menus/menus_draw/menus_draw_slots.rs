//! egui 0.37 rendering for the Save Slot cards and delete confirmation modal.

use engine::egui::{
    self, Align, Align2, Color32, Frame, Id, Layout, Order, RichText, Sense, Stroke, TextWrapMode,
    vec2,
};

use super::theme::{self, ButtonTone};
use crate::menus::{MenuAction, Menus, Screen, SlotMode, SlotState, SlotSummary};

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

fn card_saved_date(saved_at: &str) -> String {
    let formatted = format_saved_at(saved_at);
    let trimmed = formatted.strip_prefix(" · saved ").unwrap_or(&formatted);
    if trimmed.is_empty() {
        "—".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn draw_slot_picker(
    menus: &mut Menus,
    ui: &mut egui::Ui,
    slots: &[SlotSummary; 3],
    mode: SlotMode,
    scale: f32,
) -> Option<MenuAction> {
    let mut action = None;
    let card_w = 248.0 * scale; // new: DECISIONS 2026-11-21
    let card_gap = 14.0 * scale; // new: DECISIONS 2026-11-21
    let panel_inner_w = card_w * 3.0 + card_gap * 2.0;

    let (heading, subtitle) = match mode {
        SlotMode::Continue => ("CONTINUE GAME", "Select a survivor world to resume."),
        SlotMode::NewGame => ("NEW GAME", "Pick an empty slot to start a fresh world."),
    };

    let frame_resp = theme::panel_frame(scale, theme::BORDER).show(ui, |ui| {
        ui.set_width(panel_inner_w);
        ui.vertical(|ui| {
            ui.add(
                egui::Label::new(
                    RichText::new(heading)
                        .size(theme::FONT_HEADING * scale)
                        .color(theme::TEXT_PRIMARY)
                        .strong(),
                )
                .wrap_mode(TextWrapMode::Extend),
            );
            ui.add_space(2.0 * scale);
            ui.add(
                egui::Label::new(
                    RichText::new(subtitle)
                        .size(theme::FONT_HELP * scale)
                        .color(theme::TEXT_MUTED),
                )
                .wrap_mode(TextWrapMode::Extend),
            );
            ui.add_space(12.0 * scale);
            theme::paint_divider(ui, panel_inner_w, scale);
            ui.add_space(16.0 * scale);

            // Three uniform save slot cards side by side
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = card_gap;
                for (i, summary) in slots.iter().enumerate() {
                    let slot_num = (i + 1) as u8;
                    if let Some(act) = draw_slot_card(menus, ui, slot_num, summary, card_w, scale) {
                        action = Some(act);
                    }
                }
            });

            ui.add_space(18.0 * scale);
            theme::paint_divider(ui, panel_inner_w, scale);
            ui.add_space(14.0 * scale);

            let back_size = vec2(180.0 * scale, theme::BTN_HEIGHT * scale); // new: DECISIONS 2026-11-21
            if theme::menu_button(ui, "Back (Esc)", back_size, scale, ButtonTone::Default, true)
                .clicked()
            {
                menus.screen = Screen::Main;
            }
        });
    });

    theme::paint_top_accent(ui, frame_resp.response.rect, theme::ACCENT, scale);
    action
}

fn draw_slot_card(
    menus: &mut Menus,
    ui: &mut egui::Ui,
    slot_num: u8,
    summary: &SlotSummary,
    card_w: f32,
    scale: f32,
) -> Option<MenuAction> {
    let mut action = None;
    let card_pad = 16.0 * scale; // new: DECISIONS 2026-11-21
    let inner_w = card_w - card_pad * 2.0;
    let card_h = 256.0 * scale; // new: DECISIONS 2026-11-21

    let (bg, border, badge_label, badge_color) = match summary.state {
        SlotState::Filled => (theme::CARD_BG, theme::BORDER, "ACTIVE", theme::SAFE),
        SlotState::Empty => (
            theme::CARD_EMPTY_BG,
            Color32::from_rgba_unmultiplied(60, 65, 71, 130), // new: DECISIONS 2026-11-21
            "EMPTY",
            theme::TEXT_MUTED,
        ),
        SlotState::Damaged => (theme::CARD_BG, theme::DANGER, "DAMAGED", theme::DANGER_HOVER),
    };

    Frame::NONE
        .fill(bg)
        .stroke(Stroke::new(1.0, border))
        .corner_radius(theme::radius(theme::CORNER_PANEL, scale))
        .inner_margin(theme::margin(16.0, scale))
        .show(ui, |ui| {
            ui.set_width(inner_w);
            ui.set_min_height(card_h - card_pad * 2.0);
            ui.vertical(|ui| {
                // Card header: Slot number left, state badge right
                ui.horizontal(|ui| {
                    ui.set_width(inner_w);
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!("SLOT 0{slot_num}"))
                                .size(theme::FONT_SUBHEADING * scale)
                                .color(theme::TEXT_PRIMARY)
                                .strong(),
                        )
                        .wrap_mode(TextWrapMode::Extend),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        theme::status_badge(ui, badge_label, badge_color, scale);
                    });
                });

                ui.add_space(10.0 * scale);
                theme::paint_divider(ui, inner_w, scale);
                ui.add_space(12.0 * scale);

                let btn_h = theme::BTN_HEIGHT_COMPACT * scale;
                match summary.state {
                    SlotState::Filled => {
                        ui.add(
                            egui::Label::new(
                                RichText::new(format!("Day {}", summary.day))
                                    .size(23.0 * scale) // new: DECISIONS 2026-11-21
                                    .color(theme::TEXT_PRIMARY)
                                    .strong(),
                            )
                            .wrap_mode(TextWrapMode::Truncate),
                        );
                        ui.add_space(10.0 * scale);

                        let play_time = format_play_time(summary.play_time_s);
                        let diff = if summary.difficulty.is_empty() {
                            "Normal"
                        } else {
                            &summary.difficulty
                        };
                        let saved = card_saved_date(&summary.saved_at);

                        meta_row(ui, inner_w, scale, "Playtime", &play_time);
                        meta_row(ui, inner_w, scale, "Difficulty", diff);
                        meta_row(ui, inner_w, scale, "Saved", &saved);

                        ui.add_space(16.0 * scale);
                        if theme::menu_button(
                            ui,
                            "Continue",
                            vec2(inner_w, btn_h),
                            scale,
                            ButtonTone::Safe,
                            true,
                        )
                        .clicked()
                        {
                            action = Some(MenuAction::Continue { slot: slot_num });
                        }
                        ui.add_space(6.0 * scale);
                        if theme::menu_button(
                            ui,
                            "Delete",
                            vec2(inner_w, btn_h - 4.0 * scale),
                            scale,
                            ButtonTone::Danger,
                            true,
                        )
                        .clicked()
                        {
                            menus.request_delete_slot(slot_num);
                        }
                    }
                    SlotState::Empty => {
                        ui.add_space(22.0 * scale);
                        ui.add(
                            egui::Label::new(
                                RichText::new("Empty Slot")
                                    .size(theme::FONT_SUBHEADING * scale)
                                    .color(theme::TEXT_MUTED)
                                    .strong(),
                            )
                            .wrap_mode(TextWrapMode::Truncate),
                        );
                        ui.add_space(6.0 * scale);
                        ui.add(
                            egui::Label::new(
                                RichText::new("No saved world here yet.")
                                    .size(theme::FONT_HELP * scale)
                                    .color(theme::TEXT_MUTED),
                            )
                            .wrap_mode(TextWrapMode::Truncate),
                        );
                        ui.add_space(56.0 * scale);
                        if theme::menu_button(
                            ui,
                            "New game",
                            vec2(inner_w, btn_h),
                            scale,
                            ButtonTone::Primary,
                            true,
                        )
                        .clicked()
                        {
                            action = Some(MenuAction::NewGame { slot: slot_num });
                        }
                    }
                    SlotState::Damaged => {
                        ui.add_space(22.0 * scale);
                        ui.add(
                            egui::Label::new(
                                RichText::new("Damaged Save")
                                    .size(theme::FONT_SUBHEADING * scale)
                                    .color(theme::DANGER_HOVER)
                                    .strong(),
                            )
                            .wrap_mode(TextWrapMode::Truncate),
                        );
                        ui.add_space(6.0 * scale);
                        ui.add(
                            egui::Label::new(
                                RichText::new("Delete to reuse this slot.")
                                    .size(theme::FONT_HELP * scale)
                                    .color(theme::TEXT_MUTED),
                            )
                            .wrap_mode(TextWrapMode::Truncate),
                        );
                        ui.add_space(56.0 * scale);
                        if theme::menu_button(
                            ui,
                            "Delete",
                            vec2(inner_w, btn_h),
                            scale,
                            ButtonTone::Danger,
                            true,
                        )
                        .clicked()
                        {
                            menus.request_delete_slot(slot_num);
                        }
                    }
                }
            });
        });

    action
}

fn meta_row(ui: &mut egui::Ui, width: f32, scale: f32, key: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.set_width(width);
        ui.add(
            egui::Label::new(
                RichText::new(key)
                    .size(theme::FONT_HELP * scale)
                    .color(theme::TEXT_MUTED),
            )
            .wrap_mode(TextWrapMode::Extend),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(
                egui::Label::new(
                    RichText::new(value)
                        .size(theme::FONT_HELP * scale)
                        .color(theme::TEXT_PRIMARY),
                )
                .wrap_mode(TextWrapMode::Truncate),
            );
        });
    });
    ui.add_space(2.0 * scale);
}

pub fn draw_delete_confirm(
    menus: &mut Menus,
    ctx: &egui::Context,
    slot: u8,
    scale: f32,
) -> Option<MenuAction> {
    let mut action = None;
    let screen_rect = ctx.content_rect();

    // Dim backdrop blocking clicks outside the confirmation dialog
    egui::Area::new(Id::unique("menu_delete_confirm_bg"))
        .fixed_pos(screen_rect.min)
        .order(Order::Tooltip)
        .show(ctx, |ui| {
            ui.painter().rect_filled(screen_rect, 0.0, theme::MODAL_DIM);
            ui.allocate_rect(screen_rect, Sense::click_and_drag());
        });

    egui::Area::new(Id::unique("menu_delete_confirm"))
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .order(Order::Tooltip)
        .show(ctx, |ui| {
            let modal_w = 340.0 * scale; // new: DECISIONS 2026-11-21
            let frame_resp = theme::panel_frame(scale, theme::DANGER).show(ui, |ui| {
                ui.set_width(modal_w);
                ui.vertical(|ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new("DELETE SAVE")
                                .size(theme::FONT_SUBHEADING * scale)
                                .color(theme::DANGER_HOVER)
                                .strong(),
                        )
                        .wrap_mode(TextWrapMode::Extend),
                    );
                    ui.add_space(10.0 * scale);
                    ui.label(
                        RichText::new(format!(
                            "Delete Slot 0{slot}? Its world is lost for good."
                        ))
                        .size(theme::FONT_BODY * scale)
                        .color(theme::TEXT_PRIMARY),
                    );
                    ui.add_space(18.0 * scale);

                    let gap = 10.0 * scale;
                    let btn_w = (modal_w - gap) * 0.5;
                    let btn_size = vec2(btn_w, theme::BTN_HEIGHT_COMPACT * scale);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        if theme::menu_button(
                            ui,
                            "Delete",
                            btn_size,
                            scale,
                            ButtonTone::Danger,
                            true,
                        )
                        .clicked()
                        {
                            action = menus.confirm_delete();
                        }
                        if theme::menu_button(
                            ui,
                            "Keep",
                            btn_size,
                            scale,
                            ButtonTone::Default,
                            true,
                        )
                        .clicked()
                        {
                            menus.cancel_delete();
                        }
                    });
                });
            });
            theme::paint_top_accent(ui, frame_resp.response.rect, theme::DANGER, scale);
        });

    action
}
