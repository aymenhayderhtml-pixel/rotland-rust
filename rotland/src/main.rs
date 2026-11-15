//! Rotland (Rust). R0: the real world with real trees and 25 zombies, for the go/no-go speed test.
//! Options: --scene=forest|camp (forest = Godot's forest25 benchmark walk), --seed=N (42), --hour=H (12),
//! --free (fly with mouse + WASD instead of the scripted walk), plus the engine's --shot / --stats / --seconds.

mod bench;
mod crowd;
mod light;
mod props;
mod streaming;
mod terrain;

use std::sync::Arc;

use engine::render::{Camera, DrawItem, LitRenderer};
use engine::{EngineArgs, FrameTarget, Game, GameStats, Gpu, Input};
use game::{Actions, CameraRig, Player};
use glam::{Mat4, Vec3};
use world::WorldGenerator;

const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// Visual hook for the player body (to be rendered by another worker).
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct PlayerPose {
    pub position: Vec3,
    pub yaw: f32,
    pub horizontal_speed: f32,
    pub on_floor: bool,
    pub crouching: bool,
    pub first_person: bool,
}

#[derive(Default)]
struct R0 {
    renderer: Option<LitRenderer>,
    world_gen: Option<Arc<WorldGenerator>>,
    streamer: Option<streaming::Streamer>,
    library: props::PropLibrary,
    pack: Option<pack::ModelPack>,
    physics: physics::PhysicsWorld,
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
    player_ctrl: Option<Player>,
    camera_rig: Option<CameraRig>,
    mover: physics::CharacterMover,
    cursor_captured: bool,
    autowalk: bool,
    free_cam: bool,
}

impl R0 {
    fn height(&self, x: f32, z: f32) -> f32 {
        self.world_gen.as_ref().map_or(0.0, |w| w.height(x, z))
    }

    /// Pose hook exposing player state for visual character models and animations.
    #[allow(dead_code)]
    pub fn player_pose(&self) -> Option<PlayerPose> {
        let p = self.player_ctrl.as_ref()?;
        let rig = self.camera_rig.as_ref()?;
        Some(PlayerPose {
            position: p.position,
            yaw: p.yaw,
            horizontal_speed: p.horizontal_speed(),
            on_floor: p.on_floor,
            crouching: p.crouching,
            first_person: rig.is_first_person,
        })
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
        self.seed = args
            .value("--seed")
            .and_then(|s| s.parse().ok())
            .unwrap_or(42);
        self.hour = args
            .value("--hour")
            .and_then(|s| s.parse().ok())
            .unwrap_or(12.0);
        self.scene = args.value("--scene").unwrap_or("play").to_string();
        self.free_cam = args.flag("--free");
        self.autowalk = args.flag("--autowalk");
        self.walk = self.scene == "forest" && !self.free_cam;
        self.cursor_captured = self.scene == "play" && !self.free_cam;
        self.renderer = Some(LitRenderer::new(gpu));

        let started = std::time::Instant::now();
        let world_gen =
            WorldGenerator::from_data_dir(self.seed, format!("{REPO}/data")).expect("data files");
        let config = world_gen.config().clone();
        let camp = world_gen.settlements()[0].clone();
        let spawn = camp.respawn_point(&config);
        self.player = if self.scene == "forest" {
            // The same spot as Godot's benchmark: centre of the densest forest chunk near spawn
            let c = world_gen.densest_forest_near(spawn);
            let (x, z) = (
                (c.x as f32 + 0.5) * WorldGenerator::CHUNK_SIZE,
                (c.y as f32 + 0.5) * WorldGenerator::CHUNK_SIZE,
            );
            Vec3::new(x, world_gen.height(x, z), z)
        } else {
            spawn
        };

        if self.scene == "play" {
            self.player_ctrl = Some(Player::new(self.player));
            self.camera_rig = Some(CameraRig::new());
        }

        match pack::ModelPack::load(format!("{REPO}/rust/assets/models.pack")) {
            Ok(pack) => {
                let h = |x: f32, z: f32| world_gen.height(x, z);
                self.crowd =
                    crowd::Crowd::spawn(&gpu.device, &pack, self.player, 25, 15.0, 40.0, &h);
                self.pack = Some(pack);
            }
            Err(e) => log::warn!("no model pack ({e}); run `cargo run --release -p baker`"),
        }
        let world_gen = Arc::new(world_gen);
        let mut streamer = streaming::Streamer::new(world_gen.clone());
        // Load the first ring before the first frame (Godot holds the player still until the ground is ready)
        if let Some(pack) = &self.pack {
            while !streamer.is_ready_around(self.player) {
                streamer.update(
                    &gpu.device,
                    pack,
                    &mut self.library,
                    &mut self.physics,
                    self.player,
                );
                std::thread::yield_now();
            }
        }
        log::info!(
            "scene {} at {:?}: {} chunks, {} placed models, {} zombies, ready in {:.0} ms",
            self.scene,
            self.player,
            streamer.chunks.len(),
            streamer
                .chunks
                .values()
                .map(|c| c.placed.len())
                .sum::<usize>(),
            self.crowd.count(),
            started.elapsed().as_secs_f64() * 1000.0
        );
        self.streamer = Some(streamer);
        self.world_gen = Some(world_gen);
        if self.scene == "play" && !self.free_cam {
            if let (Some(rig), Some(p)) = (self.camera_rig.as_ref(), self.player_ctrl.as_ref()) {
                self.camera = rig.camera(&p.position, &self.physics);
            }
        } else if self.walk {
            self.follow(0.0);
        } else {
            self.camera.position = self.player + Vec3::new(0.0, 1.6, 0.0);
            self.camera.look_at(camp.center + Vec3::new(0.0, 1.2, 0.0));
        }
    }

    fn update(&mut self, input: &Input, dt: f32) {
        self.time += dt;
        if self.scene == "play" && !self.free_cam {
            // Esc releases the mouse cursor, left click recaptures it
            if input.pressed(winit::keyboard::KeyCode::Escape) {
                self.cursor_captured = false;
            } else if !self.cursor_captured && input.mouse_pressed(winit::event::MouseButton::Left)
            {
                self.cursor_captured = true;
            }

            let mut actions = Actions::from_default_input(input);
            if !self.cursor_captured {
                actions.look_delta = (0.0, 0.0);
            }
            if self.autowalk {
                actions.move_vec.y = -1.0;
            }

            if let (Some(p), Some(rig)) = (self.player_ctrl.as_mut(), self.camera_rig.as_mut()) {
                rig.update(&actions, p.crouching, dt);
                p.first_person = rig.is_first_person;
                p.update(&actions, rig.yaw, &self.physics, &self.mover, dt);
                self.player = p.position;
                self.camera = rig.camera(&self.player, &self.physics);
            }
        } else if self.walk {
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
            for (key, dir) in [
                (KeyCode::KeyW, f),
                (KeyCode::KeyS, -f),
                (KeyCode::KeyD, right),
                (KeyCode::KeyA, -right),
            ] {
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
        // Chunk uploads need the GPU, so streaming runs here rather than in update
        if let (Some(streamer), Some(pack)) = (self.streamer.as_mut(), self.pack.as_ref()) {
            streamer.update(
                &gpu.device,
                pack,
                &mut self.library,
                &mut self.physics,
                self.player,
            );
        }
        let (Some(renderer), Some(streamer)) = (self.renderer.as_mut(), self.streamer.as_ref())
        else {
            return;
        };
        let items: Vec<DrawItem> = streamer
            .chunks
            .values()
            .map(|c| DrawItem {
                mesh: &c.ground,
                offset: c.origin,
            })
            .collect();
        // Joint 0 is the identity: static instanced props use it
        self.bones.clear();
        self.bones.push(Mat4::IDENTITY);
        let placed = streamer.chunks.values().flat_map(|c| c.placed.iter());
        let (mut main, mut shadow) = self.library.batches(
            placed,
            &self.camera,
            target.aspect(),
            renderer.shadows.distance,
        );
        let zombies = self.crowd.pose(self.time, &mut self.bones);
        shadow.extend(zombies.iter().map(|b| engine::render::SkinnedBatch {
            mesh: b.mesh,
            instances: b.instances.clone(),
        }));
        main.extend(zombies);
        renderer.draw(
            gpu,
            target,
            encoder,
            &self.camera,
            &light::at_hour(self.hour),
            &items,
            &main,
            &shadow,
            &self.bones,
        );
    }

    fn cursor_grab(&self) -> Option<bool> {
        if self.scene == "play" && !self.free_cam {
            Some(self.cursor_captured)
        } else {
            None
        }
    }

    fn stats(&self) -> GameStats {
        let mut s = GameStats::new();
        if let Some(r) = &self.renderer {
            s.insert("drawCalls".into(), r.last.draws.into());
            s.insert("triangles".into(), r.last.triangles.into());
            s.insert("shadowDrawCalls".into(), r.last.shadow_draws.into());
            s.insert("shadowTriangles".into(), r.last.shadow_triangles.into());
        }
        s.insert("colliders".into(), self.physics.collider_count().into());
        let (x, z) = (self.player.x, self.player.z);
        if let Some(h) = self.physics.ground_height(x, z) {
            // Must equal the generator's height (same triangles): a quick health check of the collision world
            s.insert(
                "groundCheck".into(),
                format!("physics {h:.3} vs terrain {:.3}", self.height(x, z)).into(),
            );
        }
        s.insert("scene".into(), self.scene.clone().into());
        s.insert("seed".into(), self.seed.into());
        if let Some(st) = &self.streamer {
            s.insert("groundChunks".into(), st.chunks.len().into());
            s.insert(
                "placedModels".into(),
                st.chunks
                    .values()
                    .map(|c| c.placed.len())
                    .sum::<usize>()
                    .into(),
            );
        }
        s.insert("zombies".into(), self.crowd.count().into());
        if let Some(p) = &self.player_ctrl {
            s.insert(
                "playerSpeed".into(),
                format!("{:.2}", p.horizontal_speed()).into(),
            );
            s.insert("playerOnFloor".into(), p.on_floor.into());
            s.insert("playerCrouching".into(), p.crouching.into());
            s.insert("playerSprinting".into(), p.is_sprinting.into());
            s.insert(
                "playerStamina".into(),
                format!("{:.1}", p.stamina.current).into(),
            );
        }
        s
    }
}

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,wgpu_core=warn,wgpu_hal=warn"),
    )
    .init();
    engine::run(
        R0 {
            hour: 12.0,
            seed: 42,
            ..Default::default()
        },
        "Rotland (Rust)",
        EngineArgs::from_env(),
    );
}
