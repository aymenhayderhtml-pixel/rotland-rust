//! Small egui drawing helpers for the HUD mini-map.

use engine::egui::{self, Align2, Color32, FontId, Pos2, Stroke, vec2};

fn color(c: [u8; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
}

pub(super) fn draw_camp_outlines(
    painter: &egui::Painter,
    centre: Pos2,
    outlines: &[game::MinimapCampOutline],
    scale: f32,
) {
    let camp = color(game::CAMP_COLOR);
    let fill = Color32::from_rgba_unmultiplied(135, 230, 140, 48);
    let shadow = Color32::from_rgba_unmultiplied(0, 0, 0, 190);
    for outline in outlines {
        if let Some(quad) = outline.interior_quad {
            let points = quad
                .iter()
                .map(|p| Pos2::new(centre.x + p.x, centre.y + p.y))
                .collect();
            painter.add(egui::Shape::convex_polygon(points, fill, Stroke::NONE));
        }
        for &(a, b) in &outline.segments {
            let a = Pos2::new(centre.x + a.x, centre.y + a.y);
            let b = Pos2::new(centre.x + b.x, centre.y + b.y);
            painter.line_segment([a, b], Stroke::new(3.2 * scale, shadow));
            painter.line_segment([a, b], Stroke::new(1.7 * scale, camp));
        }
        for &gatepost in &outline.gateposts {
            painter.circle(
                Pos2::new(centre.x + gatepost.x, centre.y + gatepost.y),
                2.3 * scale,
                camp,
                Stroke::new(0.9 * scale, shadow),
            );
        }
    }
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
                at + vec2(-6.5, -1.0) * scale,
                at + vec2(6.5, -1.0) * scale,
                at + vec2(0.0, -7.5) * scale,
            ];
            p.add(egui::Shape::convex_polygon(
                roof,
                c,
                Stroke::new(1.2 * scale, outline),
            ));
            p.rect(
                egui::Rect::from_min_size(at + vec2(-4.5, -1.0) * scale, vec2(9.0, 6.2) * scale),
                0.0,
                c,
                Stroke::new(1.0 * scale, outline),
                egui::StrokeKind::Middle,
            );
        }
        game::MapMarkerKind::Waypoint => {
            let c = color(game::WAYPOINT_COLOR);
            let pts = vec![
                at + vec2(0.0, -6.5) * scale,
                at + vec2(5.5, 0.0) * scale,
                at + vec2(0.0, 6.5) * scale,
                at + vec2(-5.5, 0.0) * scale,
            ];
            p.add(egui::Shape::convex_polygon(
                pts,
                c,
                Stroke::new(1.3 * scale, outline),
            ));
        }
        game::MapMarkerKind::Building => {
            let fill = if marker.dim {
                color(game::DIM_COLOR)
            } else {
                color(game::BUILDING_COLOR)
            };
            p.circle(at, 7.5 * scale, fill, Stroke::new(1.4 * scale, outline));
            p.text(
                at,
                Align2::CENTER_CENTER,
                game::letter_for(&marker.building_type),
                FontId::monospace(10.5 * scale),
                outline,
            );
        }
        game::MapMarkerKind::Bounty => {
            let c = color(game::BOUNTY_COLOR);
            p.circle_stroke(at, 8.0 * scale, Stroke::new(2.2 * scale, c));
            p.circle_filled(at, 2.0 * scale, c);
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

pub(super) fn draw_player(p: &egui::Painter, at: Pos2, yaw: f32, size: f32, scale: f32) {
    let (f, r) = game::minimap_player_axes(yaw);
    let f = vec2(f.x, f.y);
    let r = vec2(r.x, r.y);
    let points = vec![
        at + f * size,
        at - f * (size * 0.55) + r * (size * 0.62),
        at - f * (size * 0.22),
        at - f * (size * 0.55) - r * (size * 0.62),
    ];
    let fill = Color32::from_rgba_unmultiplied(
        game::PLAYER_COLOR[0],
        game::PLAYER_COLOR[1],
        game::PLAYER_COLOR[2],
        225,
    );
    p.add(egui::Shape::convex_polygon(
        points,
        fill,
        Stroke::new(2.2 * scale, Color32::from_rgba_unmultiplied(0, 0, 0, 245)),
    ));
}
