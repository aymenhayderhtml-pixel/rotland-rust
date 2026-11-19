//! Persistent HUD mini-map state and cached terrain+fog patch texture.

use std::collections::HashMap;

use engine::egui;
use glam::{IVec2, Vec2};
use world::WorldGenerator;

const MAX_CACHED_CHUNKS: usize = 64;

/// Persistent mini-map state: data-driven config, cached chunk terrain, cached patch, scratch buffers.
pub struct MinimapState {
    pub config: game::MinimapConfig,
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
