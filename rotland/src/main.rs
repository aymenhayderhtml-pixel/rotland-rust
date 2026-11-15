//! Rotland (Rust). R0: the real world (seed 42) with 25 animated zombies, for the go/no-go speed test.
//! Options: --seed=N (42), --hour=H (12), plus the engine's --shot / --stats / --seconds (see rust/README.md).

mod crowd;
mod light;
mod terrain;

use engine::render::{Camera, DrawItem, LitRenderer};
use engine::{EngineArgs, FrameTarget, Game, GameStats, Gpu, Input};
use glam::Vec3;
use world::{SettlementConfig, WorldGenerator};

const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

struct R0 {
    renderer: Option<LitRenderer>,
    ground: Vec<terrain::GroundChunk>,
    crowd: crowd::Crowd,
    camera: Camera,
    hour: f32,
    time: f32,
    seed: i32,
}

impl Game for R0 {
    fn init(&mut self, gpu: &Gpu, args: &EngineArgs) {
        self.seed = args.value("--seed").and_then(|s| s.parse().ok()).unwrap_or(42);
        self.hour = args.value("--hour").and_then(|s| s.parse().ok()).unwrap_or(12.0);
        self.renderer = Some(LitRenderer::new(gpu));

        let config = SettlementConfig::load_from_file(format!("{REPO}/data/settlements.json")).expect("data/settlements.json");
        let started = std::time::Instant::now();
        let world_gen = WorldGenerator::new(self.seed, config.clone());
        let camp = world_gen.settlements()[0].clone();
        let eye = camp.respawn_point(&config);
        // Same ring as Godot's medium preset: 3 chunks each way around the player
        self.ground = terrain::build_ring(&gpu.device, &world_gen, terrain::chunk_of(eye), 3);
        log::info!("built {} ground chunks in {:.0} ms", self.ground.len(), started.elapsed().as_secs_f64() * 1000.0);

        self.camera.position = eye + Vec3::new(0.0, 1.6, 0.0);
        self.camera.look_at(camp.center + Vec3::new(0.0, 1.2, 0.0));
        let ahead = self.camera.position + self.camera.forward() * 14.0;
        let pack_path = format!("{REPO}/rust/assets/models.pack");
        self.crowd = crowd::Crowd::load(&gpu.device, &pack_path, ahead, 25, |x, z| world_gen.height(x, z));
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
        for (key, dir) in [(KeyCode::KeyW, f), (KeyCode::KeyS, -f), (KeyCode::KeyD, right), (KeyCode::KeyA, -right)] {
            if input.held(key) {
                mv += dir;
            }
        }
        self.camera.position += mv * 20.0 * dt;
    }

    fn render(&mut self, gpu: &Gpu, target: &FrameTarget, encoder: &mut wgpu::CommandEncoder) {
        let items: Vec<DrawItem> = self.ground.iter().map(|c| DrawItem { mesh: &c.mesh, offset: c.origin }).collect();
        let (batches, bones) = self.crowd.pose(self.time);
        if let Some(r) = self.renderer.as_mut() {
            r.draw(gpu, target, encoder, &self.camera, &light::at_hour(self.hour), &items, &batches, bones);
        }
    }

    fn stats(&self) -> GameStats {
        let mut s = GameStats::new();
        if let Some(r) = &self.renderer {
            s.insert("drawCalls".into(), r.last_draws.into());
            s.insert("triangles".into(), r.last_triangles.into());
        }
        s.insert("seed".into(), self.seed.into());
        s.insert("groundChunks".into(), self.ground.len().into());
        s.insert("zombies".into(), self.crowd.count().into());
        s
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info,wgpu_core=warn,wgpu_hal=warn")).init();
    let game = R0 { renderer: None, ground: Vec::new(), crowd: crowd::Crowd::default(), camera: Camera::default(), hour: 12.0, time: 0.0, seed: 42 };
    engine::run(game, "Rotland (Rust)", EngineArgs::from_env());
}
