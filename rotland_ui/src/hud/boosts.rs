//! Active medical boosts (antibiotics heal-over-time, adrenaline sprint) and night-vision battery HUD row.

use engine::egui::{self, Align2, Color32, FontId, Rect, Stroke, pos2, vec2};

use super::config::HudConfig;
use super::text::draw_outline_text;

/// Live boosts & night-vision snapshot passed from gameplay into the HUD.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BoostsHudInfo {
    pub heal_seconds: f32,
    pub adrenaline_seconds: f32,
    pub nvg_on: bool,
    pub nvg_charge_frac: f32,
}

impl BoostsHudInfo {
    pub fn has_any(&self) -> bool {
        self.heal_seconds > 0.0 || self.adrenaline_seconds > 0.0 || self.nvg_on
    }
}

/// Draws active boost timers and night-vision battery bar beside the vitals stack.
pub fn draw_boosts(
    painter: &egui::Painter,
    screen: &Rect,
    boosts: Option<&BoostsHudInfo>,
    scale: f32,
    config: &HudConfig,
) {
    let Some(info) = boosts else { return };
    if !info.has_any() {
        return;
    }

    let bg = config.bars.bg_color32();
    let corner_r = 4.0 * scale; // new: DECISIONS 2026-11-27
    let pill_h = 20.0 * scale; // new: DECISIONS 2026-11-27
    let mut x = screen.min.x + 286.0 * scale; // new: DECISIONS 2026-11-27 (right of vitals bars)
    let y = screen.max.y - 38.0 * scale; // new: DECISIONS 2026-11-27 (aligned with Health bar)
    let gap = 6.0 * scale; // new: DECISIONS 2026-11-27
    let font = FontId::proportional(11.0 * scale); // new: DECISIONS 2026-11-27

    if info.heal_seconds > 0.0 {
        let w = 52.0 * scale; // new: DECISIONS 2026-11-27
        let rect = Rect::from_min_size(pos2(x, y), vec2(w, pill_h));
        painter.rect_filled(rect, corner_r, bg);
        // Green medical cross icon
        let cx = x + 10.0 * scale; // new: DECISIONS 2026-11-27
        let cy = y + pill_h * 0.5;
        let arm = 4.0 * scale; // new: DECISIONS 2026-11-27
        let stroke = Stroke::new((2.0 * scale).max(1.5), Color32::from_rgb(95, 220, 120));
        painter.line_segment([pos2(cx - arm, cy), pos2(cx + arm, cy)], stroke);
        painter.line_segment([pos2(cx, cy - arm), pos2(cx, cy + arm)], stroke);
        draw_outline_text(
            painter,
            pos2(x + 20.0 * scale, cy),
            Align2::LEFT_CENTER,
            &format!("{:.0}s", info.heal_seconds.ceil()),
            font.clone(),
            Color32::from_rgb(225, 255, 225),
        );
        x += w + gap;
    }

    if info.adrenaline_seconds > 0.0 {
        let w = 52.0 * scale; // new: DECISIONS 2026-11-27
        let rect = Rect::from_min_size(pos2(x, y), vec2(w, pill_h));
        painter.rect_filled(rect, corner_r, bg);
        // Yellow sprint/lightning bolt icon
        let bx = x + 10.0 * scale; // new: DECISIONS 2026-11-27
        let by = y + pill_h * 0.5;
        let bolt = Stroke::new((2.0 * scale).max(1.5), Color32::from_rgb(255, 215, 70));
        painter.line_segment([pos2(bx + 2.0 * scale, by - 5.0 * scale), pos2(bx - 2.0 * scale, by)], bolt);
        painter.line_segment([pos2(bx - 2.0 * scale, by), pos2(bx + 2.0 * scale, by)], bolt);
        painter.line_segment([pos2(bx + 2.0 * scale, by), pos2(bx - 2.0 * scale, by + 5.0 * scale)], bolt);
        draw_outline_text(
            painter,
            pos2(x + 20.0 * scale, by),
            Align2::LEFT_CENTER,
            &format!("{:.0}s", info.adrenaline_seconds.ceil()),
            font.clone(),
            Color32::from_rgb(255, 240, 170),
        );
        x += w + gap;
    }

    if info.nvg_on {
        let w = 90.0 * scale; // new: DECISIONS 2026-11-27
        let rect = Rect::from_min_size(pos2(x, y), vec2(w, pill_h));
        painter.rect_filled(rect, corner_r, bg);
        draw_outline_text(
            painter,
            pos2(x + 6.0 * scale, y + pill_h * 0.5),
            Align2::LEFT_CENTER,
            "NVG",
            font,
            Color32::from_rgb(110, 255, 130),
        );
        let bar_rect = Rect::from_min_size(
            pos2(x + 36.0 * scale, y + 6.0 * scale),
            vec2(46.0 * scale, 8.0 * scale),
        );
        painter.rect_filled(bar_rect, 2.0 * scale, Color32::from_rgba_unmultiplied(15, 25, 18, 200));
        let frac = info.nvg_charge_frac.clamp(0.0, 1.0);
        if frac > 0.0 {
            let fill = Rect::from_min_size(
                bar_rect.min,
                vec2(bar_rect.width() * frac, bar_rect.height()),
            );
            let col = if frac < 0.2 {
                Color32::from_rgb(230, 80, 70)
            } else {
                Color32::from_rgb(90, 240, 115)
            };
            painter.rect_filled(fill, 2.0 * scale, col);
        }
    }
}
