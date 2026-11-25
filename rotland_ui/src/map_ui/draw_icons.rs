//! Icon rendering primitives for the map screen: camp, building, bounty, death, waypoint, and player arrow.
//!
//! Port of Godot C# scripts/UI/MapIcons.cs.

use engine::egui::{self, vec2, Align2, Color32, FontId, Pos2, Rect, Stroke, StrokeKind};

// Godot: MapIcons.cs Camp
pub fn draw_camp_icon(painter: &egui::Painter, at: Pos2) {
    let camp_col = Color32::from_rgba_premultiplied(
        game::CAMP_COLOR[0],
        game::CAMP_COLOR[1],
        game::CAMP_COLOR[2],
        game::CAMP_COLOR[3],
    );
    let outline_col = Color32::from_rgba_premultiplied(
        game::OUTLINE_COLOR[0],
        game::OUTLINE_COLOR[1],
        game::OUTLINE_COLOR[2],
        game::OUTLINE_COLOR[3],
    );

    let roof = vec![at + vec2(-9.0, -2.0), at + vec2(9.0, -2.0), at + vec2(0.0, -11.0)];
    painter.add(egui::Shape::convex_polygon(roof, camp_col, Stroke::NONE));
    painter.rect_filled(Rect::from_min_size(at + vec2(-6.0, -2.0), vec2(12.0, 9.0)), 0.0, camp_col);
    painter.rect_filled(Rect::from_min_size(at + vec2(-1.5, 3.0), vec2(3.0, 4.0)), 0.0, outline_col);

    painter.text(
        at + vec2(12.0, -6.0),
        Align2::LEFT_CENTER,
        "Camp",
        FontId::proportional(14.0),
        camp_col,
    );
}

// Godot: MapIcons.cs Building
pub fn draw_building_icon(painter: &egui::Painter, at: Pos2, letter: &str, dim: bool) {
    let fill = if dim {
        Color32::from_rgba_premultiplied(
            game::DIM_COLOR[0],
            game::DIM_COLOR[1],
            game::DIM_COLOR[2],
            game::DIM_COLOR[3],
        )
    } else {
        Color32::from_rgba_premultiplied(
            game::BUILDING_COLOR[0],
            game::BUILDING_COLOR[1],
            game::BUILDING_COLOR[2],
            game::BUILDING_COLOR[3],
        )
    };
    let outline_col = Color32::from_rgba_premultiplied(
        game::OUTLINE_COLOR[0],
        game::OUTLINE_COLOR[1],
        game::OUTLINE_COLOR[2],
        game::OUTLINE_COLOR[3],
    );

    painter.circle(at, 9.5, fill, Stroke::new(1.5, outline_col));
    painter.text(
        at,
        Align2::CENTER_CENTER,
        letter,
        FontId::monospace(13.0),
        outline_col,
    );
}

// Godot: MapIcons.cs Bounty
pub fn draw_bounty_icon(painter: &egui::Painter, at: Pos2) {
    let bounty_col = Color32::from_rgba_premultiplied(
        game::BOUNTY_COLOR[0],
        game::BOUNTY_COLOR[1],
        game::BOUNTY_COLOR[2],
        game::BOUNTY_COLOR[3],
    );
    painter.circle_stroke(at, 14.0, Stroke::new(3.0, bounty_col));
    painter.circle_filled(at, 2.5, bounty_col);
}

// Godot: MapIcons.cs Death
pub fn draw_death_icon(painter: &egui::Painter, at: Pos2) {
    let skull_col = Color32::from_rgba_premultiplied(
        game::SKULL_COLOR[0],
        game::SKULL_COLOR[1],
        game::SKULL_COLOR[2],
        game::SKULL_COLOR[3],
    );
    let outline_col = Color32::from_rgba_premultiplied(
        game::OUTLINE_COLOR[0],
        game::OUTLINE_COLOR[1],
        game::OUTLINE_COLOR[2],
        game::OUTLINE_COLOR[3],
    );

    painter.circle(at, 8.0, skull_col, Stroke::new(1.5, outline_col));
    painter.circle_filled(at + vec2(-2.5, -1.0), 1.8, outline_col);
    painter.circle_filled(at + vec2(2.5, -1.0), 1.8, outline_col);
    painter.line_segment([at + vec2(-2.5, 3.5), at + vec2(2.5, 3.5)], Stroke::new(1.2, outline_col));

    painter.text(
        at + vec2(10.0, -6.0),
        Align2::LEFT_CENTER,
        "Death",
        FontId::proportional(13.0),
        skull_col,
    );
}

// Godot: MapIcons.cs Waypoint
pub fn draw_waypoint_icon(painter: &egui::Painter, at: Pos2) {
    let wp_col = Color32::from_rgba_premultiplied(
        game::WAYPOINT_COLOR[0],
        game::WAYPOINT_COLOR[1],
        game::WAYPOINT_COLOR[2],
        game::WAYPOINT_COLOR[3],
    );
    let outline_col = Color32::from_rgba_premultiplied(
        game::OUTLINE_COLOR[0],
        game::OUTLINE_COLOR[1],
        game::OUTLINE_COLOR[2],
        game::OUTLINE_COLOR[3],
    );

    let diamond = vec![
        at + vec2(0.0, -10.0),
        at + vec2(8.0, 0.0),
        at + vec2(0.0, 10.0),
        at + vec2(-8.0, 0.0),
    ];
    painter.add(egui::Shape::convex_polygon(diamond, wp_col, Stroke::new(1.5, outline_col)));

    painter.text(
        at + vec2(11.0, -6.0),
        Align2::LEFT_CENTER,
        "WP",
        FontId::proportional(13.0),
        wp_col,
    );
}

// Godot: MapIcons.cs Player
pub fn draw_player_icon(painter: &egui::Painter, at: Pos2, yaw_rad: f32) {
    let forward = vec2(-yaw_rad.sin(), -yaw_rad.cos());
    let right = vec2(yaw_rad.cos(), -yaw_rad.sin());

    let points = vec![
        at + forward * 14.0,
        at - forward * 6.0 + right * 7.0,
        at - forward * 2.0,
        at - forward * 6.0 - right * 7.0,
    ];

    let player_col = Color32::from_rgba_premultiplied(
        game::PLAYER_COLOR[0],
        game::PLAYER_COLOR[1],
        game::PLAYER_COLOR[2],
        game::PLAYER_COLOR[3],
    );
    let outline_col = Color32::from_rgba_premultiplied(
        game::OUTLINE_COLOR[0],
        game::OUTLINE_COLOR[1],
        game::OUTLINE_COLOR[2],
        game::OUTLINE_COLOR[3],
    );

    painter.add(egui::Shape::convex_polygon(points, player_col, Stroke::new(1.5, outline_col)));
}

// new: DECISIONS 2026-11-25
pub fn draw_town_icon(painter: &egui::Painter, at: Pos2, name: &str) {
    let town_col = Color32::from_rgb(245, 240, 225);
    let outline_col = Color32::from_rgba_premultiplied(
        game::OUTLINE_COLOR[0],
        game::OUTLINE_COLOR[1],
        game::OUTLINE_COLOR[2],
        game::OUTLINE_COLOR[3],
    );
    let marker_rect = Rect::from_center_size(at, vec2(8.0, 8.0));
    painter.rect(
        marker_rect,
        0.0,
        town_col,
        Stroke::new(1.5, outline_col),
        StrokeKind::Outside,
    );
    let label_pos = at + vec2(0.0, -14.0);
    painter.text(
        label_pos + vec2(1.0, 1.0),
        Align2::CENTER_CENTER,
        name,
        FontId::proportional(16.0),
        Color32::from_black_alpha(220),
    );
    painter.text(
        label_pos,
        Align2::CENTER_CENTER,
        name,
        FontId::proportional(16.0),
        town_col,
    );
}

// new: DECISIONS 2026-11-25
pub fn draw_signpost_icon(painter: &egui::Painter, at: Pos2) {
    let sign_col = Color32::from_rgb(205, 165, 105);
    let outline_col = Color32::from_rgba_premultiplied(
        game::OUTLINE_COLOR[0],
        game::OUTLINE_COLOR[1],
        game::OUTLINE_COLOR[2],
        game::OUTLINE_COLOR[3],
    );
    painter.line_segment(
        [at + vec2(0.0, -7.0), at + vec2(0.0, 7.0)],
        Stroke::new(2.0, outline_col),
    );
    let board = Rect::from_center_size(at + vec2(0.0, -3.5), vec2(10.0, 5.0));
    painter.rect(
        board,
        0.0,
        sign_col,
        Stroke::new(1.0, outline_col),
        StrokeKind::Outside,
    );
}
