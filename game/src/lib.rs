//! # game
//!
//! Rotland gameplay logic: player controller, camera rig, input actions, stamina, health and weapon
//! data. Pure logic crate with no direct wgpu dependencies.

pub mod animals;
pub mod bounties;
pub mod building;
pub mod camera_rig;
pub mod camp;
pub mod campfire;
pub mod camp_guards;
pub mod camp_upgrades;
pub mod crafting;
pub mod damage;
pub mod dev_panel;
pub mod difficulty;
pub mod dir;
pub mod director;
pub mod dogs;
pub mod downed;
pub mod economy;
pub mod farming;
pub mod fishing;
pub mod fp_motion;
pub use fp_motion::{FpMotion, FpMotionConfig, FpMotionInput};
pub mod health;
pub mod hints;
pub mod horde;
pub mod hud_math;
pub mod flowfield;
mod flowfield_scan;
pub mod guild;
pub mod guns;
pub mod guns_fire;
pub mod input;
pub mod items;
pub mod keybinds;
pub mod ladders;
pub mod loot;
pub mod map;
pub mod melee;
pub mod player;
pub mod poi_zombies;
pub mod profile_stats;
pub mod projectiles;
pub mod rescue;
pub mod rescue_place;
pub mod skills;
pub mod spawner;
pub mod spawner_config;
pub mod survival;
pub mod swim_hands;
pub mod swimming;
pub mod team_anim;
pub use team_anim::{
    plan_npc_anim, NpcAnimInput, NpcAnimPlan, NpcAnimRole, NpcAnimTrack, RoleAnimConfig,
    TeamAnimConfig, TeamLodConfig, TeamRolesConfig,
};
pub mod trees;
pub mod weapons;
pub mod vault;
pub mod hit_react;
pub use hit_react::{HitInput, HitReactConfig, HitReactState, HitReaction, HitSide, hit_side};
pub mod death_anim;
pub use death_anim::{
    DeathAnimConfig, DeathCause, DeathChoice, DeathClipSpec, ZombieDeathClips,
};
pub use vault::{VaultConfig, VaultKind, VaultPlan, VaultState, find_vault};
pub mod melee_combo;
pub use melee_combo::{ComboMove, MeleeComboState, MeleeMovesConfig};
pub mod melee_guard;
pub use melee_guard::{GuardConfig, GuardEvent, GuardInput, GuardState, shove_targets};
pub mod vehicles;
pub mod zombie;
pub mod zombie_voices;

pub use animals::*;
pub use bounties::*;
pub use building::*;
pub use camera_rig::*;
pub use camp::*;
pub use campfire::*;
pub use camp_guards::*;
pub use camp_upgrades::*;
pub use crafting::*;
pub use damage::DamageInfo;
pub use dev_panel::*;
pub use difficulty::*;
pub use dir::*;
pub use director::*;
pub use dogs::*;
pub use downed::{BANDAGE_ITEM, Downed, DownedConfig, DownedEvent, DownedPhase, ReviveBy};
pub use economy::*;
pub use farming::*;
pub use fishing::{
    can_fish_at, FishingConfig, FishingEvent, FishingInput, FishingMachine, FishingPhase,
    FishingSession,
};
pub use swim_hands::{SwimHands, SwimHandsChange};
pub use health::{DamageResult, Health, HealthEvent, IgnoredReason};
pub use hints::*;
pub use horde::*;
pub use flowfield::FlowField;
pub use guild::*;
pub use input::{Actions, InputBindings};
pub use items::*;
pub use keybinds::*;
pub use ladders::*;
pub use loot::*;
pub use map::*;
pub use player::*;
pub use poi_zombies::*;
pub use profile_stats::*;
pub use projectiles::*;
pub use skills::*;
pub use spawner::{
    ActiveZombie, CampDensityRule, SpawnConfig, SpawnConfigData, SpawnContext, SpawnOrder,
    SpawnOrders, SpawnTypeEntry, SpawnTypeTable, Spawner, SpawnerRng,
};
pub use survival::*;
pub use swimming::{
    evaluate_water, is_on_dock, speed_multiplier_for_zone, try_gather_dirty_water, SwimmingConfig,
    WaterMovementResult,
};
pub use trees::*;
pub use vehicles::*;
pub use weapons::{WeaponAnim, WeaponDef, WeaponHold, WeaponKind, WeaponLoadError, WeaponRegistry, WeaponScope};
pub use zombie_voices::*;
