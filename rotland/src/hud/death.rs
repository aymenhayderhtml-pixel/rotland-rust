//! Death overlay screen shown when player is killed.
//!
//! Port of Godot C# `scripts/UI/HUD.cs` (DeathOverlay).

use engine::egui::{self, pos2, Align2, Color32, FontId, Rect};
use super::text::draw_outline_text;

/// Draws the death overlay with red "YOU DIED" text and death details.
pub fn draw_death_overlay(painter: &egui::Painter, screen: &Rect, msg: Option<&str>, scale: f32) {
    // 70% opacity black background
    painter.rect_filled(
        *screen,
        0.0,
        Color32::from_rgba_unmultiplied(0, 0, 0, 178),
    );

    let center_x = screen.center().x;
    let center_y = screen.center().y;

    // "YOU DIED"
    draw_outline_text(
        painter,
        pos2(center_x, center_y - 60.0 * scale),
        Align2::CENTER_CENTER,
        "YOU DIED",
        FontId::proportional(72.0 * scale),
        Color32::from_rgb(220, 38, 38),
    );

    // Lost items summary or message
    let lost_text = msg.unwrap_or("You had nothing to lose");
    draw_outline_text(
        painter,
        pos2(center_x, center_y + 10.0 * scale),
        Align2::CENTER_CENTER,
        lost_text,
        FontId::proportional(26.0 * scale),
        Color32::WHITE,
    );

    // Skills retained
    draw_outline_text(
        painter,
        pos2(center_x, center_y + 50.0 * scale),
        Align2::CENTER_CENTER,
        "Your skills are kept.",
        FontId::proportional(18.0 * scale),
        Color32::from_rgb(180, 180, 180),
    );

    // Respawning prompt
    draw_outline_text(
        painter,
        pos2(center_x, center_y + 90.0 * scale),
        Align2::CENTER_CENTER,
        "Respawning…",
        FontId::proportional(22.0 * scale),
        Color32::WHITE,
    );
}
