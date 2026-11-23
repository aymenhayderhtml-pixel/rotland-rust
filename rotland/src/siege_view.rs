//! Base-siege HUD overlays: calm under-attack banner below the compass and looked-at part health bar.

use crate::menus::theme;
use engine::egui::{
    self, Align2, Color32, Frame, Margin, Rect, Sense, Stroke, StrokeKind, pos2, vec2,
};

// Base layout metrics at 1280x720, scaled by screen height via `theme::ui_scale`.
// Alert sits below the top compass; part health sits just below screen centre and well above
// the swimming (-204) and fishing (-112) overlays and bottom hotbar.
const ALERT_OFFSET_Y: f32 = 74.0; // new: DECISIONS 2026-11-23
const ALERT_PAD_X: f32 = 16.0; // new: DECISIONS 2026-11-23
const ALERT_PAD_Y: f32 = 8.0; // new: DECISIONS 2026-11-23

const PART_HEALTH_OFFSET_Y: f32 = 46.0; // new: DECISIONS 2026-11-23
const HEALTH_PAD_X: f32 = 12.0; // new: DECISIONS 2026-11-23
const HEALTH_PAD_Y: f32 = 6.0; // new: DECISIONS 2026-11-23
const HEALTH_LABEL_GAP: f32 = 4.0; // new: DECISIONS 2026-11-23
const HEALTH_BAR_WIDTH: f32 = 140.0; // new: DECISIONS 2026-11-23
const HEALTH_BAR_HEIGHT: f32 = 5.0; // new: DECISIONS 2026-11-23
const HEALTH_BAR_INSET: f32 = 1.0; // new: DECISIONS 2026-11-23
const HEALTH_BAR_RADIUS: f32 = 2.0; // new: DECISIONS 2026-11-23
const HEALTH_MID_FRACTION: f32 = 0.5; // new: DECISIONS 2026-11-23

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

fn health_bar_color(fraction: f32) -> Color32 {
    let t = fraction.clamp(0.0, 1.0);
    if t >= HEALTH_MID_FRACTION {
        let k = (t - HEALTH_MID_FRACTION) / (1.0 - HEALTH_MID_FRACTION);
        lerp_color(theme::ACCENT_HOVER, theme::SAFE, k)
    } else {
        let k = t / HEALTH_MID_FRACTION;
        lerp_color(theme::DANGER, theme::ACCENT_HOVER, k)
    }
}

/// Draws a calm, red-edged warning banner below the compass that fades smoothly with `fraction`.
/// Draws nothing when `fraction <= 0.0`.
pub fn draw_base_alert(ctx: &egui::Context, fraction: f32) {
    if !fraction.is_finite() || fraction <= 0.0 {
        return;
    }

    let alpha = fraction.clamp(0.0, 1.0);
    let scale = theme::ui_scale(ctx.content_rect().height());
    let bg = theme::PANEL_BG.gamma_multiply(alpha);
    let edge = theme::DANGER.gamma_multiply(alpha);
    let text = theme::TEXT_PRIMARY.gamma_multiply(alpha);

    egui::Area::new(egui::Id::unique("rotland_base_siege_alert"))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_TOP, vec2(0.0, ALERT_OFFSET_Y * scale))
        .interactable(false)
        .show(ctx, |ui| {
            let inner = Frame::NONE
                .fill(bg)
                .stroke(Stroke::new(1.0, edge))
                .corner_radius(theme::radius(theme::CORNER_PANEL, scale))
                .inner_margin(scaled_margin(ALERT_PAD_X, ALERT_PAD_Y, scale))
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new("Your base is under attack")
                                .size(theme::FONT_BODY * scale)
                                .color(text),
                        )
                        .wrap_mode(egui::TextWrapMode::Extend),
                    );
                });

            theme::paint_top_accent(ui, inner.response.rect, edge, scale);
        });
}

/// Draws a compact label and thin green-to-amber-to-red health bar just below the screen centre
/// for a damaged building part the player is looking at.
pub fn draw_part_health(ctx: &egui::Context, name: &str, fraction: f32) {
    let clamped = if fraction.is_finite() {
        fraction.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let scale = theme::ui_scale(ctx.content_rect().height());
    let fill_color = health_bar_color(clamped);

    egui::Area::new(egui::Id::unique("rotland_part_health"))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, PART_HEALTH_OFFSET_Y * scale))
        .interactable(false)
        .show(ctx, |ui| {
            Frame::NONE
                .fill(theme::PANEL_BG)
                .stroke(Stroke::new(1.0, theme::BORDER))
                .corner_radius(theme::radius(theme::CORNER_CONTROL, scale))
                .inner_margin(scaled_margin(HEALTH_PAD_X, HEALTH_PAD_Y, scale))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(name)
                                    .size(theme::FONT_HELP * scale)
                                    .color(theme::TEXT_PRIMARY),
                            )
                            .wrap_mode(egui::TextWrapMode::Extend),
                        );
                        ui.add_space(HEALTH_LABEL_GAP * scale);
                        draw_thin_health_bar(ui, clamped, fill_color, scale);
                    });
                });
        });
}

fn draw_thin_health_bar(ui: &mut egui::Ui, fraction: f32, fill_color: Color32, scale: f32) {
    let (rect, _) = ui.allocate_exact_size(
        vec2(HEALTH_BAR_WIDTH * scale, HEALTH_BAR_HEIGHT * scale),
        Sense::hover(),
    );
    let painter = ui.painter();
    let r = theme::radius(HEALTH_BAR_RADIUS, scale);

    painter.rect(
        rect,
        r,
        theme::CARD_EMPTY_BG,
        Stroke::new(1.0, theme::BORDER),
        StrokeKind::Inside,
    );

    let inset = HEALTH_BAR_INSET * scale;
    let inner_w = (rect.width() - inset * 2.0).max(0.0) * fraction;
    if inner_w > 0.0 {
        let fill_rect = Rect::from_min_max(
            pos2(rect.min.x + inset, rect.min.y + inset),
            pos2(rect.min.x + inset + inner_w, rect.max.y - inset),
        );
        painter.rect_filled(fill_rect, r, fill_color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_bar_color_transitions_green_amber_red() {
        assert_eq!(health_bar_color(1.0), theme::SAFE);
        assert_eq!(health_bar_color(0.5), theme::ACCENT_HOVER);
        assert_eq!(health_bar_color(0.0), theme::DANGER);
    }

    #[test]
    fn siege_view_draws_cleanly() {
        let ctx = egui::Context::default();
        for fraction in [0.0, 0.25, 0.5, 1.0] {
            ctx.run_pass(Default::default(), |c| {
                draw_base_alert(c, fraction);
                draw_part_health(c, "Wall", fraction);
            })
            .textures_delta
            .clear();
        }
    }
}
