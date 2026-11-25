//! Centered downed bleed-out and revive progress HUD bar.

use engine::egui::{self, pos2, vec2, Align2, Color32, CornerRadius, FontId, Rect, Stroke, StrokeKind};
use game::ReviveBy;

use super::text::draw_outline_text;

const PANEL_BG: Color32 = Color32::from_rgba_premultiplied(18, 20, 22, 234); // new: DECISIONS 2026-11-25
const CARD_EMPTY_BG: Color32 = Color32::from_rgba_premultiplied(12, 13, 15, 196); // new: DECISIONS 2026-11-25
const BORDER: Color32 = Color32::from_rgb(60, 65, 71); // new: DECISIONS 2026-11-25
const TEXT_PRIMARY: Color32 = Color32::from_rgb(238, 235, 227); // new: DECISIONS 2026-11-25
const TEXT_MUTED: Color32 = Color32::from_rgb(154, 152, 146); // new: DECISIONS 2026-11-25
const SAFE: Color32 = Color32::from_rgb(98, 156, 94); // new: DECISIONS 2026-11-25
const SAFE_HOVER: Color32 = Color32::from_rgb(120, 180, 114); // new: DECISIONS 2026-11-25
const DANGER: Color32 = Color32::from_rgb(186, 62, 54); // new: DECISIONS 2026-11-25
const DANGER_HOVER: Color32 = Color32::from_rgb(214, 80, 70); // new: DECISIONS 2026-11-25
const CORNER_PANEL: f32 = 6.0; // new: DECISIONS 2026-11-25
const CORNER_CONTROL: f32 = 4.0; // new: DECISIONS 2026-11-25

const PANEL_OFFSET_Y: f32 = 84.0; // new: DECISIONS 2026-11-25
const PANEL_WIDTH: f32 = 280.0; // new: DECISIONS 2026-11-25
const PANEL_PAD_X: f32 = 16.0; // new: DECISIONS 2026-11-25
const PANEL_PAD_Y: f32 = 12.0; // new: DECISIONS 2026-11-25
const BLEED_BAR_HEIGHT: f32 = 12.0; // new: DECISIONS 2026-11-25
const REVIVE_BAR_HEIGHT: f32 = 8.0; // new: DECISIONS 2026-11-25
const ROW_GAP: f32 = 8.0; // new: DECISIONS 2026-11-25
const TITLE_SIZE: f32 = 18.0; // new: DECISIONS 2026-11-25
const HINT_SIZE: f32 = 14.0; // new: DECISIONS 2026-11-25
const BAR_INSET: f32 = 1.0; // new: DECISIONS 2026-11-25
const ZERO: f32 = 0.0; // new: DECISIONS 2026-11-25
const ONE: f32 = 1.0; // new: DECISIONS 2026-11-25

fn scaled_radius(base: f32, scale: f32) -> CornerRadius {
    CornerRadius::same((base * scale).round().clamp(ZERO, u8::MAX as f32) as u8)
}

/// Snapshot of the player's downed timer and revive state for the HUD.
#[derive(Debug, Clone, PartialEq)]
pub struct DownedHudInfo {
    pub bleed_remaining: f32,
    pub bleed_total: f32,
    pub revive_fraction: f32,
    pub revive_by: ReviveBy,
    pub has_bandage: bool,
}

/// Draws a centered downed panel with `"DOWNED 23 s"`, bleed bar, revive progress, and hint.
pub fn draw_downed_bar(painter: &egui::Painter, screen: &Rect, info: &DownedHudInfo, scale: f32) {
    let bleed_total = info.bleed_total.max(0.001); // new: DECISIONS 2026-11-25
    let bleed_frac = (info.bleed_remaining / bleed_total).clamp(ZERO, ONE);
    let revive_frac = info.revive_fraction.clamp(ZERO, ONE);
    let secs_left = info.bleed_remaining.ceil().max(ZERO) as u32;

    let reviving = info.revive_by != ReviveBy::None;
    let hint = match info.revive_by {
        ReviveBy::Teammate => "Teammate reviving…",
        ReviveBy::SelfBandage => "Bandaging…",
        ReviveBy::None if info.has_bandage => "[Hold E] Bandage",
        ReviveBy::None => "Waiting for a teammate…",
    };

    let pad_x = PANEL_PAD_X * scale;
    let pad_y = PANEL_PAD_Y * scale;
    let gap = ROW_GAP * scale;
    let bar_w = (PANEL_WIDTH * scale - pad_x * 2.0).max(ZERO);
    let bleed_h = BLEED_BAR_HEIGHT * scale;
    let revive_h = REVIVE_BAR_HEIGHT * scale;
    let title_h = TITLE_SIZE * scale;
    let hint_h = HINT_SIZE * scale;
    let total_h = pad_y * 2.0
        + title_h
        + gap
        + bleed_h
        + if reviving { gap * 0.6 + revive_h } else { ZERO }
        + gap
        + hint_h;

    let panel_rect = Rect::from_center_size(
        pos2(screen.center().x, screen.center().y + PANEL_OFFSET_Y * scale),
        vec2(PANEL_WIDTH * scale, total_h),
    );
    let panel_r = scaled_radius(CORNER_PANEL, scale);
    let border = if reviving { SAFE } else { DANGER };
    painter.rect(
        panel_rect,
        panel_r,
        PANEL_BG,
        Stroke::new(ONE, border),
        StrokeKind::Inside,
    );

    let mut cursor_y = panel_rect.min.y + pad_y;
    let cx = panel_rect.center().x;

    let title = format!("DOWNED {secs_left} s");
    draw_outline_text(
        painter,
        pos2(cx, cursor_y + title_h * 0.5),
        Align2::CENTER_CENTER,
        &title,
        FontId::proportional(title_h),
        DANGER_HOVER,
    );
    cursor_y += title_h + gap;

    let bleed_rect = Rect::from_min_size(pos2(cx - bar_w * 0.5, cursor_y), vec2(bar_w, bleed_h));
    draw_progress_bar(painter, bleed_rect, bleed_frac, DANGER, scale);
    cursor_y += bleed_h;

    if reviving {
        cursor_y += gap * 0.6;
        let revive_rect =
            Rect::from_min_size(pos2(cx - bar_w * 0.5, cursor_y), vec2(bar_w, revive_h));
        draw_progress_bar(painter, revive_rect, revive_frac, SAFE, scale);
        cursor_y += revive_h;
    }

    cursor_y += gap;
    let hint_color = if reviving {
        SAFE_HOVER
    } else if info.has_bandage {
        TEXT_PRIMARY
    } else {
        TEXT_MUTED
    };
    draw_outline_text(
        painter,
        pos2(cx, cursor_y + hint_h * 0.5),
        Align2::CENTER_CENTER,
        hint,
        FontId::proportional(hint_h),
        hint_color,
    );
}

fn draw_progress_bar(
    painter: &egui::Painter,
    rect: Rect,
    fraction: f32,
    fill_color: Color32,
    scale: f32,
) {
    let r = scaled_radius(CORNER_CONTROL, scale);
    painter.rect(
        rect,
        r,
        CARD_EMPTY_BG,
        Stroke::new(ONE, BORDER),
        StrokeKind::Inside,
    );
    let inset = BAR_INSET * scale;
    let inner_w = (rect.width() - inset * 2.0).max(ZERO) * fraction.clamp(ZERO, ONE);
    if inner_w > ZERO {
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
    fn downed_hud_draws_cleanly() {
        let ctx = egui::Context::default();
        let samples = [
            DownedHudInfo {
                bleed_remaining: 23.0,
                bleed_total: 30.0,
                revive_fraction: 0.0,
                revive_by: ReviveBy::None,
                has_bandage: true,
            },
            DownedHudInfo {
                bleed_remaining: 15.0,
                bleed_total: 30.0,
                revive_fraction: 0.5,
                revive_by: ReviveBy::SelfBandage,
                has_bandage: true,
            },
            DownedHudInfo {
                bleed_remaining: 10.0,
                bleed_total: 30.0,
                revive_fraction: 0.25,
                revive_by: ReviveBy::Teammate,
                has_bandage: false,
            },
            DownedHudInfo {
                bleed_remaining: 5.0,
                bleed_total: 30.0,
                revive_fraction: 0.0,
                revive_by: ReviveBy::None,
                has_bandage: false,
            },
        ];
        for info in &samples {
            ctx.run_pass(Default::default(), |c| {
                let screen = c.content_rect();
                egui::Area::new(egui::Id::unique("test_downed_hud")).show(c, |ui| {
                    draw_downed_bar(ui.painter(), &screen, info, 1.0);
                });
            })
            .textures_delta
            .clear();
        }
    }
}
