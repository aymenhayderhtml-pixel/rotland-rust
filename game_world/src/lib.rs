//! # game_world
//!
//! World entities and combat simulations: animals, dogs, guns firing, melee, melee guard, and projectiles.

pub mod animals;
pub mod dogs;
pub mod grenade;
pub mod guns_fire;
pub mod melee;
pub mod melee_guard;
pub mod projectiles;

pub use animals::*;
pub use dogs::*;
pub use grenade::*;
pub use guns_fire::*;
pub use melee::*;
pub use melee_guard::{GuardConfig, GuardEvent, GuardInput, GuardState, shove_targets};
pub use projectiles::*;
