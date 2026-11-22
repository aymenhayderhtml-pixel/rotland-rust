//! Small egui drawing helpers for the HUD mini-map.

use engine::egui::{self, Align2, Color32, FontId, Pos2, Stroke, vec2};
use glam::Vec2;

use super::minimap_cache::MinimapStreetsConfig;

fn color(c: [u8; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
}

pub(super) fn draw_streets_and_road(
    painter: &egui::Painter,
    centre: Pos2,
    city: Option<&world::CityLayout>,
    player_xz: Vec2,
    radius_metres: f32,
    radius_px: f32,
    disk_r: f32,
    scale: f32,
    cfg: &MinimapStreetsConfig,
) {
    let Some(city) = city else {
        return;
    };

    // 1. Draw camp road segments
    let road_w = (cfg.street_min_px * scale).max(1.5 * scale);
    for window in city.road.windows(2) {
        let (a, b) = (window[0], window[1]);
        let ab = b - a;
        let len_sq = ab.length_squared();
        let t = if len_sq < 1e-6 {
            0.0
        } else {
            ((player_xz - a).dot(ab) / len_sq).clamp(0.0, 1.0)
        };
        let closest = a + ab * t;
        if closest.distance_squared(player_xz) > radius_metres * radius_metres {
            continue;
        }

        let off_a = game::world_to_minimap_offset(a, player_xz, radius_metres, radius_px);
        let off_b = game::world_to_minimap_offset(b, player_xz, radius_metres, radius_px);
        if let Some((ca, cb)) = game::clip_segment_to_circle(off_a, off_b, disk_r) {
            let pa = Pos2::new(centre.x + ca.x, centre.y + ca.y);
            let pb = Pos2::new(centre.x + cb.x, centre.y + cb.y);
            painter.line_segment([pa, pb], Stroke::new(road_w, cfg.road_color));
        }
    }

    // 2. City streets bounding check (cheap early out when far from city)
    let margin = radius_metres + 20.0;
    if player_xz.x < city.origin.x - margin
        || player_xz.x > city.origin.x + city.size + margin
        || player_xz.y < city.origin.y - margin
        || player_xz.y > city.origin.y + city.size + margin
    {
        return;
    }

    let min_w = cfg.street_min_px.max(1.0);
    let mut blocked_crosses: Vec<Pos2> = Vec::new();

    for street in &city.streets {
        let max_reach = radius_metres + street.width * 0.5;
        if street.closest_point(player_xz).distance_squared(player_xz) > max_reach * max_reach {
            continue;
        }

        let off_a = game::world_to_minimap_offset(street.a, player_xz, radius_metres, radius_px);
        let off_b = game::world_to_minimap_offset(street.b, player_xz, radius_metres, radius_px);

        if let Some((ca, cb)) = game::clip_segment_to_circle(off_a, off_b, disk_r) {
            let pa = Pos2::new(centre.x + ca.x, centre.y + ca.y);
            let pb = Pos2::new(centre.x + cb.x, centre.y + cb.y);

            let width_ratio = if street.avenue {
                (street.width / 10.0).max(1.35)
            } else {
                (street.width / 10.0).max(1.0)
            };
            let stroke_w = (min_w * width_ratio * scale).max(min_w * scale);
            painter.line_segment([pa, pb], Stroke::new(stroke_w, cfg.street_color));

            if street.blocked {
                let mid = (street.a + street.b) * 0.5;
                let mid_off =
                    game::world_to_minimap_offset(mid, player_xz, radius_metres, radius_px);
                if mid_off.length_squared() <= disk_r * disk_r {
                    blocked_crosses.push(Pos2::new(centre.x + mid_off.x, centre.y + mid_off.y));
                }
            }
        }
    }

    let cross_sz = 3.2 * scale;
    let cross_stroke = Stroke::new(1.8 * scale, cfg.blocked_color);
    for p in blocked_crosses {
        painter.line_segment(
            [
                Pos2::new(p.x - cross_sz, p.y - cross_sz),
                Pos2::new(p.x + cross_sz, p.y + cross_sz),
            ],
            cross_stroke,
        );
        painter.line_segment(
            [
                Pos2::new(p.x + cross_sz, p.y - cross_sz),
                Pos2::new(p.x - cross_sz, p.y + cross_sz),
            ],
            cross_stroke,
        );
    }
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

pub(super) fn draw_lake_features(
    painter: &egui::Painter,
    centre: Pos2,
    lake: Option<&world::LakeLayout>,
    player_xz: Vec2,
    radius_metres: f32,
    radius_px: f32,
    disk_r: f32,
    scale: f32,
    cfg: &MinimapStreetsConfig,
) {
    let Some(lake) = lake else {
        return;
    };
    let (lake_min, lake_max) = lake.water_bounds();
    let margin = radius_metres + 32.0;
    if player_xz.x < lake_min.x - margin
        || player_xz.x > lake_max.x + margin
        || player_xz.y < lake_min.y - margin
        || player_xz.y > lake_max.y + margin
    {
        return;
    }

    // 1. Docks as short brown lines
    let dock_w = (cfg.dock_width * scale).max(1.8 * scale);
    for dock in &lake.docks {
        let off_a = game::world_to_minimap_offset(dock.start, player_xz, radius_metres, radius_px);
        let off_b = game::world_to_minimap_offset(dock.end, player_xz, radius_metres, radius_px);
        if let Some((ca, cb)) = game::clip_segment_to_circle(off_a, off_b, disk_r) {
            let pa = Pos2::new(centre.x + ca.x, centre.y + ca.y);
            let pb = Pos2::new(centre.x + cb.x, centre.y + cb.y);
            painter.line_segment([pa, pb], Stroke::new(dock_w, cfg.dock_color));
        }
    }

    // 2. Village lots as small gray squares
    for lot in &lake.village_lots {
        let lot_off = game::world_to_minimap_offset(lot.center, player_xz, radius_metres, radius_px);
        if lot_off.length_squared() <= (disk_r + 16.0 * scale) * (disk_r + 16.0 * scale) {
            let corners = lot.corners();
            let pts: Vec<Pos2> = corners
                .iter()
                .map(|&c| {
                    let off = game::world_to_minimap_offset(c, player_xz, radius_metres, radius_px);
                    Pos2::new(centre.x + off.x, centre.y + off.y)
                })
                .collect();
            painter.add(egui::Shape::convex_polygon(
                pts,
                cfg.village_lot_color,
                Stroke::new(scale, cfg.village_lot_outline),
            ));
        }
    }

    // 3. Fishing spots as small circles
    for spot in &lake.fishing_spots {
        let off = game::world_to_minimap_offset(spot.position, player_xz, radius_metres, radius_px);
        if off.length_squared() <= disk_r * disk_r {
            painter.circle_filled(
                Pos2::new(centre.x + off.x, centre.y + off.y),
                2.2 * scale,
                cfg.fishing_spot_color,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimap_lake_dock_endpoint_world_to_minimap_transform() {
        let player = Vec2::new(100.0, 200.0);
        let radius_metres = 60.0;
        let radius_px = 86.0;

        let dock_start = Vec2::new(130.0, 200.0);
        let off = game::world_to_minimap_offset(dock_start, player, radius_metres, radius_px);
        assert!((off.x - 43.0).abs() < 1e-4);
        assert!(off.y.abs() < 1e-4);
    }

    #[test]
    fn minimap_street_endpoint_world_to_minimap_transform() {
        let player = Vec2::new(100.0, 200.0);
        let radius_metres = 60.0;
        let radius_px = 86.0;

        // Street end point 30m East of player (+X)
        let street_end_east = Vec2::new(130.0, 200.0);
        let off_east =
            game::world_to_minimap_offset(street_end_east, player, radius_metres, radius_px);
        assert!((off_east.x - 43.0).abs() < 1e-4);
        assert!(off_east.y.abs() < 1e-4);

        // Street end point 40m North of player (-Y/-Z)
        let street_end_north = Vec2::new(100.0, 160.0);
        let off_north =
            game::world_to_minimap_offset(street_end_north, player, radius_metres, radius_px);
        assert!(off_north.x.abs() < 1e-4);
        assert!((off_north.y - (-40.0 / 60.0 * 86.0)).abs() < 1e-4);

        // Clipping: segment crossing the minimap boundary
        let seg_start = off_east;
        let seg_end = Vec2::new(200.0, 0.0);
        let clipped = game::clip_segment_to_circle(seg_start, seg_end, radius_px);
        assert!(clipped.is_some());
        let (ca, cb) = clipped.unwrap();
        assert_eq!(ca, seg_start);
        assert!((cb.length() - radius_px).abs() < 1e-4);
    }
}
