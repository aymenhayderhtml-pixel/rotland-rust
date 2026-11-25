//! City street, camp road, and label rendering for the big map view.

use engine::egui::{self, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Stroke};
use glam::Vec2;

use super::state::MapUiState;
use super::view::MapView;

/// Visual styling configuration for city streets and camp road on the big map.
#[derive(Debug, Clone, PartialEq)]
pub struct MapStreetsConfig {
    pub street_color: Color32,
    pub road_color: Color32,
    pub blocked_color: Color32,
    pub street_min_px: f32,
}

impl Default for MapStreetsConfig {
    fn default() -> Self {
        Self {
            street_color: Color32::from_rgba_unmultiplied(
                game::CITY_STREET_COLOR[0],
                game::CITY_STREET_COLOR[1],
                game::CITY_STREET_COLOR[2],
                game::CITY_STREET_COLOR[3],
            ),
            road_color: Color32::from_rgba_unmultiplied(
                game::CITY_ROAD_COLOR[0],
                game::CITY_ROAD_COLOR[1],
                game::CITY_ROAD_COLOR[2],
                game::CITY_ROAD_COLOR[3],
            ),
            blocked_color: Color32::from_rgba_unmultiplied(
                game::CITY_BLOCKED_COLOR[0],
                game::CITY_BLOCKED_COLOR[1],
                game::CITY_BLOCKED_COLOR[2],
                game::CITY_BLOCKED_COLOR[3],
            ),
            street_min_px: game::CITY_STREET_MIN_PX,
        }
    }
}

impl MapStreetsConfig {
    pub fn load_default() -> Self {
        Self::load(&paths::repo_path("data/minimap.json"))
    }

    pub fn load(path: &std::path::Path) -> Self {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) else {
            return Self::default();
        };
        let mut cfg = Self::default();
        if let Some(c) = val.get("street_color").and_then(parse_color) {
            cfg.street_color = c;
        }
        if let Some(c) = val.get("road_color").and_then(parse_color) {
            cfg.road_color = c;
        }
        if let Some(c) = val.get("blocked_color").and_then(parse_color) {
            cfg.blocked_color = c;
        }
        if let Some(w) = val.get("street_min_px").and_then(|v| v.as_f64()) {
            cfg.street_min_px = (w as f32).max(0.5);
        }
        cfg
    }
}

fn parse_color(v: &serde_json::Value) -> Option<Color32> {
    match v {
        serde_json::Value::String(s) => {
            let hex = s.trim_start_matches('#');
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color32::from_rgb(r, g, b))
            } else if hex.len() == 8 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Color32::from_rgba_unmultiplied(r, g, b, a))
            } else {
                None
            }
        }
        serde_json::Value::Array(arr) => {
            if arr.len() == 3 {
                let r = arr[0].as_u64()? as u8;
                let g = arr[1].as_u64()? as u8;
                let b = arr[2].as_u64()? as u8;
                Some(Color32::from_rgb(r, g, b))
            } else if arr.len() == 4 {
                let r = arr[0].as_u64()? as u8;
                let g = arr[1].as_u64()? as u8;
                let b = arr[2].as_u64()? as u8;
                let a = arr[3].as_u64()? as u8;
                Some(Color32::from_rgba_unmultiplied(r, g, b, a))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Draws city streets, camp road, blocked street crosses, city/district labels, outer towns, and trailheads.
pub fn draw_city_and_road(
    painter: &egui::Painter,
    state: &MapUiState,
    view: &MapView<'_>,
    rect: Rect,
    view_size: Vec2,
) {
    let Some(world_gen) = view.generator else {
        return;
    };

    let scale = state.scale();
    let cfg = &state.streets_cfg;
    let min_w = cfg.street_min_px;

    let to_screen_pos = |world: Vec2| -> Pos2 {
        let s = game::world_to_screen(world, state.centre, scale, view_size);
        pos2(rect.min.x + s.x, rect.min.y + s.y)
    };

    // Outer towns and trailheads (outside core)
    for town in world_gen.towns() {
        let p = to_screen_pos(town.center);
        if rect.expand(48.0).contains(p) {
            super::draw_icons::draw_town_icon(painter, p, &town.name);
        }
    }
    for &trailhead in world_gen.roads().trailheads() {
        let p = to_screen_pos(trailhead);
        if rect.expand(16.0).contains(p) {
            super::draw_icons::draw_signpost_icon(painter, p);
        }
    }

    let Some(city) = world_gen.city() else {
        return;
    };

    let (vis_min, vis_max) =
        game::map_visible_world_bounds(state.centre, scale, view_size, 32.0 / scale);

    // 1. Camp road: shows fully once discovered
    if game::is_road_discovered(&city.road, &view.map_state.explore) {
        let road_w = (min_w * scale * 1.5).max(min_w);
        for window in city.road.windows(2) {
            let (a, b) = (window[0], window[1]);
            if !game::segment_in_bounds(a, b, vis_min, vis_max, 16.0) {
                continue;
            }
            let pa = to_screen_pos(a);
            let pb = to_screen_pos(b);
            painter.line_segment([pa, pb], Stroke::new(road_w, cfg.road_color));
        }
    }

    // 2. City streets: early out if city bounding box is not in view
    let city_min = city.origin;
    let city_max = city.origin + Vec2::splat(city.size);
    if game::segment_in_bounds(city_min, city_max, vis_min, vis_max, 32.0) {
        let mut blocked_crosses: Vec<Pos2> = Vec::new();

        for street in &city.streets {
            if !game::segment_in_bounds(street.a, street.b, vis_min, vis_max, street.width) {
                continue;
            }

            let spans = game::explored_segment_spans(street.a, street.b, &view.map_state.explore);
            if spans.is_empty() {
                continue;
            }

            let width_ratio = if street.avenue { 1.5 } else { 1.0 };
            let stroke_w = (min_w * width_ratio * scale * 1.5).max(min_w * width_ratio);

            for (sa, sb) in spans {
                let pa = to_screen_pos(sa);
                let pb = to_screen_pos(sb);
                painter.line_segment([pa, pb], Stroke::new(stroke_w, cfg.street_color));
            }

            if street.blocked {
                let mid = (street.a + street.b) * 0.5;
                if view.map_state.explore.is_revealed(game::MapExplore::cell_of(mid)) {
                    blocked_crosses.push(to_screen_pos(mid));
                }
            }
        }

        // 3. Blocked street crosses
        let cross_sz = (3.5 * scale).clamp(2.5, 6.0);
        let cross_stroke = Stroke::new(1.8 * scale.max(0.6), cfg.blocked_color);
        for p in blocked_crosses {
            painter.line_segment(
                [
                    pos2(p.x - cross_sz, p.y - cross_sz),
                    pos2(p.x + cross_sz, p.y + cross_sz),
                ],
                cross_stroke,
            );
            painter.line_segment(
                [
                    pos2(p.x + cross_sz, p.y - cross_sz),
                    pos2(p.x - cross_sz, p.y + cross_sz),
                ],
                cross_stroke,
            );
        }

        // 4. City label "Ashford" at city center
        if game::is_city_center_revealed(city, &view.map_state.explore) {
            let center_pos = to_screen_pos(game::city_center(city));
            if rect.contains(center_pos) {
                painter.text(
                    center_pos + vec2(1.0, 1.0),
                    Align2::CENTER_CENTER,
                    "Ashford",
                    FontId::proportional(16.0),
                    Color32::from_black_alpha(220),
                );
                painter.text(
                    center_pos,
                    Align2::CENTER_CENTER,
                    "Ashford",
                    FontId::proportional(16.0),
                    Color32::from_rgb(245, 240, 225),
                );
            }
        }

        // 5. Small district labels ("Core", "Industrial") when zoomed in
        if state.zoom_index >= 2 {
            let shadow_col = Color32::from_black_alpha(200);
            let text_col = Color32::from_rgb(215, 210, 195);

            if game::is_district_revealed(city, world::District::Core, &view.map_state.explore)
                && let Some(core_center) = game::district_center(city, world::District::Core)
            {
                let p = to_screen_pos(core_center) + vec2(0.0, 14.0);
                if rect.contains(p) {
                    painter.text(
                        p + vec2(1.0, 1.0),
                        Align2::CENTER_CENTER,
                        "Core",
                        FontId::proportional(11.0),
                        shadow_col,
                    );
                    painter.text(
                        p,
                        Align2::CENTER_CENTER,
                        "Core",
                        FontId::proportional(11.0),
                        text_col,
                    );
                }
            }

            if game::is_district_revealed(city, world::District::Industrial, &view.map_state.explore)
                && let Some(ind_center) = game::district_center(city, world::District::Industrial)
            {
                let p = to_screen_pos(ind_center);
                if rect.contains(p) {
                    painter.text(
                        p + vec2(1.0, 1.0),
                        Align2::CENTER_CENTER,
                        "Industrial",
                        FontId::proportional(11.0),
                        shadow_col,
                    );
                    painter.text(
                        p,
                        Align2::CENTER_CENTER,
                        "Industrial",
                        FontId::proportional(11.0),
                        text_col,
                    );
                }
            }
        }
    }
}
