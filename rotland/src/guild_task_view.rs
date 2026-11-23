//! Compact on-screen guild task tracker anchored below the HUD clock on the top right.

use crate::menus::theme;
use engine::egui::{self, Align, Align2, Frame, Layout, Stroke, TextWrapMode, vec2};
use game::guild::task_hooks::TaskRow;

// Base layout metrics at 1280x720, scaled by screen height via `theme::ui_scale`.
const TRACKER_RIGHT: f32 = 16.0; // new: DECISIONS 2026-11-23
const TRACKER_TOP: f32 = 46.0; // new: DECISIONS 2026-11-23
const PANEL_WIDTH: f32 = 260.0; // new: DECISIONS 2026-11-23
const PANEL_MARGIN: f32 = 10.0; // new: DECISIONS 2026-11-23
const ROW_GAP: f32 = 4.0; // new: DECISIONS 2026-11-23
const BORDER_WIDTH: f32 = 1.0; // new: DECISIONS 2026-11-23
const MAX_ROWS: usize = 3; // data: guild.json max_active

/// Draws the active guild task tracker. Draws nothing when `rows` is empty.
pub fn draw_task_tracker(ctx: &egui::Context, rows: &[TaskRow]) {
    if rows.is_empty() {
        return;
    }

    let scale = theme::ui_scale(ctx.content_rect().height());
    let visible = &rows[..rows.len().min(MAX_ROWS)];
    let content_width = (PANEL_WIDTH - PANEL_MARGIN - PANEL_MARGIN) * scale;

    egui::Area::new(egui::Id::unique("rotland_guild_tracker"))
        .order(egui::Order::Foreground)
        .anchor(
            Align2::RIGHT_TOP,
            vec2(-TRACKER_RIGHT * scale, TRACKER_TOP * scale),
        )
        .interactable(false)
        .show(ctx, |ui| {
            Frame::NONE
                .fill(theme::PANEL_BG)
                .stroke(Stroke::new(BORDER_WIDTH, theme::BORDER))
                .corner_radius(theme::radius(theme::CORNER_CONTROL, scale))
                .inner_margin(theme::margin(PANEL_MARGIN, scale))
                .show(ui, |ui| {
                    ui.set_width(content_width);
                    for (idx, row) in visible.iter().enumerate() {
                        if idx > 0 {
                            ui.add_space(ROW_GAP * scale);
                            theme::paint_divider(ui, content_width, scale);
                            ui.add_space(ROW_GAP * scale);
                        }
                        draw_row(ui, row, scale);
                    }
                });
        });
}

fn draw_row(ui: &mut egui::Ui, row: &TaskRow, scale: f32) {
    ui.horizontal(|ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let status = if row.ready {
                egui::RichText::new("Done")
                    .size(theme::FONT_BODY * scale)
                    .color(theme::SAFE)
                    .strong()
            } else {
                egui::RichText::new(format!("{}/{}", row.progress, row.target))
                    .size(theme::FONT_BODY * scale)
                    .color(theme::TEXT_MUTED)
            };
            ui.add(egui::Label::new(status).wrap_mode(TextWrapMode::Extend));

            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(&row.title)
                            .size(theme::FONT_BODY * scale)
                            .color(theme::TEXT_PRIMARY),
                    )
                    .wrap_mode(TextWrapMode::Truncate),
                );
            });
        });
    });

    if row.ready {
        ui.label(
            egui::RichText::new("Return to the guild to claim")
                .size(theme::FONT_HELP * scale)
                .color(theme::SAFE),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guild_task_view_draw() {
        let ctx = egui::Context::default();
        let rows = [
            TaskRow {
                title: "Guild Timber Reserve".to_string(),
                progress: 3, // new: DECISIONS 2026-11-23
                target: 5,   // new: DECISIONS 2026-11-23
                ready: false,
            },
            TaskRow {
                title: "Scout Ashford Core".to_string(),
                progress: 1, // new: DECISIONS 2026-11-23
                target: 1,   // new: DECISIONS 2026-11-23
                ready: true,
            },
            TaskRow {
                title: "Clear Abandoned Gas Station".to_string(),
                progress: 0, // new: DECISIONS 2026-11-23
                target: 1,   // new: DECISIONS 2026-11-23
                ready: false,
            },
            TaskRow {
                title: "Overflow Task Ignored".to_string(),
                progress: 0, // new: DECISIONS 2026-11-23
                target: 2,   // new: DECISIONS 2026-11-23
                ready: false,
            },
        ];

        for slice in [&[][..], &rows[..1], &rows[..]] {
            ctx.run_pass(Default::default(), |c| draw_task_tracker(c, slice))
                .textures_delta
                .clear();
        }
    }
}
