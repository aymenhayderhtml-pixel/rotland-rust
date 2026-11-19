//! Small egui drawing helpers for the HUD mini-map.

use engine::egui::{self, vec2, Align2, Color32, FontId, Pos2, Stroke};

fn color(c: [u8; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
}

pub(super) fn draw_marker(p: &egui::Painter, at: Pos2, marker: &game::MinimapMarker, scale: f32) {
    let outline = color(game::OUTLINE_COLOR);
    match marker.kind {
        game::MapMarkerKind::Camp | game::MapMarkerKind::Waypoint if marker.pinned => {
            let d = vec2(marker.dir.x, marker.dir.y);
            let side = vec2(-d.y, d.x);
            let col = if marker.kind == game::MapMarkerKind::Camp {
                color(game::CAMP_COLOR)
            } else {
                color(game::WAYPOINT_COLOR)
            };
            p.add(egui::Shape::convex_polygon(
                vec![
                    at + d * 5.5 * scale,
                    at - d * 3.5 * scale + side * 4.0 * scale,
                    at - d * 3.5 * scale - side * 4.0 * scale,
                ],
                col,
                Stroke::new(scale, outline),
            ));
        }
        game::MapMarkerKind::Camp => {
            let c = color(game::CAMP_COLOR);
            let roof = vec![
                at + vec2(-5.5, -1.0) * scale,
                at + vec2(5.5, -1.0) * scale,
                at + vec2(0.0, -6.5) * scale,
            ];
            p.add(egui::Shape::convex_polygon(roof, c, Stroke::NONE));
            p.rect_filled(
                egui::Rect::from_min_size(at + vec2(-3.8, -1.0) * scale, vec2(7.6, 5.5) * scale),
                0.0,
                c,
            );
        }
        game::MapMarkerKind::Waypoint => {
            let c = color(game::WAYPOINT_COLOR);
            let pts = vec![
                at + vec2(0.0, -6.0) * scale,
                at + vec2(5.0, 0.0) * scale,
                at + vec2(0.0, 6.0) * scale,
                at + vec2(-5.0, 0.0) * scale,
            ];
            p.add(egui::Shape::convex_polygon(
                pts,
                c,
                Stroke::new(1.0 * scale, outline),
            ));
        }
        game::MapMarkerKind::Building => {
            let fill = if marker.dim {
                color(game::DIM_COLOR)
            } else {
                color(game::BUILDING_COLOR)
            };
            p.circle(at, 6.2 * scale, fill, Stroke::new(1.2 * scale, outline));
            p.text(
                at,
                Align2::CENTER_CENTER,
                game::letter_for(&marker.building_type),
                FontId::monospace(9.5 * scale),
                outline,
            );
        }
        game::MapMarkerKind::Bounty => {
            let c = color(game::BOUNTY_COLOR);
            p.circle_stroke(at, 7.5 * scale, Stroke::new(2.0 * scale, c));
            p.circle_filled(at, 1.8 * scale, c);
        }
        game::MapMarkerKind::Death => {
            p.circle(
                at,
                4.8 * scale,
                color(game::SKULL_COLOR),
                Stroke::new(1.2 * scale, outline),
            );
        }
    }
}

pub(super) fn draw_player(p: &egui::Painter, at: Pos2, yaw: f32, scale: f32) {
    let (f, r) = game::minimap_player_axes(yaw);
    let f = vec2(f.x, f.y);
    let r = vec2(r.x, r.y);
    let points = vec![
        at + f * 8.5 * scale,
        at - f * 4.5 * scale + r * 5.0 * scale,
        at - f * 1.8 * scale,
        at - f * 4.5 * scale - r * 5.0 * scale,
    ];
    p.add(egui::Shape::convex_polygon(
        points,
        color(game::PLAYER_COLOR),
        Stroke::new(1.3 * scale, color(game::OUTLINE_COLOR)),
    ));
}
