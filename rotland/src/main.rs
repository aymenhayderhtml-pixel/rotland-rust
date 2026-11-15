//! Rotland (Rust). R0: the real world with real trees and 25 zombies, for the go/no-go speed test.
//! Options: --scene=forest|camp (forest = Godot's forest25 benchmark walk), --seed=N (42), --hour=H (12),
//! --free (fly with mouse + WASD instead of the scripted walk), plus the engine's --shot / --stats / --seconds.

mod bench;
mod crowd;
mod light;
mod props;
mod terrain;

use engine::render::{Camera, DrawItem, LitRenderer};
use engine::{EngineArgs, FrameTarget, Game, GameStats, Gpu, Input};
use glam::{Mat4, Vec3};
use world::WorldGenerator;

const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
/// Medium preset in Godot: 3 chunks each way around the player.
const RING: i32 = 3;

#[derive(Default)]
struct R0 {
    renderer: Option<LitRenderer>,
    world_gen: Option<WorldGenerator>,
    ground: Vec<terrain::GroundChunk>,
    props: Option<props::Props>,
    crowd: crowd::Crowd,
    camera: Camera,
    player: Vec3,
    walk: bool,
    walk_time: f32,
    hour: f32,
    time: f32,
    seed: i32,
    scene: String,
    bones: Vec<Mat4>,
}

impl R0 {
    fn height(&self, x: f32, z: f32) -> f32 {
        self.world_gen.as_ref().map_or(0.0, |w| w.height(x, z))
    }

    /// Third-person camera like Godot's CameraRig: 4 m spring arm, 0.55 m right shoulder, looking slightly down.
    fn follow(&mut self, yaw: f32) {
        self.camera.yaw = yaw;
        self.camera.pitch = -0.25;
        let f = self.camera.forward();
        let right = f.cross(Vec3::Y).normalize_or_zero();
        let pivot = self.player + Vec3::new(0.0, 1.6, 0.0) + right * 0.55 + Vec3::Y * 0.2;
        self.camera.position = pivot - f * 4.0;
    }
}

impl Game for R0 {
    fn init(&mut self, gpu: &Gpu, args: &EngineArgs) {
        self.seed = args.value("--seed").and_then(|s| s.parse().ok()).unwrap_or(42);
        self.hour = args.value("--hour").and_then(|s| s.parse().ok()).unwrap_or(12.0);
        self.scene = args.value("--scene").unwrap_or("forest").to_string();
        self.walk = self.scene == "forest" && !args.flag("--free");
        self.renderer = Some(LitRenderer::new(gpu));

        let started = std::time::Instant::now();
        let world_gen = WorldGenerator::from_data_dir(self.seed, format!("{REPO}/data")).expect("data files");
        let config = world_gen.config().clone();
        let camp = world_gen.settlements()[0].clone();
        let spawn = camp.respawn_point(&config);
        self.player = if self.scene == "forest" {
            // The same spot as Godot's benchmark: centre of the densest forest chunk near spawn
            let c = world_gen.densest_forest_near(spawn);
            let (x, z) = ((c.x as f32 + 0.5) * WorldGenerator::CHUNK_SIZE, (c.y as f32 + 0.5) * WorldGenerator::CHUNK_SIZE);
            Vec3::new(x, world_gen.height(x, z), z)
        } else {
            spawn
        };
        let center = terrain::chunk_of(self.player);
        self.ground = terrain::build_ring(&gpu.device, &world_gen, center, RING);
        let coords: Vec<_> = self.ground.iter().map(|g| g.coord).collect();
        match pack::ModelPack::load(format!("{REPO}/rust/assets/models.pack")) {
            Ok(pack) => {
                self.props = Some(props::Props::build(&gpu.device, &pack, &world_gen, &coords));
                let h = |x: f32, z: f32| world_gen.height(x, z);
                self.crowd = crowd::Crowd::spawn(&gpu.device, &pack, self.player, 25, 15.0, 40.0, &h);
            }
            Err(e) => log::warn!("no model pack ({e}); run `cargo run --release -p baker`"),
        }
        log::info!(
            "scene {} at {:?}: {} chunks, {} props, {} zombies, ready in {:.0} ms",
            self.scene,
            self.player,
            self.ground.len(),
            self.props.as_ref().map_or(0, |p| p.count()),
            self.crowd.count(),
            started.elapsed().as_secs_f64() * 1000.0
        );
        self.world_gen = Some(world_gen);
        if self.walk {
            self.follow(0.0);
        } else {
            self.camera.position = self.player + Vec3::new(0.0, 1.6, 0.0);
            self.camera.look_at(camp.center + Vec3::new(0.0, 1.2, 0.0));
        }
    }

    fn update(&mut self, input: &Input, dt: f32) {
        self.time += dt;
        if self.walk {
            self.walk_time += dt;
            let yaw = bench::yaw_at(self.walk_time);
            let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
            self.player += forward * bench::WALK_SPEED * dt;
            self.player.y = self.height(self.player.x, self.player.z);
            self.follow(yaw);
        } else {
            self.camera.yaw -= input.mouse_delta.0 * 0.003;
            self.camera.pitch = (self.camera.pitch - input.mouse_delta.1 * 0.003).clamp(-1.5, 1.5);
            let f = self.camera.forward();
            let right = f.cross(Vec3::Y).normalize_or_zero();
            use winit::keyboard::KeyCode;
            for (key, dir) in [(KeyCode::KeyW, f), (KeyCode::KeyS, -f), (KeyCode::KeyD, right), (KeyCode::KeyA, -right)] {
                if input.held(key) {
                    self.camera.position += dir * 20.0 * dt;
                }
            }
            self.player = self.camera.position;
        }
        if let Some(w) = self.world_gen.as_ref() {
            self.crowd.update(dt, self.player, &|x, z| w.height(x, z));
        }
    }

    fn render(&mut self, gpu: &Gpu, target: &FrameTarget, encoder: &mut wgpu::CommandEncoder) {
        let Some(renderer) = self.renderer.as_mut() else { return };
        let items: Vec<DrawItem> = self.ground.iter().map(|c| DrawItem { mesh: &c.mesh, offset: c.origin }).collect();
        // Joint 0 is the identity: static instanced props use it
        self.bones.clear();
        self.bones.push(Mat4::IDENTITY);
        let (mut main, mut shadow) = match &self.props {
            Some(p) => p.batches(&self.camera, target.aspect(), renderer.shadows.distance),
            None => (Vec::new(), Vec::new()),
        };
        let zombies = self.crowd.pose(self.time, &mut self.bones);
        shadow.extend(zombies.iter().map(|b| engine::render::SkinnedBatch { mesh: b.mesh, instances: b.instances.clone() }));
        main.extend(zombies);
        renderer.draw(gpu, target, encoder, &self.camera, &light::at_hour(self.hour), &items, &main, &shadow, &self.bones);
    }

    fn stats(&self) -> GameStats {
        let mut s = GameStats::new();
        if let Some(r) = &self.renderer {
            s.insert("drawCalls".into(), r.last.draws.into());
            s.insert("triangles".into(), r.last.triangles.into());
            s.insert("shadowDrawCalls".into(), r.last.shadow_draws.into());
            s.insert("shadowTriangles".into(), r.last.shadow_triangles.into());
        }
        s.insert("scene".into(), self.scene.clone().into());
        s.insert("seed".into(), self.seed.into());
        s.insert("groundChunks".into(), self.ground.len().into());
        s.insert("props".into(), self.props.as_ref().map_or(0, |p| p.count()).into());
        s.insert("zombies".into(), self.crowd.count().into());
        s
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info,wgpu_core=warn,wgpu_hal=warn")).init();
    engine::run(R0 { hour: 12.0, seed: 42, ..Default::default() }, "Rotland (Rust)", EngineArgs::from_env());
}
