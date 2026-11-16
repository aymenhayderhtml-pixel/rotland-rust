//! Rotland (Rust). R0: the real world with real trees and 25 zombies, for the go/no-go speed test.
//! Options: --scene=forest|camp (forest = Godot's forest25 benchmark walk), --seed=N (42), --hour=H (12),
//! --free (fly with mouse + WASD instead of the scripted walk), plus the engine's --shot / --stats / --seconds.

mod bench;
mod combat;
mod crates;
mod hud;
mod light;
mod menus;
mod play;
mod player_visual;
mod props;
mod settings;
mod shooting;
mod streaming;
mod terrain;
mod zombies;

use std::sync::Arc;

use engine::render::{Camera, DrawItem, LitRenderer};
use engine::{EngineArgs, FrameTarget, Game, GameStats, Gpu, Input};
use game::{Actions, CameraRig, Player};
use glam::{Mat4, Vec3};
use world::WorldGenerator;


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
    zombies: Option<zombies::Zombies>,
    combat: Option<combat::Combat>,
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
    player_visual: Option<player_visual::PlayerVisual>,
    last_dt: f32,
    camera_rig: Option<CameraRig>,
    mover: physics::CharacterMover,
    cursor_captured: bool,
    autowalk: bool,
    /// Scripted test: swing whenever possible (for screenshots and stats without a person at the mouse).
    autoattack: bool,
    /// Scripted test: the player dies at this time (seconds), to check respawning.
    die_at: Option<f32>,
    free_cam: bool,
    /// Backpack, hotbar and spawner (play scene only).
    play: Option<play::PlayState>,
    /// A weapon model to put in the player's hand; applied in render, where the GPU is.
    pending_weapon_model: Option<Option<String>>,
    /// Smoothed frames per second for the HUD.
    fps: f32,
    /// Play mode's running clock (Godot TimeOfDay: 1 game hour per real minute, starting 08:00).
    clock: world::time::Clock,
    /// The clock stands still (benchmarks, --hour screenshots).
    clock_frozen: bool,
    flashlight_on: bool,
}

/// The pack model name of a weapon ("res://assets/models/weapons/bat.glb" -> "bat").
fn weapon_model_name(def: &game::WeaponDef) -> Option<String> {
    let file = def.model.rsplit('/').next()?;
    Some(file.trim_end_matches(".glb").to_string()).filter(|s| !s.is_empty())
}

impl R0 {
    /// Puts a weapon in hand (or empties it): the swing uses its numbers, the model shows in the hand.
    fn apply_hand(&mut self, change: play::HandChange) {
        let def = match change {
            play::HandChange::Weapon(def) => def,
            play::HandChange::ToggleFlashlight => {
                self.flashlight_on = !self.flashlight_on;
                return;
            }
        };
        self.pending_weapon_model = Some(def.as_ref().and_then(weapon_model_name));
        // Guns go to the shooting system, everything else to the melee swing
        let is_gun = def.as_ref().is_some_and(|d| d.is_gun());
        if let Some(play) = self.play.as_mut() {
            play.gun = if is_gun { def.clone() } else { None };
        }
        if let Some(c) = self.combat.as_mut() {
            c.set_weapon(if is_gun { None } else { def });
        }
    }

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
        self.scene = args.value("--scene").unwrap_or("play").to_string();
        // --hour=H fixes the time (screenshots, benchmarks); play mode otherwise starts at 08:00 and runs
        let forced_hour: Option<f32> = args.value("--hour").and_then(|s| s.parse().ok());
        let start = forced_hour.unwrap_or(if self.scene == "play" { world::time::Clock::new().hour } else { 12.0 });
        self.clock = world::time::Clock::new();
        self.clock.set_time(start);
        self.hour = start;
        self.clock_frozen = self.scene != "play" || forced_hour.is_some();
        self.flashlight_on = args.flag("--flashlight");
        self.free_cam = args.flag("--free");
        self.autowalk = args.flag("--autowalk");
        self.autoattack = args.flag("--autoattack");
        self.die_at = args.value("--die-at").and_then(|v| v.parse().ok());
        self.walk = self.scene == "forest" && !self.free_cam;
        self.cursor_captured = self.scene == "play" && !self.free_cam;
        self.renderer = Some(LitRenderer::new(gpu));

        let started = std::time::Instant::now();
        let world_gen =
            WorldGenerator::from_data_dir(self.seed, paths::repo_path("data")).expect("data files");
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
            let rig = CameraRig::new();
            let mut player = Player::new(self.player);
            // Start facing where the camera looks (away from it), not toward it
            player.yaw = rig.yaw;
            self.player_ctrl = Some(player);
            self.camera_rig = Some(rig);
        }

        match pack::ModelPack::load(paths::repo_path("rust/assets/models.pack")) {
            Ok(pack) => {
                match game::zombie::ZombieDefs::load_default() {
                    Ok(defs) => self.zombies = Some(zombies::Zombies::new(&gpu.device, &pack, defs)),
                    Err(e) => log::warn!("data/zombies.json: {e}"),
                }
                if self.scene == "play" {
                    self.combat = Some(combat::Combat::new("bat"));
                    match play::PlayState::new(self.seed as u64) {
                        Ok(state) => self.play = Some(state),
                        Err(e) => log::warn!("play systems not loaded: {e}"),
                    }
                    // Godot's starter weapon is the bat (data/start_inventory.json)
                    self.player_visual = Some(player_visual::PlayerVisual::load(&gpu.device, &pack, "human_player", Some("bat")));
                }
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
        // Zombies need the ground's collision, so they spawn after the first ring has loaded. Like the
        // forest25 benchmark: 25 walkers 15-40 m around the player (the real spawner comes in R3-07).
        // In play mode the spawner fills the world over time instead (--zombies=N still forces a ring)
        let forced = args.value("--zombies").and_then(|s| s.parse::<usize>().ok());
        let ring = if self.scene == "play" { forced.unwrap_or(0) } else { forced.unwrap_or(25) };
        if let (Some(z), true) = (self.zombies.as_mut(), ring > 0) {
            let count = ring;
            z.spawn_ring("walker", self.player, count, 15.0, 40.0, &self.physics);
        }
        if let Some(change) = self.play.as_mut().and_then(|p| p.equip_first()) {
            self.apply_hand(change);
        }
        // Test option: take out hotbar slot N at the start (e.g. --slot=2 for the pistol)
        if let Some(n) = args.value("--slot").and_then(|v| v.parse::<usize>().ok())
            && let Some(change) = self.play.as_mut().and_then(|p| p.press(n))
        {
            self.apply_hand(change);
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
            self.zombies.as_ref().map_or(0, |z| z.list.len()),
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
        self.last_dt = dt;
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

            let dead = self.combat.as_ref().is_some_and(|c| c.is_dead());
            if dead {
                actions = Actions::default();
            }
            if let Some(change) = self.play.as_mut().and_then(|p| p.hotbar_keys(input)) {
                self.apply_hand(change);
            }
            let attack = !dead && (self.autoattack || (self.cursor_captured && input.mouse_pressed(winit::event::MouseButton::Left)));
            if let (Some(p), Some(rig)) = (self.player_ctrl.as_mut(), self.camera_rig.as_mut()) {
                rig.update(&actions, p.crouching, dt);
                p.first_person = rig.is_first_person;
                p.update(&actions, rig.yaw, &self.physics, &self.mover, dt);
                if let (Some(c), Some(z)) = (self.combat.as_mut(), self.zombies.as_mut()) {
                    let ev = c.update(dt, attack, p, z, &self.physics);
                    if let (Some(clip), Some(pv)) = (ev.swing_started, self.player_visual.as_mut()) {
                        pv.animator.play_action(&clip, 1.0);
                    }
                    if ev.respawn {
                        // Clear the death pose, or it stays mixed into the legs after waking up
                        if let Some(pv) = self.player_visual.as_mut() {
                            pv.animator.reset_alive();
                        }
                        // Wake up at the camp (Godot respawns at the last settlement visited)
                        if let Some(w) = self.world_gen.as_ref() {
                            let spawn = w.settlements()[0].respawn_point(w.config());
                            *p = Player::new(spawn);
                            // Face where the camera looks, not the default direction
                            p.yaw = rig.yaw;
                        }
                    }
                }
                self.player = p.position;
                self.camera = rig.camera(&self.player, &self.physics);
                // A gun's aim zoom (the rifle scope zooms much further, CameraRig/GunHandler)
                let aiming = self.cursor_captured && input.mouse_held(winit::event::MouseButton::Right);
                if let (true, Some(gun)) = (aiming, self.play.as_ref().and_then(|pl| pl.gun.as_ref()))
                    && let Some(fov) = shooting::aim_fov(gun, rig.is_first_person)
                {
                    self.camera.fov_y_deg = fov;
                }
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
        if let (Some(t), Some(c)) = (self.die_at, self.combat.as_mut())
            && self.time >= t
        {
            self.die_at = None;
            if c.kill()
                && let Some(pv) = self.player_visual.as_mut()
            {
                pv.animator.play_die();
            }
        }
        // Guns, crates and pickups (play mode)
        let alive_now = !self.combat.as_ref().is_some_and(|c| c.is_dead());
        if let (Some(play), Some(p)) = (self.play.as_mut(), self.player_ctrl.as_ref())
            && alive_now
            && self.scene == "play"
        {
            use winit::event::MouseButton;
            use winit::keyboard::KeyCode;
            let captured = self.cursor_captured;
            let shoot = shooting::ShootInput {
                // --autoattack clicks every other frame, so semi-automatic guns see a fresh press each time
                fire_pressed: (self.autoattack && ((self.time * 10.0) as u32).is_multiple_of(2)) || (captured && input.mouse_pressed(MouseButton::Left)),
                fire_held: captured && input.mouse_held(MouseButton::Left),
                reload_pressed: input.pressed(KeyCode::KeyR),
                aiming: captured && input.mouse_held(MouseButton::Right),
            };
            if let Some(z) = self.zombies.as_mut()
                && let Some(ev) = play.shoot_tick(dt, shoot, self.camera.position, self.camera.forward(), z, &self.physics)
                && let Some(c) = self.combat.as_mut()
            {
                c.kills += ev.kills;
                c.shots += ev.shots_fired;
                c.noises.extend(ev.noise);
            }
            // Crates within reach: only the nearby chunks' spots are looked at
            let facing = Vec3::new(-p.yaw.sin(), 0.0, -p.yaw.cos());
            let near: Vec<world::LootSpot> = self
                .streamer
                .as_ref()
                .map(|st| {
                    st.chunks
                        .values()
                        .flat_map(|c| c.crates.iter())
                        .filter(|c| c.position.distance_squared(p.position) < 25.0)
                        .cloned()
                        .collect()
                })
                .unwrap_or_default();
            play.interact_tick(dt, input.pressed(KeyCode::KeyE), p.position, facing, &near);
        }
        // Zombies chase the living player; while dead they lose the target
        let alive = !self.combat.as_ref().is_some_and(|c| c.is_dead());
        let noises = self.combat.as_ref().map(|c| c.noises.clone()).unwrap_or_default();
        if let (Some(play), Some(z), Some(w), Some(p)) = (self.play.as_mut(), self.zombies.as_mut(), self.world_gen.as_ref(), self.player_ctrl.as_ref()) {
            let forward = Vec3::new(-p.yaw.sin(), 0.0, -p.yaw.cos());
            let night = !(6.0..20.0).contains(&self.hour);
            play.spawn_tick(dt, self.player, forward, night, z, &self.physics, w);
        }
        if !self.clock_frozen {
            self.clock.advance(dt);
            self.hour = self.clock.hour;
        }
        if dt > 0.0 {
            self.fps += (1.0 / dt - self.fps) * (dt * 2.0).min(1.0);
        }
        if let Some(z) = self.zombies.as_mut() {
            let frame = z.update(dt, alive.then_some(self.player), &noises, &self.physics);
            if let Some(c) = self.combat.as_mut() {
                let bites: Vec<f32> = frame.bites.iter().map(|b| b.damage).collect();
                let ev = c.take_bites(&bites);
                if ev.died {
                    if let Some(pv) = self.player_visual.as_mut() {
                        pv.animator.play_die();
                    }
                    log::info!("the player died (kills so far: {})", c.kills);
                } else if ev.bitten
                    && let Some(pv) = self.player_visual.as_mut()
                {
                    pv.animator.play_hit();
                }
            }
        }
    }

    fn render(&mut self, gpu: &Gpu, target: &FrameTarget, encoder: &mut wgpu::CommandEncoder) {
        let player_pose = self.player_pose();
        if let (Some(model), Some(pv), Some(pack)) = (self.pending_weapon_model.take(), self.player_visual.as_mut(), self.pack.as_ref()) {
            pv.set_weapon(&gpu.device, pack, model.as_deref());
        }
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
        if let (Some(pv), Some(pose)) = (self.player_visual.as_mut(), player_pose) {
            let loco = humanoid::Locomotion { speed: pose.horizontal_speed, on_floor: pose.on_floor, crouching: pose.crouching };
            // The player's yaw already uses the model's facing (checked in a screenshot: no extra turn)
            let (body, weapon) = pv.pose(self.last_dt, pose.position, pose.yaw, loco, &mut self.bones);
            // First person: the body only casts its shadow (like Godot's shadows-only body)
            if !pose.first_person {
                main.push(engine::render::SkinnedBatch { mesh: body.mesh, instances: body.instances.clone() });
                if let Some(w) = &weapon {
                    main.push(engine::render::SkinnedBatch { mesh: w.mesh, instances: w.instances.clone() });
                }
            }
            shadow.push(body);
            shadow.extend(weapon);
        }
        let zombies = self.zombies.as_ref().map(|z| z.pose(&mut self.bones)).unwrap_or_default();
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
            &light::at_hour(self.hour, self.flashlight_on.then(|| {
                // Held at the camera, pointing where it looks (Godot hangs it on the camera)
                light::flashlight(self.camera.position, self.camera.forward())
            })),
            &items,
            &main,
            &shadow,
            &self.bones,
        );
    }

    fn ui(&mut self, ctx: &engine::egui::Context) {
        let (Some(play), Some(p), Some(c)) = (self.play.as_ref(), self.player_ctrl.as_ref(), self.combat.as_ref()) else { return };
        let state = play.hud(c.health.current(), c.health.max(), p.stamina.current, p.stamina.max, self.hour, p.yaw, self.fps, c.is_dead());
        hud::draw(ctx, &state);
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
        s.insert("playerPosition".into(), format!("{:.2} {:.2} {:.2}", self.player.x, self.player.y, self.player.z).into());
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
        if let Some(z) = &self.zombies {
            s.insert("zombies".into(), z.list.len().into());
            s.insert("zombiesAlive".into(), z.alive().into());
        }
        if let Some(c) = &self.combat {
            s.insert("playerHealth".into(), format!("{:.0}", c.health.current()).into());
            s.insert("kills".into(), c.kills.into());
            s.insert("shots".into(), c.shots.into());
            if let Some(play) = &self.play {
                s.insert("ammoBag".into(), play.inventory.count("pistol_ammo").into());
                s.insert("ammoMag".into(), play.last_shot.ammo.0.into());
            }
            s.insert("playerDead".into(), c.is_dead().into());
        }
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
