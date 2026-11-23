//! Party status panel on the left HUD and floating overhead name/health tags for teammates.

use crate::menus::theme;
use engine::egui::{
    self, Align2, Color32, Frame, Margin, Pos2, Rect, Sense, Stroke, StrokeKind, pos2, vec2,
};

// Base layout metrics at 1280x720, scaled by screen height via `theme::ui_scale`.
// Placed on the left side above the food/water status bars and below the top-left hint box.
const PANEL_OFFSET_X: f32 = 16.0; // new: DECISIONS 2026-11-23
const PANEL_OFFSET_Y: f32 = -176.0; // new: DECISIONS 2026-11-23
const PANEL_PAD_X: f32 = 10.0; // new: DECISIONS 2026-11-23
const PANEL_PAD_Y: f32 = 7.0; // new: DECISIONS 2026-11-23
const ROW_SPACING_Y: f32 = 4.0; // new: DECISIONS 2026-11-23
const ROW_GAP_X: f32 = 8.0; // new: DECISIONS 2026-11-23
const ROW_NAME_WIDTH: f32 = 86.0; // new: DECISIONS 2026-11-23
const ROW_BAR_WIDTH: f32 = 64.0; // new: DECISIONS 2026-11-23
const ROW_BAR_HEIGHT: f32 = 5.0; // new: DECISIONS 2026-11-23
const BAR_INSET: f32 = 1.0; // new: DECISIONS 2026-11-23
const BAR_RADIUS: f32 = 2.0; // new: DECISIONS 2026-11-23
const HEALTH_MID_FRACTION: f32 = 0.5; // new: DECISIONS 2026-11-23

const NAME_TAG_LIFT_Y: f32 = 6.0; // new: DECISIONS 2026-11-23
const NAME_TAG_PAD_X: f32 = 6.0; // new: DECISIONS 2026-11-23
const NAME_TAG_PAD_Y: f32 = 3.0; // new: DECISIONS 2026-11-23
const NAME_TAG_FONT: f32 = 11.0; // new: DECISIONS 2026-11-23
const NAME_TAG_GAP_Y: f32 = 2.0; // new: DECISIONS 2026-11-23
const NAME_TAG_BAR_WIDTH: f32 = 56.0; // new: DECISIONS 2026-11-23
const NAME_TAG_BAR_HEIGHT: f32 = 4.0; // new: DECISIONS 2026-11-23

/// View model for one row in [`draw_party_panel`].
#[derive(Clone, Copy, Debug)]
pub struct PartyMember<'a> {
    pub name: &'a str,
    pub health_fraction: f32,
    pub downed: bool,
    pub order_label: &'a str,
}

fn scaled_margin(x: f32, y: f32, scale: f32) -> Margin {
    Margin::symmetric(
        (x * scale).round().clamp(1.0, f32::from(i8::MAX)) as i8,
        (y * scale).round().clamp(1.0, f32::from(i8::MAX)) as i8,
    )
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |ca: u8, cb: u8| -> u8 {
        (f32::from(ca) + (f32::from(cb) - f32::from(ca)) * t).round() as u8
    };
    Color32::from_rgba_unmultiplied(
        mix(a.r(), b.r()),
        mix(a.g(), b.g()),
        mix(a.b(), b.b()),
        mix(a.a(), b.a()),
    )
}

fn health_color(fraction: f32) -> Color32 {
    let t = fraction.clamp(0.0, 1.0);
    if t >= HEALTH_MID_FRACTION {
        let k = (t - HEALTH_MID_FRACTION) / (1.0 - HEALTH_MID_FRACTION);
        lerp_color(theme::ACCENT_HOVER, theme::SAFE, k)
    } else {
        let k = t / HEALTH_MID_FRACTION;
        lerp_color(theme::DANGER, theme::ACCENT_HOVER, k)
    }
}

/// Draws a compact list of hired teammates on the left side above the food/water bars.
/// Draws nothing when `members` is empty.
pub fn draw_party_panel(ctx: &egui::Context, members: &[PartyMember<'_>]) {
    if members.is_empty() {
        return;
    }

    let scale = theme::ui_scale(ctx.content_rect().height());

    egui::Area::new(egui::Id::unique("rotland_party_panel"))
        .order(egui::Order::Foreground)
        .anchor(
            Align2::LEFT_BOTTOM,
            vec2(PANEL_OFFSET_X * scale, PANEL_OFFSET_Y * scale),
        )
        .interactable(false)
        .show(ctx, |ui| {
            let inner = Frame::NONE
                .fill(theme::PANEL_BG)
                .stroke(Stroke::new(1.0, theme::BORDER))
                .corner_radius(theme::radius(theme::CORNER_PANEL, scale))
                .inner_margin(scaled_margin(PANEL_PAD_X, PANEL_PAD_Y, scale))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing =
                        vec2(ROW_GAP_X * scale, ROW_SPACING_Y * scale);

                    for member in members {
                        ui.horizontal(|ui| {
                            let name_color = if member.downed {
                                theme::TEXT_MUTED
                            } else {
                                theme::TEXT_PRIMARY
                            };
                            ui.add_sized(
                                vec2(ROW_NAME_WIDTH * scale, theme::FONT_HELP * scale),
                                egui::Label::new(
                                    egui::RichText::new(member.name)
                                        .size(theme::FONT_HELP * scale)
                                        .color(name_color),
                                )
                                .truncate(),
                            );

                            if member.downed {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new("Down until morning")
                                            .size(theme::FONT_HELP * scale)
                                            .color(theme::TEXT_DISABLED),
                                    )
                                    .wrap_mode(egui::TextWrapMode::Extend),
                                );
                            } else {
                                let frac = if member.health_fraction.is_finite() {
                                    member.health_fraction.clamp(0.0, 1.0)
                                } else {
                                    0.0
                                };
                                draw_health_bar(
                                    ui,
                                    ROW_BAR_WIDTH * scale,
                                    ROW_BAR_HEIGHT * scale,
                                    frac,
                                    scale,
                                );
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(member.order_label)
                                            .size(theme::FONT_HELP * scale)
                                            .color(theme::TEXT_MUTED),
                                    )
                                    .wrap_mode(egui::TextWrapMode::Extend),
                                );
                            }
                        });
                    }
                });

            theme::paint_top_accent(ui, inner.response.rect, theme::ACCENT, scale);
        });
}

/// Draws a tiny overhead name and thin health bar anchored above `screen_pos`.
pub fn draw_name_tag(ctx: &egui::Context, screen_pos: Pos2, name: &str, health_fraction: f32) {
    if !screen_pos.x.is_finite() || !screen_pos.y.is_finite() {
        return;
    }

    let scale = theme::ui_scale(ctx.content_rect().height());
    let frac = if health_fraction.is_finite() {
        health_fraction.clamp(0.0, 1.0)
    } else {
        0.0
    };

    egui::Area::new(egui::Id::unique(("rotland_companion_tag", name)))
        .order(egui::Order::Middle)
        .pivot(Align2::CENTER_BOTTOM)
        .fixed_pos(pos2(screen_pos.x, screen_pos.y - NAME_TAG_LIFT_Y * scale))
        .constrain(false)
        .interactable(false)
        .show(ctx, |ui| {
            Frame::NONE
                .fill(theme::PANEL_BG)
                .stroke(Stroke::new(1.0, theme::BORDER))
                .corner_radius(theme::radius(theme::CORNER_CONTROL, scale))
                .inner_margin(scaled_margin(NAME_TAG_PAD_X, NAME_TAG_PAD_Y, scale))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(name)
                                    .size(NAME_TAG_FONT * scale)
                                    .color(theme::TEXT_PRIMARY),
                            )
                            .wrap_mode(egui::TextWrapMode::Extend),
                        );
                        ui.add_space(NAME_TAG_GAP_Y * scale);
                        draw_health_bar(
                            ui,
                            NAME_TAG_BAR_WIDTH * scale,
                            NAME_TAG_BAR_HEIGHT * scale,
                            frac,
                            scale,
                        );
                    });
                });
        });
}

fn draw_health_bar(ui: &mut egui::Ui, width: f32, height: f32, fraction: f32, scale: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let painter = ui.painter();
    let r = theme::radius(BAR_RADIUS, scale);

    painter.rect(
        rect,
        r,
        theme::CARD_EMPTY_BG,
        Stroke::new(1.0, theme::BORDER),
        StrokeKind::Inside,
    );

    let inset = BAR_INSET * scale;
    let inner_w = (rect.width() - inset * 2.0).max(0.0) * fraction;
    if inner_w > 0.0 {
        let fill_rect = Rect::from_min_max(
            pos2(rect.min.x + inset, rect.min.y + inset),
            pos2(rect.min.x + inset + inner_w, rect.max.y - inset),
        );
        painter.rect_filled(fill_rect, r, health_color(fraction));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn companion_view_draws_empty_active_downed_and_name_tags() {
        let ctx = egui::Context::default();
        let members = [
            PartyMember {
                name: "Mark Vance",
                health_fraction: 0.85,
                downed: false,
                order_label: "Follow",
            },
            PartyMember {
                name: "Sarah Jenkins",
                health_fraction: 0.0,
                downed: true,
                order_label: "Stay",
            },
        ];

        ctx.run_pass(Default::default(), |c| {
            draw_party_panel(c, &[]);
        })
        .textures_delta
        .clear();

        ctx.run_pass(Default::default(), |c| {
            draw_party_panel(c, &members);
            draw_name_tag(c, pos2(640.0, 300.0), "Mark Vance", 0.85);
            draw_name_tag(c, pos2(700.0, 300.0), "Elena Rostova", 0.3);
        })
        .textures_delta
        .clear();
    }
}
