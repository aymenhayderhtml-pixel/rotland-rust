//! Big-map drawing for New Halden: outline, explored streets and the centre label.
//! Nothing is drawn until the city is discovered (no hint before that).

use engine::egui::{self, Align2, Color32, FontId, Pos2, Rect, Stroke, pos2, vec2};
use glam::Vec2;
use world::metro::safehouse::{rooftop_camp_tower, safe_house_tower};
use world::metro::{ClimbKind, MetroLayout, MetroWorld, Reserved, army_line_site, climb_towers, place_sites};

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
    /// Map labels for the R44 safe house tower and rooftop camp tower (empty = no label).
    pub rooftop: RooftopMapLabels,
}

/// Labels for the two R44 rooftop towers, from `data/safe_house.json` (`mapLabel`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RooftopMapLabels {
    pub safe_house: String,
    pub rooftop_camp: String,
}

impl Default for MetroMapConfig {
    fn default() -> Self {
        Self {
            outline_color: Color32::from_rgb(154, 174, 154), // data: map.json metro_outline_color
            street_color: Color32::from_rgb(69, 69, 74),     // data: map.json metro_street_color
            label_size: 16.0,                                // data: map.json metro_label_size
            label_color: Color32::from_rgb(245, 240, 225),   // data: map.json metro_label_color
            rooftop: RooftopMapLabels::default(),
        }
    }
}

/// The rooftop tower labels from the safe house data (the `mapLabel` keys). Missing data = no label.
fn rooftop_labels_from_repo() -> RooftopMapLabels {
    RooftopMapLabels {
        safe_house: game::safe_house::SafeHouseConfig::from_repo()
            .map(|c| c.map_label)
            .unwrap_or_default(),
        rooftop_camp: world::metro::safehouse::safe_house_layout()
            .map(|l| l.rooftop_camp.map_label.clone())
            .unwrap_or_default(),
    }
}

/// The label for `tower`, when it is the safe house tower or the rooftop camp tower.
pub fn rooftop_label_for(
    tower: usize,
    safe: Option<usize>,
    camp: Option<usize>,
    labels: &RooftopMapLabels,
) -> Option<&str> {
    let text = if safe == Some(tower) {
        labels.safe_house.as_str()
    } else if camp == Some(tower) {
        labels.rooftop_camp.as_str()
    } else {
        ""
    };
    (!text.is_empty()).then_some(text)
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
        let mut cfg = match std::fs::read_to_string(paths::repo_path("data/map.json")) {
            Ok(text) => Self::load(&text),
            Err(_) => Self::default(),
        };
        cfg.rooftop = rooftop_labels_from_repo();
        cfg
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
#[allow(clippy::too_many_arguments)]
pub fn draw_metro_map(
    painter: &egui::Painter,
    state: &MapUiState,
    explore: &game::MapExplore,
    layout: &MetroLayout,
    metro: Option<&MetroWorld>,
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

    for place in place_sites(layout) {
        let label = match place.kind {
            Reserved::Hospital => "Hospital",
            Reserved::Police => "Police",
            Reserved::FireStation => "Fire Station",
            _ => continue,
        };
        let p = to_screen(place.pos);
        if rect.contains(p) {
            draw_small_label(painter, p, label, cfg);
        }
    }
    if let Some(line) = army_line_site(layout) {
        let p = to_screen(line.origin);
        if rect.contains(p) {
            draw_small_label(painter, p, "Army Line", cfg);
        }
    }
    if let Some(m) = metro {
        for plan in climb_towers(m) {
            let label = if plan.kind == ClimbKind::Landmark {
                "Radio Mast"
            } else {
                "Stairs"
            };
            let p = to_screen(m.towers[plan.tower].pos);
            if rect.contains(p) {
                draw_small_label(painter, p, label, cfg);
            }
        }
        let safe = safe_house_tower(m);
        let camp = rooftop_camp_tower(m);
        for idx in [safe, camp].into_iter().flatten() {
            let Some(label) = rooftop_label_for(idx, safe, camp, &cfg.rooftop) else {
                continue;
            };
            let p = to_screen(m.towers[idx].pos);
            if rect.contains(p) {
                draw_small_label(painter, p, label, cfg);
            }
        }
    }
}

fn draw_small_label(painter: &egui::Painter, at: Pos2, text: &str, cfg: &MetroMapConfig) {
    let font = FontId::proportional(11.0);
    painter.text(
        at + vec2(SHADOW_OFFSET_PX, SHADOW_OFFSET_PX),
        Align2::CENTER_CENTER,
        text,
        font.clone(),
        Color32::from_black_alpha(200),
    );
    painter.text(at, Align2::CENTER_CENTER, text, font, cfg.label_color);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rooftop_label_is_only_for_the_right_towers() {
        let labels = RooftopMapLabels { safe_house: "Safe House".into(), rooftop_camp: "Rooftop Camp".into() };
        assert_eq!(rooftop_label_for(3, Some(3), Some(9), &labels), Some("Safe House"));
        assert_eq!(rooftop_label_for(9, Some(3), Some(9), &labels), Some("Rooftop Camp"));
        assert_eq!(rooftop_label_for(4, Some(3), Some(9), &labels), None);
        assert_eq!(rooftop_label_for(3, None, None, &labels), None);
    }

    #[test]
    fn empty_label_text_draws_nothing() {
        let labels = RooftopMapLabels { safe_house: String::new(), rooftop_camp: "Rooftop Camp".into() };
        assert_eq!(rooftop_label_for(3, Some(3), Some(9), &labels), None);
        assert_eq!(rooftop_label_for(9, Some(3), Some(9), &labels), Some("Rooftop Camp"));
    }
}
