//! Key help strip showing active key bindings.
//!
//! Port of Godot C# `scripts/UI/KeyBar.cs`:
//! Bottom-right strip above ammo counter, displaying current keybinds.

use engine::egui::{self, pos2, Align2, Color32, FontId, Rect};
use super::text::draw_outline_text;

/// Draws the key help strip in the bottom-right corner above the ammo readout.
pub fn draw_key_bar(painter: &egui::Painter, screen: &Rect, key_help: Option<&str>, scale: f32) {
    let Some(text) = key_help else { return };
    if text.is_empty() {
        return;
    }

    draw_outline_text(
        painter,
        pos2(screen.max.x - 16.0 * scale, screen.max.y - 106.0 * scale),
        Align2::RIGHT_BOTTOM,
        text,
        FontId::proportional(15.0 * scale),
        Color32::from_rgba_unmultiplied(255, 255, 255, 217),
    );
}
