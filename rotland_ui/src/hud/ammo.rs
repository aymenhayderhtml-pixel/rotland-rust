//! Ammo readout widget with dynamic key binding prompt.
//!
//! Port of Godot C# `scripts/UI/HUD.cs` and GitHub issue #18 (UI-002):
//! Shows "Out of ammo: bash [KEY]" with the real Attack key bind when empty.

use engine::egui::{self, pos2, Align2, Color32, FontId, Rect};
use super::text::draw_outline_text;

/// Formats the ammo string for display or testing.
pub fn format_ammo_text(ammo: Option<(u32, u32)>, attack_key: &str) -> Option<String> {
    let (mag, spare) = ammo?;
    if spare == u32::MAX {
        Some(format!("{mag}"))
    } else if mag == 0 && spare == 0 {
        let key_str = if attack_key.is_empty() { "LMB" } else { attack_key };
        Some(format!("Out of ammo: bash [{key_str}]"))
    } else {
        Some(format!("{mag} / {spare}"))
    }
}

/// Draws the ammo readout in the bottom-right corner.
pub fn draw_ammo(
    painter: &egui::Painter,
    screen: &Rect,
    ammo: Option<(u32, u32)>,
    attack_key: &str,
    scale: f32,
) {
    let Some(ammo_text) = format_ammo_text(ammo, attack_key) else { return };
    let font_size = if ammo.is_some_and(|(m, s)| m == 0 && s == 0) {
        20.0 * scale
    } else {
        32.0 * scale
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_ammo_text_rules() {
        assert_eq!(format_ammo_text(None, "LMB"), None);
        assert_eq!(format_ammo_text(Some((12, 24)), "LMB"), Some("12 / 24".to_string()));
        assert_eq!(format_ammo_text(Some((0, 0)), "LMB"), Some("Out of ammo: bash [LMB]".to_string()));
        assert_eq!(format_ammo_text(Some((20, u32::MAX)), "LMB"), Some("20".to_string()));
        assert_eq!(format_ammo_text(Some((0, u32::MAX)), "LMB"), Some("0".to_string()));
    }
}
