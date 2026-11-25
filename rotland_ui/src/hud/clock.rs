//! Clock widget showing day and time in the top-right corner.
//!
//! Port of Godot C# `scripts/UI/ClockLabel.cs`.

use engine::egui::{self, pos2, Align2, Color32, FontId, Rect};
use super::text::draw_outline_text;

/// Draws the game clock "Day N · HH:MM" in the top-right corner.
pub fn draw_clock(painter: &egui::Painter, screen: &Rect, hour: f32, day: u32, scale: f32) {
    let hours = (hour.floor() as u32) % 24;
    let minutes = ((hour.fract() * 60.0).floor() as u32).min(59);
    let clock_text = format!("Day {} · {:02}:{:02}", day, hours, minutes);

    draw_outline_text(
        painter,
        pos2(screen.max.x - 16.0 * scale, screen.min.y + 16.0 * scale),
        Align2::RIGHT_TOP,
        &clock_text,
        FontId::proportional(18.0 * scale),
        Color32::WHITE,
    );
}
