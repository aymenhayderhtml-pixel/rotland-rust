//! Ammo readout widget with dynamic key binding prompt.
//!
//! Port of Godot C# `scripts/UI/HUD.cs` and GitHub issue #18 (UI-002):
//! Shows "Out of ammo: bash [KEY]" with the real Attack key bind when empty.

use engine::egui::{self, pos2, Align2, Color32, FontId, Rect};
use super::text::draw_outline_text;

/// Draws the ammo readout in the bottom-right corner.
pub fn draw_ammo(
    painter: &egui::Painter,
    screen: &Rect,
    ammo: Option<(u32, u32)>,
    attack_key: &str,
    scale: f32,
) {
    let Some((mag, spare)) = ammo else { return };

    let (ammo_text, font_size) = if mag == 0 && spare == 0 {
        let key_str = if attack_key.is_empty() { "LMB" } else { attack_key };
        (format!("Out of ammo: bash [{key_str}]"), 20.0 * scale)
    } else {
        (format!("{mag} / {spare}"), 32.0 * scale)
    };

    draw_outline_text(
        painter,
        pos2(screen.max.x - 24.0 * scale, screen.max.y - 36.0 * scale),
        Align2::RIGHT_BOTTOM,
        &ammo_text,
        FontId::proportional(font_size),
        Color32::WHITE,
    );
}
