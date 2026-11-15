//! Zombie logic, pure (no GPU): data from `data/zombies.json`, state, senses and bite timing.

pub mod attack;
pub mod defs;
pub mod senses;
pub mod state;

pub use attack::{AttackHit, ZombieAttack};
pub use defs::{ZombieBehaviourDef, ZombieDef, ZombieDefs};
pub use senses::{hears, Noise, Senses};
pub use state::ZombieState;
