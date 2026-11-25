//! Centre-top biome / danger-tier entry banner.
//!
//! Draws a short notification (e.g. `Entering: Desert` / `Danger 3` or
//! `Back in home lands`) below the compass and safe-zone label.

use engine::egui::{self, pos2, Align2, Color32, FontId, Rect};

use super::text::draw_outline_text;

const BANNER_TOP_Y: f32 = 128.0; // new: DECISIONS 2026-11-25
const TITLE_FONT_SIZE: f32 = 26.0; // new: DECISIONS 2026-11-25
const SUBTITLE_OFFSET_Y: f32 = 30.0; // new: DECISIONS 2026-11-25
const SUBTITLE_FONT_SIZE: f32 = 18.0; // new: DECISIONS 2026-11-25

/// Display payload for the region entry banner.
#[derive(Clone, Debug, PartialEq)]
pub struct RegionBannerInfo {
    pub title: String,
    pub subtitle: Option<String>,
    pub alpha: f32,
}

/// Draws the region entry banner at the top-centre of the screen when active.
pub fn draw_region_banner(
    painter: &egui::Painter,
    screen: &Rect,
    banner: Option<&RegionBannerInfo>,
    scale: f32,
) {
    let Some(info) = banner else {
        return;
    };
    let alpha = (info.alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    if alpha == 0 {
        return;
    }

    let cx = screen.center().x;
    let y = screen.min.y + BANNER_TOP_Y * scale;

    let title_color = Color32::from_rgba_unmultiplied(245, 236, 212, alpha);
    draw_outline_text(
        painter,
        pos2(cx, y),
        Align2::CENTER_TOP,
        &info.title,
        FontId::proportional(TITLE_FONT_SIZE * scale),
        title_color,
    );

    if let Some(sub) = &info.subtitle {
        let sub_color = Color32::from_rgba_unmultiplied(255, 168, 102, alpha);
        draw_outline_text(
            painter,
            pos2(cx, y + SUBTITLE_OFFSET_Y * scale),
            Align2::CENTER_TOP,
            sub,
            FontId::proportional(SUBTITLE_FONT_SIZE * scale),
            sub_color,
        );
    }
}
