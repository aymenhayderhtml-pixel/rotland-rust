//! Wild animals and hunting logic ported from scripts/Animals/*.cs.
// Godot: scripts/Animals/

mod ai;
mod ai_states;
mod butcher;
mod combat;
mod config;
mod herd;
mod hunting;
mod instance;
mod loot;
mod manager;
mod manager_update;
mod motion;
mod pool;
mod rng;
mod senses;
mod spawner;
mod state;

pub use ai::{AnimalAI, AnimalAIOutput};
pub use butcher::AnimalButcher;
pub use combat::{AnimalAttackHit, AnimalCombat, FIGHT_COOLDOWN, FIGHT_TICK};
pub use config::{
    AnimalBehaviourData, AnimalButcherData, AnimalConfigData, AnimalSpawnData, AnimalSpeciesData,
};
pub use herd::AnimalHerd;
pub use hunting::AnimalHunting;
pub use instance::{AnimalInstance, PARKED_Y};
pub use loot::{AnimalLoot, ANTLER_ITEM, HIDE_ITEM, MEAT_ITEM};
pub use manager::{AnimalDrawState, AnimalFrame, AnimalUpdateContext, Animals};
pub use motion::{AnimalMotion, GRAVITY, STOP_DISTANCE};
pub use pool::AnimalPool;
pub use rng::AnimalRng;
pub use senses::AnimalSenses;
pub use spawner::{
    pick_spawn_spot, plan_herd, slope_deg, try_ground_at, FLOOR_OFFSET, HERD_SPACING,
    MAX_SLOPE_DEG, SLOPE_PROBE_DIST, SPAWN_ATTEMPTS_PER_TICK,
};
pub use state::AnimalState;
