//! FPS readout widget.
//!
//! Positioned below the reserved 180 px top-left minimap box (#7) so it never collides
//! with the minimap, compass, or clock.

use engine::egui::{self, pos2, Align2, Color32, FontId, Rect};
use super::text::draw_outline_text;

/// Reserved size of future minimap in top-left corner.
pub const RESERVED_MINIMAP_SIZE: f32 = 180.0;

/// Draws the FPS counter below the reserved minimap region.
pub fn draw_fps(painter: &egui::Painter, screen: &Rect, fps: f32, scale: f32) {
    let fps_text = format!("{:.0} FPS", fps);
    let x = screen.min.x + 16.0 * scale;
    let y = screen.min.y + (16.0 + RESERVED_MINIMAP_SIZE + 8.0) * scale;

    draw_outline_text(
        painter,
        pos2(x, y),
        Align2::LEFT_TOP,
        &fps_text,
        FontId::proportional(16.0 * scale),
        Color32::from_rgb(220, 220, 220),
    );
}
