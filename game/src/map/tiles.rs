//! Chunk tile rendering: height and biome based procedural colors.
//!
//! Port of Godot C# scripts/UI/MapTiles.cs.

use glam::{IVec2, Vec2};
use world::{BiomeId, WorldGenerator};

use super::explore::MapExplore;

// Godot: MapTiles.cs TexelsPerChunk
pub const TEXELS_PER_CHUNK: usize = 16;

// Godot: MapTiles.cs CliffSlope
pub const CLIFF_SLOPE: f32 = 0.7;

// Godot: MapTiles.cs TexelMetres
pub const TEXEL_METRES: f32 = WorldGenerator::CHUNK_SIZE / TEXELS_PER_CHUNK as f32;

// Godot: MapTiles.cs Unexplored (0.2, 0.19, 0.17, 1.0)
pub const UNEXPLORED_COLOR: [u8; 4] = [51, 48, 43, 255];

// Godot: MapTiles.cs Cliff ("6b5236" -> 107, 82, 54)
pub const CLIFF_COLOR: [f32; 3] = [107.0 / 255.0, 82.0 / 255.0, 54.0 / 255.0];

/// Renders a 16x16 RGBA8 tile image for a 64m chunk.
// Godot: MapTiles.cs Render
pub fn render_chunk_tiles(
    generator: &WorldGenerator,
    explore: &MapExplore,
    chunk: IVec2,
) -> Vec<u8> {
    let n = TEXELS_PER_CHUNK;
    let origin_x = chunk.x as f32 * WorldGenerator::CHUNK_SIZE;
    let origin_z = chunk.y as f32 * WorldGenerator::CHUNK_SIZE;

    // Heights are sampled on a grid one texel wider for slope calculations
    let mut heights = [[0.0f32; TEXELS_PER_CHUNK + 2]; TEXELS_PER_CHUNK + 2];
    #[allow(clippy::needless_range_loop)]
    for k in 0..n + 2 {
        for m in 0..n + 2 {
            let sx = origin_x + (k as f32 - 1.0) * TEXEL_METRES + TEXEL_METRES * 0.5;
            let sz = origin_z + (m as f32 - 1.0) * TEXEL_METRES + TEXEL_METRES * 0.5;
            heights[k][m] = generator.height(sx, sz);
        }
    }

    let mut pixels = vec![0u8; n * n * 4];
    for j in 0..n {
        for i in 0..n {
            let x = origin_x + i as f32 * TEXEL_METRES + TEXEL_METRES * 0.5;
            let z = origin_z + j as f32 * TEXEL_METRES + TEXEL_METRES * 0.5;
            let cell = MapExplore::cell_of(Vec2::new(x, z));

            let rgba = if explore.is_revealed(cell) {
                shade_texel(generator, &heights, i + 1, j + 1, x, z)
            } else {
                UNEXPLORED_COLOR
            };

            let idx = (j * n + i) * 4;
            pixels[idx] = rgba[0];
            pixels[idx + 1] = rgba[1];
            pixels[idx + 2] = rgba[2];
            pixels[idx + 3] = rgba[3];
        }
    }

    pixels
}

// Godot: MapTiles.cs Shade
fn shade_texel(
    generator: &WorldGenerator,
    heights: &[[f32; TEXELS_PER_CHUNK + 2]; TEXELS_PER_CHUNK + 2],
    k: usize,
    m: usize,
    x: f32,
    z: f32,
) -> [u8; 4] {
    let h = heights[k][m];
    let slope_x = (heights[k + 1][m] - heights[k - 1][m]) / (2.0 * TEXEL_METRES);
    let slope_z = (heights[k][m + 1] - heights[k][m - 1]) / (2.0 * TEXEL_METRES);

    let is_cliff = slope_x.abs().max(slope_z.abs()) > CLIFF_SLOPE;
    let ground = if is_cliff {
        CLIFF_COLOR
    } else if generator.biome(x, z) == BiomeId::Forest {
        [
            WorldGenerator::FOREST_GRASS.r,
            WorldGenerator::FOREST_GRASS.g,
            WorldGenerator::FOREST_GRASS.b,
        ]
    } else {
        [
            WorldGenerator::PLAINS_GRASS.r,
            WorldGenerator::PLAINS_GRASS.g,
            WorldGenerator::PLAINS_GRASS.b,
        ]
    };

    let light = 0.85 + 0.3 * ((h + 10.0) / 35.0).clamp(0.0, 1.0);
    [
        (ground[0] * light * 255.0).round().clamp(0.0, 255.0) as u8,
        (ground[1] * light * 255.0).round().clamp(0.0, 255.0) as u8,
        (ground[2] * light * 255.0).round().clamp(0.0, 255.0) as u8,
        255,
    ]
}
