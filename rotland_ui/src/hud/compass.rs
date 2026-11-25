//! Compass strip with directional ticks, camp marker, and waypoint diamond.
//!
//! Port of Godot C# `scripts/UI/Compass.cs` and `scripts/UI/CompassMath.cs`.

use engine::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};
use game::hud_math::{
    bearing_to_view_deg, direction_name, marker_x, shows_behind, HALF_VIEW_DEG,
};
use super::text::draw_outline_text;

/// Information for camp indicator on the compass.
#[derive(Debug, Clone, PartialEq)]
pub struct CampHudInfo {
    pub angle_deg: f32,
    pub distance_metres: f32,
}

impl CampHudInfo {
    pub const fn new(angle_deg: f32, distance_metres: f32) -> Self {
        Self {
            angle_deg,
            distance_metres,
        }
    }
}

/// Information for player waypoint diamond indicator on the compass.
#[derive(Debug, Clone, PartialEq)]
pub struct WaypointHudInfo {
    pub angle_deg: f32,
    pub distance_metres: f32,
}

impl WaypointHudInfo {
    pub const fn new(angle_deg: f32, distance_metres: f32) -> Self {
        Self {
            angle_deg,
            distance_metres,
        }
    }
}

/// Draws the compass strip along the top-centre of the screen.
pub fn draw_compass(
    painter: &egui::Painter,
    screen: &Rect,
    yaw_rad: f32,
    in_safe_zone: bool,
    camp: Option<&CampHudInfo>,
    waypoint: Option<&WaypointHudInfo>,
    scale: f32,
) {
    let strip_w = (600.0 * scale).min(screen.width() - 32.0 * scale);
    let strip_h = 40.0 * scale;
    let center_x = screen.center().x;
    let strip_rect = Rect::from_min_size(
        pos2(center_x - strip_w * 0.5, screen.min.y + 10.0 * scale),
        vec2(strip_w, strip_h),
    );

    // Strip background: Color(0, 0, 0, 0.62)
    painter.rect_filled(strip_rect, 0.0, Color32::from_rgba_unmultiplied(0, 0, 0, 158));

    // Bottom line: Color(1, 1, 1, 0.3)
    painter.line_segment(
        [
            pos2(strip_rect.min.x, strip_rect.max.y),
            pos2(strip_rect.max.x, strip_rect.max.y),
        ],
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 76)),
    );

    // Center view tick: Color(1, 1, 1, 0.7)
    painter.line_segment(
        [
            pos2(center_x, strip_rect.min.y),
            pos2(center_x, strip_rect.min.y + 6.0 * scale),
        ],
        Stroke::new(2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 178)),
    );

    let text_color = Color32::from_rgb(245, 245, 245);
    let north_color = Color32::from_rgb(255, 209, 76);
    let camp_color = Color32::from_rgb(178, 255, 178);
    let waypoint_color = Color32::from_rgb(255, 204, 51);

    // 1. Degree ticks and direction labels
    for bearing in (0..360).step_by(15) {
        let rel = bearing_to_view_deg(bearing as f32, yaw_rad);
        if rel.abs() > HALF_VIEW_DEG {
            continue;
        }
        let x = strip_rect.min.x + marker_x(rel, strip_w);
        let major = bearing % 45 == 0;
        let tick_len = if major { 12.0 * scale } else { 6.0 * scale };
        painter.line_segment(
            [
                pos2(x, strip_rect.max.y),
                pos2(x, strip_rect.max.y - tick_len),
            ],
            Stroke::new(1.0, text_color),
        );
        if major {
            let label = direction_name(bearing);
            let color = if bearing == 0 { north_color } else { text_color };
            draw_outline_text(
                painter,
                pos2(x, strip_rect.min.y + 14.0 * scale),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(15.0 * scale),
                color,
            );
        }
    }

    // 2. Camp indicator / Safe zone
    if in_safe_zone {
        draw_outline_text(
            painter,
            pos2(strip_rect.center().x, strip_rect.max.y + 24.0 * scale),
            Align2::CENTER_CENTER,
            "IN CAMP",
            FontId::proportional(16.0 * scale),
            camp_color,
        );
    } else if let Some(c) = camp {
        let mx = strip_rect.min.x + marker_x(c.angle_deg, strip_w);
        let tri = vec![
            pos2(mx - 6.0 * scale, strip_rect.max.y + 1.0 * scale),
            pos2(mx + 6.0 * scale, strip_rect.max.y + 1.0 * scale),
            pos2(mx, strip_rect.max.y + 8.0 * scale),
        ];
        painter.add(egui::Shape::convex_polygon(tri, camp_color, Stroke::NONE));

        let name = if shows_behind(true, c.angle_deg, c.distance_metres) {
            "CAMP (behind)"
        } else {
            "CAMP"
        };
        draw_outline_text(
            painter,
            pos2(mx, strip_rect.max.y + 24.0 * scale),
            Align2::CENTER_CENTER,
            name,
            FontId::proportional(16.0 * scale),
            camp_color,
        );

        let dist_text = format!("{} m", c.distance_metres.round() as i32);
        draw_outline_text(
            painter,
            pos2(mx, strip_rect.max.y + 44.0 * scale),
            Align2::CENTER_CENTER,
            &dist_text,
            FontId::proportional(16.0 * scale),
            text_color,
        );
    }

    // 3. Waypoint diamond indicator
    if let Some(wp) = waypoint {
        let wx = strip_rect.min.x + marker_x(wp.angle_deg, strip_w);
        let wy = strip_rect.center().y;
        let diamond = vec![
            pos2(wx, wy - 7.0 * scale),
            pos2(wx + 6.0 * scale, wy),
            pos2(wx, wy + 7.0 * scale),
            pos2(wx - 6.0 * scale, wy),
        ];
        painter.add(egui::Shape::convex_polygon(diamond, waypoint_color, Stroke::NONE));

        let wp_text = format!("WAYPOINT {} m", wp.distance_metres.round() as i32);
        draw_outline_text(
            painter,
            pos2(wx, strip_rect.max.y + 64.0 * scale),
            Align2::CENTER_CENTER,
            &wp_text,
            FontId::proportional(16.0 * scale),
            waypoint_color,
        );
    }
}
