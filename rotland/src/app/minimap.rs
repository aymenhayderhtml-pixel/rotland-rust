//! Round north-up HUD mini-map: visibility gating, marker gathering, and overlay rendering.

use std::f32::consts::TAU;

use engine::egui::{self, Align2, Color32, FontId, Stroke, pos2, vec2};
use glam::Vec2;

use crate::app::state::{ActorsState, UiState, WorldState};
use crate::play::PlayState;

pub use super::minimap_cache::MinimapState;
use super::minimap_draw::{draw_camp_outlines, draw_marker, draw_player};

const CIRCLE_SEGMENTS: usize = 48;
const CAMP_GATE_HALF_METRES: f32 = 3.2;

/// Top-level frame helper called once per frame from `frame_ui::draw_ui`.
/// Automatically hides the mini-map whenever settings disable it or menus/hub/camp/death are open.
pub fn draw_minimap_hud(
    ctx: &egui::Context,
    world: &WorldState,
    actors: &ActorsState,
    play_state: &PlayState,
    ui: &mut UiState,
) {
    let show_minimap = ui.menus.as_ref().is_none_or(|m| m.settings.show_minimap);
    let menu_open = ui.menus.as_ref().is_some_and(|m| m.is_open());
    let screen_open = ui.hub.is_some() || ui.camp_ui.is_some();
    let dead = actors.combat.as_ref().is_some_and(|c| c.is_dead());
    if !show_minimap || menu_open || screen_open || dead {
        return;
    }

    let Some(player) = actors.player_ctrl.as_ref() else {
        return;
    };
    let player_xz = Vec2::new(player.position.x, player.position.z);
    let player_yaw = player.yaw;

    ui.minimap.camp_sites_scratch.clear();
    ui.minimap.camps_scratch.clear();
    if let Some(generator) = world.world_gen.as_ref() {
        let half_size = generator.config().safe_zone_half;
        for s in generator.settlements() {
            let center = Vec2::new(s.center.x, s.center.z);
            ui.minimap.camps_scratch.push(center);
            ui.minimap.camp_sites_scratch.push(game::MinimapCampSite {
                center,
                yaw: s.yaw,
                half_size,
                gate_half: CAMP_GATE_HALF_METRES,
            });
        }
        ui.minimap
            .ensure_terrain_texture(ctx, generator, &play_state.map.explore, player_xz);
    }

    ui.minimap.zombies_scratch.clear();
    if ui.minimap.config.show_zombies
        && let Some(zombies) = actors.zombies.as_ref()
    {
        for z in &zombies.list {
            if !z.health.is_dead() {
                ui.minimap
                    .zombies_scratch
                    .push(Vec2::new(z.position.x, z.position.z));
            }
        }
    }

    draw_minimap_overlay(
        ctx,
        &ui.minimap,
        &play_state.map,
        play_state.camp.bounties.active(),
        player_xz,
        player_yaw,
    );
}

fn draw_minimap_overlay(
    ctx: &egui::Context,
    state: &MinimapState,
    map: &game::MapState,
    bounties: &[game::BountyDef],
    player_xz: Vec2,
    yaw: f32,
) {
    egui::Area::new(egui::Id::unique("rotland_minimap_hud"))
        .fixed_pos(pos2(0.0, 0.0))
        .interactable(false)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let painter = ui.painter();
            let screen = ctx.content_rect();
            let scale = game::hud_math::hud_scale(screen.height());
            let (centre_v, radius) = game::minimap_circle(
                Vec2::new(screen.width(), screen.height()),
                scale,
                &state.config,
            );
            let centre = pos2(screen.min.x + centre_v.x, screen.min.y + centre_v.y);
            let cfg = &state.config;
            let alpha = (cfg.opacity * 255.0).round().clamp(0.0, 255.0) as u8;
            let bg_alpha = (cfg.opacity * 230.0).round().clamp(0.0, 255.0) as u8;
            let disk_r = (radius - cfg.border_width_px * scale * 0.5).max(1.0);
            painter.circle_filled(
                centre,
                radius,
                Color32::from_rgba_unmultiplied(18, 18, 20, bg_alpha),
            );
            if let (Some(texture), Some((min, max))) = (&state.texture, state.texture_bounds) {
                let span = max - min;
                if span.x > 0.0 && span.y > 0.0 {
                    let tint = Color32::from_rgba_unmultiplied(245, 245, 245, alpha);
                    let mut mesh = egui::Mesh::with_texture(texture.id());
                    let uv = |w: Vec2| {
                        pos2(
                            ((w.x - min.x) / span.x).clamp(0.0, 1.0),
                            ((w.y - min.y) / span.y).clamp(0.0, 1.0),
                        )
                    };
                    mesh.vertices.push(egui::epaint::Vertex {
                        pos: centre,
                        uv: uv(player_xz),
                        color: tint,
                    });
                    for i in 0..=CIRCLE_SEGMENTS {
                        let a = i as f32 / CIRCLE_SEGMENTS as f32 * TAU;
                        let d = Vec2::new(a.cos(), a.sin());
                        mesh.vertices.push(egui::epaint::Vertex {
                            pos: pos2(centre.x + d.x * disk_r, centre.y + d.y * disk_r),
                            uv: uv(player_xz + d * cfg.radius_metres),
                            color: tint,
                        });
                    }
                    for i in 0..CIRCLE_SEGMENTS as u32 {
                        mesh.add_triangle(0, i + 1, i + 2);
                    }
                    painter.add(egui::Shape::mesh(mesh));
                }
            }
            let camp_outlines = game::collect_minimap_camp_outlines(
                &state.camp_sites_scratch,
                player_xz,
                cfg.radius_metres,
                radius,
                disk_r - 1.5 * scale,
            );
            draw_camp_outlines(painter, centre, &camp_outlines, scale);

            let rim = (radius - 7.5 * scale).max(radius * 0.75);
            let markers = game::collect_minimap_markers(
                map,
                state.camps_scratch.iter().copied(),
                bounties,
                player_xz,
                cfg.radius_metres,
                radius,
                rim,
            );
            for marker in &markers {
                let at = pos2(centre.x + marker.offset_px.x, centre.y + marker.offset_px.y);
                draw_marker(painter, at, marker, scale);
            }
            if cfg.show_zombies {
                let dots = game::collect_minimap_zombies(
                    state.zombies_scratch.iter().copied(),
                    player_xz,
                    cfg.radius_metres,
                    radius,
                    rim,
                );
                for dot in dots {
                    painter.circle(
                        pos2(centre.x + dot.x, centre.y + dot.y),
                        2.6 * scale,
                        Color32::from_rgb(235, 55, 55),
                        Stroke::new(0.8 * scale, Color32::BLACK),
                    );
                }
            }
            draw_player(
                painter,
                centre,
                yaw,
                cfg.player_arrow_size_px * scale,
                scale,
            );
            painter.circle_stroke(
                centre,
                radius + 0.5 * scale,
                Stroke::new(
                    cfg.border_width_px * scale + 1.4 * scale,
                    Color32::from_rgba_unmultiplied(0, 0, 0, 200),
                ),
            );
            painter.circle_stroke(
                centre,
                radius,
                Stroke::new(
                    cfg.border_width_px * scale,
                    Color32::from_rgba_unmultiplied(210, 210, 210, 180),
                ),
            );
            for d in [vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(-1.0, 0.0)] {
                painter.line_segment(
                    [
                        centre + d * (radius - 4.5 * scale),
                        centre + d * (radius - scale),
                    ],
                    Stroke::new(1.2 * scale, Color32::LIGHT_GRAY),
                );
            }
            let n = pos2(centre.x, centre.y - radius);
            painter.circle_filled(
                n,
                8.0 * scale,
                Color32::from_rgba_unmultiplied(18, 18, 22, 235),
            );
            painter.text(
                n,
                Align2::CENTER_CENTER,
                "N",
                FontId::proportional(11.0 * scale),
                Color32::from_rgb(255, 209, 76),
            );
        });
}
