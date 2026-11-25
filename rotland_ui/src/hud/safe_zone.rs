//! "Safe zone" notification label.
//!
//! Port of Godot C# `scripts/UI/SafeZoneLabel.cs`:
//! Small green label at the top-centre, shown while the player is inside a settlement safe zone.

use engine::egui::{self, pos2, Align2, Color32, FontId, Rect};
use super::text::draw_outline_text;

/// Draws the "Safe zone" label below the compass when inside a safe zone.
pub fn draw_safe_zone(painter: &egui::Painter, screen: &Rect, in_safe_zone: bool, scale: f32) {
    if !in_safe_zone {
        return;
    }

    let safe_color = Color32::from_rgb(178, 255, 178);
    draw_outline_text(
        painter,
        pos2(screen.center().x, screen.min.y + 96.0 * scale),
        Align2::CENTER_TOP,
        "Safe zone",
        FontId::proportional(22.0 * scale),
        safe_color,
    );
}
