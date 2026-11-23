//! Interactive and screenshot viewer for humanoid player animation and weapon attachment.
//!
//! Shows the player model on flat ground at noon cycling idle -> walk -> run every 3 s
//! with a bat in hand, camera 3 m in front.

#[path = "../src/light.rs"]
mod light;
#[path = "../src/player_visual.rs"]
#[allow(dead_code)]
mod player_visual;

use engine::render::{Camera, DrawItem, GpuMesh, LitRenderer, SkinnedBatch};
use engine::{EngineArgs, FrameTarget, Game, GameStats, Gpu, Input};
use glam::{Mat4, Vec3};
use humanoid::Locomotion;
use pack::{ModelPack, PackVertex};
use player_visual::PlayerVisual;


#[derive(Default)]
struct HumanoidView {
    renderer: Option<LitRenderer>,
    ground: Option<GpuMesh>,
    visual: Option<PlayerVisual>,
    camera: Camera,
    frame: u64,
    time: f32,
    current_speed: f32,
    bones: Vec<Mat4>,
    /// `--speed=N` holds one speed instead of cycling idle, walk and run.
    fixed_speed: Option<f32>,
    /// `--vel=x,z` moves the body this way (model faces -Z), to check side and back steps.
    velocity: Option<Vec3>,
}

fn make_ground(device: &wgpu::Device) -> GpuMesh {
    let size = 20.0;
    let y = 0.0;
    let col = [75, 115, 55, 255]; // grass green
    let n = [0.0, 1.0, 0.0];
    let verts = vec![
        PackVertex { position: [-size, y, -size], normal: n, color: col, bone: 0, _pad: [0; 3] },
        PackVertex { position: [ size, y, -size], normal: n, color: col, bone: 0, _pad: [0; 3] },
        PackVertex { position: [ size, y,  size], normal: n, color: col, bone: 0, _pad: [0; 3] },
        PackVertex { position: [-size, y, -size], normal: n, color: col, bone: 0, _pad: [0; 3] },
        PackVertex { position: [ size, y,  size], normal: n, color: col, bone: 0, _pad: [0; 3] },
        PackVertex { position: [-size, y,  size], normal: n, color: col, bone: 0, _pad: [0; 3] },
    ];
    GpuMesh::new(device, "flat_ground", &verts)
}

impl Game for HumanoidView {
    fn init(&mut self, gpu: &Gpu, args: &EngineArgs) {
        self.renderer = Some(LitRenderer::new(gpu));
        self.ground = Some(make_ground(&gpu.device));

        let pack_path = paths::repo_path("assets/models.pack");
        let pack = ModelPack::load(&pack_path)
            .or_else(|_| ModelPack::load("assets/models.pack"))
            .expect("failed to load models.pack; run baker first");

        // Test options: --weapon=rifle (or none), --aim, --speed=4, --cam-angle=45 (degrees around the player)
        let weapon = args.value("--weapon").unwrap_or("bat");
        let held = (weapon != "none").then_some(weapon);
        let mut visual = PlayerVisual::load(&gpu.device, &pack, "human_player", held);
        visual.animator.set_weapon(held);
        visual.animator.set_aiming(args.flag("--aim"));
        if let Some(deg) = args.value("--pitch").and_then(|v| v.parse::<f32>().ok()) {
            visual.animator.set_look_pitch(deg.to_radians());
        }
        self.visual = Some(visual);
        self.fixed_speed = args.value("--speed").and_then(|v| v.parse().ok());
        self.velocity = args.value("--vel").and_then(|v| {
            let (x, z) = v.split_once(',')?;
            Some(Vec3::new(x.parse().ok()?, 0.0, z.parse().ok()?))
        });
        let angle = args.value("--cam-angle").and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0).to_radians();

        // Camera 3 m in front of player (human_player faces -Z, so 3 m in front is at -Z looking at +Z)
        self.camera.position = Vec3::new(3.0 * angle.sin(), 1.1, -3.0 * angle.cos());
        self.camera.look_at(Vec3::new(0.0, 1.0, 0.0));
    }

    fn update(&mut self, _input: &Input, _dt: f32) {
        self.frame += 1;
        let dt = 1.0 / 60.0;
        self.time += dt;

        // Cycle idle -> walk -> run every 3 s (60 fps = 180 frames per phase):
        // 0..3s (frames 0..180): idle (0 m/s) -> frame 60 is idle
        // 3..6s (frames 180..360): walk (4 m/s) -> frame 250 is walk
        // 6..9s (frames 360..540): run (7 m/s) -> frame 450 is run
        let phase = ((self.time / 3.0) as usize) % 3;
        self.current_speed = match phase {
            0 => 0.0,
            1 => 4.0,
            _ => 7.0,
        };
        if let Some(speed) = self.fixed_speed {
            self.current_speed = speed;
        }
    }

    fn render(&mut self, gpu: &Gpu, target: &FrameTarget, encoder: &mut wgpu::CommandEncoder) {
        let (Some(renderer), Some(ground), Some(visual)) =
            (self.renderer.as_mut(), self.ground.as_ref(), self.visual.as_mut())
        else {
            return;
        };

        self.bones.clear();
        self.bones.push(Mat4::IDENTITY); // Joint 0 is identity for static meshes

        if let Some(v) = self.velocity {
            visual.animator.set_move_velocity(v, 0.0);
        }
        let dt = 1.0 / 60.0;
        let (body_batch, weapon_batch) = visual.pose(
            dt,
            Vec3::ZERO,
            0.0,
            Locomotion {
                speed: self.current_speed,
                on_floor: true,
                crouching: false,
            },
            &mut self.bones,
        );

        let mut main = vec![body_batch];
        let mut shadow = vec![SkinnedBatch {
            mesh: main[0].mesh,
            instances: main[0].instances.clone(),
        }];

        if let Some(wb) = weapon_batch {
            shadow.push(SkinnedBatch {
                mesh: wb.mesh,
                instances: wb.instances.clone(),
            });
            main.push(wb);
        }

        let items = [DrawItem {
            mesh: ground,
            offset: Vec3::ZERO,
        }];

        let lighting = light::at_hour(12.0, None); // noon
        renderer.draw(
            gpu,
            target,
            encoder,
            &self.camera,
            &lighting,
            &items,
            &main,
            &shadow,
            &self.bones,
        );
    }

    fn stats(&self) -> GameStats {
        let mut s = GameStats::new();
        s.insert("speed".into(), format!("{:.1}", self.current_speed).into());
        s.insert("time".into(), format!("{:.2}", self.time).into());
        s
    }
}

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,wgpu_core=warn,wgpu_hal=warn"),
    )
    .init();
    engine::run(HumanoidView::default(), "Humanoid View", EngineArgs::from_env());
}
