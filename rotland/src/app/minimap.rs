//! Round north-up HUD mini-map rendered with egui shapes and cached terrain tiles.

use std::collections::HashMap;
use std::f32::consts::TAU;

use engine::egui::{self, Align2, Color32, FontId, Stroke, pos2, vec2};
use glam::{IVec2, Vec2};
use world::WorldGenerator;

use crate::app::state::{ActorsState, UiState, WorldState};
use crate::play::PlayState;

use super::minimap_draw::{draw_marker, draw_player};

const CIRCLE_SEGMENTS: usize = 48;
const MAX_CACHED_CHUNKS: usize = 64;

pub struct MinimapState {
    pub config: game::MinimapConfig,
    chunk_pixels: HashMap<IVec2, (u64, Vec<u8>)>,
    texture: Option<egui::TextureHandle>,
    texture_bounds: Option<(Vec2, Vec2)>,
    cached_key: Option<(usize, IVec2, i32, u64)>,
    camps_scratch: Vec<Vec2>,
    zombies_scratch: Vec<Vec2>,
}

impl Default for MinimapState {
    fn default() -> Self {
        Self {
            config: game::MinimapConfig::load_default(),
            chunk_pixels: HashMap::new(),
            texture: None,
            texture_bounds: None,
            cached_key: None,
            camps_scratch: Vec::with_capacity(4),
            zombies_scratch: Vec::with_capacity(32),
        }
    }
}

impl MinimapState {
    fn ensure_terrain_texture(
        &mut self,
        ctx: &egui::Context,
        generator: &WorldGenerator,
        explore: &game::MapExplore,
        player_xz: Vec2,
    ) {
        let (center_chunk, reach) =
            game::minimap_chunk_window(player_xz, self.config.radius_metres);
        let gen_id = generator as *const WorldGenerator as usize;
        let mut mask_hash = 0u64;
        for dz in -reach..=reach {
            for dx in -reach..=reach {
                let chunk = IVec2::new(center_chunk.x + dx, center_chunk.y + dz);
                mask_hash = mask_hash
                    .wrapping_add(explore.mask_of(chunk))
                    .rotate_left(7)
                    ^ 0x9E37_79B9_7F4A_7C15;
            }
        }
        let key = (gen_id, center_chunk, reach, mask_hash);
        if self.texture.is_some() && self.cached_key == Some(key) {
            return;
        }
        if self
            .cached_key
            .is_some_and(|(old_gen, ..)| old_gen != gen_id)
            || self.chunk_pixels.len() > MAX_CACHED_CHUNKS
        {
            self.chunk_pixels.clear();
        }

        let side_chunks = (reach * 2 + 1) as usize;
        let n = game::TEXELS_PER_CHUNK;
        let side = side_chunks * n;
        let mut image_data = game::UNEXPLORED_COLOR.repeat(side * side);
        let min_chunk = IVec2::new(center_chunk.x - reach, center_chunk.y - reach);
        for dz in 0..side_chunks {
            for dx in 0..side_chunks {
                let chunk = IVec2::new(min_chunk.x + dx as i32, min_chunk.y + dz as i32);
                let mask = explore.mask_of(chunk);
                if mask == 0 {
                    continue;
                }
                let pixels = match self.chunk_pixels.get(&chunk) {
                    Some((cached_mask, buf)) if *cached_mask == mask => buf.as_slice(),
                    _ => {
                        let rendered = game::render_chunk_tiles(generator, explore, chunk);
                        self.chunk_pixels.insert(chunk, (mask, rendered));
                        self.chunk_pixels[&chunk].1.as_slice()
                    }
                };
                for row in 0..n {
                    let src = row * n * 4;
                    let dst = ((dz * n + row) * side + dx * n) * 4;
                    image_data[dst..dst + n * 4].copy_from_slice(&pixels[src..src + n * 4]);
                }
            }
        }
        let image = egui::ColorImage::from_rgba_unmultiplied([side, side], &image_data);
        self.texture =
            Some(ctx.load_texture("rotland_minimap_tiles", image, egui::TextureOptions::LINEAR));
        let chunk_size = WorldGenerator::CHUNK_SIZE;
        self.texture_bounds = Some((
            Vec2::new(
                min_chunk.x as f32 * chunk_size,
                min_chunk.y as f32 * chunk_size,
            ),
            Vec2::new(
                (min_chunk.x + side_chunks as i32) as f32 * chunk_size,
                (min_chunk.y + side_chunks as i32) as f32 * chunk_size,
            ),
        ));
        self.cached_key = Some(key);
    }
}

pub fn draw_minimap_hud(
    ctx: &egui::Context,
    world: &WorldState,
    actors: &ActorsState,
    play_state: &PlayState,
    ui: &mut UiState,
) {
    if !ui.menus.as_ref().is_none_or(|m| m.settings.show_minimap)
        || ui.menus.as_ref().is_some_and(|m| m.is_open())
        || ui.hub.is_some()
        || ui.camp_ui.is_some()
        || actors.combat.as_ref().is_some_and(|c| c.is_dead())
    {
        return;
    }
    let Some(player) = actors.player_ctrl.as_ref() else {
        return;
    };
    let player_xz = Vec2::new(player.position.x, player.position.z);
    let yaw = player.yaw;
    ui.minimap.camps_scratch.clear();
    if let Some(generator) = world.world_gen.as_ref() {
        for settlement in generator.settlements() {
            ui.minimap
                .camps_scratch
                .push(Vec2::new(settlement.center.x, settlement.center.z));
        }
        ui.minimap
            .ensure_terrain_texture(ctx, generator, &play_state.map.explore, player_xz);
    }
    ui.minimap.zombies_scratch.clear();
    if ui.minimap.config.show_zombies
        && let Some(zombies) = actors.zombies.as_ref()
    {
        for zombie in &zombies.list {
            if !zombie.health.is_dead() {
                ui.minimap
                    .zombies_scratch
                    .push(Vec2::new(zombie.position.x, zombie.position.z));
            }
        }
    }
    draw_overlay(
        ctx,
        &ui.minimap,
        &play_state.map,
        play_state.camp.bounties.active(),
        player_xz,
        yaw,
    );
}

fn draw_overlay(
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
            painter.circle_filled(
                centre,
                radius,
                Color32::from_rgba_unmultiplied(18, 18, 20, (cfg.opacity * 220.0) as u8),
            );
            if let (Some(texture), Some((min, max))) = (&state.texture, state.texture_bounds) {
                let span = max - min;
                if span.x > 0.0 && span.y > 0.0 {
                    let disk_r = (radius - cfg.border_width_px * scale * 0.5).max(1.0);
                    let tint = Color32::from_rgba_unmultiplied(235, 235, 235, alpha);
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
            let rim = (radius - 7.0 * scale).max(radius * 0.75);
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
            draw_player(painter, centre, yaw, scale);
            painter.circle_stroke(
                centre,
                radius,
                Stroke::new(cfg.border_width_px * scale, Color32::from_gray(210)),
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

