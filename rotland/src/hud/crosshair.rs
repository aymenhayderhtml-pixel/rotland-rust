//! Center crosshair widget with hip-fire dot and four directional ticks.
//!
//! Port of Godot C# `scripts/UI/Crosshair.cs` with owner's rule from GitHub #4:
//! Hidden when holding melee weapon, under rifle scope, or when dead.

use engine::egui::{self, pos2, Color32, Pos2, Stroke};

/// Draws the center crosshair: central dot with four ticks (up, down, left, right).
pub fn draw_crosshair(painter: &egui::Painter, center: Pos2, scale: f32) {
    let line_color = Color32::from_rgba_unmultiplied(255, 255, 255, 230);
    let hip_gap = 12.0 * scale;
    let line_len = 8.0 * scale;
    let line_width = 2.0 * scale;

    // Center dot
    painter.circle_filled(center, 2.0 * scale, line_color);

    let stroke = Stroke::new(line_width, line_color);

    // Up
    painter.line_segment(
        [
            pos2(center.x, center.y - hip_gap),
            pos2(center.x, center.y - hip_gap - line_len),
        ],
        stroke,
    );
    // Down
    painter.line_segment(
        [
            pos2(center.x, center.y + hip_gap),
            pos2(center.x, center.y + hip_gap + line_len),
        ],
        stroke,
    );
    // Left
    painter.line_segment(
        [
            pos2(center.x - hip_gap, center.y),
            pos2(center.x - hip_gap - line_len, center.y),
        ],
        stroke,
    );
    // Right
    painter.line_segment(
        [
            pos2(center.x + hip_gap, center.y),
            pos2(center.x + hip_gap + line_len, center.y),
        ],
        stroke,
    );
}
