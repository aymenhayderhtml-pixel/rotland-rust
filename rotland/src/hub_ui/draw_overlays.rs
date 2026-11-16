//! egui 0.37 rendering for LevelUpBanner and XpToastStack HUD overlays.

use engine::egui::{self, pos2, vec2, Align2, Color32, FontId, Order, Pos2};

use super::state::{HubUiState, LEVEL_UP_BANNER_DURATION, XP_TOAST_LIFETIME};

pub const TOAST_RIGHT_MARGIN: f32 = 24.0;
pub const TOAST_BOTTOM_MARGIN: f32 = 200.0;
pub const TOAST_LINE_HEIGHT: f32 = 26.0;

/// Draws the Level Up announcement banner and XP toast stack on top of the game screen.
pub fn draw_overlays(ctx: &egui::Context, state: &HubUiState) {
    if state.level_up_banner.is_none() && state.xp_toasts.is_empty() {
        return;
    }

    egui::Area::new(egui::Id::unique("hub_hud_overlays"))
        .fixed_pos(pos2(0.0, 0.0))
        .interactable(false)
        .order(Order::Foreground)
        .show(ctx, |ui| {
            let painter = ui.painter();
            let screen = ctx.content_rect();

            // 1. Level Up Banner (centered large text)
            if let Some(banner) = &state.level_up_banner {
                let frac = (banner.time_left / LEVEL_UP_BANNER_DURATION).clamp(0.0, 1.0);
                let alpha = if frac < 0.2 { (frac / 0.2 * 255.0) as u8 } else { 255 };
                let banner_color = Color32::from_rgba_unmultiplied(255, 209, 76, alpha);

                draw_outline_text(
                    painter,
                    screen.center(),
                    Align2::CENTER_CENTER,
                    &banner.text,
                    FontId::proportional(56.0),
                    banner_color,
                    alpha,
                );
            }

            // 2. XP Toast Stack (bottom right, above stamina bar)
            let base_x = screen.max.x - TOAST_RIGHT_MARGIN;
            let base_y = screen.max.y - TOAST_BOTTOM_MARGIN;

            for (i, toast) in state.xp_toasts.iter().enumerate() {
                let y = base_y - (i as f32 * TOAST_LINE_HEIGHT);
                let alpha_f = (1.0 - toast.age / XP_TOAST_LIFETIME).clamp(0.0, 1.0);
                let alpha = (alpha_f * 255.0) as u8;
                let text = format!("+{} {} XP", toast.amount.round() as i32, toast.skill_name);

                draw_outline_text(
                    painter,
                    pos2(base_x, y),
                    Align2::RIGHT_BOTTOM,
                    &text,
                    FontId::proportional(20.0),
                    Color32::from_rgba_unmultiplied(240, 240, 240, alpha),
                    alpha,
                );
            }
        });
}

fn draw_outline_text(
    painter: &egui::Painter,
    pos: Pos2,
    anchor: Align2,
    text: &str,
    font_id: FontId,
    color: Color32,
    alpha: u8,
) {
    let outline_alpha = (alpha as f32 * 0.85) as u8;
    let outline_color = Color32::from_rgba_unmultiplied(0, 0, 0, outline_alpha);
    let offsets = [
        vec2(-1.0, 0.0),
        vec2(1.0, 0.0),
        vec2(0.0, -1.0),
        vec2(0.0, 1.0),
    ];
    for offset in offsets {
        painter.text(pos + offset, anchor, text, font_id.clone(), outline_color);
    }
    painter.text(pos, anchor, text, font_id, color);
}
