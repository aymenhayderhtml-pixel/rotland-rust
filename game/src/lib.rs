//! # game
//!
//! Rotland gameplay logic: player controller, camera rig, input actions, stamina, health and weapon
//! data. Pure logic crate with no direct wgpu dependencies.

pub mod bounties;
pub mod camera_rig;
pub mod damage;
pub mod health;
pub mod flowfield;
mod flowfield_scan;
pub mod guns;
pub mod guns_fire;
pub mod input;
pub mod items;
pub mod loot;
pub mod melee;
pub mod player;
pub mod spawner;
pub mod spawner_config;
pub mod weapons;
pub mod zombie;

pub use bounties::*;
pub use camera_rig::*;
pub use damage::DamageInfo;
pub use health::{DamageResult, Health, HealthEvent, IgnoredReason};
pub use flowfield::FlowField;
pub use input::{Actions, InputBindings};
pub use items::*;
pub use loot::*;
pub use player::*;
pub use spawner::{
    ActiveZombie, CampDensityRule, SpawnConfig, SpawnConfigData, SpawnContext, SpawnOrder,
    SpawnOrders, SpawnTypeEntry, SpawnTypeTable, Spawner, SpawnerRng,
};
pub use weapons::{WeaponAnim, WeaponDef, WeaponHold, WeaponKind, WeaponLoadError, WeaponRegistry, WeaponScope};
