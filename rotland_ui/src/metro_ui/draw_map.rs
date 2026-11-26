//! Big-map drawing for New Halden: outline, explored streets and the centre label.
//! Nothing is drawn until the city is discovered (no hint before that).

use engine::egui::{self, Align2, Color32, FontId, Pos2, Rect, Stroke, pos2, vec2};
use glam::Vec2;
use world::metro::MetroLayout;

use crate::map_ui::MapUiState;

use super::outline::{discovered, site_corners, streets_on_map};

/// Outline line width in screen pixels.
const OUTLINE_WIDTH_PX: f32 = 2.0; // new: DECISIONS 2026-11-26 (outline width from the task)
/// Avenue width multiplier and the zoom boost, same as the Ashford streets in `draw_city.rs`.
const AVENUE_RATIO: f32 = 1.5; // moved: rotland_ui/src/map_ui/draw_city.rs (avenue width)
const ZOOM_BOOST: f32 = 1.5; // moved: rotland_ui/src/map_ui/draw_city.rs (street zoom boost)
/// Extra world margin around the view when culling streets, in screen pixels.
const CULL_MARGIN_PX: f32 = 32.0; // moved: rotland_ui/src/map_ui/draw_city.rs (cull margin)
/// Label shadow offset in screen pixels (same as Ashford).
const SHADOW_OFFSET_PX: f32 = 1.0; // moved: rotland_ui/src/map_ui/draw_city.rs (label shadow)
/// Shadow alpha under the label (same as Ashford).
const SHADOW_ALPHA: u8 = 220; // moved: rotland_ui/src/map_ui/draw_city.rs (label shadow)

/// Colours and label size from `data/map.json` (`metro_*` keys).
#[derive(Debug, Clone, PartialEq)]
pub struct MetroMapConfig {
    pub outline_color: Color32,
    pub street_color: Color32,
    pub label_size: f32,
    pub label_color: Color32,
}

impl Default for MetroMapConfig {
    fn default() -> Self {
        Self {
            outline_color: Color32::from_rgb(154, 174, 154), // data: map.json metro_outline_color
            street_color: Color32::from_rgb(69, 69, 74),     // data: map.json metro_street_color
            label_size: 16.0,                                // data: map.json metro_label_size
            label_color: Color32::from_rgb(245, 240, 225),   // data: map.json metro_label_color
        }
    }
}

impl MetroMapConfig {
    /// Reads the `metro_*` keys of a `data/map.json` text. Missing or bad keys keep their defaults.
    pub fn load(json: &str) -> Self {
        let mut cfg = Self::default();
        let Ok(root) = serde_json::from_str::<serde_json::Value>(json) else {
            return cfg;
        };
        if let Some(c) = root.get("metro_outline_color").and_then(parse_hex) {
            cfg.outline_color = c;
        }
        if let Some(c) = root.get("metro_street_color").and_then(parse_hex) {
            cfg.street_color = c;
        }
        if let Some(s) = root
            .get("metro_label_size")
            .and_then(serde_json::Value::as_f64)
        {
            cfg.label_size = s as f32;
        }
        if let Some(c) = root.get("metro_label_color").and_then(parse_hex) {
            cfg.label_color = c;
        }
        cfg
    }

    /// Reads `data/map.json` from the repo; a missing file gives the defaults.
    pub fn load_default() -> Self {
        match std::fs::read_to_string(paths::repo_path("data/map.json")) {
            Ok(text) => Self::load(&text),
            Err(_) => Self::default(),
        }
    }
}

/// Parses a `#rrggbb` string.
fn parse_hex(v: &serde_json::Value) -> Option<Color32> {
    let hex = v.as_str()?.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let channel = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    Some(Color32::from_rgb(
        channel(0..2)?,
        channel(2..4)?,
        channel(4..6)?,
    ))
}

/// Draws New Halden on the big map: outline, explored streets and the label, once discovered.
pub fn draw_metro_map(
    painter: &egui::Painter,
    state: &MapUiState,
    explore: &game::MapExplore,
    layout: &MetroLayout,
    cfg: &MetroMapConfig,
    rect: Rect,
    view_size: Vec2,
) {
    let site = &layout.site;
    if !discovered(site, explore) {
        return;
    }
    let scale = state.scale();
    let to_screen = |world: Vec2| -> Pos2 {
        let s = game::world_to_screen(world, state.centre, scale, view_size);
        pos2(rect.min.x + s.x, rect.min.y + s.y)
    };

    let corners = site_corners(site).map(to_screen);
    let outline = Stroke::new(OUTLINE_WIDTH_PX, cfg.outline_color);
    for k in 0..corners.len() {
        painter.line_segment([corners[k], corners[(k + 1) % corners.len()]], outline);
    }

    let (vis_min, vis_max) =
        game::map_visible_world_bounds(state.centre, scale, view_size, CULL_MARGIN_PX / scale);
    let min_w = state.streets_cfg.street_min_px;
    for street in streets_on_map(layout, explore) {
        let a = site.to_world(street.a);
        let b = site.to_world(street.b);
        if !game::segment_in_bounds(a, b, vis_min, vis_max, street.width) {
            continue;
        }
        let ratio = if street.avenue { AVENUE_RATIO } else { 1.0 };
        let stroke_w = (min_w * ratio * scale * ZOOM_BOOST).max(min_w * ratio);
        painter.line_segment(
            [to_screen(a), to_screen(b)],
            Stroke::new(stroke_w, cfg.street_color),
        );
    }

    let centre = to_screen(site.center);
    if rect.contains(centre) {
        draw_label(painter, centre, &site.name, cfg);
    }
}

fn draw_label(painter: &egui::Painter, at: Pos2, text: &str, cfg: &MetroMapConfig) {
    let font = FontId::proportional(cfg.label_size);
    painter.text(
        at + vec2(SHADOW_OFFSET_PX, SHADOW_OFFSET_PX),
        Align2::CENTER_CENTER,
        text,
        font.clone(),
        Color32::from_black_alpha(SHADOW_ALPHA),
    );
    painter.text(at, Align2::CENTER_CENTER, text, font, cfg.label_color);
}
