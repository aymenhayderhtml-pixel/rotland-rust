//! # game
//!
//! Rotland gameplay logic: player controller, camera rig, input actions, stamina, health and weapon
//! data. Pure logic crate with no direct wgpu dependencies.

pub mod animals;
pub mod bounties;
pub mod camera_rig;
pub mod camp;
pub mod campfire;
pub mod camp_guards;
pub mod crafting;
pub mod damage;
pub mod difficulty;
pub mod dogs;
pub mod economy;
pub mod farming;
pub mod health;
pub mod hints;
pub mod flowfield;
mod flowfield_scan;
pub mod guns;
pub mod guns_fire;
pub mod input;
pub mod items;
pub mod ladders;
pub mod loot;
pub mod map;
pub mod melee;
pub mod player;
pub mod skills;
pub mod spawner;
pub mod spawner_config;
pub mod survival;
pub mod weapons;
pub mod vehicles;
pub mod zombie;

pub use animals::*;
pub use bounties::*;
pub use camera_rig::*;
pub use camp::*;
pub use campfire::*;
pub use camp_guards::*;
pub use crafting::*;
pub use damage::DamageInfo;
pub use difficulty::*;
pub use dogs::*;
pub use economy::*;
pub use farming::*;
pub use health::{DamageResult, Health, HealthEvent, IgnoredReason};
pub use hints::*;
pub use flowfield::FlowField;
pub use input::{Actions, InputBindings};
pub use items::*;
pub use ladders::*;
pub use loot::*;
pub use map::*;
pub use player::*;
pub use skills::*;
pub use spawner::{
    ActiveZombie, CampDensityRule, SpawnConfig, SpawnConfigData, SpawnContext, SpawnOrder,
    SpawnOrders, SpawnTypeEntry, SpawnTypeTable, Spawner, SpawnerRng,
};
pub use survival::*;
pub use vehicles::*;
pub use weapons::{WeaponAnim, WeaponDef, WeaponHold, WeaponKind, WeaponLoadError, WeaponRegistry, WeaponScope};
