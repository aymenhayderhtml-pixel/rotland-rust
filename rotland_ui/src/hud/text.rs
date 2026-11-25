//! Shared text rendering with high-contrast dark drop-shadow outline.

use engine::egui::{self, vec2, Align2, Color32, FontId, Pos2};

/// Draws text with a 1px dark drop-shadow outline for high contrast against any game background.
pub fn draw_outline_text(
    painter: &egui::Painter,
    pos: Pos2,
    anchor: Align2,
    text: &str,
    font_id: FontId,
    color: Color32,
) {
    let outline_color = Color32::from_rgba_unmultiplied(0, 0, 0, 230);
    let offsets = [
        vec2(-1.0, 0.0),
        vec2(1.0, 0.0),
        vec2(0.0, -1.0),
        vec2(0.0, 1.0),
        vec2(-1.0, -1.0),
        vec2(1.0, -1.0),
        vec2(-1.0, 1.0),
        vec2(1.0, 1.0),
    ];
    for offset in offsets {
        painter.text(pos + offset, anchor, text, font_id.clone(), outline_color);
    }
    painter.text(pos, anchor, text, font_id, color);
}
