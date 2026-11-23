//! Fishing HUD overlays: cast hint, waiting indicator, bite timing bar, and catch result.

use crate::menus::theme;
use engine::egui::{
    self, Align2, Color32, Frame, Margin, Rect, Sense, Stroke, StrokeKind, pos2, vec2,
};
use game::fishing::{FishingMachine, FishingPhase};

// Base layout metrics at 1280x720, scaled by screen height via `theme::ui_scale`.
const ANCHOR_OFFSET_Y: f32 = -112.0; // new: DECISIONS 2026-11-23
const PAD_X: f32 = 16.0; // new: DECISIONS 2026-11-23
const PAD_Y: f32 = 10.0; // new: DECISIONS 2026-11-23
const BAR_WIDTH: f32 = 240.0; // new: DECISIONS 2026-11-23
const BAR_HEIGHT: f32 = 18.0; // new: DECISIONS 2026-11-23
const SWEET_SPOT_INSET_Y: f32 = 2.0; // new: DECISIONS 2026-11-23
const SWEET_SPOT_RADIUS: f32 = 2.5; // new: DECISIONS 2026-11-23
const NEEDLE_WIDTH: f32 = 4.0; // new: DECISIONS 2026-11-23
const NEEDLE_OVERHANG: f32 = 6.0; // new: DECISIONS 2026-11-23
const NEEDLE_RADIUS: f32 = 2.0; // new: DECISIONS 2026-11-23
const DOT_COUNT: usize = 3; // new: DECISIONS 2026-11-23
const DOT_RADIUS: f32 = 2.8; // new: DECISIONS 2026-11-23
const DOT_SPACING: f32 = 9.0; // new: DECISIONS 2026-11-23
const HINT_DOT_RADIUS: f32 = 3.2; // new: DECISIONS 2026-11-23

fn scaled_margin(x: f32, y: f32, scale: f32) -> Margin {
    Margin::symmetric(
        (x * scale).round().clamp(1.0, f32::from(i8::MAX)) as i8,
        (y * scale).round().clamp(1.0, f32::from(i8::MAX)) as i8,
    )
}

fn pulse_dot_alpha(time: f64, index: usize) -> f32 {
    // new: DECISIONS 2026-11-23
    const SPEED: f64 = 2.4;
    // new: DECISIONS 2026-11-23
    const PHASE_STEP: f64 = 0.85;
    // new: DECISIONS 2026-11-23
    const MIN_ALPHA: f32 = 0.25;
    // new: DECISIONS 2026-11-23
    const MAX_ALPHA: f32 = 0.95;

    let wave = ((time * SPEED - index as f64 * PHASE_STEP).sin() * 0.5 + 0.5) as f32;
    MIN_ALPHA + (MAX_ALPHA - MIN_ALPHA) * wave
}

fn result_alpha(progress: f32) -> f32 {
    // Hold full opacity briefly so the result reads clearly, then fade out smoothly.
    // new: DECISIONS 2026-11-23
    const HOLD_FRACTION: f32 = 0.45;
    let p = progress.clamp(0.0, 1.0);
    if p <= HOLD_FRACTION {
        1.0
    } else {
        ((1.0 - p) / (1.0 - HOLD_FRACTION)).clamp(0.0, 1.0)
    }
}

/// Draws a quiet "Click to cast" prompt when the player holds a fishing rod facing water.
pub fn draw_fishing_hint(ctx: &egui::Context) {
    let scale = theme::ui_scale(ctx.content_rect().height());
    let time = ctx.input(|i| i.time);
    ctx.request_repaint();

    egui::Area::new(egui::Id::unique("rotland_fishing_hint"))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, ANCHOR_OFFSET_Y * scale))
        .interactable(false)
        .show(ctx, |ui| {
            let inner = Frame::NONE
                .fill(theme::PANEL_BG)
                .stroke(Stroke::new(1.0, theme::BORDER))
                .corner_radius(theme::radius(theme::CORNER_PANEL, scale))
                .inner_margin(scaled_margin(PAD_X, PAD_Y * 0.8, scale))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let dot_size = vec2(HINT_DOT_RADIUS * 2.5 * scale, theme::FONT_BODY * scale);
                        let (dot_rect, _) = ui.allocate_exact_size(dot_size, Sense::hover());
                        let alpha = pulse_dot_alpha(time, 0);
                        ui.painter().circle_filled(
                            dot_rect.center(),
                            HINT_DOT_RADIUS * scale,
                            theme::ACCENT.gamma_multiply(alpha),
                        );

                        ui.label(
                            egui::RichText::new("Click to cast")
                                .size(theme::FONT_BODY * scale)
                                .color(theme::TEXT_PRIMARY),
                        );
                    });
                });

            theme::paint_top_accent(ui, inner.response.rect, theme::ACCENT, scale);
        });
}

/// Draws the fishing status panel and bite timing bar low in the centre of the screen.
pub fn draw_fishing_bar(ctx: &egui::Context, machine: &FishingMachine) {
    if machine.phase == FishingPhase::Idle {
        return;
    }

    let scale = theme::ui_scale(ctx.content_rect().height());
    let time = ctx.input(|i| i.time);
    ctx.request_repaint();

    let (border_color, accent_color, panel_alpha) = match machine.phase {
        FishingPhase::Waiting => (theme::BORDER, theme::TEXT_MUTED, 1.0),
        FishingPhase::Bite => (theme::ACCENT, theme::ACCENT, 1.0),
        FishingPhase::Caught => {
            let a = result_alpha(machine.result_progress());
            (theme::SAFE.gamma_multiply(a), theme::SAFE.gamma_multiply(a), a)
        }
        FishingPhase::Escaped => {
            let a = result_alpha(machine.result_progress());
            (
                theme::DANGER.gamma_multiply(a),
                theme::DANGER.gamma_multiply(a),
                a,
            )
        }
        FishingPhase::Idle => return,
    };

    egui::Area::new(egui::Id::unique("rotland_fishing_bar"))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, ANCHOR_OFFSET_Y * scale))
        .interactable(false)
        .show(ctx, |ui| {
            let inner = Frame::NONE
                .fill(theme::PANEL_BG.gamma_multiply(panel_alpha))
                .stroke(Stroke::new(1.0, border_color))
                .corner_radius(theme::radius(theme::CORNER_PANEL, scale))
                .inner_margin(scaled_margin(PAD_X, PAD_Y, scale))
                .show(ui, |ui| match machine.phase {
                    FishingPhase::Waiting => draw_waiting_contents(ui, scale, time),
                    FishingPhase::Bite => draw_bite_contents(ui, machine, scale),
                    FishingPhase::Caught => {
                        ui.label(
                            egui::RichText::new("Caught a fish!")
                                .size(theme::FONT_SUBHEADING * scale)
                                .color(theme::SAFE_HOVER.gamma_multiply(panel_alpha))
                                .strong(),
                        );
                    }
                    FishingPhase::Escaped => {
                        ui.label(
                            egui::RichText::new("The fish got away...")
                                .size(theme::FONT_BODY * scale)
                                .color(theme::DANGER_HOVER.gamma_multiply(panel_alpha)),
                        );
                    }
                    FishingPhase::Idle => {}
                });

            theme::paint_top_accent(ui, inner.response.rect, accent_color, scale);
        });
}

fn draw_waiting_contents(ui: &mut egui::Ui, scale: f32, time: f64) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Waiting for a bite")
                .size(theme::FONT_BODY * scale)
                .color(theme::TEXT_MUTED),
        );

        let dots_w = (DOT_COUNT as f32) * DOT_SPACING * scale;
        let (rect, _) = ui.allocate_exact_size(vec2(dots_w, theme::FONT_BODY * scale), Sense::hover());
        let painter = ui.painter();
        let cy = rect.center().y + 1.0 * scale;
        for i in 0..DOT_COUNT {
            let cx = rect.left() + (i as f32 + 0.5) * DOT_SPACING * scale;
            let alpha = pulse_dot_alpha(time, i);
            painter.circle_filled(
                pos2(cx, cy),
                DOT_RADIUS * scale,
                theme::TEXT_PRIMARY.gamma_multiply(alpha),
            );
        }
    });
}

fn draw_bite_contents(ui: &mut egui::Ui, machine: &FishingMachine, scale: f32) {
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("Click now!")
                .size(theme::FONT_SUBHEADING * scale)
                .color(theme::ACCENT_HOVER)
                .strong(),
        );
        // new: DECISIONS 2026-11-23
        ui.add_space(6.0 * scale);

        let (rect, _) =
            ui.allocate_exact_size(vec2(BAR_WIDTH * scale, BAR_HEIGHT * scale), Sense::hover());
        let painter = ui.painter();

        // Rounded track
        let track_radius = theme::radius(theme::CORNER_CONTROL, scale);
        painter.rect(
            rect,
            track_radius,
            theme::CARD_EMPTY_BG,
            Stroke::new(1.0, theme::BORDER),
            StrokeKind::Inside,
        );

        // Soft green sweet-spot zone
        let inset = SWEET_SPOT_INSET_Y * scale;
        let sweet_min_x = rect.min.x + machine.sweet_spot_min.clamp(0.0, 1.0) * rect.width();
        let sweet_max_x = rect.min.x + machine.sweet_spot_max.clamp(0.0, 1.0) * rect.width();
        let sweet_rect = Rect::from_min_max(
            pos2(sweet_min_x, rect.min.y + inset),
            pos2(sweet_max_x, rect.max.y - inset),
        );
        painter.rect_filled(
            sweet_rect,
            theme::radius(SWEET_SPOT_RADIUS, scale),
            theme::SAFE,
        );

        // White timing needle with dark outline for contrast
        let needle_x = rect.min.x + machine.progress() * rect.width();
        let needle_rect = Rect::from_center_size(
            pos2(needle_x, rect.center().y),
            vec2(NEEDLE_WIDTH * scale, rect.height() + NEEDLE_OVERHANG * scale),
        );
        painter.rect(
            needle_rect,
            theme::radius(NEEDLE_RADIUS, scale),
            Color32::WHITE,
            Stroke::new(1.0, theme::PANEL_BG),
            StrokeKind::Outside,
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fishing_view_draw() {
        let ctx = egui::Context::default();
        let mut machine = FishingMachine::new(&game::fishing::FishingConfig::default());
        ctx.run_pass(Default::default(), draw_fishing_hint).textures_delta.clear();
        for phase in [FishingPhase::Idle, FishingPhase::Waiting, FishingPhase::Bite, FishingPhase::Caught, FishingPhase::Escaped] {
            machine.phase = phase;
            ctx.run_pass(Default::default(), |c| draw_fishing_bar(c, &machine)).textures_delta.clear();
        }
    }

    #[test]
    fn test_result_alpha_fades_out() {
        assert_eq!(result_alpha(0.0), 1.0);
        // new: DECISIONS 2026-11-23
        let mid = result_alpha(0.75);
        assert!(mid > 0.0 && mid < 1.0);
        assert_eq!(result_alpha(1.0), 0.0);
    }
}
