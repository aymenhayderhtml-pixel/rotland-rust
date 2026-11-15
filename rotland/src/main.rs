//! Rotland (Rust). R0: the engine draws a lit test landscape (temporary until the `world` crate lands).

use engine::render::{Camera, ColorVertex, DrawItem, GpuMesh, Lighting, LitRenderer};
use engine::{EngineArgs, FrameTarget, Game, GameStats, Gpu, Input};
use glam::Vec3;

const CHUNK: f32 = 64.0;
const CELL: f32 = 2.0;

struct R0 {
    renderer: Option<LitRenderer>,
    chunks: Vec<(GpuMesh, Vec3)>,
    camera: Camera,
    time: f32,
}

fn height(x: f32, z: f32) -> f32 {
    (x * 0.02).sin() * 6.0 + (z * 0.03).cos() * 4.0 + (x * 0.11 + z * 0.07).sin() * 0.8
}

/// Flat-shaded test chunk: two triangles per 2 m cell, clockwise from above like the Godot terrain.
fn test_chunk(cx: i32, cz: i32) -> Vec<ColorVertex> {
    let (ox, oz) = (cx as f32 * CHUNK, cz as f32 * CHUNK);
    let n = (CHUNK / CELL) as i32;
    let mut out = Vec::with_capacity((n * n * 6) as usize);
    let p = |i: i32, j: i32| Vec3::new(i as f32 * CELL, height(ox + i as f32 * CELL, oz + j as f32 * CELL), j as f32 * CELL);
    for j in 0..n {
        for i in 0..n {
            let (a, b, c, d) = (p(i, j), p(i + 1, j), p(i, j + 1), p(i + 1, j + 1));
            for tri in [[a, b, c], [b, d, c]] {
                let normal = (tri[2] - tri[0]).cross(tri[1] - tri[0]).normalize();
                let shade = ((i * 7 + j * 13) % 5) as u8 * 3;
                let color = [0x5d - shade, 0x88 - shade, 0x36, 255];
                for v in tri {
                    out.push(ColorVertex { position: v.to_array(), normal: normal.to_array(), color });
                }
            }
        }
    }
    out
}

fn noon() -> Lighting {
    let hex = |h: u32| Vec3::new(((h >> 16) & 255) as f32, ((h >> 8) & 255) as f32, (h & 255) as f32) / 255.0;
    // DayNightPalette at 12:00: sun angle = pi * 6 / 13
    let angle = std::f32::consts::PI * 6.0 / 13.0;
    Lighting {
        sun_toward: Vec3::new(-angle.cos(), angle.sin(), 0.4).normalize(),
        sun_energy: 0.9,
        sun_color: Vec3::new(1.0, 0.98, 0.93),
        sky_top: hex(0x5b86c4),
        sky_horizon: hex(0xc9d6e0),
        ground_horizon: hex(0xc9d6e0),
        ground_bottom: hex(0xc9d6e0),
        ambient_energy: 0.6,
        fog_begin: 120.0,
        fog_end: 190.0,
        fog_density: 1.0,
        fog_color: hex(0xc9d6e0),
    }
}

impl Game for R0 {
    fn init(&mut self, gpu: &Gpu, _args: &EngineArgs) {
        self.renderer = Some(LitRenderer::new(gpu));
        for cz in -3..=3 {
            for cx in -3..=3 {
                let mesh = GpuMesh::new(&gpu.device, "test chunk", &test_chunk(cx, cz));
                self.chunks.push((mesh, Vec3::new(cx as f32 * CHUNK, 0.0, cz as f32 * CHUNK)));
            }
        }
        self.camera.position = Vec3::new(10.0, height(10.0, 10.0) + 8.0, 10.0);
        self.camera.look_at(Vec3::new(-60.0, 0.0, -80.0));
    }

    fn update(&mut self, input: &Input, dt: f32) {
        self.time += dt;
        // Free-fly for looking around by hand: mouse to turn, WASD to move
        self.camera.yaw -= input.mouse_delta.0 * 0.003;
        self.camera.pitch = (self.camera.pitch - input.mouse_delta.1 * 0.003).clamp(-1.5, 1.5);
        let f = self.camera.forward();
        let right = f.cross(Vec3::Y).normalize_or_zero();
        let mut mv = Vec3::ZERO;
        use winit::keyboard::KeyCode;
        if input.held(KeyCode::KeyW) { mv += f; }
        if input.held(KeyCode::KeyS) { mv -= f; }
        if input.held(KeyCode::KeyD) { mv += right; }
        if input.held(KeyCode::KeyA) { mv -= right; }
        self.camera.position += mv * 20.0 * dt;
    }

    fn render(&mut self, gpu: &Gpu, target: &FrameTarget, encoder: &mut wgpu::CommandEncoder) {
        let items: Vec<DrawItem> = self.chunks.iter().map(|(mesh, offset)| DrawItem { mesh, offset: *offset }).collect();
        if let Some(r) = self.renderer.as_mut() {
            r.draw(gpu, target, encoder, &self.camera, &noon(), &items);
        }
    }

    fn stats(&self) -> GameStats {
        let mut s = GameStats::new();
        if let Some(r) = &self.renderer {
            s.insert("drawCalls".into(), r.last_draws.into());
            s.insert("triangles".into(), r.last_triangles.into());
        }
        s
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info,wgpu_core=warn,wgpu_hal=warn")).init();
    let game = R0 { renderer: None, chunks: Vec::new(), camera: Camera::default(), time: 0.0 };
    engine::run(game, "Rotland (Rust)", EngineArgs::from_env());
}
