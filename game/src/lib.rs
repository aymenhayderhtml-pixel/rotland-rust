//! # game
//!
//! Rotland gameplay logic: player controller, camera rig, input actions, stamina, health and weapon
//! data. Pure logic crate with no direct wgpu dependencies.

pub mod camera_rig;
pub mod damage;
pub mod health;
pub mod input;
pub mod player;
pub mod weapons;

pub use camera_rig::*;
pub use damage::DamageInfo;
pub use health::{DamageResult, Health, HealthEvent, IgnoredReason};
pub use input::{Actions, InputBindings};
pub use player::*;
pub use weapons::{WeaponAnim, WeaponDef, WeaponHold, WeaponKind, WeaponLoadError, WeaponRegistry, WeaponScope};
