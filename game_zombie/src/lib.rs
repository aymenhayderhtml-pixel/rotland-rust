//! # game_zombie
//!
//! Zombie AI, hordes, flowfields, spawners, director, POI zombies, and zombie voices.

pub mod director;
pub mod flowfield;
mod flowfield_scan;
pub mod horde;
pub mod poi_zombies;
pub mod spawner;
pub mod spawner_config;
pub mod zombie;
pub mod zombie_voices;

pub use director::*;
pub use flowfield::FlowField;
pub use horde::*;
pub use poi_zombies::*;
pub use spawner::{
    ActiveZombie, CampDensityRule, SpawnConfig, SpawnConfigData, SpawnContext, SpawnOrder,
    SpawnOrders, SpawnTypeEntry, SpawnTypeTable, Spawner, SpawnerRng,
};
pub use zombie::*;
pub use zombie_voices::*;
