//! Pure mini-map terrain and smooth explored-fog patch rendering.
//!
//! Renders a cached local RGBA8 patch around the player's current 8m map cell by bilinearly
//! sampling per-chunk terrain colours and smoothing the `MapExplore` cell mask so explored
//! areas fill cleanly without blocky 8m stair-steps or low-res blur.

use glam::{IVec2, Vec2};

use super::explore::{CELL_SIZE, MapExplore};
use super::tiles::{TEXEL_METRES, TEXELS_PER_CHUNK};

/// Side length in pixels of the cached local mini-map terrain+fog patch.
pub const MINIMAP_PATCH_SIZE: usize = 128;

/// Clean dark fog colour for unexplored ground on the mini-map.
pub const MINIMAP_UNEXPLORED_RGB: [u8; 3] = [28, 28, 30];

/// Returns `(min_world, max_world)` covering `±(radius_metres + CELL_SIZE)` around `player_cell`'s centre.
/// Keeping one extra cell of margin lets the player walk anywhere inside `player_cell` with smooth
/// sub-pixel UV panning and zero texture rebuilds until entering a new cell.
pub fn minimap_patch_bounds(player_cell: IVec2, radius_metres: f32) -> (Vec2, Vec2) {
    let centre = (player_cell.as_vec2() + Vec2::splat(0.5)) * CELL_SIZE;
    let half = radius_metres.max(1.0) + CELL_SIZE;
    (centre - Vec2::splat(half), centre + Vec2::splat(half))
}

/// Smoothly samples the `MapExplore` 8m cell grid at `world_xz` in `[0.0, 1.0]`.
/// Fully revealed regions return `1.0`, fully unrevealed regions return `0.0`, and boundaries
/// transition smoothly without jagged 8m stair-step blocks.
pub fn smooth_explored_coverage(explore: &MapExplore, world_xz: Vec2) -> f32 {
    let gx = world_xz.x / CELL_SIZE - 0.5;
    let gz = world_xz.y / CELL_SIZE - 0.5;
    let x0 = gx.floor() as i32;
    let z0 = gz.floor() as i32;
    let fx = gx - x0 as f32;
    let fz = gz - z0 as f32;

    let r = |cx: i32, cz: i32| -> f32 {
        if explore.is_revealed(IVec2::new(cx, cz)) {
            1.0
        } else {
            0.0
        }
    };

    let c00 = r(x0, z0);
    let c10 = r(x0 + 1, z0);
    let c01 = r(x0, z0 + 1);
    let c11 = r(x0 + 1, z0 + 1);

    let top = c00 + (c10 - c00) * fx;
    let bot = c01 + (c11 - c01) * fx;
    let cov = top + (bot - top) * fz;

    let t = ((cov - 0.38) / 0.24).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Bilinearly samples a cached 16x16 per-chunk terrain RGB grid at `world_xz`.
pub fn sample_chunk_terrain_bilinear<'a>(
    world_xz: Vec2,
    get_chunk: &impl Fn(IVec2) -> Option<&'a [[u8; 3]]>,
) -> [u8; 3] {
    let tx = world_xz.x / TEXEL_METRES - 0.5;
    let tz = world_xz.y / TEXEL_METRES - 0.5;
    let x0 = tx.floor() as i32;
    let z0 = tz.floor() as i32;
    let fx = tx - x0 as f32;
    let fz = tz - z0 as f32;

    let sample = |ix: i32, iz: i32| -> [f32; 3] {
        let n = TEXELS_PER_CHUNK as i32;
        let chunk = IVec2::new(ix.div_euclid(n), iz.div_euclid(n));
        let lx = ix.rem_euclid(n) as usize;
        let lz = iz.rem_euclid(n) as usize;
        if let Some(pixels) = get_chunk(chunk)
            && let Some(&rgb) = pixels.get(lz * TEXELS_PER_CHUNK + lx)
        {
            return [rgb[0] as f32, rgb[1] as f32, rgb[2] as f32];
        }
        [
            MINIMAP_UNEXPLORED_RGB[0] as f32,
            MINIMAP_UNEXPLORED_RGB[1] as f32,
            MINIMAP_UNEXPLORED_RGB[2] as f32,
        ]
    };

    let c00 = sample(x0, z0);
    let c10 = sample(x0 + 1, z0);
    let c01 = sample(x0, z0 + 1);
    let c11 = sample(x0 + 1, z0 + 1);

    let mut out = [0u8; 3];
    for ch in 0..3 {
        let top = c00[ch] + (c10[ch] - c00[ch]) * fx;
        let bot = c01[ch] + (c11[ch] - c01[ch]) * fx;
        let val = top + (bot - top) * fz;
        out[ch] = val.round().clamp(0.0, 255.0) as u8;
    }
    out
}

/// Fills `out_rgba` with a `size x size` RGBA8 mini-map patch covering `[min_world, max_world]`.
pub fn render_minimap_patch<'a>(
    explore: &MapExplore,
    min_world: Vec2,
    max_world: Vec2,
    size: usize,
    get_chunk: impl Fn(IVec2) -> Option<&'a [[u8; 3]]>,
    out_rgba: &mut Vec<u8>,
) {
    out_rgba.resize(size * size * 4, 255);
    let span = max_world - min_world;
    let inv = 1.0 / (size.max(1) as f32);

    for py in 0..size {
        let wz = min_world.y + (py as f32 + 0.5) * inv * span.y;
        for px in 0..size {
            let wx = min_world.x + (px as f32 + 0.5) * inv * span.x;
            let world_xz = Vec2::new(wx, wz);
            let cov = smooth_explored_coverage(explore, world_xz);

            let rgb = if cov <= 0.001 {
                MINIMAP_UNEXPLORED_RGB
            } else {
                let terrain = sample_chunk_terrain_bilinear(world_xz, &get_chunk);
                if cov >= 0.999 {
                    terrain
                } else {
                    [
                        lerp_u8(MINIMAP_UNEXPLORED_RGB[0], terrain[0], cov),
                        lerp_u8(MINIMAP_UNEXPLORED_RGB[1], terrain[1], cov),
                        lerp_u8(MINIMAP_UNEXPLORED_RGB[2], terrain[2], cov),
                    ]
                }
            };

            let idx = (py * size + px) * 4;
            out_rgba[idx] = rgb[0];
            out_rgba[idx + 1] = rgb[1];
            out_rgba[idx + 2] = rgb[2];
            out_rgba[idx + 3] = 255;
        }
    }
}

#[inline]
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t)
        .round()
        .clamp(0.0, 255.0) as u8
}
