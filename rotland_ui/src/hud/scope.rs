//! Rifle scope overlay with circular aperture and crosshair reticle.
//!
//! Port of Godot C# `scripts/UI/ScopeOverlay.cs`:
//! Blackout area around a clear circle (radius = 0.36 of shorter screen side),
//! with thin crosshair reticle lines.

use engine::egui::{self, pos2, Color32, Rect, Stroke};
use game::hud_math::{scope_clear_radius, scope_reticle_ends, scope_reticle_gap};

/// Draws the rifle scope overlay across the whole screen.
pub fn draw_scope_overlay(painter: &egui::Painter, screen: &Rect) {
    let center = screen.center();
    let radius = scope_clear_radius(screen.width(), screen.height());

    // 1. Blackout surrounding area
    // Corner blackout via large circle stroke matching Godot's DrawArc
    let thickness = screen.size().length();
    painter.circle_stroke(
        center,
        radius + thickness * 0.5,
        Stroke::new(thickness, Color32::BLACK),
    );

    // Complementary edge blackout rectangles to guarantee complete border coverage
    let dark = Color32::BLACK;
    painter.rect_filled(
        Rect::from_min_max(screen.min, pos2(screen.max.x, center.y - radius)),
        0.0,
        dark,
    );
    painter.rect_filled(
        Rect::from_min_max(pos2(screen.min.x, center.y + radius), screen.max),
        0.0,
        dark,
    );
    painter.rect_filled(
        Rect::from_min_max(pos2(screen.min.x, center.y - radius), pos2(center.x - radius, center.y + radius)),
        0.0,
        dark,
    );
    painter.rect_filled(
        Rect::from_min_max(pos2(center.x + radius, center.y - radius), pos2(screen.max.x, center.y + radius)),
        0.0,
        dark,
    );

    // 2. Reticle lines with central gap
    let gap = scope_reticle_gap(radius);
    let ends = scope_reticle_ends(radius);
    let reticle_color = Color32::from_rgba_unmultiplied(0, 0, 0, 217); // 85% alpha
    let stroke = Stroke::new(2.0, reticle_color);

    // Horizontal left and right
    painter.line_segment([pos2(center.x - ends, center.y), pos2(center.x - gap, center.y)], stroke);
    painter.line_segment([pos2(center.x + gap, center.y), pos2(center.x + ends, center.y)], stroke);

    // Vertical top and bottom
    painter.line_segment([pos2(center.x, center.y - ends), pos2(center.x, center.y - gap)], stroke);
    painter.line_segment([pos2(center.x, center.y + gap), pos2(center.x, center.y + ends)], stroke);
}
