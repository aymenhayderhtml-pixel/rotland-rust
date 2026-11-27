//! State groupings and core data structures for Rotland.

use std::collections::HashMap;
use std::sync::Arc;

use engine::render::{Camera, LitRenderer};
use game::{
    CameraRig, GuardPost, GuardTarget, HintContext, Player, PoiZombieSite,
    VoiceZombie,
};
use glam::{Mat4, Vec3};
use winit::keyboard::KeyCode;
use world::{LootSpot, WorldGenerator};

use crate::{
    camp_npcs, camp_ui, campfire_mesh, car_solid, combat, dog_view, fp_weapon, hub_ui, menus,
    play, player_visual, props, streaming, wildlife, zombies,
};

/// Visual hook for the player body (to be rendered by another worker).
#[derive(Clone, Copy, Debug)]
pub struct PlayerPose {
    pub position: Vec3,
    pub yaw: f32,
    pub horizontal_speed: f32,
    pub velocity: Vec3,
    pub on_floor: bool,
    pub crouching: bool,
    pub sprinting: bool,
    pub stamina_frac: f32,
    pub water_depth: f32,
    pub swimming: bool,
    pub climbing: bool,
    pub on_rungs: bool,
    pub first_person: bool,
}

/// The keys of the rebindable actions this frame (Godot KeyBindings; Settings > Controls).
#[derive(Clone)]
pub struct KeyMap {
    pub movement: game::InputBindings,
    pub interact: KeyCode,
    pub reload: KeyCode,
    pub dog_stay: KeyCode,
    pub inventory: KeyCode,
    pub skills: KeyCode,
    pub bounties: KeyCode,
    pub profile: KeyCode,
    pub map: KeyCode,
    pub slots: [KeyCode; 6],
}

impl KeyMap {
    pub fn from_bindings(kb: &game::KeyBindings) -> Self {
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
pub fn weapon_model_name(def: &game::WeaponDef) -> Option<String> {
    let file = def.model.rsplit('/').next()?;
    Some(file.trim_end_matches(".glb").to_string()).filter(|s| !s.is_empty())
}

/// GPU renderer, camera, models, and visual batches.
#[derive(Default)]
pub struct RenderState {
    pub renderer: Option<LitRenderer>,
    pub camera: Camera,
    pub bones: Vec<Mat4>,
    pub library: props::PropLibrary,
    pub pack: Option<pack::ModelPack>,
    pub campfire_meshes: Option<campfire_mesh::CampfireMeshes>,
    pub pending_weapon_model: Option<Option<String>>,
    pub car_markers: crate::car_visuals::CarMarkers,
    pub car_emissives: Option<crate::car_visuals::CarEmissiveMeshes>,
    pub car_smoke: crate::car_visuals::CarSmokeSystem,
    pub fps: f32,
    pub flashlight_on: bool,
    pub scratch_beds: Vec<props::Placed>,
    /// Toxic cloud puffs (R32-07).
    pub toxic_cloud_view: crate::toxic_cloud_view::ToxicCloudView,
    pub toxic_puff_scratch: Vec<crate::toxic_cloud_view::PuffInstance>,
    /// Short-range ground detail layer around the player (R35-01).
    pub ground_detail: crate::ground_detail::GroundDetailLayer,
    /// Pooled explosion VFX and camera shake (R36-05).
    pub explosion_view: crate::explosion_view::ExplosionView,
    /// Gun feel view kick, FP kick, and muzzle flash (R36-01).
    pub gun_feel: crate::gun_feel_view::GunFeelView,
    /// Weapon attachment meshes and mount-point offsets (R37-08).
    pub attachment_view: Option<crate::attachment_view::AttachmentView>,
    /// Crows and flapping flags/plastic in New Halden (R41-05).
    pub metro_life: crate::metro_life::MetroLife,
}

/// Terrain, streaming chunks, static physics, trees, and POI markers.
#[derive(Default)]
pub struct WorldState {
    pub world_gen: Option<Arc<WorldGenerator>>,
    pub streamer: Option<streaming::Streamer>,
    pub physics: physics::PhysicsWorld,
    pub trees: game::TreeFelling,
    pub falling: HashMap<String, (glam::IVec2, streaming::TreeSlot)>,
    pub poi_markers: HashMap<String, Vec<Vec3>>,
    pub poi_zombies: Option<game::PoiZombies>,
    pub near_crates_scratch: Vec<LootSpot>,
    pub poi_zombie_sites: Vec<PoiZombieSite>,
    pub poi_sites_generation: u64,
    pub tracked_zombie_scratch: Vec<u64>,
    pub cleared_poi_scratch: Vec<String>,
    pub army_line_brute_day: Option<i32>,
    pub camps: Vec<glam::Vec2>,
    pub last_chunk_count: usize,
}

impl WorldState {
    pub fn height(&self, x: f32, z: f32) -> f32 {
        self.world_gen.as_ref().map_or(0.0, |w| w.height(x, z))
    }
}

/// Player, combat, zombies, wildlife, companion dogs, and vehicles.
#[derive(Default)]
pub struct ActorsState {
    pub player: Vec3,
    pub player_ctrl: Option<Player>,
    pub player_visual: Option<player_visual::PlayerVisual>,
    pub backpack_view: Option<crate::backpack_view::BackpackView>,
    pub camera_rig: Option<CameraRig>,
    pub mover: physics::CharacterMover,
    pub combat: Option<combat::Combat>,
    pub zombies: Option<zombies::Zombies>,
    /// Active bloater toxic clouds (R32-07).
    pub toxic_clouds: game::zombie::ToxicClouds,
    pub wildlife: Option<wildlife::Wildlife>,
    pub dogs: Option<dog_view::DogView>,
    /// Teammates' person models on screen (R23-N5).
    pub team_models: Option<crate::companion_model::CompanionModels>,
    pub cars: Option<game::Cars>,
    pub car_input: game::CarInput,
    pub car_camera: game::CarCamera,
    pub car_solids: car_solid::ParkedCarSolids,
    pub fp_sway: fp_weapon::FpSway,
    pub fp_motion: game::FpMotion,
    pub vault: game::VaultState,
    pub hit_react: game::HitReactState,
    pub guard: game::GuardState,
    pub shove_scratch: Vec<(u64, Vec3)>,
    pub open_trunk: Option<String>,
    pub camp_guards: Option<game::CampGuards>,
    pub camp_npcs: Option<camp_npcs::CampNpcs>,
    pub rescue_view: Option<crate::rescue_view::RescueView>,
    /// Camp gunsmith bench and outer-town gun shopkeepers (R37-08).
    pub gunsmith_world: crate::gunsmith_play::GunsmithWorld,
    pub zombie_voices: Option<game::ZombieVoices>,
    pub climb: game::LadderClimb,
    pub downed: crate::downed_play::DownedPlay,
    pub grenades: crate::grenade_play::GrenadePlay,
    pub was_dead: bool,
    pub bites_scratch: Vec<f32>,
    pub zombie_pos_scratch: Vec<(u64, Vec3)>,
    pub voice_zombies_scratch: Vec<VoiceZombie>,
    pub guard_posts_scratch: Vec<GuardPost>,
    pub guard_targets_scratch: Vec<GuardTarget>,
    pub zombie_status_scratch: Vec<(u64, Vec3, bool)>,
    pub noises_scratch: Vec<(Vec3, f32)>,
    /// What the zombies hear this frame: the player's noises plus last frame's screams (reused buffer).
    pub heard_scratch: Vec<game::zombie::Noise>,
    /// Screams from last frame, heard by the other zombies this frame (ZombieAI.Actions SpotPlayer).
    pub scream_noises: Vec<game::zombie::Noise>,
    pub animal_status_scratch: Vec<(u64, Vec3, bool)>,
    pub car_solids_scratch: Vec<(String, Mat4, bool)>,
    pub car_targets_scratch: Vec<(u64, Vec3)>,
    pub car_gone_scratch: Vec<glam::IVec2>,
}

impl ActorsState {
    /// Pose hook exposing player state for visual character models and animations.
    pub fn player_pose(&self) -> Option<PlayerPose> {
        let p = self.player_ctrl.as_ref()?;
        let rig = self.camera_rig.as_ref()?;
        Some(PlayerPose {
            position: p.position,
            yaw: p.yaw,
            horizontal_speed: p.horizontal_speed(),
            velocity: p.velocity,
            on_floor: p.on_floor,
            crouching: p.crouching,
            sprinting: p.is_sprinting,
            stamina_frac: if p.stamina.max > 0.0 {
                (p.stamina.current / p.stamina.max).clamp(0.0, 1.0)
            } else {
                1.0
            },
            water_depth: p.water_depth,
            swimming: p.in_deep_water,
            climbing: self.climb.is_active(),
            on_rungs: self.climb.is_on_rungs(),
            first_person: rig.is_first_person,
        })
    }
}

/// Backpack, hotbar, camp management, game clock, and survival stats.
pub struct PlaySide {
    pub play: Option<play::PlayState>,
    pub clock: world::time::Clock,
    pub clock_frozen: bool,
    pub hour: f32,
    pub elapsed_hours: f32,
    pub skill_seen: HashMap<String, (u32, f32)>,
    pub hint_text: Option<String>,
    pub voices_played: u32,
    pub pois_cleared: u32,
    pub hint_ctx: HintContext,
    pub hint_inventory_hash: u64,
    pub active_hint_id: Option<String>,
    pub build_mode: super::frame_input::build_mode::BuildModeState,
    pub water: super::frame_water::WaterPlay,
    /// Base siege simulation (zombies hitting the player's walls); `None` when data/siege.json fails to load.
    pub siege: Option<game::building::siege::SiegeSystem>,
    /// Hired teammates' runtime bodies (R23-N5); `None` when data/companions.json fails to load.
    pub companions: Option<game::guild::companion::CompanionSquad>,
    /// Reused buffers and the guild sync key for the teammates.
    pub team: super::frame_team::TeamScratch,
    /// Guild task hooks (fetch hand-ins, explore places, ready notices); `None` when data/guild.json fails to load (R24-N6).
    pub guild_hooks: Option<game::guild::GuildTaskHooks>,
    /// Tracker rows and named places for the guild hooks (R24-N6).
    pub guild_panel: super::frame_guild::GuildTaskPanel,
    /// World survivor rescue state (R29-02).
    pub rescue: Option<game::rescue::RescueState>,
    /// Region name banner when crossing biomes (R31-05).
    pub region_banner: crate::region_banner::RegionBannerTracker,
    /// Weather state, particles and fog for the current biome (R32-03).
    pub weather: Option<crate::weather_play::WeatherPlay>,
    /// Body warmth bar and warm gear (R32-05).
    pub temperature: Option<crate::temperature_play::TemperaturePlay>,
    /// Procedural clouds and biome fog tint (R35-04).
    pub sky: Option<crate::sky_play::SkyPlay>,
    /// New Halden's skyline and haze; None in a world without the metro city.
    pub metro: Option<crate::metro_play::MetroPlay>,
    /// Tallest-landmark radio goal runtime (R43-07).
    pub metro_radio: crate::metro_radio::MetroRadioRuntime,
}

impl Default for PlaySide {
    fn default() -> Self {
        Self {
            play: None,
            clock: world::time::Clock::new(),
            clock_frozen: false,
            hour: 12.0,
            elapsed_hours: 0.0,
            skill_seen: HashMap::new(),
            hint_text: None,
            voices_played: 0,
            pois_cleared: 0,
            hint_ctx: HintContext::default(),
            hint_inventory_hash: 0,
            active_hint_id: None,
            build_mode: super::frame_input::build_mode::BuildModeState::new(),
            water: super::frame_water::WaterPlay::default(),
            siege: super::frame_siege::load_system(),
            companions: super::frame_team::load_squad(),
            team: super::frame_team::TeamScratch::default(),
            guild_hooks: super::frame_guild::load_hooks(),
            region_banner: crate::region_banner::RegionBannerTracker::default(),
            guild_panel: super::frame_guild::GuildTaskPanel::default(),
            rescue: None,
            weather: None,
            temperature: None,
            sky: None,
            metro: None,
            metro_radio: crate::metro_radio::MetroRadioRuntime::default(),
        }
    }
}

/// Menus, camp screens, player hub, HUD toasts, and key state.
#[derive(Default)]
pub struct UiState {
    pub menus: Option<menus::Menus>,
    pub camp_ui: Option<camp_ui::CampUiState>,
    pub hub: Option<hub_ui::HubUiState>,
    pub overlays: Option<hub_ui::HubUiState>,
    pub gunsmith: Option<crate::gunsmith_play::GunsmithSession>,
    pub keys: KeyMap,
    pub cursor_captured: bool,
    pub e_used: bool,
    /// Right mouse held this frame (aiming); the HUD shows the scope picture with a scoped gun.
    pub right_held: bool,
    /// Item pictures for the hotbar and bag, loaded once (R10-04).
    pub icons: crate::hud::IconCache,
    /// HUD mini-map config and cached local terrain tiles.
    pub minimap: crate::app::minimap::MinimapState,
    /// F1 developer panel (R12-03).
    pub dev_panel: crate::dev_panel::DevPanelState,
    /// Persistent HUD manager for transient effects, directional damage, pickup popups, notifications, and vitals trails.
    pub hud: crate::hud::Hud,
}

impl UiState {
    /// A camp screen or the hub is open: the mouse is free and the player's hands are busy.
    pub fn screen_open(&self) -> bool {
        self.camp_ui.is_some() || self.hub.is_some() || self.gunsmith.is_some() || self.dev_panel.open
    }
}

/// Save slots, session timers, and command line test flags.
pub struct SessionState {
    pub slots: Option<save::Slots>,
    pub slot_cards: [menus::SlotSummary; 3],
    pub active_slot: u8,
    pub loaded_save: Option<save::SaveFile>,
    pub pending_world: Option<(i32, Option<save::SaveFile>)>,
    pub play_time: f64,
    pub quit: bool,
    pub time: f32,
    pub last_dt: f32,
    pub seed: i32,
    pub scene: String,
    pub walk: bool,
    pub walk_time: f32,
    pub free_cam: bool,
    pub autowalk: bool,
    pub autoattack: bool,
    pub die_at: Option<f32>,
    pub save_at: Option<f32>,
    pub forced_zombies: Option<usize>,
    /// --at=X,Z: start the player there (screenshots, dev).
    pub start_at: Option<(f32, f32)>,
    pub start_yaw: Option<f32>,
    pub give: Vec<(String, u32)>,
    pub near_animals: Vec<String>,
    pub test_climb: bool,
    pub test_chop: bool,
    pub test_first_person: bool,
    pub test_poi: bool,
    pub test_backpack: Option<crate::backpack_view::BackpackKind>,
    pub test_explosion: bool,
    /// `--car-yaw=<deg>`: holds the car camera at this angle around the car (screenshots).
    pub test_car_yaw: Option<f32>,
    pub anim_shot: super::anim_shot::AnimShotState,
    pub test_flags: super::test_flags::TestFlags,
    pub bench: Option<super::bench::BenchState>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            slots: None,
            slot_cards: [
                menus::SlotSummary::default(),
                menus::SlotSummary::default(),
                menus::SlotSummary::default(),
            ],
            active_slot: 1,
            loaded_save: None,
            pending_world: None,
            play_time: 0.0,
            quit: false,
            time: 0.0,
            last_dt: 0.0,
            seed: 42,
            scene: "play".to_string(),
            walk: false,
            walk_time: 0.0,
            free_cam: false,
            autowalk: false,
            autoattack: false,
            die_at: None,
            save_at: None,
            forced_zombies: None,
            start_at: None,
            start_yaw: None,
            give: Vec::new(),
            near_animals: Vec::new(),
            test_climb: false,
            test_chop: false,
            test_first_person: false,
            test_poi: false,
            test_backpack: None,
            test_explosion: false,
            test_car_yaw: None,
            anim_shot: super::anim_shot::AnimShotState::default(),
            test_flags: super::test_flags::TestFlags::default(),
            bench: None,
        }
    }
}

/// Sound and audio effects.
#[derive(Default)]
pub struct AudioState {
    pub audio: Option<audio::Audio>,
    pub step_timer: f32,
    pub car_horn_cooldown: f32,
    pub trunk_was_open: bool,
}

/// The top-level Rotland game instance grouping all subsystem states.
#[derive(Default)]
pub struct R0 {
    pub render: RenderState,
    pub world: WorldState,
    pub actors: ActorsState,
    pub play: PlaySide,
    pub ui: UiState,
    pub session: SessionState,
    pub audio: AudioState,
}

impl R0 {
    /// Visual hook exposing player state for character models and animations.
    pub fn player_pose(&self) -> Option<PlayerPose> {
        self.actors.player_pose()
    }
}
