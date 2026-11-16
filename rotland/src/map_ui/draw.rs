//! Drawing routines for the full-screen map page (egui 0.37).
//!
//! Port of Godot C# scripts/UI/MapView.cs and scripts/UI/HubMapPage.cs.

use engine::egui::{self, pos2, vec2, Color32, FontId, Key, Pos2, Rect, Sense};
use glam::Vec2;

use super::action::MapAction;
use super::draw_icons::{
    draw_bounty_icon, draw_building_icon, draw_camp_icon, draw_death_icon, draw_player_icon,
    draw_waypoint_icon,
};
use super::state::MapUiState;
use super::view::MapView;

const WAYPOINT_HIT_PIXELS: f32 = 14.0;

const LEGEND_TEXT: &str =
    "Arrow: you | House: camp | C H G B S T: cabin, house, gas, barn, shed, tower (grey = searched) \
    | Red ring: bounty | Skull: death | Gold diamond: waypoint. \
    Drag to pan, wheel to zoom, right-click to set or clear waypoint.";

/// Central drawing function for the full-screen map UI.
/// Returns an action if a button or interaction was triggered this frame.
// Godot: HubMapPage.cs and MapView.cs
pub fn draw_map(
    ui: &mut egui::Ui,
    state: &mut MapUiState,
    view: &MapView<'_>,
) -> Option<MapAction> {
    let mut action = None;

    // 1. Esc key closes the map
    if ui.input(|i| i.key_pressed(Key::Escape)) {
        return Some(MapAction::Close);
    }

    ui.vertical(|ui| {
        // 2. Header controls row
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
            if ui.button("Centre on me").clicked() {
                state.centre_on_player(view.player_pos);
            }
            if ui.button("Zoom in").clicked() {
                state.zoom(1);
            }
            if ui.button("Zoom out").clicked() {
                state.zoom(-1);
            }
            let zoom_pct = (state.scale() / game::SCALES[game::MAP_DEFAULT_ZOOM] * 100.0).round() as i32;
            ui.label(format!("{zoom_pct}%"));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    action = Some(MapAction::Close);
                }
            });
        });

        ui.add_space(4.0);

        // 3. Main map viewport area
        let available_size = ui.available_size() - vec2(0.0, 28.0); // leave room for legend
        let (rect, response) = ui.allocate_exact_size(
            vec2(available_size.x.max(200.0), available_size.y.max(200.0)),
            Sense::click_and_drag(),
        );

        let view_size = Vec2::new(rect.width(), rect.height());

        // Mouse wheel zooming
        let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
        if response.hovered() && scroll_y.abs() > 0.0 {
            if scroll_y > 0.0 {
                state.zoom(1);
            } else {
                state.zoom(-1);
            }
        }

        // Drag panning
        if response.dragged() {
            state.pan(response.drag_delta());
        }

        // Right-click waypoint toggle
        if response.secondary_clicked()
            && let Some(pointer_pos) = response.interact_pointer_pos()
        {
            let local_screen = Vec2::new(pointer_pos.x - rect.min.x, pointer_pos.y - rect.min.y);
            if let Some(existing_wp) = view.map_state.waypoint {
                let wp_screen = game::world_to_screen(existing_wp, state.centre, state.scale(), view_size);
                if (wp_screen - local_screen).length() <= WAYPOINT_HIT_PIXELS {
                    action = Some(MapAction::RemoveMarker);
                } else {
                    let world_pt = game::screen_to_world(local_screen, state.centre, state.scale(), view_size);
                    action = Some(MapAction::AddMarker(world_pt));
                }
            } else {
                let world_pt = game::screen_to_world(local_screen, state.centre, state.scale(), view_size);
                action = Some(MapAction::AddMarker(world_pt));
            }
        }

        // 4. Painting
        let painter = ui.painter_at(rect);

        // Unexplored parchment background
        painter.rect_filled(
            rect,
            0.0,
            Color32::from_rgb(
                game::UNEXPLORED_COLOR[0],
                game::UNEXPLORED_COLOR[1],
                game::UNEXPLORED_COLOR[2],
            ),
        );

        // Tile texture overlay
        state.ensure_texture(ui.ctx(), view.generator, &view.map_state.explore);
        if let (Some(texture), Some((min_world, max_world))) = (&state.texture, state.texture_bounds) {
            let s_min = game::world_to_screen(min_world, state.centre, state.scale(), view_size);
            let s_max = game::world_to_screen(max_world, state.centre, state.scale(), view_size);
            let img_rect = Rect::from_min_max(
                pos2(rect.min.x + s_min.x, rect.min.y + s_min.y),
                pos2(rect.min.x + s_max.x, rect.min.y + s_max.y),
            );
            painter.image(
                texture.id(),
                img_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        let to_screen_pos = |world: Vec2| -> Pos2 {
            let s = game::world_to_screen(world, state.centre, state.scale(), view_size);
            pos2(rect.min.x + s.x, rect.min.y + s.y)
        };

        // Draw collected markers
        let markers = game::collect_markers(
            view.map_state,
            view.camps.iter().copied(),
            view.bounties,
        );

        for marker in markers {
            let at = to_screen_pos(marker.position);
            match marker.kind {
                game::MapMarkerKind::Camp => draw_camp_icon(&painter, at),
                game::MapMarkerKind::Building => {
                    let letter = game::letter_for(&marker.building_type);
                    draw_building_icon(&painter, at, letter, marker.dim);
                }
                game::MapMarkerKind::Bounty => draw_bounty_icon(&painter, at),
                game::MapMarkerKind::Death => draw_death_icon(&painter, at),
                game::MapMarkerKind::Waypoint => draw_waypoint_icon(&painter, at),
            }
        }

        // Draw player arrow
        if let Some(player_pos) = view.player_pos {
            let at = to_screen_pos(player_pos);
            draw_player_icon(&painter, at, view.player_yaw);
        }

        ui.add_space(4.0);

        // 5. Legend
        ui.label(
            egui::RichText::new(LEGEND_TEXT)
                .font(FontId::proportional(12.0))
                .color(Color32::from_rgb(180, 180, 180)),
        );
    });

    action
}
