//! Interactive UI state for the map page: camera centre, zoom level, and cached texture.
//!
//! Port of pan/zoom logic in scripts/UI/MapView.cs.

use engine::egui;
use glam::{IVec2, Vec2};

/// Persistent interactive state of the map view.
pub struct MapUiState {
    pub centre: Vec2,
    pub zoom_index: usize,
    pub(crate) texture: Option<egui::TextureHandle>,
    pub(crate) texture_bounds: Option<(Vec2, Vec2)>, // (min_world, max_world)
    pub(crate) cached_version: u64,
}

impl Default for MapUiState {
    fn default() -> Self {
        Self {
            centre: Vec2::ZERO,
            zoom_index: game::MAP_DEFAULT_ZOOM,
            texture: None,
            texture_bounds: None,
            cached_version: 0,
        }
    }
}

impl MapUiState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Screen pixels per metre at current zoom level.
    // Godot: MapView.cs Scale
    #[inline]
    pub fn scale(&self) -> f32 {
        game::SCALES[self.zoom_index]
    }

    /// Zooms in (+1) or out (-1), clamped to the available scales.
    // Godot: MapView.cs Zoom
    pub fn zoom(&mut self, direction: i32) {
        self.zoom_index = game::step_zoom(self.zoom_index, direction);
    }

    /// Pans the map by screen pixel delta.
    // Godot: MapView.cs Pan
    pub fn pan(&mut self, screen_delta: egui::Vec2) {
        let delta = Vec2::new(screen_delta.x, screen_delta.y) / self.scale();
        self.centre -= delta;
    }

    /// Centres the camera on a specific world XZ coordinate.
    // Godot: MapView.cs CentreOn
    pub fn centre_on(&mut self, world: Vec2) {
        self.centre = world;
    }

    /// Centres on the player's position if present.
    // Godot: MapView.cs CentreOnPlayer
    pub fn centre_on_player(&mut self, player_pos: Option<Vec2>) -> bool {
        if let Some(pos) = player_pos {
            self.centre_on(pos);
            true
        } else {
            false
        }
    }

    /// Computes a lightweight fingerprint of explored chunks to detect changes.
    pub fn explored_fingerprint(explore: &game::MapExplore) -> u64 {
        let mut hash = 0u64;
        for chunk in explore.chunks() {
            let mask = explore.mask_of(chunk);
            hash = hash
                .wrapping_add(mask)
                .wrapping_add((chunk.x as u64) << 32)
                .wrapping_add(chunk.y as u32 as u64)
                .rotate_left(5);
        }
        hash
    }

    /// Rebuilds the tile texture if the explored chunks have changed.
    pub fn ensure_texture(
        &mut self,
        ctx: &egui::Context,
        generator: Option<&world::WorldGenerator>,
        explore: &game::MapExplore,
    ) {
        let Some(world_gen) = generator else {
            return;
        };
        let fingerprint = Self::explored_fingerprint(explore);
        if fingerprint == 0 {
            self.texture = None;
            self.texture_bounds = None;
            self.cached_version = 0;
            return;
        }

        if self.texture.is_some() && self.cached_version == fingerprint {
            return;
        }

        // Find bounding box of all explored chunks
        let mut min_chunk = IVec2::splat(i32::MAX);
        let mut max_chunk = IVec2::splat(i32::MIN);
        let mut count = 0;

        for chunk in explore.chunks() {
            if explore.mask_of(chunk) != 0 {
                min_chunk = min_chunk.min(chunk);
                max_chunk = max_chunk.max(chunk);
                count += 1;
            }
        }

        if count == 0 {
            self.texture = None;
            self.texture_bounds = None;
            self.cached_version = 0;
            return;
        }

        let chunks_w = (max_chunk.x - min_chunk.x + 1) as usize;
        let chunks_h = (max_chunk.y - min_chunk.y + 1) as usize;
        let texels_w = chunks_w * game::TEXELS_PER_CHUNK;
        let texels_h = chunks_h * game::TEXELS_PER_CHUNK;

        let mut image_data = game::UNEXPLORED_COLOR.repeat(texels_w * texels_h);

        for chunk in explore.chunks() {
            if explore.mask_of(chunk) == 0 {
                continue;
            }
            let chunk_pixels = game::render_chunk_tiles(world_gen, explore, chunk);
            let ox = (chunk.x - min_chunk.x) as usize * game::TEXELS_PER_CHUNK;
            let oz = (chunk.y - min_chunk.y) as usize * game::TEXELS_PER_CHUNK;

            for row in 0..game::TEXELS_PER_CHUNK {
                let src_start = row * game::TEXELS_PER_CHUNK * 4;
                let src_end = src_start + game::TEXELS_PER_CHUNK * 4;
                let dst_start = ((oz + row) * texels_w + ox) * 4;
                let dst_end = dst_start + game::TEXELS_PER_CHUNK * 4;
                image_data[dst_start..dst_end].copy_from_slice(&chunk_pixels[src_start..src_end]);
            }
        }

        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [texels_w, texels_h],
            &image_data,
        );

        let handle = ctx.load_texture("rotland_map_tiles", color_image, egui::TextureOptions::NEAREST);
        self.texture = Some(handle);

        let min_world = Vec2::new(
            min_chunk.x as f32 * world::WorldGenerator::CHUNK_SIZE,
            min_chunk.y as f32 * world::WorldGenerator::CHUNK_SIZE,
        );
        let max_world = Vec2::new(
            (max_chunk.x + 1) as f32 * world::WorldGenerator::CHUNK_SIZE,
            (max_chunk.y + 1) as f32 * world::WorldGenerator::CHUNK_SIZE,
        );
        self.texture_bounds = Some((min_world, max_world));
        self.cached_version = fingerprint;
    }
}
