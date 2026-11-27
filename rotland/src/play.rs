//! The play-mode systems that sit around the player: the backpack and hotbar (what is in hand), the
//! zombie spawner, and the HUD numbers. Kept out of main.rs so that file stays small.

use std::sync::Arc;

use game::director::{Director, DirectorConfig, HordeEvent};
use game::items::{start_inventory, Hotbar, Inventory, ItemRegistry};
use game::spawner::Spawner;
use game::spawner_config::SpawnConfig;
use game::{LootState, LootTableRegistry, WeaponDef, WeaponRegistry, WorldItems};
use crate::shooting::{ShootEvents, Shooting};

mod access;
mod equipped;
mod hotbar;
mod hud;
mod interact;
mod items_use;
mod needs;
mod shoot;
mod spawn;

/// How long a pickup message stays on screen (Godot's toast).
const TOAST_SECONDS: f32 = 3.0;
/// Seconds between two uses of food, drink or bandages (ItemUse.cs UseCooldown).
const USE_COOLDOWN: f32 = 0.8;

fn load_tier_zombie_mults() -> Vec<f32> {
    let Ok(text) = std::fs::read_to_string(paths::repo_path("data/world_layout.json")) else {
        return Vec::new();
    };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    val.get("tiers")
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .map(|e| {
                    e.get("zombie_mult")
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32)
                        .unwrap_or(1.0) // new: DECISIONS 2026-11-25
                })
                .collect()
        })
        .unwrap_or_default()
}

pub struct PlayState {
    pub items: Arc<ItemRegistry>,
    pub weapons: WeaponRegistry,
    pub wear_config: game::WearConfig,
    pub attachments: game::AttachmentRegistry,
    pub inventory: Inventory,
    pub hotbar: Hotbar,
    spawner: Spawner,
    tier_zombie_mults: Vec<f32>,
    loot_tables: LootTableRegistry,
    loot_state: LootState,
    pub world_items: WorldItems,
    loot_multiplier: f32,
    world_seed: i32,
    shooting: Shooting,
    /// The gun in hand, if any (melee weapons go to combat.rs instead).
    pub gun: Option<WeaponDef>,
    /// The last shooting frame (ammo for the HUD).
    pub last_shot: ShootEvents,
    toast: Option<(String, f32)>,
    prompt: Option<String>,
    pub hunger: game::NeedStat,
    pub thirst: game::NeedStat,
    pub survival: game::SurvivalData,
    use_cooldown: f32,
    /// A hand warmer was used from the hotbar this frame; the warmth system consumes it (R32-05).
    pub hand_warmer_pending: bool,
    /// Health is full this frame (bandages are refused then, like Godot).
    pub health_full: bool,
    /// Wallet, bank, trader, skills, bounties and difficulty.
    pub camp: crate::camp::Camp,
    /// Something was picked up or a crate opened this frame (pickup sound).
    pub picked_up: bool,
    /// Body animation for something the player just did (taken by frame_actors).
    pub anim_cue: Option<humanoid::ActionCue>,
    /// What the player has explored (Godot WorldMap, saved under "map").
    pub map: game::MapState,
    /// Lifetime numbers for the profile page (Godot PlayerStats, save id "stats").
    pub stats: game::ProfileStats,
    /// The tips for new players (Godot HintSystem, saved under "hints"); None if data/hints.json is broken.
    pub hints: Option<game::HintSystem>,
    /// Items collected this frame for HUD pickup popups.
    pub pending_pickups: Vec<(String, u32)>,
    /// AI director pacing pressure and wave hordes.
    pub director: Director,
    /// Player health fraction (0.0..1.0) for director input.
    pub player_health: f32,
    /// Cumulative kills count seen from zombies.
    pub last_total_kills: u64,
    /// Events emitted for HUD and camp defense.
    pub pending_horde_events: Vec<HordeEvent>,
    horde_test_spawned: bool,
    pub armor_cfg: game::ArmorConfig,
    pub boosts_cfg: game::BoostsConfig,
    pub boosts: game::Boosts,
    pub night_vision: game::NightVision,
    pub metro_herds: crate::play_metro::MetroHerdRuntime,
    pub surgery: crate::play_metro::SurgeryUse,
}

/// What changed in hand this frame.
#[allow(clippy::large_enum_variant)] // made a few times a minute at most; boxing would only add noise
pub enum HandChange {
    /// A weapon was taken out (its definition) or put away (None).
    Weapon(Option<WeaponDef>),
    /// The flashlight was switched on or off.
    ToggleFlashlight,
    /// Food, drink or a bandage changed health (heal if above 0, sickness if below).
    Health(f32),
    /// The campfire kit was used: place a fire in front (needs the ground, so main.rs does it).
    PlaceCampfire,
    /// A holdable tool (such as the fishing rod) was taken out; the value is its model id.
    Tool(String),
    ToggleNightVision,
    UseAdrenaline,
    StartSurgery,
    RepairCarParts,
}

impl PlayState {
    pub fn new(seed: u64) -> Result<Self, String> {
        let items = ItemRegistry::from_repo().map_err(|e| format!("data/items.json: {e:?}"))?;
        let weapons = WeaponRegistry::load(&paths::repo_path("data/weapons.json")).map_err(|e| format!("data/weapons.json: {e:?}"))?;
        let wear_config = game::WearConfig::from_repo().map_err(|e| format!("data/weapon_wear.json: {e}"))?;
        let attachments = game::AttachmentRegistry::from_repo().map_err(|e| format!("data/attachments.json: {e}"))?;
        let config = SpawnConfig::load_default().map_err(|e| format!("data/spawning.json: {e}"))?;
        let tier_zombie_mults = load_tier_zombie_mults();
        let loot_tables = LootTableRegistry::from_repo().map_err(|e| format!("data/loot_tables.json: {e:?}"))?;
        let loot_multiplier = game::DifficultyConfigData::from_repo().map(|d| d.default_loot_multiplier()).unwrap_or(1.0);
        let survival = game::SurvivalData::from_repo().map_err(|e| format!("data/survival.json: {e}"))?;
        let hunger = game::NeedStat::from_config("hunger", &survival).ok_or("survival.json has no hunger")?;
        let thirst = game::NeedStat::from_config("thirst", &survival).ok_or("survival.json has no thirst")?;
        let mut inventory = start_inventory(&items);
        let items = Arc::new(items);
        let camp = crate::camp::Camp::new(seed as i32, items.clone())?;
        inventory.set_registry(items.clone());
        let director_cfg = DirectorConfig::load_default()
            .unwrap_or_else(|e| {
                log::warn!("data/director.json: {e}; using default");
                DirectorConfig::default()
            });
        let director = Director::new(director_cfg);
        let hotbar = Hotbar::new_game(&inventory);
        Ok(Self {
            items,
            weapons,
            wear_config,
            attachments,
            inventory,
            hotbar,
            spawner: Spawner::new(config, seed),
            tier_zombie_mults,
            loot_tables,
            loot_state: LootState::default(),
            world_items: WorldItems::default(),
            loot_multiplier,
            world_seed: seed as i32,
            shooting: Shooting::new(seed ^ 0x5157),
            gun: None,
            last_shot: ShootEvents::default(),
            toast: None,
            prompt: None,
            hunger,
            thirst,
            survival,
            use_cooldown: 0.0,
            hand_warmer_pending: false,
            health_full: true,
            camp,
            picked_up: false,
            anim_cue: None,
            map: game::MapState::new(),
            stats: game::ProfileStats::default(),
            hints: game::HintSystem::from_repo().map_err(|e| log::warn!("data/hints.json: {e}")).ok(),
            pending_pickups: Vec::new(),
            director,
            player_health: 1.0,
            last_total_kills: 0,
            pending_horde_events: Vec::new(),
            horde_test_spawned: false,
            armor_cfg: game::ArmorConfig::from_repo().unwrap_or_default(),
            boosts_cfg: game::BoostsConfig::from_repo().unwrap_or_default(),
            boosts: game::Boosts::new(),
            night_vision: game::NightVision::new(),
            metro_herds: crate::play_metro::MetroHerdRuntime::default(),
            surgery: crate::play_metro::SurgeryUse::default(),
        })
    }
}
