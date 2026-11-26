//! New Halden's grey-green haze: pure fog maths (no GPU). The haze colour is capped to the base
//! fog colour's luminance, so night fog never brightens.

use glam::{Vec2, Vec3};
use world::metro::MetroSite;

/// Luma weights (Rec. 709), same as the sky and light code.
const LUMA_R: f32 = 0.2126; // moved: rotland/src/light.rs (luma weights)
const LUMA_G: f32 = 0.7152; // moved: rotland/src/light.rs (luma weights)
const LUMA_B: f32 = 0.0722; // moved: rotland/src/light.rs (luma weights)
/// Tints darker than this are treated as black (same as light.rs).
const MIN_TINT_LUMA: f32 = 1e-4; // moved: rotland/src/light.rs (tint floor)
/// 8-bit colour scale used by `data/sky.json` colours.
const COLOR_MAX: f32 = 255.0; // new: DECISIONS 2026-11-26 (8-bit colour scale)

/// Haze values from `data/sky.json` `metro_haze`.
#[derive(Debug, Clone, PartialEq)]
pub struct MetroHazeConfig {
    /// Haze colour, 0..1.
    pub color: Vec3,
    /// Fog begin distance (m) at full haze.
    pub fog_start: f32,
    /// Fog end distance (m) at full haze.
    pub fog_end: f32,
    /// Distance outside the city square over which the haze fades out (m).
    pub blend_m: f32,
    /// Far skyline fog begin (m).
    pub skyline_fog_start: f32,
    /// Far skyline fog end (m).
    pub skyline_fog_end: f32,
}

impl Default for MetroHazeConfig {
    fn default() -> Self {
        Self {
            color: rgb8([150, 168, 150]), // data: sky.json metro_haze color
            fog_start: 70.0,              // data: sky.json metro_haze fog_start
            fog_end: 190.0,               // data: sky.json metro_haze fog_end
            blend_m: 250.0,               // data: sky.json metro_haze blend_m
            skyline_fog_start: 3000.0,    // data: sky.json metro_haze skyline_fog_start
            skyline_fog_end: 6000.0,      // data: sky.json metro_haze skyline_fog_end
        }
    }
}

fn rgb8(rgb: [u8; 3]) -> Vec3 {
    Vec3::new(f32::from(rgb[0]), f32::from(rgb[1]), f32::from(rgb[2])) / COLOR_MAX
}

impl MetroHazeConfig {
    /// Reads `metro_haze` from a `data/sky.json` text. Missing or bad keys keep their defaults.
    pub fn load(json: &str) -> Self {
        let mut cfg = Self::default();
        let Ok(root) = serde_json::from_str::<serde_json::Value>(json) else {
            return cfg;
        };
        let Some(h) = root.get("metro_haze") else {
            return cfg;
        };
        if let Some(c) = read_color(h, "color") {
            cfg.color = c;
        }
        cfg.fog_start = read_f32(h, "fog_start").unwrap_or(cfg.fog_start);
        cfg.fog_end = read_f32(h, "fog_end").unwrap_or(cfg.fog_end);
        cfg.blend_m = read_f32(h, "blend_m").unwrap_or(cfg.blend_m);
        cfg.skyline_fog_start = read_f32(h, "skyline_fog_start").unwrap_or(cfg.skyline_fog_start);
        cfg.skyline_fog_end = read_f32(h, "skyline_fog_end").unwrap_or(cfg.skyline_fog_end);
        cfg
    }

    /// Reads `data/sky.json` from the repo; a missing file gives the defaults.
    pub fn load_default() -> Self {
        match std::fs::read_to_string(paths::repo_path("data/sky.json")) {
            Ok(text) => Self::load(&text),
            Err(_) => Self::default(),
        }
    }
}

fn read_f32(obj: &serde_json::Value, key: &str) -> Option<f32> {
    obj.get(key)
        .and_then(serde_json::Value::as_f64)
        .map(|v| v as f32)
}

fn read_color(obj: &serde_json::Value, key: &str) -> Option<Vec3> {
    let arr = obj.get(key)?.as_array()?;
    if arr.len() != 3 {
        return None;
    }
    let mut rgb = [0u8; 3];
    for (slot, v) in rgb.iter_mut().zip(arr) {
        *slot = u8::try_from(v.as_u64()?).ok()?;
    }
    Some(rgb8(rgb))
}

/// Signed distance from world point `p` to the city square: negative inside.
pub fn city_signed_distance(p: Vec2, site: &MetroSite) -> f32 {
    site.signed_distance(p)
}

/// Haze strength: 1.0 inside or on the city edge, smooth fall-off to 0.0 at `blend_m` outside.
pub fn haze_weight(signed_dist: f32, blend_m: f32) -> f32 {
    if signed_dist <= 0.0 {
        return 1.0;
    }
    if blend_m <= 0.0 {
        return 0.0;
    }
    1.0 - world::smooth_step(0.0, blend_m, signed_dist)
}

/// Blends the base fog toward the haze by `weight`: colour, begin and end all lerp.
pub fn blend_fog(
    base_color: Vec3,
    base_begin: f32,
    base_end: f32,
    cfg: &MetroHazeConfig,
    weight: f32,
) -> (Vec3, f32, f32) {
    let w = weight.clamp(0.0, 1.0);
    let target = capped_haze(base_color, cfg.color);
    (
        base_color.lerp(target, w),
        base_begin + (cfg.fog_start - base_begin) * w,
        base_end + (cfg.fog_end - base_end) * w,
    )
}

/// Fog for the far skyline: capped haze colour with the skyline begin and end distances.
pub fn skyline_fog(cfg: &MetroHazeConfig, base_color: Vec3) -> (Vec3, f32, f32) {
    (
        capped_haze(base_color, cfg.color),
        cfg.skyline_fog_start,
        cfg.skyline_fog_end,
    )
}

/// The haze colour scaled down so it is never brighter than `base` (keeps hue).
fn capped_haze(base: Vec3, tint: Vec3) -> Vec3 {
    let base_luma = luma(base).max(0.0);
    let tint_luma = luma(tint);
    if tint_luma <= MIN_TINT_LUMA {
        return Vec3::ZERO;
    }
    tint * (base_luma / tint_luma).min(1.0)
}

fn luma(c: Vec3) -> f32 {
    c.x * LUMA_R + c.y * LUMA_G + c.z * LUMA_B
}
