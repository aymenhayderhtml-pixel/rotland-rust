//! Persistent HUD mini-map state and cached terrain+fog patch texture.

use std::collections::HashMap;
use std::path::Path;

use engine::egui;
use glam::{IVec2, Vec2};
use world::WorldGenerator;

const MAX_CACHED_CHUNKS: usize = 64;

/// Visual styling configuration for city streets and camp road on the mini-map.
#[derive(Debug, Clone, PartialEq)]
pub struct MinimapStreetsConfig {
    pub street_color: egui::Color32,
    pub road_color: egui::Color32,
    pub blocked_color: egui::Color32,
    pub street_min_px: f32,
    pub water_color: egui::Color32,
    pub dock_color: egui::Color32,
    pub dock_width: f32,
    pub village_lot_color: egui::Color32,
    pub village_lot_outline: egui::Color32,
    pub fishing_spot_color: egui::Color32,
}

impl Default for MinimapStreetsConfig {
    fn default() -> Self {
        Self {
            street_color: egui::Color32::from_rgb(69, 69, 74),
            road_color: egui::Color32::from_rgb(90, 83, 72),
            blocked_color: egui::Color32::from_rgb(217, 53, 43),
            street_min_px: 2.0,
            water_color: egui::Color32::from_rgba_unmultiplied(
                game::LAKE_WATER_COLOR[0],
                game::LAKE_WATER_COLOR[1],
                game::LAKE_WATER_COLOR[2],
                game::LAKE_WATER_COLOR[3],
            ),
            dock_color: egui::Color32::from_rgba_unmultiplied(
                game::LAKE_DOCK_COLOR[0],
                game::LAKE_DOCK_COLOR[1],
                game::LAKE_DOCK_COLOR[2],
                game::LAKE_DOCK_COLOR[3],
            ),
            dock_width: game::LAKE_DOCK_WIDTH,
            village_lot_color: egui::Color32::from_rgba_unmultiplied(
                game::LAKE_VILLAGE_LOT_COLOR[0],
                game::LAKE_VILLAGE_LOT_COLOR[1],
                game::LAKE_VILLAGE_LOT_COLOR[2],
                game::LAKE_VILLAGE_LOT_COLOR[3],
            ),
            village_lot_outline: egui::Color32::from_rgba_unmultiplied(
                game::LAKE_VILLAGE_LOT_OUTLINE[0],
                game::LAKE_VILLAGE_LOT_OUTLINE[1],
                game::LAKE_VILLAGE_LOT_OUTLINE[2],
                game::LAKE_VILLAGE_LOT_OUTLINE[3],
            ),
            fishing_spot_color: egui::Color32::from_rgba_unmultiplied(
                game::LAKE_FISHING_SPOT_COLOR[0],
                game::LAKE_FISHING_SPOT_COLOR[1],
                game::LAKE_FISHING_SPOT_COLOR[2],
                game::LAKE_FISHING_SPOT_COLOR[3],
            ),
        }
    }
}

impl MinimapStreetsConfig {
    pub fn load_default() -> Self {
        let mut cfg = Self::load(&paths::repo_path("data/minimap.json"));
        // Load lake map styling overrides if present
        let map_path = paths::repo_path("data/map.json");
        if let Ok(text) = std::fs::read_to_string(&map_path)
            && let Ok(val) = serde_json::from_str::<serde_json::Value>(&text)
        {
            if let Some(c) = val.get("water_color").and_then(parse_color_value) {
                cfg.water_color = c;
            }
            if let Some(c) = val.get("dock_color").and_then(parse_color_value) {
                cfg.dock_color = c;
            }
            if let Some(w) = val.get("dock_width").and_then(|v| v.as_f64()) {
                cfg.dock_width = (w as f32).max(0.5);
            }
            if let Some(c) = val.get("village_lot_color").and_then(parse_color_value) {
                cfg.village_lot_color = c;
            }
            if let Some(c) = val.get("village_lot_outline").and_then(parse_color_value) {
                cfg.village_lot_outline = c;
            }
            if let Some(c) = val.get("fishing_spot_color").and_then(parse_color_value) {
                cfg.fishing_spot_color = c;
            }
        }
        cfg
    }

    pub fn load(path: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) else {
            return Self::default();
        };
        let mut cfg = Self::default();
        if let Some(c) = val.get("street_color").and_then(parse_color_value) {
            cfg.street_color = c;
        }
        if let Some(c) = val.get("road_color").and_then(parse_color_value) {
            cfg.road_color = c;
        }
        if let Some(c) = val.get("blocked_color").and_then(parse_color_value) {
            cfg.blocked_color = c;
        }
        if let Some(w) = val.get("street_min_px").and_then(|v| v.as_f64()) {
            cfg.street_min_px = (w as f32).max(0.5);
        }
        cfg
    }
}

fn parse_color_value(v: &serde_json::Value) -> Option<egui::Color32> {
    match v {
        serde_json::Value::String(s) => {
            let hex = s.trim_start_matches('#');
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(egui::Color32::from_rgb(r, g, b))
            } else if hex.len() == 8 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(egui::Color32::from_rgba_unmultiplied(r, g, b, a))
            } else {
                None
            }
        }
        serde_json::Value::Array(arr) => {
            if arr.len() == 3 {
                let r = arr[0].as_u64()? as u8;
                let g = arr[1].as_u64()? as u8;
                let b = arr[2].as_u64()? as u8;
                Some(egui::Color32::from_rgb(r, g, b))
            } else if arr.len() == 4 {
                let r = arr[0].as_u64()? as u8;
                let g = arr[1].as_u64()? as u8;
                let b = arr[2].as_u64()? as u8;
                let a = arr[3].as_u64()? as u8;
                Some(egui::Color32::from_rgba_unmultiplied(r, g, b, a))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Persistent mini-map state: data-driven config, cached chunk terrain, cached patch, scratch buffers.
pub struct MinimapState {
    pub config: game::MinimapConfig,
    pub streets_cfg: MinimapStreetsConfig,
    chunk_terrain: HashMap<IVec2, Vec<[u8; 3]>>,
    patch_rgba_scratch: Vec<u8>,
    pub(super) texture: Option<egui::TextureHandle>,
    pub(super) texture_bounds: Option<(Vec2, Vec2)>,
    cached_key: Option<(usize, IVec2, u32, u64)>,
    pub(super) camp_sites_scratch: Vec<game::MinimapCampSite>,
    pub(super) camps_scratch: Vec<Vec2>,
    pub(super) zombies_scratch: Vec<Vec2>,
}

impl Default for MinimapState {
    fn default() -> Self {
        Self {
            config: game::MinimapConfig::load_default(),
            streets_cfg: MinimapStreetsConfig::load_default(),
            chunk_terrain: HashMap::new(),
            patch_rgba_scratch: Vec::new(),
            texture: None,
            texture_bounds: None,
            cached_key: None,
            camp_sites_scratch: Vec::with_capacity(4),
            camps_scratch: Vec::with_capacity(4),
            zombies_scratch: Vec::with_capacity(32),
        }
    }
}

impl MinimapState {
    /// Rebuilds the cached 128x128 local terrain+fog patch only when the player moves into a new
    /// 8m map cell, reveals new cells inside the window, or loads a different world.
    pub(super) fn ensure_terrain_texture(
        &mut self,
        ctx: &egui::Context,
        generator: &WorldGenerator,
        explore: &game::MapExplore,
        player_xz: Vec2,
    ) {
        let player_cell = game::MapExplore::cell_of(player_xz);
        let (min_world, max_world) =
            game::minimap_patch_bounds(player_cell, self.config.radius_metres);
        let gen_id = generator as *const WorldGenerator as usize;

        let min_chunk = IVec2::new(
            ((min_world.x - game::TEXEL_METRES) / WorldGenerator::CHUNK_SIZE).floor() as i32,
            ((min_world.y - game::TEXEL_METRES) / WorldGenerator::CHUNK_SIZE).floor() as i32,
        );
        let max_chunk = IVec2::new(
            ((max_world.x + game::TEXEL_METRES) / WorldGenerator::CHUNK_SIZE).floor() as i32,
            ((max_world.y + game::TEXEL_METRES) / WorldGenerator::CHUNK_SIZE).floor() as i32,
        );

        let mut mask_hash = 0u64;
        for cz in min_chunk.y..=max_chunk.y {
            for cx in min_chunk.x..=max_chunk.x {
                let mask = explore.mask_of(IVec2::new(cx, cz));
                mask_hash = mask_hash.wrapping_add(mask).rotate_left(7) ^ 0x9E37_79B9_7F4A_7C15;
            }
        }

        let key = (
            gen_id,
            player_cell,
            self.config.radius_metres.to_bits(),
            mask_hash,
        );
        if self.texture.is_some() && self.cached_key == Some(key) {
            return;
        }

        if self
            .cached_key
            .is_some_and(|(old_gen, ..)| old_gen != gen_id)
            || self.chunk_terrain.len() > MAX_CACHED_CHUNKS
        {
            self.chunk_terrain.clear();
        }

        for cz in min_chunk.y..=max_chunk.y {
            for cx in min_chunk.x..=max_chunk.x {
                let chunk = IVec2::new(cx, cz);
                self.chunk_terrain
                    .entry(chunk)
                    .or_insert_with(|| game::render_chunk_terrain_rgb(generator, chunk));
            }
        }

        let size = game::MINIMAP_PATCH_SIZE;
        let terrain_cache = &self.chunk_terrain;
        game::render_minimap_patch(
            explore,
            min_world,
            max_world,
            size,
            |chunk| terrain_cache.get(&chunk).map(Vec::as_slice),
            &mut self.patch_rgba_scratch,
        );

        // Water cells show blue (matching big map) where depth_at > 0
        if let Some(lake) = generator.lake() {
            let (lake_min, lake_max) = lake.water_bounds();
            if !(max_world.x < lake_min.x
                || min_world.x > lake_max.x
                || max_world.y < lake_min.y
                || min_world.y > lake_max.y)
            {
                let span = max_world - min_world;
                let inv = 1.0 / (size.max(1) as f32);
                let w_col = self.streets_cfg.water_color;
                for py in 0..size {
                    let wz = min_world.y + (py as f32 + 0.5) * inv * span.y;
                    for px in 0..size {
                        let wx = min_world.x + (px as f32 + 0.5) * inv * span.x;
                        let pos = Vec2::new(wx, wz);
                        if lake.contains_region(pos) && lake.depth_at(pos) > 0.0 {
                            let cov = game::smooth_explored_coverage(explore, pos);
                            if cov > 0.001 {
                                let idx = (py * size + px) * 4;
                                let rgb = if cov >= 0.999 {
                                    [w_col.r(), w_col.g(), w_col.b()]
                                } else {
                                    [
                                        lerp_u8(game::MINIMAP_UNEXPLORED_RGB[0], w_col.r(), cov),
                                        lerp_u8(game::MINIMAP_UNEXPLORED_RGB[1], w_col.g(), cov),
                                        lerp_u8(game::MINIMAP_UNEXPLORED_RGB[2], w_col.b(), cov),
                                    ]
                                };
                                self.patch_rgba_scratch[idx] = rgb[0];
                                self.patch_rgba_scratch[idx + 1] = rgb[1];
                                self.patch_rgba_scratch[idx + 2] = rgb[2];
                            }
                        }
                    }
                }
            }
        }

        let color_image =
            egui::ColorImage::from_rgba_unmultiplied([size, size], &self.patch_rgba_scratch);
        let handle = ctx.load_texture(
            "rotland_minimap_tiles",
            color_image,
            egui::TextureOptions::LINEAR,
        );
        self.texture = Some(handle);
        self.texture_bounds = Some((min_world, max_world));
        self.cached_key = Some(key);
    }
}

#[inline]
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t)
        .round()
        .clamp(0.0, 255.0) as u8
}
