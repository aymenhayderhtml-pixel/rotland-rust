//! Zombie logic, pure (no GPU): data from `data/zombies.json`, state, senses and bite timing.

pub mod attack;
pub mod brain;
mod brain_actions;
pub mod defs;
mod rng;
pub mod senses;
pub mod state;

pub use attack::{AttackHit, ZombieAttack};
pub use brain::{AnimClip, BrainInput, BrainOutput, ZombieBrain};
pub use defs::{ZombieBehaviourDef, ZombieDef, ZombieDefs};
pub use senses::{hears, Noise, Senses};
pub use state::ZombieState;
