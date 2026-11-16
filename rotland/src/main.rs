//! Rotland (Rust). R0: the real world with real trees and 25 zombies, for the go/no-go speed test.
//! Options: --scene=forest|camp (forest = Godot's forest25 benchmark walk), --seed=N (42), --hour=H (12),
//! --free (fly with mouse + WASD instead of the scripted walk), plus the engine's --shot / --stats / --seconds.

mod bench;
mod camp;
mod camp_ui;
mod combat;
mod crates;
mod hud;
mod light;
mod menus;
mod play;
mod player_visual;
mod props;
mod session;
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
    /// Main menu, pause, settings and the slot picker (play scene only).
    menus: Option<menus::Menus>,
    /// The three slot cards, re-read when the slots change (not every frame).
    slot_cards: [menus::SlotSummary; 3],
    /// The Rust game's save folder (--save-dir for tests, so they never touch the real one).
    slots: Option<save::Slots>,
    /// The slot this session saves into (1-3).
    active_slot: u8,
    /// The file this session came from: written over on save, so systems not ported yet survive.
    loaded_save: Option<save::SaveFile>,
    /// A new world to build at the next frame (seed, save to restore), from New game / Continue.
    pending_world: Option<(i32, Option<save::SaveFile>)>,
    /// --zombies=N: a ring of zombies at the start (tests and benchmarks).
    forced_zombies: Option<usize>,
    /// Seconds played in this save (stats.playTime).
    play_time: f64,
    quit: bool,
    /// Scripted test: save into the active slot at this time (seconds).
    save_at: Option<f32>,
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
            play::HandChange::Health(amount) => {
                if let Some(c) = self.combat.as_mut() {
                    if amount > 0.0 {
                        c.health.heal(amount);
                    } else {
                        // Spoiled food: sickness damage (no flinch)
                        c.take_bites(&[-amount]);
                    }
                }
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

    /// Does what the player picked in a menu.
    fn menu_action(&mut self, action: menus::MenuAction) {
        use menus::MenuAction;
        match action {
            MenuAction::Resume => {
                if let Some(m) = self.menus.as_mut() {
                    m.set_screen(menus::Screen::None);
                }
                self.cursor_captured = true;
            }
            MenuAction::NewGame { slot } => {
                self.active_slot = slot;
                // A fresh world: the seed comes from the clock, like Godot's new game
                let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(42, |d| d.as_nanos() as i32);
                self.pending_world = Some((seed, None));
                self.close_menus();
            }
            MenuAction::Continue { slot } => {
                let read = self.slots.as_ref().map(|s| s.read(slot as usize));
                match read {
                    Some(Ok(file)) => {
                        self.active_slot = slot;
                        self.pending_world = Some((session::seed_of(&file).unwrap_or(self.seed), Some(file)));
                        self.close_menus();
                    }
                    Some(Err(e)) => log::warn!("slot {slot} not loaded: {e:?}"),
                    None => {}
                }
            }
            MenuAction::DeleteSlot { slot } => {
                if let Some(slots) = &self.slots {
                    if let Err(e) = slots.delete(slot as usize) {
                        log::warn!("slot {slot} not deleted: {e:?}");
                    }
                    self.slot_cards = session::summaries(slots);
                }
            }
            MenuAction::SaveAndQuitToMenu => {
                self.save_now();
                if let Some(m) = self.menus.as_mut() {
                    m.set_screen(menus::Screen::Main);
                }
            }
            MenuAction::Quit => {
                // Quitting from the pause menu keeps the game; from the main menu there is nothing to save
                if self.menus.as_ref().is_some_and(|m| *m.screen() == menus::Screen::Paused) {
                    self.save_now();
                }
                self.quit = true;
            }
            MenuAction::SettingsChanged(new) => {
                if let Some(m) = self.menus.as_mut() {
                    m.settings = new;
                }
            }
        }
    }

    fn close_menus(&mut self) {
        if let Some(m) = self.menus.as_mut() {
            m.set_screen(menus::Screen::None);
        }
        self.cursor_captured = true;
    }

    /// Writes the game into the active slot (over the file it came from).
    fn save_now(&mut self) {
        let (Some(slots), Some(p), Some(rig), Some(c), Some(play), Some(w)) =
            (self.slots.as_ref(), self.player_ctrl.as_ref(), self.camera_rig.as_ref(), self.combat.as_ref(), self.play.as_ref(), self.world_gen.as_ref())
        else {
            return;
        };
        let snap = session::Snapshot {
            seed: self.seed,
            player: p,
            rig,
            health: c.health.current(),
            spawn: w.settlements()[0].respawn_point(w.config()),
            clock: &self.clock,
            play,
            play_time: self.play_time,
        };
        let file = session::capture(self.loaded_save.as_ref(), &snap);
        match slots.write(self.active_slot as usize, &file) {
            Ok(()) => {
                log::info!("saved slot {} to {}", self.active_slot, slots.path_for(self.active_slot as usize).display());
                self.slot_cards = session::summaries(slots);
                self.loaded_save = Some(file);
            }
            Err(e) => log::warn!("save to slot {} failed: {e:?}", self.active_slot),
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

impl R0 {
    /// Builds the world for `self.seed` (and puts a save back): ground, props, zombies, player, backpack.
    /// Runs at start-up and again when New game / Continue picks another world.
    fn start_world(&mut self, gpu: &Gpu, save_file: Option<save::SaveFile>) {
        self.physics = physics::PhysicsWorld::default();
        self.streamer = None;
        self.zombies = None;
        self.combat = None;
        self.play = None;
        self.player_visual = None;
        self.player_ctrl = None;
        self.camera_rig = None;
        self.play_time = 0.0;
        if self.scene == "play" && !self.clock_frozen {
            self.clock = world::time::Clock::new();
            self.hour = self.clock.hour;
        }
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

        if let Some(pack) = &self.pack {
            match game::zombie::ZombieDefs::load_default() {
                Ok(defs) => self.zombies = Some(zombies::Zombies::new(&gpu.device, pack, defs)),
                Err(e) => log::warn!("data/zombies.json: {e}"),
            }
            if self.scene == "play" {
                self.combat = Some(combat::Combat::new("bat"));
                match play::PlayState::new(self.seed as u64) {
                    Ok(state) => self.play = Some(state),
                    Err(e) => log::warn!("play systems not loaded: {e}"),
                }
                // Godot's starter weapon is the bat (data/start_inventory.json)
                self.player_visual = Some(player_visual::PlayerVisual::load(&gpu.device, pack, "human_player", Some("bat")));
            }
        }
        // A saved game: the player where they were, the clock, the backpack and the emptied crates
        if let Some(file) = &save_file {
            if let (Some(r), Some(p), Some(rig)) = (session::restore(file), self.player_ctrl.as_mut(), self.camera_rig.as_mut()) {
                p.position = r.position;
                p.yaw = r.body_yaw;
                rig.yaw = r.camera_yaw;
                rig.pitch = r.camera_pitch;
                self.player = r.position;
                // --hour (screenshots) wins over the saved hour
                if !self.clock_frozen {
                    self.clock.set_time(r.hour);
                    self.hour = self.clock.hour;
                }
                self.clock.day = r.day.max(1);
                self.play_time = r.play_time;
                if let Some(c) = self.combat.as_mut() {
                    c.health.set_current(r.health);
                }
            }
            if let Some(play) = self.play.as_mut() {
                session::restore_play(file, play);
            }
        }
        self.loaded_save = save_file;
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
        let forced = self.forced_zombies;
        let ring = if self.scene == "play" { forced.unwrap_or(0) } else { forced.unwrap_or(25) };
        if let (Some(z), true) = (self.zombies.as_mut(), ring > 0) {
            let count = ring;
            z.spawn_ring("walker", self.player, count, 15.0, 40.0, &self.physics);
        }
        if let Some(change) = self.play.as_mut().and_then(|p| p.equip_first()) {
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
        self.save_at = args.value("--save-at").and_then(|v| v.parse().ok());
        self.walk = self.scene == "forest" && !self.free_cam;
        self.cursor_captured = self.scene == "play" && !self.free_cam;
        self.renderer = Some(LitRenderer::new(gpu));

        // Saves: --save-dir keeps tests away from the real folder; --import-godot=N copies a Godot slot
        // (read only) into Rust slot N; --load=N continues slot N; --menu starts on the main menu
        let slots = save::Slots::new(args.value("--save-dir").map(std::path::PathBuf::from).unwrap_or_else(save::default_base_dir));
        let settings_file = match args.value("--save-dir") {
            Some(dir) => std::path::PathBuf::from(dir).join("settings.json"),
            None => settings::settings_path(),
        };
        let mut save_file = None;
        if let Some(n) = args.value("--import-godot").and_then(|v| v.parse::<usize>().ok()) {
            match slots.import_godot(n) {
                Ok(f) => {
                    log::info!("imported Godot slot {n} into {}", slots.path_for(n).display());
                    self.active_slot = n as u8;
                    save_file = Some(f);
                }
                Err(e) => log::warn!("import of Godot slot {n} failed: {e:?}"),
            }
        }
        if let Some(n) = args.value("--load").and_then(|v| v.parse::<usize>().ok()) {
            match slots.read(n) {
                Ok(f) => {
                    self.active_slot = n as u8;
                    save_file = Some(f);
                }
                Err(e) => log::warn!("slot {n} not loaded: {e:?}"),
            }
        }
        if self.active_slot == 0 {
            self.active_slot = 1;
        }
        if let Some(seed) = save_file.as_ref().and_then(session::seed_of) {
            self.seed = seed;
        }
        if self.scene == "play" && !self.free_cam {
            let mut m = if args.flag("--menu") { menus::Menus::new(settings::Settings::load(&settings_file)) } else { menus::Menus::in_game(settings::Settings::load(&settings_file)) };
            m.set_save_path(settings_file);
            self.menus = Some(m);
        }
        self.slot_cards = session::summaries(&slots);
        self.slots = Some(slots);
        self.forced_zombies = args.value("--zombies").and_then(|s| s.parse::<usize>().ok());

        match pack::ModelPack::load(paths::repo_path("rust/assets/models.pack")) {
            Ok(pack) => self.pack = Some(pack),
            Err(e) => log::warn!("no model pack ({e}); run `cargo run --release -p baker`"),
        }
        self.start_world(gpu, save_file);
        // Test option: take out hotbar slot N at the start (e.g. --slot=2 for the pistol)
        if let Some(n) = args.value("--slot").and_then(|v| v.parse::<usize>().ok())
            && let Some(change) = self.play.as_mut().and_then(|p| p.press(n))
        {
            self.apply_hand(change);
        }
    }

    fn update(&mut self, input: &Input, dt: f32) {
        self.last_dt = dt;
        self.time += dt;
        if self.scene == "play" && !self.free_cam {
            // Esc opens the pause menu (and steps back inside the menus); the game stands still while a
            // menu is open
            if input.pressed(winit::keyboard::KeyCode::Escape)
                && let Some(m) = self.menus.as_mut()
            {
                let was_open = m.is_open();
                m.on_escape();
                if !was_open && m.is_open() {
                    if let Some(slots) = &self.slots {
                        self.slot_cards = session::summaries(slots);
                    }
                } else if was_open && !m.is_open() {
                    self.cursor_captured = true;
                }
            }
            if self.menus.as_ref().is_some_and(|m| m.is_open()) {
                if dt > 0.0 {
                    self.fps += (1.0 / dt - self.fps) * (dt * 2.0).min(1.0);
                }
                return;
            }
            if !self.cursor_captured && input.mouse_pressed(winit::event::MouseButton::Left) {
                self.cursor_captured = true;
            }
            self.play_time += dt as f64;

            let mut actions = Actions::from_default_input(input);
            if !self.cursor_captured {
                actions.look_delta = (0.0, 0.0);
            }
            if let Some(m) = &self.menus {
                let sens = m.settings.mouse_sensitivity;
                actions.look_delta.0 *= sens;
                actions.look_delta.1 *= if m.settings.invert_y { -sens } else { sens };
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
                        if let Some(play) = self.play.as_mut() {
                            play.needs_respawn();
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
        if let Some(t) = self.save_at
            && self.time >= t
        {
            self.save_at = None;
            self.save_now();
        }
        // Hunger and thirst drain; empty ones hurt, being well fed heals (play mode)
        if let (Some(play), Some(p), Some(c)) = (self.play.as_mut(), self.player_ctrl.as_mut(), self.combat.as_mut()) {
            let dead = c.is_dead();
            let (damage, heal) = play.needs_tick(dt, p.is_sprinting, dead, (c.health.current(), c.health.max()), &mut p.stamina);
            if heal > 0.0 {
                c.health.heal(heal);
            }
            if damage > 0.0 && c.take_bites(&[damage]).died {
                if let Some(pv) = self.player_visual.as_mut() {
                    pv.animator.play_die();
                }
                log::info!("the player died of hunger or thirst");
            }
            play.health_full = c.health.current() >= c.health.max();
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
        // Kills count for bounties; the trader restocks and the board refreshes with the days
        let night = !(6.0..20.0).contains(&self.hour);
        if let (Some(play), Some(z)) = (self.play.as_mut(), self.zombies.as_mut()) {
            for kind in z.take_kills() {
                play.camp.on_zombie_killed(&kind, night);
            }
            play.camp.tick(self.clock.day, self.hour);
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
                    if let Some(play) = self.play.as_mut() {
                        let seed = (self.time * 1000.0) as u64 ^ self.seed as u64;
                        play.camp.on_death(&mut play.inventory, seed);
                        play.hotbar.on_inventory_changed(&play.inventory);
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
        // New game / Continue: build the picked world here, where the GPU is
        if let Some((seed, file)) = self.pending_world.take() {
            self.seed = seed;
            self.start_world(gpu, file);
        }
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
        let on_main = self.menus.as_ref().is_some_and(|m| matches!(m.screen(), menus::Screen::Main | menus::Screen::SlotPicker { .. }));
        if let (Some(play), Some(p), Some(c), false) = (self.play.as_ref(), self.player_ctrl.as_ref(), self.combat.as_ref(), on_main) {
            let show_fps = self.menus.as_ref().is_none_or(|m| m.settings.show_fps);
            let fps = if show_fps { self.fps } else { 0.0 };
            let state = play.hud(c.health.current(), c.health.max(), p.stamina.current, p.stamina.max, self.hour, p.yaw, fps, c.is_dead());
            hud::draw(ctx, &state);
        }
        let Some(m) = self.menus.as_mut() else { return };
        if let Some(action) = m.ui(ctx, &self.slot_cards) {
            self.menu_action(action);
        }
    }

    fn cursor_grab(&self) -> Option<bool> {
        if self.scene == "play" && !self.free_cam {
            Some(self.cursor_captured && !self.menus.as_ref().is_some_and(|m| m.wants_cursor()))
        } else {
            None
        }
    }

    fn should_quit(&self) -> bool {
        self.quit
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
