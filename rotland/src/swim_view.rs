//! Swimming stamina and drowning warning overlay placed above the hotbar and fishing panel.

use crate::menus::theme;
use engine::egui::{
    self, Align2, Color32, Frame, Margin, Rect, Sense, Stroke, StrokeKind, pos2, vec2,
};
use game::swimming::WaterWarning;

// Base layout metrics at 1280x720, scaled by screen height via `theme::ui_scale`.
// Placed cleanly above the fishing bar (-112) and hotbar so they never overlap.
const ANCHOR_OFFSET_Y: f32 = -204.0; // new: DECISIONS 2026-11-23
const PAD_X: f32 = 16.0; // new: DECISIONS 2026-11-23
const PAD_Y: f32 = 10.0; // new: DECISIONS 2026-11-23
const BAR_WIDTH: f32 = 210.0; // new: DECISIONS 2026-11-23
const BAR_HEIGHT: f32 = 8.0; // new: DECISIONS 2026-11-23
const BAR_INSET: f32 = 1.0; // new: DECISIONS 2026-11-23
const BAR_RADIUS: f32 = 3.0; // new: DECISIONS 2026-11-23

fn scaled_margin(x: f32, y: f32, scale: f32) -> Margin {
    Margin::symmetric(
        (x * scale).round().clamp(1.0, f32::from(i8::MAX)) as i8,
        (y * scale).round().clamp(1.0, f32::from(i8::MAX)) as i8,
    )
}

fn pulse_01(time: f64, speed: f64) -> f32 {
    ((time * speed).sin() * 0.5 + 0.5) as f32
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

/// Draws the deep-water stamina/drowning warning. Draws nothing and requests no repaint for `None`.
pub fn draw_water_warning(
    ctx: &egui::Context,
    warning: WaterWarning,
    stamina_fraction: f32,
) {
    if warning == WaterWarning::None {
        return;
    }

    let scale = theme::ui_scale(ctx.content_rect().height());
    let time = ctx.input(|i| i.time);
    ctx.request_repaint();

    let fraction = if stamina_fraction.is_finite() {
        stamina_fraction.clamp(0.0, 1.0)
    } else {
        0.0
    };

    let (border_color, accent_color) = match warning {
        WaterWarning::Tired => {
            // new: DECISIONS 2026-11-23
            let p = pulse_01(time, 3.0);
            let c = lerp_color(theme::ACCENT, theme::ACCENT_HOVER, p);
            (c, c)
        }
        WaterWarning::Drowning => {
            // new: DECISIONS 2026-11-23
            let p = pulse_01(time, 7.0);
            let c = lerp_color(theme::DANGER, theme::DANGER_HOVER, p);
            (c, c)
        }
        WaterWarning::None => return,
    };

    egui::Area::new(egui::Id::unique("rotland_swim_warning"))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, ANCHOR_OFFSET_Y * scale))
        .interactable(false)
        .show(ctx, |ui| {
            let inner = Frame::NONE
                .fill(theme::PANEL_BG)
                .stroke(Stroke::new(1.0, border_color))
                .corner_radius(theme::radius(theme::CORNER_PANEL, scale))
                .inner_margin(scaled_margin(PAD_X, PAD_Y, scale))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| match warning {
                        WaterWarning::Tired => {
                            ui.label(
                                egui::RichText::new("Getting tired, swim to shore")
                                    .size(theme::FONT_BODY * scale)
                                    .color(theme::TEXT_PRIMARY),
                            );
                            // new: DECISIONS 2026-11-23
                            ui.add_space(6.0 * scale);
                            draw_stamina_bar(ui, fraction, accent_color, scale);
                        }
                        WaterWarning::Drowning => {
                            ui.label(
                                egui::RichText::new("Drowning!")
                                    .size(theme::FONT_SUBHEADING * scale)
                                    .color(accent_color)
                                    .strong(),
                            );
                            // new: DECISIONS 2026-11-23
                            ui.add_space(5.0 * scale);
                            // new: DECISIONS 2026-11-23
                            let flash_alpha = 0.25 + 0.35 * pulse_01(time, 7.0);
                            draw_empty_danger_bar(
                                ui,
                                theme::DANGER.gamma_multiply(flash_alpha),
                                scale,
                            );
                        }
                        WaterWarning::None => {}
                    });
                });

            theme::paint_top_accent(ui, inner.response.rect, accent_color, scale);
        });
}

fn draw_stamina_bar(ui: &mut egui::Ui, fraction: f32, fill_color: Color32, scale: f32) {
    let (rect, _) =
        ui.allocate_exact_size(vec2(BAR_WIDTH * scale, BAR_HEIGHT * scale), Sense::hover());
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
        painter.rect_filled(fill_rect, r, fill_color);
    }
}

fn draw_empty_danger_bar(ui: &mut egui::Ui, tint: Color32, scale: f32) {
    let (rect, _) =
        ui.allocate_exact_size(vec2(BAR_WIDTH * scale, BAR_HEIGHT * scale), Sense::hover());
    let painter = ui.painter();
    let r = theme::radius(BAR_RADIUS, scale);

    painter.rect(
        rect,
        r,
        tint,
        Stroke::new(1.0, theme::DANGER),
        StrokeKind::Inside,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swim_view_draw() {
        let ctx = egui::Context::default();
        for (warning, fraction) in [(WaterWarning::None, 1.0), (WaterWarning::Tired, 0.2), (WaterWarning::Drowning, 0.0)] {
            ctx.run_pass(Default::default(), |c| draw_water_warning(c, warning, fraction)).textures_delta.clear();
        }
    }
}
