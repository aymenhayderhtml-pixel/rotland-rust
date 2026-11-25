//! Game-loop weather wrapper: simulation state, biome-aware ticking, fog/sky lighting overrides,
//! save snapshot conversion, and particle rendering.

use engine::render::weather::{WeatherParams, WeatherRenderer};
use engine::render::Camera;
use engine::{FrameTarget, Gpu};
use glam::{Vec2, Vec3};
use save::systems::world::WeatherSaveData;
use world::generator::WorldGenerator;
use world::weather::{WeatherData, WeatherKind, WeatherState};

use crate::light::{self, WeatherLight};

const MIN_DRAW_INTENSITY: f32 = 1e-3; // new: DECISIONS 2026-11-25

/// Maps a [`WeatherKind`] to the shader `kind_index` (`0` clear .. `4` fog).
pub fn weather_kind_index(kind: WeatherKind) -> u32 {
    match kind {
        WeatherKind::Clear => 0,
        WeatherKind::Rain => 1,
        WeatherKind::Snow => 2,
        WeatherKind::Sandstorm => 3,
        WeatherKind::Fog => 4,
    }
}

/// Active weather system for the running game session.
pub struct WeatherPlay {
    pub state: WeatherState,
    pub data: WeatherData,
    pub renderer: WeatherRenderer,
    forced: Option<WeatherKind>,
}

impl WeatherPlay {
    /// Creates a new weather controller backed by `data/weather.json` and a GPU particle renderer.
    pub fn new(gpu: &Gpu, seed: i32, data: WeatherData) -> Self {
        Self {
            state: WeatherState::new(seed),
            data,
            renderer: WeatherRenderer::new(gpu, engine::target::COLOR_FORMAT, engine::gpu::DEPTH_FORMAT),
            forced: None,
        }
    }

    /// Forces `kind` at full intensity (`1.0`) for the rest of the run (used by `--weather=<kind>`).
    pub fn force(&mut self, kind: WeatherKind) {
        self.forced = Some(kind);
        self.state.kind = kind;
        self.state.intensity = 1.0;
        let wind_speed = self.kind_wind(kind);
        self.state.wind = Vec2::new(wind_speed, 0.0);
    }

    /// Advances weather using the player's primary biome unless locked by `--weather`.
    pub fn tick(&mut self, dt_game: f32, player_pos: Vec3, generator: &WorldGenerator) {
        if self.forced.is_some() {
            return;
        }
        let biome = generator.biome(player_pos.x, player_pos.z);
        self.state.update(dt_game, biome, &self.data);
    }

    /// Builds GPU shader parameters for the current weather frame.
    pub fn params(&self, camera_pos: Vec3, time: f32) -> WeatherParams {
        WeatherParams {
            kind_index: weather_kind_index(self.state.kind),
            intensity: self.state.intensity,
            wind: [self.state.wind.x, self.state.wind.y],
            tint: self.kind_tint(self.state.kind),
            camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z],
            time,
        }
    }

    /// Computes fog distance scaling and biome/weather colour tints for [`crate::light::at_hour_weather`].
    pub fn light_overrides(&self, player_pos: Vec3, generator: &WorldGenerator) -> WeatherLight {
        let sample = generator.biome_sample(player_pos.x, player_pos.z);
        let biome_def = generator.biome_registry().get(sample.primary);
        let intensity = if self.state.kind == WeatherKind::Clear {
            0.0
        } else {
            self.state.intensity
        };
        WeatherLight {
            fog_mult: self.state.fog_mult(),
            weather_tint: Vec3::from(self.kind_tint(self.state.kind)),
            intensity,
            biome_fog_tint: light::rgb_u8_to_vec3(biome_def.fog_tint),
            biome_outer: sample.outer,
        }
    }

    /// Current sight visibility factor (`0..1`) for zombie senses.
    pub fn visibility(&self) -> f32 {
        self.state.visibility()
    }

    /// Current noise masking factor (`0..1`) for zombie senses.
    pub fn noise_mask(&self) -> f32 {
        self.state.noise_mask()
    }

    /// Snapshots the active weather for world save serialization.
    pub fn to_save(&self) -> WeatherSaveData {
        WeatherSaveData {
            kind: self.state.kind.key().to_string(),
            intensity: self.state.intensity,
            wind: [self.state.wind.x, self.state.wind.y],
        }
    }

    /// Restores weather from a saved world snapshot (`None` loads as clear weather).
    pub fn apply_save(&mut self, saved: Option<&WeatherSaveData>) {
        if self.forced.is_some() {
            return;
        }
        match saved {
            Some(w) => {
                self.state.kind = WeatherKind::from_key(&w.kind).unwrap_or(WeatherKind::Clear);
                self.state.intensity = w.intensity.clamp(0.0, 1.0);
                self.state.wind = Vec2::from(w.wind);
            }
            None => {
                self.state.kind = WeatherKind::Clear;
                self.state.intensity = 0.0;
                self.state.wind = Vec2::ZERO;
            }
        }
    }

    /// Draws weather particles after the water pass when active weather is visible.
    pub fn draw(
        &mut self,
        gpu: &Gpu,
        target: &FrameTarget,
        encoder: &mut wgpu::CommandEncoder,
        camera: &Camera,
        time: f32,
    ) {
        if self.state.kind == WeatherKind::Clear || self.state.intensity <= MIN_DRAW_INTENSITY {
            return;
        }
        let params = self.params(camera.position, time);
        let view_proj = camera.view_proj(target.aspect());
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("weather"),
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

    fn kind_tint(&self, kind: WeatherKind) -> [f32; 3] {
        self.data.kind(kind).tint
    }

    fn kind_wind(&self, kind: WeatherKind) -> f32 {
        self.data.kind(kind).wind
    }
}
