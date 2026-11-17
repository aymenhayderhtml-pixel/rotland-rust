//! Rotland (Rust). R0: the real world with real trees and 25 zombies, for the go/no-go speed test.
//! Options: --scene=forest|camp (forest = Godot's forest25 benchmark walk), --seed=N (42), --hour=H (12),
//! --free (fly with mouse + WASD instead of the scripted walk), plus the engine's --shot / --stats / --seconds.

mod bench;
mod camp;
mod camp_npcs;
mod campfire_mesh;
mod wildlife;
mod dog_view;
mod camp_ui;
mod hub_ui;
mod combat;
mod crates;
mod hud;
mod light;
mod map_ui;
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
mod bubbles;
mod car_solid;
mod keys_ui;
mod poi_links;
mod car_lights;
mod fp_weapon;
mod trunk_play;
mod display;

use std::collections::HashMap;
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
    /// --yaw=DEG: start facing this way (screenshots).
    start_yaw: Option<f32>,
    /// --give=hoe,seeds_wheat:5: items put in the bag at the start (tests).
    give: Vec<(String, u32)>,
    /// --animals=deer,boar,rabbit: put these animals in front of the player at the start (screenshots).
    near_animals: Vec<String>,
    /// Seconds played in this save (stats.playTime).
    play_time: f64,
    quit: bool,
    /// Scripted test: save into the active slot at this time (seconds).
    save_at: Option<f32>,
    /// The open camp screen (shop, bank, crafting, bounties), if any.
    camp_ui: Option<camp_ui::CampUiState>,
    /// The player's hub (Tab bag and crafting, L skills, J bounties, P profile, M map).
    hub: Option<hub_ui::HubUiState>,
    /// On a ladder (Godot LadderClimb).
    climb: game::LadderClimb,
    /// E was used this frame (a ladder grabbed): nothing else takes it.
    e_used: bool,
    /// --climb: after a second, stand at the nearest ladder and grab it (tests; --autowalk then climbs).
    test_climb: bool,
    /// Test option --chop: stand at the nearest tree with the axe in hand.
    test_chop: bool,
    /// Test option --first-person: start in the first-person camera.
    test_first_person: bool,
    /// Test option --poi: stand 20 m from the nearest building after a second.
    test_poi: bool,
    campfire_meshes: Option<campfire_mesh::CampfireMeshes>,
    /// The level-up banner and "+XP" toasts (always on, also when the hub is shut).
    overlays: Option<hub_ui::HubUiState>,
    /// Each skill's (level, xp) last frame, to spot XP gains.
    skill_seen: HashMap<String, (u32, f32)>,
    /// The tip showing now (HintPanel), with the key names filled in.
    hint_text: Option<String>,
    /// The camp's trader, banker, guards and residents.
    camp_npcs: Option<camp_npcs::CampNpcs>,
    /// The camp guards' rifles (Godot SettlementGuard).
    camp_guards: Option<game::CampGuards>,
    /// Deer, rabbits and boars (play scene).
    wildlife: Option<wildlife::Wildlife>,
    /// The player's dog and strays.
    dogs: Option<dog_view::DogView>,
    /// The camp's car and any others (game::vehicles).
    cars: Option<game::Cars>,
    /// This frame's driving keys (only while in a car).
    car_input: game::CarInput,
    /// Solid boxes for parked cars (removed while driven).
    car_solids: car_solid::ParkedCarSolids,
    /// First-person weapon bob and kick.
    fp_sway: fp_weapon::FpSway,
    /// The car model's trunk and headlight markers (model space): trunk, left, right.
    car_markers: (Vec3, Vec3, Vec3),
    /// The car whose trunk screen is open.
    open_trunk: Option<String>,
    /// Zombie groans and attack snarls near the player (Godot ZombieVoices).
    zombie_voices: Option<game::ZombieVoices>,
    /// Zombies waiting inside buildings (Godot PoiManager; visited buildings saved in worldstate).
    poi_zombies: Option<game::PoiZombies>,
    /// Each building model's spawn markers (model space), read from the pack once.
    poi_markers: HashMap<String, Vec<Vec3>>,
    /// Zombie voice sounds started (stats).
    voices_played: u32,
    /// Buildings cleared this session (stats).
    pois_cleared: u32,
    /// This frame's keys for the rebindable actions.
    keys: KeyMap,
    /// The player was dead last frame (a death is counted once).
    was_dead: bool,
    /// Axe hits and felled trees (Godot TreeFelling, saved in worldstate).
    trees: game::TreeFelling,
    /// Trees falling over right now: id -> (chunk, slot).
    falling: HashMap<String, (glam::IVec2, streaming::TreeSlot)>,
    /// Sound (silent when muted or without a sound device).
    audio: Option<audio::Audio>,
    /// Seconds to the next footstep.
    step_timer: f32,
}

use winit::keyboard::KeyCode;

/// The keys of the rebindable actions this frame (Godot KeyBindings; Settings > Controls).
#[derive(Clone)]
struct KeyMap {
    movement: game::InputBindings,
    interact: KeyCode,
    reload: KeyCode,
    dog_stay: KeyCode,
    inventory: KeyCode,
    skills: KeyCode,
    bounties: KeyCode,
    profile: KeyCode,
    map: KeyCode,
    slots: [KeyCode; 6],
}

impl KeyMap {
    fn from_bindings(kb: &game::KeyBindings) -> Self {
        let k = |action: &str, fallback: KeyCode| kb.keycode_of(action).unwrap_or(fallback);
        Self {
            movement: game::InputBindings::from_keybindings(kb),
            interact: k("interact", KeyCode::KeyE),
            reload: k("reload", KeyCode::KeyR),
            dog_stay: k("dog_stay", KeyCode::KeyG),
            inventory: k("inventory", KeyCode::Tab),
            skills: k("toggle_skills", KeyCode::KeyL),
            bounties: k("toggle_bounties", KeyCode::KeyJ),
            profile: k("toggle_profile", KeyCode::KeyP),
            map: k("toggle_map", KeyCode::KeyM),
            slots: [
                k("slot_1", KeyCode::Digit1),
                k("slot_2", KeyCode::Digit2),
                k("slot_3", KeyCode::Digit3),
                k("slot_4", KeyCode::Digit4),
                k("slot_5", KeyCode::Digit5),
                k("slot_6", KeyCode::Digit6),
            ],
        }
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        Self::from_bindings(&game::KeyBindings::default())
    }
}

/// The pack model name of a weapon ("res://assets/models/weapons/bat.glb" -> "bat").
fn weapon_model_name(def: &game::WeaponDef) -> Option<String> {
    let file = def.model.rsplit('/').next()?;
    Some(file.trim_end_matches(".glb").to_string()).filter(|s| !s.is_empty())
}

impl R0 {
    /// A camp screen or the hub is open: the mouse is free and the player's hands are busy.
    fn screen_open(&self) -> bool {
        self.camp_ui.is_some() || self.hub.is_some()
    }

    /// Puts a weapon in hand (or empties it): the swing uses its numbers, the model shows in the hand.
    fn apply_hand(&mut self, change: play::HandChange) {
        let def = match change {
            play::HandChange::Weapon(def) => def,
            play::HandChange::ToggleFlashlight => {
                self.flashlight_on = !self.flashlight_on;
                return;
            }
            play::HandChange::PlaceCampfire => {
                // Godot CampfirePlacer: 1.5 m in front on flat ground, away from other fires
                if let (Some(play), Some(p)) = (self.play.as_mut(), self.player_ctrl.as_ref()) {
                    let facing = Vec3::new(-p.yaw.sin(), 0.0, -p.yaw.cos());
                    let physics = &self.physics;
                    let ground = |spot: Vec3| physics.raycast(spot + Vec3::Y * 3.0, Vec3::NEG_Y, 6.0).map(|h| (h.point, h.normal));
                    let text = match play.camp.campfires.place(p.position, facing, Some(&mut play.inventory), ground) {
                        Ok(_) => "Campfire placed".to_string(),
                        Err(e) => e.message().to_string(),
                    };
                    play.hotbar.on_inventory_changed(&play.inventory);
                    play.show_toast(text);
                }
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
        let mut file = session::capture(self.loaded_save.as_ref(), &snap);
        if let Some(d) = &self.dogs {
            file.systems.extra.insert("dog".into(), d.sim.to_save());
        }
        if let Some(c) = &self.cars {
            file.systems.extra.insert("cars".into(), c.to_save());
        }
        let mut ws = file.systems.worldstate.clone().unwrap_or_default();
        ws.fallen_trees = serde_json::from_value(self.trees.to_save()["fallen_trees"].clone()).unwrap_or_default();
        if let Some(pz) = &self.poi_zombies {
            ws.visited_pois = pz.visited_ids();
        }
        file.systems.worldstate = Some(ws);
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
        self.camp_npcs = None;
        self.camp_guards = None;
        self.wildlife = None;
        self.dogs = None;
        self.cars = None;
        self.car_solids = car_solid::ParkedCarSolids::default();
        self.zombie_voices = Some(game::ZombieVoices::new(self.seed));
        self.poi_zombies = Some(game::PoiZombies::new(self.seed));
        self.trees = game::TreeFelling::new();
        self.trees.set_seed(self.seed as u64);
        self.falling.clear();
        self.skill_seen.clear();
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
            let mut rig = CameraRig::new();
            let mut player = Player::new(self.player);
            // Start facing where the camera looks (away from it), not toward it
            player.yaw = rig.yaw;
            self.player_ctrl = Some(player);
            if self.test_first_person {
                rig.is_first_person = true;
            }
            self.camera_rig = Some(rig);
        }

        if let Some(pack) = &self.pack {
            match game::zombie::ZombieDefs::load_default() {
                Ok(defs) => self.zombies = Some(zombies::Zombies::new(&gpu.device, pack, defs)),
                Err(e) => log::warn!("data/zombies.json: {e}"),
            }
            if self.scene == "play" {
                match wildlife::Wildlife::new(&gpu.device, pack, self.seed) {
                    Ok(w) => self.wildlife = Some(w),
                    Err(e) => log::warn!("data/animals.json: {e}"),
                }
                match game::Cars::from_repo() {
                    Ok(mut c) => {
                        c.seed = self.seed;
                        self.cars = Some(c);
                        self.car_solids = car_solid::ParkedCarSolids::from_pack(pack);
                        let marker = |name: &str, fallback: Vec3| {
                            pack.find("car_hatchback_red")
                                .and_then(|m| m.markers.iter().find(|k| k.name == name))
                                .map_or(fallback, |k| Vec3::from(k.translation))
                        };
                        self.car_markers = (
                            marker("trunk", trunk_play::DEFAULT_TRUNK_LOCAL),
                            marker("headlight_L", Vec3::new(-0.55, 0.66, -1.93)),
                            marker("headlight_R", Vec3::new(0.55, 0.66, -1.93)),
                        );
                    }
                    Err(e) => log::warn!("data/vehicles.json: {e}"),
                }
                match dog_view::DogView::new(&gpu.device, pack) {
                    Ok(d) => self.dogs = Some(d),
                    Err(e) => log::warn!("data/dog.json: {e}"),
                }
                self.combat = Some(combat::Combat::new("bat"));
                match play::PlayState::new(self.seed as u64) {
                    Ok(state) => self.play = Some(state),
                    Err(e) => log::warn!("play systems not loaded: {e}"),
                }
                // Godot's starter weapon is the bat (data/start_inventory.json)
                self.player_visual = Some(player_visual::PlayerVisual::load(&gpu.device, pack, "human_player", Some("bat")));
            }
        }
        // The camp's people (play scene)
        if let (Some(pack), true) = (&self.pack, self.scene == "play") {
            match game::CampLifeDirector::from_data_dir(camp.clone(), paths::repo_path("data")) {
                Ok(director) => self.camp_npcs = Some(camp_npcs::CampNpcs::new(&gpu.device, pack, director)),
                Err(e) => log::warn!("camp life not loaded: {e}"),
            }
            match game::CampGuards::from_repo() {
                Ok(g) => self.camp_guards = Some(g.with_camp_center(camp.center).with_seed(self.seed)),
                Err(e) => log::warn!("camp guards not loaded: {e}"),
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
            if let (Some(c), Some(v)) = (self.cars.as_mut(), file.systems.extra.get("cars")) {
                c.from_save(v);
            }
            if let Some(ws) = &file.systems.worldstate {
                self.trees.from_save(&serde_json::json!({ "fallen_trees": ws.fallen_trees }));
                if let Some(pz) = self.poi_zombies.as_mut() {
                    pz.from_save(&serde_json::json!({ "visited_pois": ws.visited_pois }));
                }
            }
            if let (Some(d), Some(v)) = (self.dogs.as_mut(), file.systems.extra.get("dog")) {
                d.sim.from_save(v);
            }
            if let Some(play) = self.play.as_mut() {
                session::restore_play(file, play);
            }
        }
        self.climb = game::LadderClimb::default();
        // The camp's parked car outside the gate, half a tank (Godot CarManager.EnsureCampCar)
        if let (Some(cars), Some(site)) = (self.cars.as_mut(), world_gen.settlements().first())
            && !cars.has_record(game::CAMP_CAR_ID)
        {
            let local = site.center + site.local(7.0, 0.0, -27.0);
            let at = Vec3::new(local.x, world_gen.height(local.x, local.z) + 1.0, local.z);
            let cfg = cars.config.clone();
            cars.spawn_car(game::CAMP_CAR_ID, &cfg.camp_colour, at, site.yaw, cfg.camp_fuel_frac * cfg.tank_litres, cfg.health);
        }
        if self.campfire_meshes.is_none() {
            self.campfire_meshes = Some(campfire_mesh::CampfireMeshes::new(&gpu.device));
        }
        // The camp's four beds (a loaded save may already have them) and its fire (the model's marker)
        if let Some(play) = self.play.as_mut() {
            if let (Some(pack), Some(site)) = (self.pack.as_ref(), world_gen.settlements().first())
                && let Some(m) = pack.find("settlement")
                && let Some(fire) = game::settlement_campfire_from_markers(&m.markers, Mat4::from_translation(site.center) * Mat4::from_rotation_y(site.yaw))
            {
                play.camp.campfires.add(fire);
            }
            play.camp.farm.ensure_camp_plots(world_gen.settlements());
            for (id, n) in &self.give {
                play.inventory.add(id, *n);
            }
        }
        if let (Some(yaw), Some(p), Some(rig)) = (self.start_yaw, self.player_ctrl.as_mut(), self.camera_rig.as_mut()) {
            p.yaw = yaw;
            rig.yaw = yaw;
        }
        // --animals=...,campfire also lays a burning fire 4 m ahead (screenshots)
        if let (true, Some(play), Some(p)) = (self.near_animals.iter().any(|a| a == "campfire"), self.play.as_mut(), self.player_ctrl.as_ref()) {
            let at = self.player + Vec3::new(-p.yaw.sin(), 0.0, -p.yaw.cos()) * 4.0;
            play.camp.campfires.add(game::Campfire::create(Vec3::new(at.x, world_gen.height(at.x, at.z), at.z)));
        }
        self.near_animals.retain(|a| a != "campfire");
        // ...and `dog` gives the player a grown dog beside them
        if let (true, Some(d), Some(p)) = (self.near_animals.iter().any(|a| a == "dog"), self.dogs.as_mut(), self.player_ctrl.as_ref()) {
            let at = self.player + Vec3::new(p.yaw.cos(), 0.0, -p.yaw.sin()) * 2.0;
            let health = d.sim.config.max_health;
            d.sim.spawn_dog(Vec3::new(at.x, world_gen.height(at.x, at.z), at.z), health, game::DogMode::Follow, "Rex", self.clock.day, game::DogGrowth::NO_START);
        }
        self.near_animals.retain(|a| a != "dog");
        // ...and `drive` seats the player in the camp's car (with --autowalk it drives off)
        if let (true, Some(cars)) = (self.near_animals.iter().any(|a| a == "drive"), self.cars.as_mut())
            && let Some(car) = cars.get_mut(game::CAMP_CAR_ID)
        {
            car.set_driven(true);
            cars.driven_car_id = Some(game::CAMP_CAR_ID.to_string());
        }
        self.near_animals.retain(|a| a != "drive");
        if let (Some(w), Some(p)) = (self.wildlife.as_mut(), self.player_ctrl.as_ref()) {
            let ahead = Vec3::new(-p.yaw.sin(), 0.0, -p.yaw.cos());
            let side = Vec3::new(-ahead.z, 0.0, ahead.x);
            let n = self.near_animals.len() as f32;
            for (i, species) in self.near_animals.iter().enumerate() {
                let at = self.player + ahead * 9.0 + side * (i as f32 - (n - 1.0) / 2.0) * 3.0;
                w.sim.spawn_at(species, Vec3::new(at.x, world_gen.height(at.x, at.z), at.z), None, None);
            }
        }
        self.loaded_save = save_file;
        let world_gen = Arc::new(world_gen);
        let mut streamer = streaming::Streamer::new(world_gen.clone());
        streamer.felled = self.trees.fallen.iter().filter_map(|id| game::parse_tree_id(id)).collect();
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
        // Scripted runs (screenshots, timed runs, stats) are silent, so tests never make noise
        let muted = args.flag("--mute") || ["--shot", "--seconds", "--frames", "--stats"].iter().any(|f| args.value(f).is_some());
        if self.scene == "play" {
            self.audio = Some(audio::Audio::new(muted));
        }
        // Test option: open a camp screen at the start (shop, bank, crafting, bounties)
        self.hub = args.value("--hub").and_then(|h| {
            use hub_ui::HubTab as T;
            let tab = match h {
                "bag" => T::Bag,
                "skills" => T::Skills,
                "bounties" => T::Bounties,
                "profile" => T::Profile,
                "map" => T::Map,
                _ => return None,
            };
            Some(hub_ui::HubUiState::new(hub_ui::HubScreen::Tab(tab)))
        });
        self.camp_ui = match args.value("--camp-screen") {
            Some("shop") => Some(camp_ui::CampUiState::new(camp_ui::CampScreen::Shop)),
            Some("bank") => Some(camp_ui::CampUiState::new(camp_ui::CampScreen::Bank)),
            Some("crafting") => Some(camp_ui::CampUiState::new(camp_ui::CampScreen::Crafting)),
            Some("bounties") => Some(camp_ui::CampUiState::new(camp_ui::CampScreen::Bounties)),
            _ => None,
        };
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
        self.test_climb = args.flag("--climb");
        self.test_chop = args.flag("--chop");
        self.test_first_person = args.flag("--first-person");
        self.test_poi = args.flag("--poi");
        self.near_animals = args.value("--animals").map(|s| s.split(',').filter(|e| !e.is_empty()).map(str::to_string).collect()).unwrap_or_default();
        self.start_yaw = args.value("--yaw").and_then(|s| s.parse::<f32>().ok()).map(f32::to_radians);
        self.give = args
            .value("--give")
            .map(|s| {
                s.split(',')
                    .filter(|e| !e.is_empty())
                    .map(|e| match e.split_once(':') {
                        Some((id, n)) => (id.to_string(), n.parse().unwrap_or(1)),
                        None => (e.to_string(), 1),
                    })
                    .collect()
            })
            .unwrap_or_default();

        match pack::ModelPack::load(paths::repo_path("rust/assets/models.pack")) {
            Ok(pack) => self.pack = Some(pack),
            Err(e) => log::warn!("no model pack ({e}); run `cargo run --release -p baker`"),
        }
        self.start_world(gpu, save_file);
        // Test option: take out hotbar slot N at the start (e.g. --slot=2 for the pistol)
        // Test option --chop: the axe in hand
        if self.test_chop
            && let Some(axe) = self.play.as_ref().and_then(|p| p.weapons.get("axe").cloned())
        {
            self.apply_hand(play::HandChange::Weapon(Some(axe)));
        }
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
            // Tab opens crafting (Godot's inventory screen), J the bounties; again closes them.
            // Shop and bank open by talking to the camp's trader and banker.
            // The hub's keys (Godot project.godot): Tab bag, L skills, J bounties, P profile, M map; the
            // same key again closes it
            use hub_ui::{HubScreen, HubTab};
            // The player's own keys (Settings > Controls)
            self.keys = self.menus.as_ref().map(|m| KeyMap::from_bindings(&m.keys)).unwrap_or_default();
            let keys = self.keys.clone();
            if let Some(m) = self.menus.as_mut() {
                m.pressed_key = input.pressed_keys().into_iter().next();
            }
            for (key, tab) in [(keys.inventory, HubTab::Bag), (keys.skills, HubTab::Skills), (keys.bounties, HubTab::Bounties), (keys.profile, HubTab::Profile), (keys.map, HubTab::Map)] {
                if input.pressed(key) && self.camp_ui.is_none() && !self.menus.as_ref().is_some_and(|m| m.is_open()) {
                    self.hub = match &self.hub {
                        Some(h) if h.screen() == HubScreen::Tab(tab) => None,
                        _ => Some(hub_ui::HubUiState::new(HubScreen::Tab(tab))),
                    };
                }
            }
            // Esc closes an open camp or hub screen (the screen itself sees the key), otherwise the menus
            if input.pressed(winit::keyboard::KeyCode::Escape)
                && !self.screen_open()
                && !self.menus.as_ref().is_some_and(|m| m.keys_ui.is_waiting())
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

            let mut actions = Actions::from_input(input, &keys.movement);
            if !self.cursor_captured || self.screen_open() {
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
            // G: the dog stays or follows again (Godot dog_stay)
            if input.pressed(keys.dog_stay)
                && !self.screen_open()
                && let Some(d) = self.dogs.as_mut()
                && d.sim.has_dog()
            {
                d.sim.toggle_stay();
                let mode = d.sim.dog().map(|g| format!("{:?}", g.mode())).unwrap_or_default();
                if let Some(play) = self.play.as_mut() {
                    play.show_toast(if mode == "Stay" { "Dog: stay".into() } else { "Dog: follow".into() });
                }
            }
            if let Some(change) = self.play.as_mut().and_then(|p| p.hotbar_keys(input, &keys.slots)) {
                self.apply_hand(change);
            }
            let attack = !dead && !self.screen_open() && (self.autoattack || (self.cursor_captured && input.mouse_pressed(winit::event::MouseButton::Left)));
            if let (Some(p), Some(rig)) = (self.player_ctrl.as_mut(), self.camera_rig.as_mut()) {
                rig.update(&actions, p.crouching, dt);
                p.first_person = rig.is_first_person;
                // In a car: the keys drive it, E gets out, the player rides in the driver's seat
                self.car_input = game::CarInput::default();
                self.e_used = false;
                let driven = self.cars.as_ref().and_then(|c| c.driven_car_id.clone());
                if let (Some(id), Some(cars)) = (driven, self.cars.as_mut()) {
                    use winit::keyboard::KeyCode as K;
                    if input.pressed(self.keys.interact) && self.hub.is_none() && self.camp_ui.is_none() {
                        let spot = cars.exit(&id, &self.physics);
                        p.position = spot + Vec3::Y * 0.1;
                        p.velocity = Vec3::ZERO;
                        self.e_used = true;
                    } else if let Some(car) = cars.get(&id) {
                        self.car_input = game::CarInput {
                            throttle: -actions.move_vec.y,
                            steer: actions.move_vec.x,
                            handbrake: input.held(K::Space),
                            horn: input.held(K::KeyH),
                            lights_toggle: input.pressed(K::KeyL),
                            ..Default::default()
                        };
                        let f = car.body.rotation * Vec3::NEG_Z;
                        let car_yaw = (-f.x).atan2(-f.z);
                        p.position = car.body.position + Vec3::Y * 0.3;
                        p.velocity = Vec3::ZERO;
                        p.yaw = car_yaw;
                        // The camera swings back behind the car unless the mouse turns it
                        if actions.look_delta.0.abs() < 0.01 {
                            let diff = (car_yaw - rig.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                            rig.yaw += diff * (dt * 2.5).min(1.0);
                        }
                        self.e_used = input.pressed(self.keys.interact);
                    }
                }
                let in_car = self.cars.as_ref().is_some_and(|c| c.driven_car_id.is_some());
                // Ladders first: while climbing, the ladder moves the player instead of walking
                let mut grab = input.pressed(self.keys.interact) && self.camp_ui.is_none() && self.hub.is_none() && !dead;
                if self.test_poi && self.time > 1.0 {
                    self.test_poi = false;
                    let me = p.position;
                    let nearest = self.streamer.as_ref().and_then(|s| {
                        s.chunks.values().flat_map(|c| c.pois.iter()).min_by(|a, b| a.position.distance(me).total_cmp(&b.position.distance(me))).map(|q| q.position)
                    });
                    if let Some(poi) = nearest {
                        let away = Vec3::new(me.x - poi.x, 0.0, me.z - poi.z).normalize_or(Vec3::X);
                        let at = poi + away * 20.0;
                        p.position = Vec3::new(at.x, self.physics.ground_height(at.x, at.z).unwrap_or(at.y) + 0.05, at.z);
                        p.yaw = away.x.atan2(away.z);
                        rig.yaw = p.yaw;
                    }
                }
                if self.test_chop && self.time > 1.0 {
                    self.test_chop = false;
                    let me = p.position;
                    let nearest = self.streamer.as_ref().and_then(|s| {
                        s.chunks.values().flat_map(|c| c.trees.iter()).min_by(|a, b| a.position.distance(me).total_cmp(&b.position.distance(me))).map(|t| t.position)
                    });
                    if let Some(tree) = nearest {
                        let away = Vec3::new(me.x - tree.x, 0.0, me.z - tree.z).normalize_or(Vec3::X);
                        let at = tree + away * 1.3;
                        p.position = Vec3::new(at.x, self.physics.ground_height(at.x, at.z).unwrap_or(at.y) + 0.05, at.z);
                        p.yaw = away.x.atan2(away.z);
                        rig.yaw = p.yaw;
                    }
                }
                if self.test_climb && self.time > 1.0 {
                    self.test_climb = false;
                    let nearest = self.streamer.as_ref().and_then(|s| {
                        s.chunks.values().flat_map(|c| c.ladders.iter()).min_by(|a, b| a.foot.distance(p.position).total_cmp(&b.foot.distance(p.position))).cloned()
                    });
                    if let Some(rail) = nearest {
                        p.position = rail.foot_interaction_point();
                        grab = true;
                    }
                }
                let rails: Vec<game::LadderRail> = if grab || self.climb.is_active() {
                    self.streamer.as_ref().map(|s| s.chunks.values().flat_map(|c| c.ladders.iter().cloned()).collect()).unwrap_or_default()
                } else {
                    Vec::new()
                };
                let was_climbing = self.climb.is_active();
                let climb_in = game::ClimbInput::new(-actions.move_vec.y, actions.jump, grab);
                let climbing = !dead && !in_car && game::climb_tick(&mut self.climb, p, &climb_in, &rails, dt);
                self.e_used = self.e_used || (grab && (climbing || was_climbing));
                if in_car {
                    // the car moves the player (above)
                } else if climbing {
                    rig.yaw = p.yaw;
                } else {
                    p.update(&actions, rig.yaw, &self.physics, &self.mover, dt);
                }
                let mut swung = None;
                if let (Some(c), Some(z)) = (self.combat.as_mut(), self.zombies.as_mut()) {
                    let animal_bonus = self.play.as_ref().map_or(0.0, |pl| pl.camp.skills.get_bonus("animalDamage"));
                    let mut prey = wildlife::Prey { zombies: z, animals: self.wildlife.as_mut().map(|w| &mut w.sim), animal_bonus };
                    let ev = c.update(dt, attack, p, &mut prey, &self.physics);
                    swung = ev.swung.clone();
                    if ev.swing_started.is_some() {
                        self.fp_sway.trigger_swing();
                    }
                    self.fp_sway.update(dt, p.horizontal_speed());
                    if let Some(a) = self.audio.as_mut() {
                        if ev.swing_started.is_some()
                            && let Some(sound) = c.weapon_id().and_then(|w| a.data.weapon_sound(w, "swing")).map(str::to_string)
                        {
                            a.play_at(&sound, p.position);
                        }
                        if c.hits > 0 {
                            a.event("meleeHit", Some(p.position));
                        }
                    }
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
                // An axe swing that lands near a tree chops it (Godot MeleeHandler + TreeStrike/TreeFelling)
                if let Some((weapon, dir)) = swung
                    && weapon.tags.iter().any(|t| t == "chop")
                    && let Some(st) = self.streamer.as_mut()
                {
                    let me = p.position;
                    let hit = {
                        let near = st.chunks.iter().flat_map(|(c, ch)| {
                            ch.trees
                                .iter()
                                .filter(move |t| t.position.distance_squared(me) < 36.0)
                                .map(move |t| game::TreeCandidate::new(*c, t.index, t.position, &t.model))
                        });
                        game::find_tree_strike(me + Vec3::Y, dir, weapon.range, weapon.arc_deg, near)
                    };
                    if let Some(hit) = hit {
                        let id = game::tree_id(hit.chunk, hit.index);
                        let model = st.chunks.get(&hit.chunk).and_then(|ch| ch.trees.iter().find(|t| t.index == hit.index)).map(|t| t.model.clone()).unwrap_or_default();
                        let r = self.trees.strike(&id, hit.point, &model, Some(&weapon.id), None, me);
                        if let (Some(sound), Some(a)) = (r.sound_event, self.audio.as_mut()) {
                            a.play_at(sound, hit.point);
                        }
                        if r.felled {
                            if let Some(slot) = st.fell(&mut self.physics, hit.chunk, hit.index) {
                                self.falling.insert(id, (hit.chunk, slot));
                            }
                            // The wood lands 1.5 m to the +X side, on the ground (Godot TreeFelling.Fell)
                            if r.wood_given > 0 && let Some(play) = self.play.as_mut() {
                                let (x, z) = (hit.point.x + 1.5, hit.point.z);
                                let y = self.physics.ground_height(x, z).unwrap_or(hit.point.y) + 0.125;
                                play.drop_on_ground("wood", r.wood_given, Vec3::new(x, y, z));
                                play.show_toast(format!("Tree felled: +{} wood", r.wood_given));
                            }
                        }
                    }
                }
                self.player = p.position;
                self.camera = rig.camera(&self.player, &self.physics);
                // Driving: a chase camera further back and higher than on foot (Godot CarCamera)
                if self.cars.as_ref().is_some_and(|c| c.driven_car_id.is_some()) {
                    self.camera.pitch = self.camera.pitch.clamp(-0.35, 0.15);
                    let f = self.camera.forward();
                    let mut at = self.player + Vec3::Y * 2.2 - f * 7.0;
                    // Never under a hill behind the car
                    if let Some(h) = self.physics.ground_height(at.x, at.z) {
                        at.y = at.y.max(h + 1.2);
                    }
                    self.camera.position = at;
                }
                // The player's field of view (Settings), before a gun's aim zoom
                if let Some(m) = &self.menus {
                    self.camera.fov_y_deg = display::apply_fov(self.camera.fov_y_deg, &m.settings);
                }
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
            if let Some(toast) = play.camp.craft_tick(dt, &mut play.inventory, dead) {
                if let Some(a) = self.audio.as_mut() {
                    a.event("craftDone", None);
                }
                play.hotbar.on_inventory_changed(&play.inventory);
                play.show_toast(toast);
            }
        }
        // Guns, crates and pickups (play mode)
        let alive_now = !self.combat.as_ref().is_some_and(|c| c.is_dead());
        if let (Some(play), Some(p)) = (self.play.as_mut(), self.player_ctrl.as_mut())
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
                reload_pressed: input.pressed(self.keys.reload),
                aiming: captured && input.mouse_held(MouseButton::Right),
            };
            let animal_bonus = play.camp.skills.get_bonus("animalDamage");
            if let Some(z) = self.zombies.as_mut()
                && let Some(ev) = play.shoot_tick(
                    dt,
                    shoot,
                    self.camera.position,
                    self.camera.forward(),
                    &mut wildlife::Prey { zombies: z, animals: self.wildlife.as_mut().map(|w| &mut w.sim), animal_bonus },
                    &self.physics,
                )
                && let Some(c) = self.combat.as_mut()
            {
                c.kills += ev.kills;
                c.shots += ev.shots_fired;
                if ev.shots_fired > 0 {
                    self.fp_sway.trigger_shot();
                }
                // No bullets in the gun or the bag: the click swings it as a club (Godot GunBash)
                if ev.bash
                    && let Some(gun) = play.gun.as_ref()
                {
                    c.bash(gun, &mut p.stamina);
                }
                if let Some(a) = self.audio.as_mut() {
                    let at = self.camera.position;
                    if ev.shots_fired > 0
                        && let Some(sound) = play.gun.as_ref().and_then(|g| a.data.weapon_sound(&g.id, "shot")).map(str::to_string)
                    {
                        a.play_at(&sound, at);
                    }
                    if ev.reload_started {
                        a.event("reload", Some(at));
                    }
                    if ev.empty_click {
                        a.event("dryFire", Some(at));
                    }
                }
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
            // E talks to the camp person in front (trader, banker, bounty board) before crates and items
            let npc = self.camp_npcs.as_ref().and_then(|c| c.facing(p.position, facing));
            let pressed = input.pressed(self.keys.interact) && self.camp_ui.is_none() && self.hub.is_none() && !self.e_used;
            play.camp.player_at = p.position;
            // A car within reach: E drives it; with a fuel can, F pours it in (Godot CarDriver / CarFuel)
            let mut car_prompt = None;
            // The back of a parked car: E opens its trunk (Godot PlayerInteractor / TrunkScreen)
            if let (None, Some(cars)) = (&npc, self.cars.as_ref())
                && cars.driven_car_id.is_none()
                && let Some(id) = trunk_play::trunk_in_front(cars, p.position, facing, self.car_markers.0)
            {
                car_prompt = Some(trunk_play::OPEN_TRUNK_PROMPT.to_string());
                if pressed {
                    self.open_trunk = Some(id);
                    self.hub = Some(hub_ui::HubUiState::new(hub_ui::HubScreen::Trunk));
                }
            }
            if let (None, None, Some(cars)) = (&npc, &car_prompt, self.cars.as_mut())
                && cars.driven_car_id.is_none()
                && let Some(id) = cars
                    .live_cars()
                    .filter(|c| !c.condition.is_wrecked() && c.body.position.distance(p.position) < 3.5)
                    .min_by(|a, b| a.body.position.distance(p.position).total_cmp(&b.body.position.distance(p.position)))
                    .map(|c| c.id.clone())
            {
                let can = play.inventory.has("fuel_can", 1) && cars.get(&id).is_some_and(|c| c.tank.fraction() < 1.0);
                car_prompt = Some(if can { "[E] Drive   [F] Pour in fuel".to_string() } else { "[E] Drive".to_string() });
                if input.pressed(KeyCode::KeyF) && can && cars.refuel(&id, &mut play.inventory) {
                    play.show_toast("Poured in 10 litres".into());
                    play.hotbar.on_inventory_changed(&play.inventory);
                }
                if pressed && cars.try_enter(p.position, facing).is_some() {
                    play.show_toast("W/S drive, A/D steer, Space handbrake, H horn, L lights, E get out".into());
                    car_prompt = None;
                }
            }
            let npc = if car_prompt.is_some() { None } else { npc };
            // A corpse in front (or one being butchered) comes next: E starts the butchering timer
            let mut corpse_prompt = None;
            if let (None, Some(w)) = (&npc, self.wildlife.as_mut()) {
                let (prompt, toast) = w.butcher_tick(dt, pressed, p.position, facing, &mut play.inventory, &mut play.camp.skills);
                if let Some(t) = toast {
                    play.show_toast(t);
                    play.hotbar.on_inventory_changed(&play.inventory);
                }
                corpse_prompt = prompt;
            }
            // A campfire in front: E cooks (opens the bag's crafting) or relights it with wood
            let mut fire_prompt = None;
            let mut open_crafting = false;
            if npc.is_none() && corpse_prompt.is_none() {
                let fires = play.camp.campfires.campfires_mut();
                if let Some(fire) = fires.iter_mut().filter(|f| f.distance_flat(p.position) < 2.0).min_by(|a, b| a.distance_flat(p.position).total_cmp(&b.distance_flat(p.position))) {
                    fire_prompt = Some(fire.prompt(Some(&play.inventory)).to_string());
                    if pressed {
                        match fire.interact(&mut play.inventory) {
                            game::CampfireInteractResult::OpenCrafting => open_crafting = true,
                            game::CampfireInteractResult::Relit => {
                                play.show_toast("The fire is burning again".into());
                                play.hotbar.on_inventory_changed(&play.inventory);
                            }
                            game::CampfireInteractResult::NeedsWood => play.show_toast("Needs wood".into()),
                        }
                    }
                }
            }
            if open_crafting {
                let mut hub = hub_ui::HubUiState::new(hub_ui::HubScreen::Tab(hub_ui::HubTab::Bag));
                hub.set_status("Cooking at the campfire");
                self.hub = Some(hub);
            }
            let corpse_prompt = corpse_prompt.or(fire_prompt).or(car_prompt);
            let free = npc.is_none() && corpse_prompt.is_none();
            // A bed in front comes before crates; tilling with a hoe only when nothing else is offered
            let bed = free && play.camp.bed_in_front(p.position, facing).is_some();
            play.interact_tick(dt, pressed && free && !bed, p.position, facing, &near);
            if free && (bed || !play.has_prompt()) {
                let physics = &self.physics;
                let ground = |spot: Vec3| physics.raycast(spot + Vec3::Y * 3.0, Vec3::NEG_Y, 6.0).map(|h| (h.point, h.normal));
                let farm_press = pressed && !play.picked_up;
                if let Some((prompt, toast)) = play.camp.farm_interact(p.position, facing, &mut play.inventory, farm_press, ground) {
                    if !prompt.is_empty() {
                        play.set_prompt(prompt);
                    }
                    if let Some(t) = toast {
                        play.show_toast(t);
                        play.hotbar.on_inventory_changed(&play.inventory);
                    }
                }
            }
            if let Some(text) = corpse_prompt {
                play.set_prompt(text);
            }
            if let (true, Some(a)) = (play.picked_up, self.audio.as_mut()) {
                a.event("pickup", None);
            }
            // E on the player's own dog opens its menu (Godot DogMenu)
            if let (None, Some(d)) = (&npc, self.dogs.as_ref())
                && let Some((_, name)) = d.own_dog_in_front(p.position, facing)
            {
                play.set_prompt(format!("[E] {name}"));
                if pressed {
                    self.hub = Some(hub_ui::HubUiState::new(hub_ui::HubScreen::DogMenu));
                }
            }
            if let (Some((_, role, _)), Some(d)) = (&npc, self.dogs.as_mut())
                && role == "breeder"
            {
                // The breeder sells puppies (Godot DogBreeder)
                let price = d.sim.config.price;
                play.set_prompt(if d.sim.has_dog() { "[E] Dog breeder (you already have a dog)".into() } else { format!("[E] Buy a puppy (${price})") });
                if pressed {
                    let now = (self.clock.day as f64 - 1.0) * 24.0 + self.clock.hour as f64;
                    let text = match d.sim.buy_from_breeder(&mut play.camp.wallet, p.position + facing * 1.5, self.clock.day, now) {
                        Ok(_) => "You bought a puppy. Press E on it for its menu, G to make it stay".to_string(),
                        Err(e) => e,
                    };
                    play.show_toast(text);
                }
            } else if let Some((_, role, prompt)) = npc {
                play.set_prompt(prompt);
                let screen = match role.as_str() {
                    "trader" => Some(camp_ui::CampScreen::Shop),
                    "banker" => Some(camp_ui::CampScreen::Bank),
                    r if r.starts_with("bounty") => Some(camp_ui::CampScreen::Bounties),
                    _ => None,
                };
                if let (true, Some(s)) = (pressed, screen) {
                    self.camp_ui = Some(camp_ui::CampUiState::new(s));
                }
            }
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
            let before = self.clock.day as f32 * 24.0 + self.clock.hour;
            self.clock.advance(dt);
            self.hour = self.clock.hour;
            // Fires burn in game hours (Godot Campfire: 5 h per load of wood)
            let hours = self.clock.day as f32 * 24.0 + self.clock.hour - before;
            if let Some(play) = self.play.as_mut() {
                play.camp.campfires.advance_hours(hours.max(0.0));
            }
        }
        if let Some(play) = self.play.as_mut() {
            for f in play.camp.campfires.campfires_mut() {
                f.visuals.update(dt);
            }
        }
        if let Some(c) = self.camp_npcs.as_mut() {
            let player = alive.then_some(self.player);
            c.update(dt, self.hour, player);
        }
        // Sound: heard from the camera; footsteps while walking on the ground (Footsteps.cs);
        // the day and night ambience cross over with the clock
        if let Some(a) = self.audio.as_mut() {
            a.set_listener(self.camera.position, self.camera.forward());
            if let Some(m) = &self.menus {
                a.set_volumes(m.settings.master_volume, m.settings.music_volume, m.settings.sfx_volume);
            }
            a.tick(dt, !(6.0..20.0).contains(&self.hour));
            if let Some(p) = self.player_ctrl.as_ref() {
                self.step_timer -= dt;
                if p.on_floor && p.horizontal_speed() >= 0.5 && alive {
                    if self.step_timer <= 0.0 {
                        a.play_ui("footstep_grass");
                        self.step_timer = if p.is_sprinting { 0.33 } else { 0.5 };
                    }
                } else {
                    self.step_timer = 0.0;
                }
            }
        }
        // Kills count for bounties; the trader restocks and the board refreshes with the days
        let night = !(6.0..20.0).contains(&self.hour);
        let menu_open = self.camp_ui.is_some() || self.hub.is_some() || self.menus.as_ref().is_some_and(|m| m.is_open());
        if let (Some(play), Some(z)) = (self.play.as_mut(), self.zombies.as_mut()) {
            // Tips for new players: what is near and what the player has (Godot HintTriggers)
            if let Some(hints) = play.hints.as_mut() {
                let mut ctx = game::HintContext::new();
                let at = self.player;
                ctx.nearest_zombie_dist = z.list.iter().filter(|q| !q.health.is_dead()).map(|q| q.position.distance(at)).reduce(f32::min);
                ctx.nearest_crate_dist = self.streamer.as_ref().and_then(|s| s.chunks.values().flat_map(|c| c.crates.iter()).map(|c| c.position.distance(at)).reduce(f32::min));
                if let Some(c) = &self.camp_npcs {
                    for n in c.director.fixed_npcs() {
                        let d = n.position.distance(at);
                        if n.role.starts_with("bounty") {
                            ctx.nearest_board_dist = Some(ctx.nearest_board_dist.map_or(d, |b: f32| b.min(d)));
                        }
                        let e = ctx.npc_distances.entry(n.role.clone()).or_insert(d);
                        *e = e.min(d);
                    }
                }
                ctx.needs.insert("food".into(), play.hunger.current());
                ctx.needs.insert("water".into(), play.thirst.current());
                ctx.hour = self.hour;
                ctx.bag_weight_ratio = Some(play.inventory.total_weight() / play.inventory.max_weight().max(1.0));
                ctx.inventory_items = play.inventory.slots().iter().flatten().map(|s| s.item_id.clone()).collect();
                ctx.is_menu_open = menu_open;
                ctx.is_dead = !alive;
                if let Some(g) = self.world_gen.as_ref() {
                    hints.on_safe_zone_changed(g.settlements().iter().any(|s| s.contains(at, 0.0, g.config())));
                }
                if play.picked_up {
                    hints.on_event("item_picked");
                }
                self.hint_text = hints.tick(dt, &ctx).map(|h| fill_keys(&h.text));
            }
            // Profile numbers: deaths, distance travelled (teleports skipped), kills and headshots
            let dead_now = self.combat.as_ref().is_some_and(|c| c.is_dead());
            if dead_now && !self.was_dead {
                play.stats.record_death();
            }
            self.was_dead = dead_now;
            if !dead_now {
                play.stats.update_position(glam::Vec2::new(self.player.x, self.player.z));
            }
            play.stats.record_day(self.clock.day);
            for (kind, headshot) in z.take_kills() {
                play.stats.record_kill(&kind, headshot);
                play.camp.on_zombie_killed(&kind, night);
                if let Some(a) = self.audio.as_mut() {
                    a.event("zombieDeath", Some(self.player));
                }
            }
            play.camp.tick(self.clock.day, self.hour);
            // XP gains show as toasts, level-ups as a banner (Godot XpToastStack / LevelUpBanner)
            let overlays = self.overlays.get_or_insert_with(|| hub_ui::HubUiState::new(hub_ui::HubScreen::Tab(hub_ui::HubTab::Bag)));
            let first = self.skill_seen.is_empty();
            for def in play.camp.skills.registry().all() {
                let now = (play.camp.skills.get_level(&def.id), play.camp.skills.get_xp(&def.id));
                let before = self.skill_seen.insert(def.id.clone(), now);
                if let (false, Some((level, xp))) = (first, before) {
                    if now.0 > level {
                        overlays.trigger_level_up(&def.name, now.0);
                        overlays.add_xp_toast(&def.id, &def.name, now.1.max(1.0));
                    } else if now.1 > xp + 0.01 {
                        overlays.add_xp_toast(&def.id, &def.name, now.1 - xp);
                    }
                }
            }
            overlays.update_overlays(dt);
            // The map fills in around the player
            play.map.reveal(glam::Vec2::new(self.player.x, self.player.z));
            // Crops grow with the clock; zombies walking over field beds can trample them
            let camp = &mut play.camp;
            camp.farm.sync_clock(&self.clock, Some(&camp.skills));
            if camp.farm.has_vulnerable_crops() && !camp.farm.trample_step(z.active().into_iter().map(|a| (a.id, a.position))).is_empty() {
                play.show_toast("A zombie trampled a crop".into());
            }
        }
        if dt > 0.0 {
            self.fps += (1.0 / dt - self.fps) * (dt * 2.0).min(1.0);
        }
        if let Some(z) = self.zombies.as_mut() {
            let frame = z.update(dt, alive.then_some(self.player), &noises, &self.physics);
            let mut bites: Vec<f32> = frame.bites.iter().map(|b| b.damage).collect();
            // Buildings near the player wake the zombies waiting inside; a building is cleared when they are
            // all dead (Godot PoiManager; clear bounties count it)
            if let (Some(pz), Some(st), Some(pack)) = (self.poi_zombies.as_mut(), self.streamer.as_ref(), self.pack.as_ref()) {
                let markers = &mut self.poi_markers;
                let sites: Vec<game::PoiZombieSite> = st
                    .chunks
                    .values()
                    .flat_map(|c| c.pois.iter())
                    .map(|poi| {
                        let spawns = markers
                            .entry(poi.building_id.clone())
                            .or_insert_with(|| {
                                pack.find(&poi.building_id)
                                    .map(|m| m.markers.iter().filter(|k| k.name.starts_with("spawn_")).map(|k| Vec3::from(k.translation)).collect())
                                    .unwrap_or_default()
                            })
                            .clone();
                        game::PoiZombieSite::new(&poi.id, &poi.building_id, poi.position, poi.yaw, spawns)
                    })
                    .collect();
                let alive = z.list.iter().filter(|q| !q.health.is_dead()).count();
                for spawn in pz.update(self.player, &sites, alive) {
                    if let Some(id) = z.spawn_kind(&spawn.kind, spawn.position) {
                        pz.on_zombie_spawned(&spawn.poi_id, id);
                    }
                }
                // Far behind: gone for good (the building does not wait for them)
                let at: Vec<(u64, Vec3)> = z.list.iter().map(|q| (q.id, q.position)).collect();
                let far = pz.check_despawns(self.player, &at);
                z.despawn(&far);
                let mut cleared: Vec<String> = far.iter().filter_map(|id| pz.on_zombie_despawned(*id)).collect();
                // Dead, or removed by the spawner
                let tracked: Vec<u64> = pz.zombie_to_poi.keys().copied().collect();
                for id in tracked {
                    if z.list.iter().find(|q| q.id == id).is_none_or(|q| q.health.is_dead())
                        && let Some(poi) = pz.on_zombie_died(id)
                    {
                        cleared.push(poi);
                    }
                }
                if let Some(play) = self.play.as_mut() {
                    for poi in cleared {
                        play.camp.bounties.on_poi_cleared(&poi);
                        self.pois_cleared += 1;
                    }
                }
            }
            // Zombies groan now and then and snarl as they attack (Godot ZombieVoices)
            if let (Some(voices), Some(a)) = (self.zombie_voices.as_mut(), self.audio.as_mut()) {
                let list: Vec<game::VoiceZombie> = z
                    .list
                    .iter()
                    .map(|q| {
                        let state = if q.health.is_dead() { game::VoiceState::Dead } else { q.brain.state().into() };
                        game::VoiceZombie::new(q.id, q.position, state)
                    })
                    .collect();
                for ev in voices.update(dt, self.player, &list) {
                    self.voices_played += 1;
                    a.play_at(&ev.sound_id, ev.position);
                }
            }
            // Camp guards shoot zombies near the camp and turn to face them
            if let (Some(g), Some(npcs)) = (self.camp_guards.as_mut(), self.camp_npcs.as_mut()) {
                let posts: Vec<game::GuardPost> = npcs
                    .director
                    .fixed_npcs()
                    .iter()
                    .filter(|n| n.role == "guard")
                    .map(|n| game::GuardPost { id: n.id.clone(), position: n.position, yaw: n.yaw })
                    .collect();
                let targets: Vec<game::GuardTarget> =
                    z.list.iter().map(|q| game::GuardTarget { id: q.id, position: q.position, alive: !q.health.is_dead() }).collect();
                let physics = &self.physics;
                // Clear when nothing solid sits between the eye and the target (the last 0.3 m is the target's ground)
                let clear = |a: Vec3, b: Vec3| {
                    let d = b - a;
                    let len = d.length();
                    len < 0.5 || physics.raycast(a, d / len, len - 0.3).is_none()
                };
                let gf = g.update(dt, &posts, &targets, &clear);
                for shot in gf.shots {
                    let kb = (shot.to - shot.from).normalize_or_zero() * 2.0;
                    z.hit_unclaimed(shot.target_id, shot.damage, kb);
                }
                if let Some(a) = self.audio.as_mut() {
                    for (id, at) in &gf.sounds {
                        a.play_at(id, *at);
                    }
                }
                npcs.aim = gf.yaws.into_iter().collect();
            }
            // Wild animals: graze, flee, boars charge the player and fight zombies (Godot Animal/AnimalSpawner)
            if let (Some(w), Some(p), Some(g)) = (self.wildlife.as_mut(), self.player_ctrl.as_ref(), self.world_gen.as_ref()) {
                let zl: Vec<(u64, Vec3, bool)> = z.list.iter().map(|q| (q.id, q.position, q.health.is_dead())).collect();
                let heard: Vec<(Vec3, f32)> = noises.iter().map(|n| (n.position, n.radius)).collect();
                let biome = match g.biome(self.player.x, self.player.z) {
                    world::BiomeId::Plains => "plains",
                    world::BiomeId::Forest => "forest",
                };
                let no_spawn = |at: Vec3| g.settlements().iter().any(|s| s.contains(at, g.config().safe_zone_padding, g.config()));
                let skills = self.play.as_ref().map(|pl| &pl.camp.skills);
                let ctx = game::AnimalUpdateContext {
                    player_position: Some(self.player),
                    player_forward: Vec3::new(-p.yaw.sin(), 0.0, -p.yaw.cos()),
                    player_is_crouching: p.crouching,
                    player_is_sprinting: p.is_sprinting,
                    player_is_dead: !alive,
                    player_hunting_stealth_bonus: skills.map_or(0.0, |sk| sk.get_bonus("animalStealth")),
                    player_hunting_level: skills.map_or(1, |sk| sk.get_level("hunting")),
                    is_night: !(6.0..20.0).contains(&self.hour),
                    biome_name: Some(biome),
                    noises: &heard,
                    zombies: &zl,
                    in_no_spawn_zone: Some(&no_spawn),
                };
                let af = w.update(dt, &ctx, &self.physics);
                for hit in af.attacks {
                    if hit.is_player {
                        if alive {
                            bites.push(hit.damage);
                        }
                    } else if let Some(target) = hit.target_id {
                        let mut d = game::DamageInfo::new(hit.damage);
                        d.hit_point = hit.hit_point;
                        d.damage_type = "animal".into();
                        z.hit(target, d, hit.knockback);
                    }
                }
                if let Some(a) = self.audio.as_mut() {
                    for (sound, at) in &af.sounds {
                        a.play_at(sound, *at);
                    }
                }
                // Kills by the player give hunting XP and count for hunt bounties
                if let Some(play) = self.play.as_mut() {
                    for species in w.sim.take_kills() {
                        if let Some(xp) = w.sim.config.species(&species).map(|s| s.xp).filter(|&x| x > 0) {
                            play.camp.skills.add_xp("hunting", xp as f32);
                        }
                        play.camp.bounties.on_animal_killed(&species);
                    }
                }
            }
            // The trunk screen shuts when the player walks off or the car drives away (Godot TrunkScreen)
            if let Some(id) = &self.open_trunk {
                let gone = self.cars.as_ref().and_then(|c| c.get(id)).is_none_or(|c| c.is_driven() || c.body.position.distance(self.player) > 3.5);
                let shown = self.hub.as_ref().is_some_and(|h| h.screen() == hub_ui::HubScreen::Trunk);
                if gone || !shown {
                    self.open_trunk = None;
                    if shown {
                        self.hub = None;
                    }
                }
            }
            // Felled trees tip over away from the player, then vanish
            for f in self.trees.update(dt) {
                if let (Some((coord, slot)), Some(st)) = (self.falling.get(&f.id), self.streamer.as_mut())
                    && let Some(pl) = st.chunks.get_mut(coord).and_then(|ch| slot.placed.and_then(|i| ch.placed.get_mut(i)))
                {
                    let scale = if f.finished { 0.0001 } else { slot.scale };
                    let axis = Vec3::Y.cross(f.fall_dir).normalize_or(Vec3::X);
                    pl.matrix = Mat4::from_translation(slot.position)
                        * Mat4::from_axis_angle(axis, f.tilt_angle)
                        * Mat4::from_rotation_y(slot.yaw)
                        * Mat4::from_scale(Vec3::splat(scale));
                }
                if f.finished {
                    self.falling.remove(&f.id);
                }
            }
            // Cars: parked ones only live on loaded ground; the driven one takes the keys and runs things over
            if let Some(cars) = self.cars.as_mut() {
                cars.is_night = !(6.0..20.0).contains(&self.hour);
                if let Some(st) = &self.streamer {
                    for c in st.chunks.keys() {
                        cars.respawn_chunk(*c);
                    }
                    let gone: Vec<glam::IVec2> = cars
                        .live_cars()
                        .filter(|c| !c.is_driven())
                        .map(|c| game::chunk_of(c.body.position, WorldGenerator::CHUNK_SIZE))
                        .filter(|k| !st.chunks.contains_key(k))
                        .collect();
                    for k in gone {
                        cars.store_chunk(k);
                    }
                }
                let cf = cars.update(dt, &self.car_input, &self.physics);
                let solids: Vec<(String, Mat4, bool)> = cars.live_cars().map(|c| (c.id.clone(), c.draw_state().world_matrix, c.is_driven())).collect();
                self.car_solids.sync(&mut self.physics, &solids);
                let mut targets: Vec<(u64, Vec3)> = z.list.iter().filter(|q| !q.health.is_dead()).map(|q| (q.id, q.position)).collect();
                if let Some(w) = &self.wildlife {
                    targets.extend(w.sim.list.iter().filter(|a| !a.is_dead() && !a.retired).map(|a| (a.id, a.position)));
                }
                for (id, dmg, kb) in cars.run_over(&targets) {
                    match self.wildlife.as_mut() {
                        Some(w) if id >= wildlife::FIRST_ANIMAL_ID => {
                            w.sim.hit(id, dmg, kb);
                        }
                        _ => {
                            z.hit(id, dmg, kb);
                        }
                    }
                    if let Some(a) = self.audio.as_mut() {
                        a.play_at("car_impact", self.player);
                    }
                }
                if let Some(a) = self.audio.as_mut() {
                    if cf.horn && ((self.time * 4.0) as u32).is_multiple_of(4) {
                        a.play_at("car_horn", self.player);
                    }
                    if cf.crash {
                        a.play_at("car_impact", self.player);
                    }
                }
                // The gauge (Godot CarGauge) sits where the E prompt goes while driving
                if let (Some(car), Some(play)) = (cars.driven_car(), self.play.as_mut()) {
                    let cfg = &cars.config;
                    play.set_prompt(game::describe_gauge(
                        car.speed_kmh(),
                        car.tank.litres(),
                        car.tank.fraction(),
                        car.condition.current(),
                        cfg.health,
                        car.condition.is_wrecked(),
                        car.tank.is_empty(),
                    ));
                }
            }
            // The dog follows, barks and bites zombies and animals (Godot Dog / DogCombat)
            // Buildings near the player go on the map (Godot MapBuilding / WorldMap)
            if let (Some(st), Some(play), Some(g)) = (self.streamer.as_ref(), self.play.as_mut(), self.world_gen.as_ref()) {
                play.note_buildings(self.player, st.chunks.values().map(|c| (c.pois.as_slice(), c.crates.as_slice())), g.poi_config());
            }
            // Stray dogs wait at the loaded POI buildings
            let dog_sites: Vec<game::DogPoiSite> = match (self.streamer.as_ref(), self.world_gen.as_ref()) {
                (Some(st), Some(g)) => st.chunks.values().flat_map(|c| poi_links::build_dog_poi_sites(&c.pois, g.poi_config())).collect(),
                _ => Vec::new(),
            };
            if let (Some(d), Some(p)) = (self.dogs.as_mut(), self.player_ctrl.as_ref()) {
                let zl: Vec<(u64, Vec3, bool)> = z.list.iter().map(|q| (q.id, q.position, q.health.is_dead())).collect();
                let al: Vec<(u64, Vec3, bool)> = self
                    .wildlife
                    .as_ref()
                    .map(|w| w.sim.list.iter().filter(|a| !a.retired).map(|a| (a.id, a.position, a.is_dead())).collect())
                    .unwrap_or_default();
                let camp = self.world_gen.as_ref().and_then(|g| g.settlements().first().map(|s| s.respawn_point(g.config())));
                let ctx = game::DogUpdateContext {
                    player_position: Some(self.player),
                    player_forward: Vec3::new(-p.yaw.sin(), 0.0, -p.yaw.cos()),
                    player_dead: !alive,
                    player_sprinting: p.is_sprinting,
                    player_velocity: p.velocity,
                    zombies: &zl,
                    animals: &al,
                    is_night: !(6.0..20.0).contains(&self.hour),
                    camp_position: camp,
                    now_hours: (self.clock.day as f64 - 1.0) * 24.0 + self.clock.hour as f64,
                    day: self.clock.day,
                    poi_sites: &dog_sites,
                    world_seed: self.seed,
                };
                let f = d.update(dt, &ctx, &self.physics);
                for bite in f.bites {
                    match self.wildlife.as_mut() {
                        Some(w) if bite.target_id >= wildlife::FIRST_ANIMAL_ID => {
                            w.sim.hit(bite.target_id, bite.damage, bite.knockback);
                        }
                        _ => {
                            z.hit(bite.target_id, bite.damage, bite.knockback);
                        }
                    }
                }
                if let Some(a) = self.audio.as_mut() {
                    for (sound, at) in &f.sounds {
                        a.play_at(sound, *at);
                    }
                }
                if let (Some(t), Some(play)) = (f.toasts.last(), self.play.as_mut()) {
                    play.show_toast(t.clone());
                }
            }
            if let Some(c) = self.combat.as_mut() {
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
                } else if ev.bitten {
                    if let Some(pv) = self.player_visual.as_mut() {
                        pv.animator.play_hit();
                    }
                    if let Some(a) = self.audio.as_mut() {
                        a.event("playerHurt", None);
                    }
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
        // Farm beds: the soil, and the crop's model for its stage (Godot PlotNode)
        let mut beds: Vec<props::Placed> = Vec::new();
        if let (Some(play), Some(pack)) = (self.play.as_ref(), self.pack.as_ref()) {
            for v in play.camp.farm.plot_views() {
                if let Some(slot) = self.library.get(&gpu.device, pack, "soil_plot", false) {
                    beds.push(self.library.place(slot, v.position, 0.0, 1.0));
                }
                if let Some(model) = &v.stage_model
                    && let Some(slot) = self.library.get(&gpu.device, pack, model, false)
                {
                    beds.push(self.library.place(slot, v.position, 0.0, 1.0));
                }
            }
        }
        // Cars: the hatchback model at the body's place (its wheels are part of the model for now)
        if let (Some(cars), Some(pack)) = (self.cars.as_ref(), self.pack.as_ref()) {
            for d in cars.draw_states() {
                if let Some(slot) = self.library.get(&gpu.device, pack, &d.model_name, false) {
                    let mut placed = self.library.place(slot, Vec3::ZERO, 0.0, 1.0);
                    placed.centre = d.world_matrix.transform_point3(placed.centre);
                    placed.matrix = d.world_matrix;
                    beds.push(placed);
                }
            }
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
        let placed = streamer.chunks.values().flat_map(|c| c.placed.iter()).chain(beds.iter());
        let (mut main, mut shadow) = self.library.batches(
            placed,
            &self.camera,
            target.aspect(),
            renderer.shadows.distance,
        );
        let driving = self.cars.as_ref().is_some_and(|c| c.driven_car_id.is_some());
        if let (Some(pv), Some(pose), false) = (self.player_visual.as_mut(), player_pose, driving) {
            let loco = humanoid::Locomotion { speed: pose.horizontal_speed, on_floor: pose.on_floor, crouching: pose.crouching };
            // The player's yaw already uses the model's facing (checked in a screenshot: no extra turn)
            let (body, weapon) = pv.pose(self.last_dt, pose.position, pose.yaw, loco, &mut self.bones);
            // First person: the body only casts its shadow (like Godot's shadows-only body)
            if !pose.first_person {
                main.push(engine::render::SkinnedBatch { mesh: body.mesh, instances: body.instances.clone() });
                if let Some(w) = &weapon {
                    main.push(engine::render::SkinnedBatch { mesh: w.mesh, instances: w.instances.clone() });
                }
            } else if let Some(w) = &weapon
                && let Some(def) = self.play.as_ref().and_then(|p| p.gun.as_ref()).or_else(|| self.combat.as_ref().and_then(|c| c.weapon_def()))
            {
                // First person: the weapon sits in front of the camera (Godot FirstPersonMount)
                let at = fp_weapon::fp_weapon_matrix_with_physics(&self.camera, def, self.fp_sway, &self.physics);
                main.push(engine::render::SkinnedBatch { mesh: w.mesh, instances: vec![engine::render::SkinInstance::new(at, 0)] });
            }
            shadow.push(body);
            shadow.extend(weapon);
        }
        let people = self.camp_npcs.as_ref().map(|c| c.pose(Some(self.player), &mut self.bones)).unwrap_or_default();
        shadow.extend(people.iter().map(|b| engine::render::SkinnedBatch { mesh: b.mesh, instances: b.instances.clone() }));
        main.extend(people);
        let zombies = self.zombies.as_ref().map(|z| z.pose(&mut self.bones)).unwrap_or_default();
        let mut animals = self.wildlife.as_ref().map(|w| w.pose(&mut self.bones)).unwrap_or_default();
        animals.extend(self.dogs.as_ref().map(|d| d.pose(&mut self.bones)).unwrap_or_default());
        shadow.extend(zombies.iter().map(|b| engine::render::SkinnedBatch {
            mesh: b.mesh,
            instances: b.instances.clone(),
        }));
        main.extend(zombies);
        shadow.extend(animals.iter().map(|b| engine::render::SkinnedBatch { mesh: b.mesh, instances: b.instances.clone() }));
        main.extend(animals);
        // Campfires: placed ones get stones and logs (the camp's are in its model); lit ones a flame
        if let (Some(meshes), Some(play)) = (self.campfire_meshes.as_ref(), self.play.as_ref()) {
            let fires = play.camp.campfires.campfires();
            // The fire's position is lifted 0.4 m above the ground (Godot ROOT_LIFT)
            let ground = |f: &game::Campfire| f.position - Vec3::Y * game::ROOT_LIFT;
            let bases: Vec<_> = fires.iter().filter(|f| !f.permanent).map(|f| engine::render::SkinInstance::new(Mat4::from_translation(ground(f)), 0)).collect();
            let flames: Vec<_> = fires
                .iter()
                .filter(|f| f.is_lit())
                .map(|f| engine::render::SkinInstance::new(Mat4::from_translation(ground(f)) * Mat4::from_scale(f.visuals.flame_scale.max(Vec3::splat(0.2))), 0))
                .collect();
            if !bases.is_empty() {
                shadow.push(engine::render::SkinnedBatch { mesh: &meshes.base, instances: bases.clone() });
                main.push(engine::render::SkinnedBatch { mesh: &meshes.base, instances: bases });
            }
            if !flames.is_empty() {
                main.push(engine::render::SkinnedBatch { mesh: &meshes.flame, instances: flames });
            }
        }
        let mut lighting = light::at_hour(self.hour, self.flashlight_on.then(|| {
            // Held at the camera, pointing where it looks (Godot hangs it on the camera)
            light::flashlight(self.camera.position, self.camera.forward())
        }));
        // The nearest car with its lights on lights the road (Godot CarLights); the renderer has 3 spots
        if let Some(cars) = &self.cars
            && let Some(car) = cars
                .live_cars()
                .filter(|c| c.lights_on)
                .min_by(|a, b| a.body.position.distance(self.player).total_cmp(&b.body.position.distance(self.player)))
        {
            let (_, left, right) = self.car_markers;
            let [l, r] = car_lights::headlight_spots(car.draw_state().world_matrix, left, right, true, &cars.config);
            lighting.spots[1] = l;
            lighting.spots[2] = r;
        }
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

    fn ui(&mut self, ctx: &engine::egui::Context) {
        let on_main = self.menus.as_ref().is_some_and(|m| matches!(m.screen(), menus::Screen::Main | menus::Screen::SlotPicker { .. }));
        // Speech bubbles over camp people chatting near the player (Godot CampChatBubble)
        if let (Some(npcs), false) = (self.camp_npcs.as_ref(), on_main) {
            let range = npcs.director.data.bubble_range;
            let list: Vec<(Vec3, &str, f32)> = npcs
                .director
                .residents()
                .iter()
                .filter(|r| r.visible && r.bubble.is_saying() && r.position.distance(self.player) < range)
                .map(|r| (r.position + Vec3::Y * 2.0, r.bubble.text.as_str(), r.bubble.alpha()))
                .collect();
            let r = ctx.content_rect();
            bubbles::draw_bubbles(ctx, self.camera.view_proj(r.width() / r.height().max(1.0)), (r.width(), r.height()), &list);
        }
        if let (Some(play), Some(p), Some(c), false) = (self.play.as_ref(), self.player_ctrl.as_ref(), self.combat.as_ref(), on_main) {
            let show_fps = self.menus.as_ref().is_none_or(|m| m.settings.show_fps);
            let fps = if show_fps { self.fps } else { 0.0 };
            let state = play.hud(c.health.current(), c.health.max(), p.stamina.current, p.stamina.max, self.hour, p.yaw, fps, c.is_dead());
            hud::draw(ctx, &state);
            if let (Some(t), false) = (&self.hint_text, c.is_dead()) {
                hud::draw_hint(ctx, t);
            }
        }
        // Camp screens over the game (not over the main menu)
        if let (Some(play), false) = (self.play.as_mut(), on_main) {
            let dead = self.combat.as_ref().is_some_and(|c| c.is_dead());
            let action = {
                let view = camp_ui::CampView {
                    wallet: Some(&play.camp.wallet),
                    inventory: Some(&play.inventory),
                    bank: Some(&play.camp.bank),
                    trader_stock: Some(&play.camp.trader),
                    crafting: Some(&play.camp.crafting),
                    recipe_registry: Some(&play.camp.recipes),
                    bounty_manager: Some(&play.camp.bounties),
                    item_registry: Some(&play.items),
                    economy_config: Some(&play.camp.economy),
                    near_lit_campfire: play.camp.campfires.near_campfire(self.player),
                    player_pos: self.player,
                    is_dead: dead,
                };
                camp_ui::draw_bounty_tracker(ctx, &view);
                match self.camp_ui.as_mut() {
                    Some(state) => camp_ui::draw(ctx, state.screen(), state, &view),
                    None => None,
                }
            };
            match action {
                Some(camp_ui::CampAction::Close) => self.camp_ui = None,
                Some(a) => {
                    let items = play.items.clone();
                    let status = play.camp.apply(a, &mut play.inventory, &items, dead);
                    play.hotbar.on_inventory_changed(&play.inventory);
                    if let (Some(s), Some(state)) = (status, self.camp_ui.as_mut()) {
                        state.set_status(s);
                    }
                }
                None => {}
            }
        }
        if let (Some(o), false) = (&self.overlays, on_main) {
            hub_ui::draw_overlays(ctx, o);
        }
        // The hub (bag, crafting, skills, bounties, profile)
        let mut hand = None;
        let mut close_hub = false;
        let camps: Vec<glam::Vec2> = self.world_gen.as_ref().map(|g| g.settlements().iter().map(|s| glam::Vec2::new(s.center.x, s.center.z)).collect()).unwrap_or_default();
        let yaw = self.player_ctrl.as_ref().map_or(0.0, |p| p.yaw);
        let day = self.clock.day;
        if let (Some(play), Some(state), false) = (self.play.as_mut(), self.hub.as_mut(), on_main) {
            let dog = self.dogs.as_ref().and_then(|d| d.sim.dog().filter(|g| !g.is_dead()).map(|g| (g, d.sim.config.feed_items.clone())));
            // The map opens centred on the player (Godot MapView)
            if state.map_ui.centre == glam::Vec2::ZERO {
                state.map_ui.centre_on(glam::Vec2::new(self.player.x, self.player.z));
            }
            let dead = self.combat.as_ref().is_some_and(|c| c.is_dead());
            let trunk_car = self.open_trunk.as_deref().and_then(|id| self.cars.as_ref()?.get(id));
            let action = {
                let view = hub_ui::HubView {
                    inventory: Some(&play.inventory),
                    hotbar: Some(&play.hotbar),
                    item_registry: Some(&play.items),
                    crafting: Some(&play.camp.crafting),
                    recipe_registry: Some(&play.camp.recipes),
                    skills: Some(&play.camp.skills),
                    skill_registry: Some(play.camp.skills.registry()),
                    bounty_manager: Some(&play.camp.bounties),
                    wallet: Some(&play.camp.wallet),
                    bank: Some(&play.camp.bank),
                    difficulty_settings: Some(&play.camp.difficulty),
                    difficulty_config: Some(&play.camp.difficulty_config),
                    days_survived: self.clock.day,
                    play_time_seconds: self.play_time,
                    kills: play.stats.kills(),
                    kills_by_type: Some(play.stats.kills_by_type()),
                    headshots: play.stats.headshots(),
                    deaths: play.stats.deaths(),
                    distance_metres: play.stats.distance_metres(),
                    is_dead: dead,
                    near_lit_campfire: play.camp.campfires.near_campfire(self.player),
                    dog: dog.as_ref().map(|(g, food)| hub_ui::DogView {
                        name: g.name().to_string(),
                        mode: format!("{:?}", g.mode()),
                        hunger: 0.0,
                        health: g.health().current(),
                        max_health: g.health().max(),
                        age: (day - g.bought_day).max(1) as u32,
                        has_food_in_bag: food.iter().any(|f| play.inventory.has(f, 1)),
                        pack_inventory: Some(g.pack()),
                        pack_max_weight: g.pack().max_weight(),
                        pack_slots: g.pack().slot_count(),
                    }),
                    map: Some(crate::map_ui::MapView {
                        player_pos: Some(glam::Vec2::new(self.player.x, self.player.z)),
                        player_yaw: yaw,
                        map_state: &play.map,
                        generator: self.world_gen.as_deref(),
                        camps: &camps,
                        bounties: play.camp.bounties.active(),
                    }),
                    health: self.combat.as_ref().map(|c| (c.health.current(), c.health.max())),
                    trunk_inventory: trunk_car.map(|c| &c.trunk),
                    trunk_max_weight: trunk_car.map_or(40.0, |c| c.trunk.max_weight()),
                    ..Default::default()
                };
                hub_ui::draw(ctx, state, &view)
            };
            match action {
                Some(hub_ui::HubAction::Close) => self.hub = None,
                Some(hub_ui::HubAction::TrunkMove { from_trunk, slot, count }) => {
                    if let (Some(id), Some(cars)) = (self.open_trunk.as_deref(), self.cars.as_mut())
                        && let Some(car) = cars.get_mut(id)
                    {
                        let action = hub_ui::HubAction::TrunkMove { from_trunk, slot, count };
                        if let Some(toast) = trunk_play::apply_trunk_action(&action, car, &mut play.inventory, &play.items) {
                            play.show_toast(toast);
                        }
                        play.hotbar.on_inventory_changed(&play.inventory);
                    }
                }
                Some(hub_ui::HubAction::Craft { recipe }) => {
                    let items = play.items.clone();
                    let status = play.camp.apply(camp_ui::CampAction::Craft { recipe }, &mut play.inventory, &items, dead);
                    play.hotbar.on_inventory_changed(&play.inventory);
                    if let Some(s) = status {
                        state.set_status(s);
                    }
                }
                Some(hub_ui::HubAction::DogCommand(hub_ui::DogCommand::Close)) if state.screen() == hub_ui::HubScreen::DogMenu => close_hub = true,
                Some(hub_ui::HubAction::DogCommand(cmd)) => {
                    if let Some(d) = self.dogs.as_mut() {
                        let status = dog_command(&mut d.sim, cmd, state, &mut play.inventory);
                        play.hotbar.on_inventory_changed(&play.inventory);
                        if let Some(s) = status {
                            state.set_status(s);
                        }
                    }
                }
                Some(hub_ui::HubAction::RenameDog { name }) => {
                    if let Some(d) = self.dogs.as_mut() {
                        d.sim.set_name(&name);
                    }
                    state.set_screen(hub_ui::HubScreen::DogMenu);
                }
                Some(hub_ui::HubAction::Map(map_ui::MapAction::AddMarker(at))) => play.map.set_waypoint(at),
                Some(hub_ui::HubAction::Map(_)) => play.map.clear_waypoint(),
                Some(hub_ui::HubAction::CancelCraft) => {
                    let items = play.items.clone();
                    if let Some(s) = play.camp.apply(camp_ui::CampAction::CancelCraft, &mut play.inventory, &items, dead) {
                        state.set_status(s);
                    }
                }
                Some(a) => {
                    let (change, status) = play.bag_action(&a, state.split_count(), self.player);
                    hand = change;
                    if let Some(s) = status {
                        state.set_status(s);
                    }
                }
                None => {}
            }
        }
        if close_hub {
            self.hub = None;
        }
        if let Some(change) = hand {
            self.apply_hand(change);
        }
        let Some(m) = self.menus.as_mut() else { return };
        if let Some(action) = m.ui(ctx, &self.slot_cards) {
            self.menu_action(action);
        }
    }

    /// Render scale and V-Sync from Settings, applied while playing (--scale / --vsync still win).
    fn display(&self) -> Option<engine::DisplayRequest> {
        self.menus.as_ref().map(|m| display::display_request(&m.settings))
    }

    fn cursor_grab(&self) -> Option<bool> {
        if self.scene == "play" && !self.free_cam {
            Some(self.cursor_captured && !self.screen_open() && !self.menus.as_ref().is_some_and(|m| m.wants_cursor()))
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
        if let Some(a) = &self.audio {
            // Counted even when muted, so scripted runs can check sounds without playing them
            s.insert("soundsPlayed".into(), a.played.into());
            s.insert("zombieVoices".into(), self.voices_played.into());
            s.insert("poisCleared".into(), self.pois_cleared.into());
            if let Some(pz) = &self.poi_zombies {
                s.insert("poiZombiesAwake".into(), pz.awake_count().into());
                s.insert("poisVisited".into(), pz.visited_ids().len().into());
            }
            if let Some(z) = &self.zombies {
                let near = z.list.iter().filter(|q| !q.health.is_dead()).map(|q| q.position.distance(self.player)).fold(f32::MAX, f32::min);
                s.insert("nearestZombie".into(), format!("{near:.1}").into());
            }
            s.insert("lastSound".into(), a.last_played.clone().into());
            s.insert("soundLive".into(), a.is_live().into());
        }
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
        if let Some(st) = &self.streamer {
            s.insert("ladders".into(), st.chunks.values().map(|c| c.ladders.len()).sum::<usize>().into());
        }
        s.insert("climbing".into(), self.climb.is_active().into());
        if let Some(play) = &self.play {
            s.insert("campfires".into(), play.camp.campfires.campfires().len().into());
            if let Some(g) = self.dogs.as_ref().and_then(|d| d.sim.dog()) {
                s.insert("dogPosition".into(), format!("{:.2} {:.2} {:.2}", g.position().x, g.position().y, g.position().z).into());
                s.insert("dogClip".into(), g.current_clip().to_string().into());
            }
            if let Some(car) = self.cars.as_ref().and_then(|c| c.get(game::CAMP_CAR_ID)) {
                let at = car.body.position;
                s.insert("campCar".into(), format!("{:.2} {:.2} {:.2} {:.0}kmh fuel {:.1}", at.x, at.y, at.z, car.speed_kmh(), car.tank.litres()).into());
            }
            s.insert("mapChunks".into(), play.map.explore.chunk_count().into());
            s.insert("hint".into(), self.hint_text.clone().unwrap_or_default().into());
            s.insert("felledTrees".into(), self.trees.fallen.len().into());
            s.insert("guardShots".into(), self.camp_guards.as_ref().map_or(0, |g| g.total_shots_fired()).into());
            s.insert("parkedSolids".into(), self.car_solids.count().into());
            s.insert("mapBuildings".into(), play.map.found.len().into());
            s.insert("strayDogs".into(), self.dogs.as_ref().map_or(0, |d| d.sim.strays().len()).into());
            s.insert(
                "profile".into(),
                format!("kills {} headshots {} deaths {} distance {:.0}", play.stats.kills(), play.stats.headshots(), play.stats.deaths(), play.stats.distance_metres()).into(),
            );
            s.insert(
                "bubbles".into(),
                self.camp_npcs.as_ref().map_or(0, |c| c.director.residents().iter().filter(|r| r.visible && r.bubble.is_saying()).count()).into(),
            );
            s.insert("campChats".into(), self.camp_npcs.as_ref().map_or(0, |c| c.director.chats().len()).into());
            s.insert("campfiresLit".into(), play.camp.campfires.campfires().iter().filter(|f| f.is_lit()).count().into());
        }
        if let Some(w) = &self.wildlife {
            s.insert("animals".into(), w.sim.live_count().into());
            let species: Vec<String> = w.sim.list.iter().filter(|a| !a.is_dead()).map(|a| a.species_id.clone()).collect();
            s.insert("animalSpecies".into(), species.join(",").into());
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

/// A tip's key names ({interact}, {slot_1}...) as the keys this game uses.
fn fill_keys(text: &str) -> String {
    let keys = [
        ("{interact}", "E"),
        ("{inventory}", "Tab"),
        ("{sprint}", "Shift"),
        ("{toggle_camera}", "V"),
        ("{dog_stay}", "G"),
    ];
    let mut out = text.to_string();
    for (k, v) in keys {
        out = out.replace(k, v);
    }
    for n in 1..=6 {
        out = out.replace(&format!("{{slot_{n}}}"), &n.to_string());
    }
    out
}

/// A button on the dog's menu or pack screen (Godot DogMenu / DogPackScreen). Returns a status line.
fn dog_command(dogs: &mut game::Dogs, cmd: hub_ui::DogCommand, state: &mut hub_ui::HubUiState, bag: &mut game::items::Inventory) -> Option<String> {
    use hub_ui::DogCommand as C;
    let id = dogs.dog()?.id();
    match cmd {
        C::OpenPack => {
            state.set_screen(hub_ui::HubScreen::DogPack);
            None
        }
        C::Feed => Some(dogs.feed(id, bag).unwrap_or_else(|e| e)),
        C::ToggleStay => {
            dogs.toggle_stay();
            dogs.dog().map(|d| format!("{:?}", d.mode()))
        }
        C::StorePack { bag_slot, count } => {
            let pack = dogs.dog_mut()?.pack_mut();
            Some(move_stack(bag, pack, bag_slot, count))
        }
        C::TakePack { pack_slot, count } => {
            let pack = dogs.dog_mut()?.pack_mut();
            Some(move_stack(pack, bag, pack_slot, count))
        }
        C::Close => {
            state.set_screen(hub_ui::HubScreen::DogMenu);
            None
        }
    }
}

/// Moves `count` (0 = all) of a slot's items into another inventory; what does not fit goes back.
fn move_stack(from: &mut game::items::Inventory, to: &mut game::items::Inventory, slot: usize, count: u32) -> String {
    let Some(stack) = from.slot(slot).cloned() else { return String::new() };
    let n = if count == 0 { stack.count } else { count.min(stack.count) };
    let taken = from.remove_from_slot(slot, n);
    let left = to.add(&stack.item_id, taken);
    if left > 0 {
        from.add(&stack.item_id, left);
    }
    format!("Moved {}", taken - left)
}
