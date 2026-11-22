//! Lake, docks, village lots, and fishing spots rendering for the big map view.

use std::path::Path;

use engine::egui::{self, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Stroke};
use glam::Vec2;

use super::state::MapUiState;
use super::view::MapView;

/// Visual styling configuration for the lake on the big map.
#[derive(Debug, Clone, PartialEq)]
pub struct MapLakeConfig {
    pub lake_name: String,
    pub water_color: Color32,
    pub shore_color: Color32,
    pub shore_width: f32,
    pub dock_color: Color32,
    pub dock_width: f32,
    pub village_lot_color: Color32,
    pub village_lot_outline: Color32,
    pub fishing_spot_color: Color32,
    pub fishing_spot_radius: f32,
    pub label_color: Color32,
    pub label_size: f32,
}

impl Default for MapLakeConfig {
    fn default() -> Self {
        Self {
            lake_name: "Lake".to_string(),
            water_color: Color32::from_rgba_unmultiplied(
                game::LAKE_WATER_COLOR[0],
                game::LAKE_WATER_COLOR[1],
                game::LAKE_WATER_COLOR[2],
                game::LAKE_WATER_COLOR[3],
            ),
            shore_color: Color32::from_rgba_unmultiplied(
                game::LAKE_SHORE_COLOR[0],
                game::LAKE_SHORE_COLOR[1],
                game::LAKE_SHORE_COLOR[2],
                game::LAKE_SHORE_COLOR[3],
            ),
            shore_width: game::LAKE_SHORE_WIDTH,
            dock_color: Color32::from_rgba_unmultiplied(
                game::LAKE_DOCK_COLOR[0],
                game::LAKE_DOCK_COLOR[1],
                game::LAKE_DOCK_COLOR[2],
                game::LAKE_DOCK_COLOR[3],
            ),
            dock_width: game::LAKE_DOCK_WIDTH,
            village_lot_color: Color32::from_rgba_unmultiplied(
                game::LAKE_VILLAGE_LOT_COLOR[0],
                game::LAKE_VILLAGE_LOT_COLOR[1],
                game::LAKE_VILLAGE_LOT_COLOR[2],
                game::LAKE_VILLAGE_LOT_COLOR[3],
            ),
            village_lot_outline: Color32::from_rgba_unmultiplied(
                game::LAKE_VILLAGE_LOT_OUTLINE[0],
                game::LAKE_VILLAGE_LOT_OUTLINE[1],
                game::LAKE_VILLAGE_LOT_OUTLINE[2],
                game::LAKE_VILLAGE_LOT_OUTLINE[3],
            ),
            fishing_spot_color: Color32::from_rgba_unmultiplied(
                game::LAKE_FISHING_SPOT_COLOR[0],
                game::LAKE_FISHING_SPOT_COLOR[1],
                game::LAKE_FISHING_SPOT_COLOR[2],
                game::LAKE_FISHING_SPOT_COLOR[3],
            ),
            fishing_spot_radius: game::LAKE_FISHING_SPOT_RADIUS,
            label_color: Color32::from_rgba_unmultiplied(
                game::LAKE_LABEL_COLOR[0],
                game::LAKE_LABEL_COLOR[1],
                game::LAKE_LABEL_COLOR[2],
                game::LAKE_LABEL_COLOR[3],
            ),
            label_size: game::LAKE_LABEL_SIZE,
        }
    }
}

impl MapLakeConfig {
    pub fn load_default() -> Self {
        Self::load(&paths::repo_path("data/map.json"))
    }

    pub fn load(path: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) else {
            return Self::default();
        };
        let mut cfg = Self::default();
        if let Some(name) = val.get("lake_name").and_then(|v| v.as_str()) {
            cfg.lake_name = name.to_string();
        }
        if let Some(c) = val.get("water_color").and_then(parse_color) {
            cfg.water_color = c;
        }
        if let Some(c) = val.get("shore_color").and_then(parse_color) {
            cfg.shore_color = c;
        }
        if let Some(w) = val.get("shore_width").and_then(|v| v.as_f64()) {
            cfg.shore_width = (w as f32).max(0.5);
        }
        if let Some(c) = val.get("dock_color").and_then(parse_color) {
            cfg.dock_color = c;
        }
        if let Some(w) = val.get("dock_width").and_then(|v| v.as_f64()) {
            cfg.dock_width = (w as f32).max(0.5);
        }
        if let Some(c) = val.get("village_lot_color").and_then(parse_color) {
            cfg.village_lot_color = c;
        }
        if let Some(c) = val.get("village_lot_outline").and_then(parse_color) {
            cfg.village_lot_outline = c;
        }
        if let Some(c) = val.get("fishing_spot_color").and_then(parse_color) {
            cfg.fishing_spot_color = c;
        }
        if let Some(r) = val.get("fishing_spot_radius").and_then(|v| v.as_f64()) {
            cfg.fishing_spot_radius = (r as f32).max(1.0);
        }
        if let Some(c) = val.get("label_color").and_then(parse_color) {
            cfg.label_color = c;
        }
        if let Some(sz) = val.get("label_size").and_then(|v| v.as_f64()) {
            cfg.label_size = (sz as f32).max(8.0);
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

/// Draws the lake polygon, shore outline, docks, village lots, fishing spots, and label on the big map.
pub fn draw_lake(
    painter: &egui::Painter,
    state: &MapUiState,
    view: &MapView<'_>,
    rect: Rect,
    view_size: Vec2,
) {
    let Some(world_gen) = view.generator else {
        return;
    };
    let Some(lake_layout) = world_gen.lake() else {
        return;
    };

    let lake_data = game::LakeMapData::from_layout(lake_layout);
    if !game::is_lake_discovered(&lake_data, &view.map_state.explore) {
        return;
    }

    let scale = state.scale();
    let cfg = &state.lake_cfg;

    let (vis_min, vis_max) =
        game::map_visible_world_bounds(state.centre, scale, view_size, 64.0 / scale);
    let (w_min, w_max) = lake_data.water_bounds;
    if w_max.x < vis_min.x || w_min.x > vis_max.x || w_max.y < vis_min.y || w_min.y > vis_max.y {
        return;
    }

    let to_screen_pos = |world: Vec2| -> Pos2 {
        let s = game::world_to_screen(world, state.centre, scale, view_size);
        pos2(rect.min.x + s.x, rect.min.y + s.y)
    };

    // 1. Water polygon (fan triangulated from center)
    let center_screen = to_screen_pos(lake_data.center);
    let shore_screen: Vec<Pos2> = lake_data
        .shore
        .iter()
        .map(|&p| to_screen_pos(p))
        .collect();
    let n = shore_screen.len();

    if n >= 3 {
        let mut mesh = egui::Mesh::default();
        mesh.vertices.push(egui::epaint::Vertex {
            pos: center_screen,
            uv: Pos2::ZERO,
            color: cfg.water_color,
        });
        for &p in &shore_screen {
            mesh.vertices.push(egui::epaint::Vertex {
                pos: p,
                uv: Pos2::ZERO,
                color: cfg.water_color,
            });
        }
        for i in 0..n {
            let next = (i + 1) % n;
            mesh.add_triangle(0, (i + 1) as u32, (next + 1) as u32);
        }
        painter.add(egui::Shape::mesh(mesh));

        // 2. Darker shore outline
        let shore_w = (cfg.shore_width * scale).max(1.5);
        let stroke = Stroke::new(shore_w, cfg.shore_color);
        for i in 0..n {
            let next = (i + 1) % n;
            painter.line_segment([shore_screen[i], shore_screen[next]], stroke);
        }
    }

    // 3. Docks as short brown lines
    let dock_w = (cfg.dock_width * scale).max(2.0);
    let dock_stroke = Stroke::new(dock_w, cfg.dock_color);
    for dock in &lake_data.docks {
        let pa = to_screen_pos(dock.start);
        let pb = to_screen_pos(dock.end);
        painter.line_segment([pa, pb], dock_stroke);
    }

    // 4. Village lots as small gray squares
    let lot_stroke = Stroke::new(1.0, cfg.village_lot_outline);
    for lot in &lake_data.village_lots {
        let screen_corners = lot.corners.map(to_screen_pos);
        painter.add(egui::Shape::convex_polygon(
            screen_corners.to_vec(),
            cfg.village_lot_color,
            lot_stroke,
        ));
    }

    // 5. Fishing spots as small circles
    let spot_r = (cfg.fishing_spot_radius * scale).clamp(2.5, 6.0);
    let spot_stroke = Stroke::new(1.0, Color32::from_black_alpha(200));
    for spot in &lake_data.fishing_spots {
        let p = to_screen_pos(spot.position);
        painter.circle(p, spot_r, cfg.fishing_spot_color, spot_stroke);
    }

    // 6. "Lake" label
    let label_pos = to_screen_pos(lake_data.label_pos);
    if rect.contains(label_pos) {
        let font_id = FontId::proportional(cfg.label_size);
        painter.text(
            label_pos + vec2(1.0, 1.0),
            Align2::CENTER_CENTER,
            &cfg.lake_name,
            font_id.clone(),
            Color32::from_black_alpha(220),
        );
        painter.text(
            label_pos,
            Align2::CENTER_CENTER,
            &cfg.lake_name,
            font_id,
            cfg.label_color,
        );
    }
}
