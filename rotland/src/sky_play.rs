//! Drifting low-poly clouds and per-biome fog tinting driven by `data/sky.json` (R35-04).

use std::collections::HashMap;
use std::path::Path;

use engine::render::sky::{SkyParams, SkyRenderer};
use engine::render::{Camera, Lighting};
use engine::{FrameTarget, Gpu};
use glam::{Vec2, Vec3};
use serde::Deserialize;
use world::biome::BiomeId;
use world::generator::WorldGenerator;
use world::weather::{WeatherKind, WeatherState};

const UV_WRAP: f32 = 1000.0; // new: DECISIONS 2026-11-25
const MIN_DAY_BRIGHTNESS: f32 = 0.08; // new: DECISIONS 2026-11-25
const SUN_BRIGHTNESS_WEIGHT: f32 = 0.7; // new: DECISIONS 2026-11-25
const AMBIENT_BRIGHTNESS_WEIGHT: f32 = 0.8; // new: DECISIONS 2026-11-25
const MOON_BRIGHTNESS_WEIGHT: f32 = 0.25; // new: DECISIONS 2026-11-25
const OVERCAST_SHADOW_SCALE: f32 = 0.85; // new: DECISIONS 2026-11-25
const LUMA_R: f32 = 0.2126; // new: DECISIONS 2026-11-25
const LUMA_G: f32 = 0.7152; // new: DECISIONS 2026-11-25
const LUMA_B: f32 = 0.0722; // new: DECISIONS 2026-11-25
const MIN_TINT_LUMA: f32 = 1e-4; // new: DECISIONS 2026-11-25

/// Cloud cover, greyness and speed multiplier for one weather kind in `data/sky.json`.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct SkyWeatherDef {
    pub cover: f32,
    pub grey: f32,
    pub speed_mult: f32,
}

#[derive(Debug, Deserialize)]
struct SkyFile {
    cloud_scale: f32,
    cloud_softness: f32,
    cloud_alpha: f32,
    wind_uv_per_mps: f32,
    sunlit_color: [f32; 3],
    shadow_color: [f32; 3],
    overcast_tint: [f32; 3],
    fog_tint_blend: f32,
    weather: HashMap<String, SkyWeatherDef>,
    biome_fog_tint: HashMap<String, [u8; 3]>,
}

/// Validated sky, cloud, and per-biome fog configuration loaded from `data/sky.json`.
#[derive(Debug, Clone, PartialEq)]
pub struct SkyData {
    pub cloud_scale: f32,
    pub cloud_softness: f32,
    pub cloud_alpha: f32,
    pub wind_uv_per_mps: f32,
    pub sunlit_color: [f32; 3],
    pub shadow_color: [f32; 3],
    pub overcast_tint: [f32; 3],
    pub fog_tint_blend: f32,
    weather: [SkyWeatherDef; WeatherKind::COUNT],
    biome_fog_tint: [Vec3; BiomeId::COUNT],
}

impl SkyData {
    /// Loads `data/sky.json` via `paths::repo_path`.
    pub fn load() -> Result<Self, String> {
        let path = paths::repo_path("data/sky.json");
        Self::load_from_file(path)
    }

    /// Loads sky configuration from an explicit path.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&json)
    }

    /// Parses and validates sky configuration from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let parsed: SkyFile =
            serde_json::from_str(json).map_err(|e| format!("failed to parse sky JSON: {e}"))?;

        if !parsed.cloud_scale.is_finite() || parsed.cloud_scale <= 0.0 {
            return Err(format!("invalid cloud_scale in sky.json: {}", parsed.cloud_scale));
        }
        if !parsed.cloud_softness.is_finite() || parsed.cloud_softness <= 0.0 {
            return Err(format!("invalid cloud_softness in sky.json: {}", parsed.cloud_softness));
        }
        if !parsed.cloud_alpha.is_finite() || !(0.0..=1.0).contains(&parsed.cloud_alpha) {
            return Err(format!("invalid cloud_alpha in sky.json: {}", parsed.cloud_alpha));
        }
        if !parsed.wind_uv_per_mps.is_finite() || parsed.wind_uv_per_mps < 0.0 {
            return Err(format!(
                "invalid wind_uv_per_mps in sky.json: {}",
                parsed.wind_uv_per_mps
            ));
        }
        if !parsed.fog_tint_blend.is_finite() || !(0.0..=1.0).contains(&parsed.fog_tint_blend) {
            return Err(format!(
                "invalid fog_tint_blend in sky.json: {}",
                parsed.fog_tint_blend
            ));
        }
        validate_rgb("sunlit_color", parsed.sunlit_color)?;
        validate_rgb("shadow_color", parsed.shadow_color)?;
        validate_rgb("overcast_tint", parsed.overcast_tint)?;

        let weather = parse_weather(parsed.weather)?;
        let biome_fog_tint = parse_biome_tints(parsed.biome_fog_tint)?;

        Ok(Self {
            cloud_scale: parsed.cloud_scale,
            cloud_softness: parsed.cloud_softness,
            cloud_alpha: parsed.cloud_alpha,
            wind_uv_per_mps: parsed.wind_uv_per_mps,
            sunlit_color: parsed.sunlit_color,
            shadow_color: parsed.shadow_color,
            overcast_tint: parsed.overcast_tint,
            fog_tint_blend: parsed.fog_tint_blend,
            weather,
            biome_fog_tint,
        })
    }

    /// Cloud parameters for `kind`.
    pub fn weather_def(&self, kind: WeatherKind) -> SkyWeatherDef {
        self.weather[kind.index()]
    }

    /// Normalized `0..=1` fog tint for `biome`.
    pub fn biome_fog_tint(&self, biome: BiomeId) -> Vec3 {
        self.biome_fog_tint[biome.index()]
    }
}

fn validate_rgb(name: &str, rgb: [f32; 3]) -> Result<(), String> {
    for (idx, ch) in rgb.iter().enumerate() {
        if !ch.is_finite() || !(0.0..=1.0).contains(ch) {
            return Err(format!("sky.json {name}[{idx}] out of 0..=1: {ch}"));
        }
    }
    Ok(())
}

fn parse_weather(raw: HashMap<String, SkyWeatherDef>) -> Result<[SkyWeatherDef; WeatherKind::COUNT], String> {
    let mut slots: [Option<SkyWeatherDef>; WeatherKind::COUNT] = std::array::from_fn(|_| None);
    for (key, def) in raw {
        let Some(kind) = WeatherKind::from_key(&key) else {
            return Err(format!("unknown weather key in sky.json: '{key}'"));
        };
        if !def.cover.is_finite() || !(0.0..=1.0).contains(&def.cover) {
            return Err(format!("sky.json weather.{key}.cover out of 0..=1: {}", def.cover));
        }
        if !def.grey.is_finite() || !(0.0..=1.0).contains(&def.grey) {
            return Err(format!("sky.json weather.{key}.grey out of 0..=1: {}", def.grey));
        }
        if !def.speed_mult.is_finite() || def.speed_mult < 0.0 {
            return Err(format!(
                "sky.json weather.{key}.speed_mult invalid: {}",
                def.speed_mult
            ));
        }
        slots[kind.index()] = Some(def);
    }
    let missing: Vec<&'static str> = WeatherKind::ALL
        .iter()
        .filter(|k| slots[k.index()].is_none())
        .map(|k| k.key())
        .collect();
    if !missing.is_empty() {
        return Err(format!("missing weather key(s) in sky.json: {}", missing.join(", ")));
    }
    let [Some(clear), Some(rain), Some(snow), Some(sandstorm), Some(fog)] = slots else {
        return Err("internal error assembling sky weather table".to_string());
    };
    Ok([clear, rain, snow, sandstorm, fog])
}

fn parse_biome_tints(raw: HashMap<String, [u8; 3]>) -> Result<[Vec3; BiomeId::COUNT], String> {
    let mut slots: [Option<Vec3>; BiomeId::COUNT] = std::array::from_fn(|_| None);
    for (key, rgb) in raw {
        let Some(id) = BiomeId::from_key(&key) else {
            return Err(format!("unknown biome key in sky.json: '{key}'"));
        };
        slots[id.index()] = Some(Vec3::new(
            f32::from(rgb[0]) / 255.0,
            f32::from(rgb[1]) / 255.0,
            f32::from(rgb[2]) / 255.0,
        ));
    }
    let missing: Vec<&'static str> = BiomeId::ALL
        .iter()
        .filter(|b| slots[b.index()].is_none())
        .map(|b| b.key())
        .collect();
    if !missing.is_empty() {
        return Err(format!("missing biome key(s) in sky.json: {}", missing.join(", ")));
    }
    let [Some(plains), Some(forest), Some(desert), Some(snow), Some(swamp), Some(mountains)] = slots else {
        return Err("internal error assembling sky biome fog table".to_string());
    };
    Ok([plains, forest, desert, snow, swamp, mountains])
}

/// Active cloud simulation and per-biome fog tint controller.
pub struct SkyPlay {
    pub data: SkyData,
    pub renderer: SkyRenderer,
    wind_offset: Vec2,
}

impl SkyPlay {
    /// Creates a new sky controller backed by `data/sky.json` and a GPU cloud renderer.
    pub fn new(gpu: &Gpu, data: SkyData) -> Self {
        Self {
            data,
            renderer: SkyRenderer::new(gpu, engine::target::COLOR_FORMAT, engine::gpu::DEPTH_FORMAT),
            wind_offset: Vec2::ZERO,
        }
    }

    /// Advances cloud wind drift by `dt` seconds.
    pub fn tick(&mut self, dt: f32, weather: Option<&WeatherState>) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let (wind, speed_mult) = match weather {
            Some(w) => {
                let clear_mult = self.data.weather_def(WeatherKind::Clear).speed_mult;
                let active_mult = self.data.weather_def(w.kind).speed_mult;
                let mult = clear_mult + (active_mult - clear_mult) * w.intensity.clamp(0.0, 1.0);
                (w.wind, mult)
            }
            None => (Vec2::X, self.data.weather_def(WeatherKind::Clear).speed_mult),
        };
        self.wind_offset = (self.wind_offset + wind * (dt * self.data.wind_uv_per_mps * speed_mult))
            .rem_euclid(Vec2::splat(UV_WRAP));
    }

    /// Blends `lighting.fog_color` toward the player's current biome tint from `data/sky.json`.
    /// Leaves the home land core (`sample.outer <= 0.0`) completely unchanged.
    pub fn apply_biome_fog(&self, lighting: &mut Lighting, player_pos: Vec3, generator: &WorldGenerator) {
        let sample = generator.biome_sample(player_pos.x, player_pos.z);
        if sample.outer <= 0.0 {
            return;
        }
        let t_primary = self.data.biome_fog_tint(sample.primary);
        let t_secondary = self.data.biome_fog_tint(sample.secondary);
        let biome_tint = t_primary.lerp(t_secondary, sample.blend.clamp(0.0, 1.0));
        let capped = darken_tint_to_base(lighting.fog_color, biome_tint);
        let weight = (sample.outer.clamp(0.0, 1.0) * self.data.fog_tint_blend).clamp(0.0, 1.0);
        lighting.fog_color = lighting.fog_color.lerp(capped, weight);
    }

    /// Computes per-frame [`SkyParams`] from the current weather, camera, and day/night lighting.
    pub fn params(&self, camera: &Camera, lighting: &Lighting, weather: Option<&WeatherState>) -> SkyParams {
        let clear = self.data.weather_def(WeatherKind::Clear);
        let (cover, grey) = match weather {
            Some(w) if w.kind != WeatherKind::Clear => {
                let active = self.data.weather_def(w.kind);
                let t = w.intensity.clamp(0.0, 1.0);
                (
                    clear.cover + (active.cover - clear.cover) * t,
                    clear.grey + (active.grey - clear.grey) * t,
                )
            }
            _ => (clear.cover, clear.grey),
        };

        let day_brightness = (lighting.sun_energy * SUN_BRIGHTNESS_WEIGHT
            + lighting.ambient_energy * AMBIENT_BRIGHTNESS_WEIGHT
            + lighting.moon_energy * MOON_BRIGHTNESS_WEIGHT)
            .clamp(MIN_DAY_BRIGHTNESS, 1.0);
        let overcast = Vec3::from(self.data.overcast_tint);
        let sunlit = Vec3::from(self.data.sunlit_color).lerp(overcast, grey) * day_brightness;
        let shadow =
            Vec3::from(self.data.shadow_color).lerp(overcast * OVERCAST_SHADOW_SCALE, grey)
                * day_brightness;

        SkyParams {
            cover,
            softness: self.data.cloud_softness,
            scale: self.data.cloud_scale,
            max_alpha: self.data.cloud_alpha,
            wind_offset: [self.wind_offset.x, self.wind_offset.y],
            sunlit_color: sunlit.to_array(),
            shadow_color: shadow.to_array(),
            sun_dir: lighting.sun_toward.normalize_or_zero().to_array(),
            camera_pos: camera.position.to_array(),
        }
    }

    /// Draws the clouds over uncovered sky pixels after `LitRenderer::draw`.
    pub fn draw(
        &self,
        gpu: &Gpu,
        target: &FrameTarget,
        encoder: &mut wgpu::CommandEncoder,
        camera: &Camera,
        lighting: &Lighting,
        weather: Option<&WeatherState>,
    ) {
        let params = self.params(camera, lighting, weather);
        let view_proj = camera.view_proj(target.aspect());
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("sky_clouds"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.color,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &target.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        self.renderer.draw(&mut pass, &gpu.queue, view_proj, &params);
    }
}

fn luminance(c: Vec3) -> f32 {
    c.x * LUMA_R + c.y * LUMA_G + c.z * LUMA_B
}

fn darken_tint_to_base(base: Vec3, tint: Vec3) -> Vec3 {
    let base_luma = luminance(base).max(0.0);
    let tint_luma = luminance(tint);
    if tint_luma <= MIN_TINT_LUMA {
        return Vec3::ZERO;
    }
    let scale = (base_luma / tint_luma).min(1.0);
    tint * scale
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_sky_json() -> &'static str {
        include_str!("../../data/sky.json")
    }

    #[test]
    fn sky_json_parses_and_validates() {
        let data = SkyData::from_json(test_sky_json()).expect("valid data/sky.json");
        assert!(data.weather_def(WeatherKind::Rain).cover > data.weather_def(WeatherKind::Clear).cover);
        assert!(data.weather_def(WeatherKind::Rain).grey > data.weather_def(WeatherKind::Clear).grey);
    }

    #[test]
    fn biome_fog_tint_never_brightens_dark_fog() {
        let data = SkyData::from_json(test_sky_json()).expect("valid data/sky.json");
        let dark_night = Vec3::new(0.02, 0.02, 0.04);
        let snow_tint = data.biome_fog_tint(BiomeId::Snow);
        let capped = darken_tint_to_base(dark_night, snow_tint);
        assert!(luminance(capped) <= luminance(dark_night) + 1e-5);
    }
}
