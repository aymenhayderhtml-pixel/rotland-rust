//! Chunk tile rendering: height and biome based procedural colors.
//!
//! Port of Godot C# scripts/UI/MapTiles.cs.

use glam::{IVec2, Vec2};
use world::WorldGenerator;

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

// new: DECISIONS 2026-11-25
pub const HIGHWAY_MAP_COLOR: [u8; 4] = [72, 72, 76, 255];
// new: DECISIONS 2026-11-25
pub const TRAIL_MAP_COLOR: [u8; 4] = [166, 132, 92, 255];
// new: DECISIONS 2026-11-25
const HIGHWAY_MAP_HALF_WIDTH_M: f32 = 4.0;
// new: DECISIONS 2026-11-25
const TRAIL_MAP_HALF_WIDTH_M: f32 = 3.0;

fn sample_chunk_heights(
    generator: &WorldGenerator,
    origin_x: f32,
    origin_z: f32,
) -> [[f32; TEXELS_PER_CHUNK + 2]; TEXELS_PER_CHUNK + 2] {
    let n = TEXELS_PER_CHUNK;
    let mut heights = [[0.0f32; TEXELS_PER_CHUNK + 2]; TEXELS_PER_CHUNK + 2];
    #[allow(clippy::needless_range_loop)]
    for k in 0..n + 2 {
        for m in 0..n + 2 {
            let sx = origin_x + (k as f32 - 1.0) * TEXEL_METRES + TEXEL_METRES * 0.5;
            let sz = origin_z + (m as f32 - 1.0) * TEXEL_METRES + TEXEL_METRES * 0.5;
            heights[k][m] = generator.height(sx, sz);
        }
    }
    heights
}

/// Renders a 16x16 RGB8 terrain-only image for a 64m chunk (without fog), suitable for caching.
pub fn render_chunk_terrain_rgb(generator: &WorldGenerator, chunk: IVec2) -> Vec<[u8; 3]> {
    let n = TEXELS_PER_CHUNK;
    let origin_x = chunk.x as f32 * WorldGenerator::CHUNK_SIZE;
    let origin_z = chunk.y as f32 * WorldGenerator::CHUNK_SIZE;
    let heights = sample_chunk_heights(generator, origin_x, origin_z);
    let mut pixels = vec![[0u8; 3]; n * n];
    for j in 0..n {
        for i in 0..n {
            let x = origin_x + i as f32 * TEXEL_METRES + TEXEL_METRES * 0.5;
            let z = origin_z + j as f32 * TEXEL_METRES + TEXEL_METRES * 0.5;
            let rgba = shade_texel(generator, &heights, i + 1, j + 1, x, z);
            pixels[j * n + i] = [rgba[0], rgba[1], rgba[2]];
        }
    }
    pixels
}

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
    let heights = sample_chunk_heights(generator, origin_x, origin_z);
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
            pixels[idx..idx + 4].copy_from_slice(&rgba);
        }
    }
    pixels
}

fn rgb_u8_to_f32(c: [u8; 3]) -> [f32; 3] {
    [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0]
}

fn lerp_rgb(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] * (1.0 - t) + b[0] * t,
        a[1] * (1.0 - t) + b[1] * t,
        a[2] * (1.0 - t) + b[2] * t,
    ]
}

fn ground_color(generator: &WorldGenerator, x: f32, z: f32) -> [f32; 3] {
    let sample = generator.biome_sample(x, z);
    let core = if generator.forest_weight(x, z) > 0.5 {
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

    if sample.outer <= 0.0 {
        return core;
    }

    let registry = generator.biome_registry();
    let primary = rgb_u8_to_f32(registry.get(sample.primary).map_color);
    let secondary = rgb_u8_to_f32(registry.get(sample.secondary).map_color);
    let outer = lerp_rgb(primary, secondary, sample.blend);
    lerp_rgb(core, outer, sample.outer)
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
    let layout = generator.world_layout();
    let origin = layout.origin();
    let core_r = layout.data().core_radius;
    let dx = x - origin.x;
    let dz = z - origin.y;
    if dx * dx + dz * dz > core_r * core_r {
        if generator.rivers().water_at(x, z).is_some() {
            return super::lake::LAKE_WATER_COLOR;
        }
        if let Some((dist, kind)) = generator.roads().distance_to_road(Vec2::new(x, z)) {
            let max_dist = match kind {
                world::RoadKind::Highway => HIGHWAY_MAP_HALF_WIDTH_M,
                world::RoadKind::Trail => TRAIL_MAP_HALF_WIDTH_M,
            };
            if dist <= max_dist {
                return match kind {
                    world::RoadKind::Highway => HIGHWAY_MAP_COLOR,
                    world::RoadKind::Trail => TRAIL_MAP_COLOR,
                };
            }
        }
    }

    let h = heights[k][m];
    let slope_x = (heights[k + 1][m] - heights[k - 1][m]) / (2.0 * TEXEL_METRES);
    let slope_z = (heights[k][m + 1] - heights[k][m - 1]) / (2.0 * TEXEL_METRES);

    let is_cliff = slope_x.abs().max(slope_z.abs()) > CLIFF_SLOPE;
    let ground = if is_cliff {
        CLIFF_COLOR
    } else {
        ground_color(generator, x, z)
    };

    let light = 0.85 + 0.3 * ((h + 10.0) / 35.0).clamp(0.0, 1.0);
    [
        (ground[0] * light * 255.0).round().clamp(0.0, 255.0) as u8,
        (ground[1] * light * 255.0).round().clamp(0.0, 255.0) as u8,
        (ground[2] * light * 255.0).round().clamp(0.0, 255.0) as u8,
        255,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_rgb_interpolates_and_clamps() {
        let a = [0.0, 0.5, 1.0];
        let b = [1.0, 0.5, 0.0];
        let mid = lerp_rgb(a, b, 0.5);
        assert!((mid[0] - 0.5).abs() < 1e-6);
        assert!((mid[1] - 0.5).abs() < 1e-6);
        assert!((mid[2] - 0.5).abs() < 1e-6);
        assert_eq!(lerp_rgb(a, b, -1.0), a);
        assert_eq!(lerp_rgb(a, b, 2.0), b);
    }

    #[test]
    fn road_texel_outside_core_has_road_color() {
        let generator = WorldGenerator::from_data_dir(42, paths::repo_path("data"))
            .expect("load data dir");
        let origin = generator.world_layout().origin();
        let core_r = generator.world_layout().data().core_radius;

        let mut found = None;
        'outer: for road in generator.roads().roads() {
            for &pt in &road.points {
                if pt.distance(origin) > core_r + 10.0
                    && generator.rivers().water_at(pt.x, pt.y).is_none()
                {
                    found = Some((pt, road.kind));
                    break 'outer;
                }
            }
        }
        let (pt, kind) = found.expect("expected a road point outside core");

        let chunk = IVec2::new(
            (pt.x / WorldGenerator::CHUNK_SIZE).floor() as i32,
            (pt.y / WorldGenerator::CHUNK_SIZE).floor() as i32,
        );
        let ox = chunk.x as f32 * WorldGenerator::CHUNK_SIZE;
        let oz = chunk.y as f32 * WorldGenerator::CHUNK_SIZE;
        let i = (((pt.x - ox) / TEXEL_METRES).floor() as usize).min(TEXELS_PER_CHUNK - 1);
        let j = (((pt.y - oz) / TEXEL_METRES).floor() as usize).min(TEXELS_PER_CHUNK - 1);

        let pixels = render_chunk_terrain_rgb(&generator, chunk);
        let expected = match kind {
            world::RoadKind::Highway => [
                HIGHWAY_MAP_COLOR[0],
                HIGHWAY_MAP_COLOR[1],
                HIGHWAY_MAP_COLOR[2],
            ],
            world::RoadKind::Trail => [
                TRAIL_MAP_COLOR[0],
                TRAIL_MAP_COLOR[1],
                TRAIL_MAP_COLOR[2],
            ],
        };
        assert_eq!(pixels[j * TEXELS_PER_CHUNK + i], expected);
    }

    #[test]
    fn core_tile_is_unchanged() {
        let generator = WorldGenerator::from_data_dir(42, paths::repo_path("data"))
            .expect("load data dir");
        let plain = WorldGenerator::new(42, generator.config().clone());
        let a = render_chunk_terrain_rgb(&generator, IVec2::ZERO);
        let b = render_chunk_terrain_rgb(&plain, IVec2::ZERO);
        assert_eq!(a, b);
    }
}
