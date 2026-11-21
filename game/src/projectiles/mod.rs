//! Projectile physics, simulation, and weapon firing logic.

pub mod sim;
pub mod types;

pub use sim::{bow_shot, spear_throw, ProjectileSim};
pub use types::{Hit, Projectile, ProjectileEvent, ProjectileKind};
