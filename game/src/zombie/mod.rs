//! Zombie logic, pure (no GPU): data from `data/zombies.json`, state, senses and bite timing.

pub mod attack;
pub mod brain;
mod brain_actions;
mod brain_extras;
pub mod corpses;
pub mod defs;
pub mod grid;
pub mod herd;
pub mod hideouts;
mod rng;
pub mod senses;
pub mod specials;
pub mod state;
pub mod steering;

pub use attack::{AttackHit, ZombieAttack};
pub use brain::{AnimClip, BrainInput, BrainOutput, ZombieBrain};
pub use corpses::ZombieCorpses;
pub use defs::{ZombieBehaviourDef, ZombieDef, ZombieDefs};
pub use grid::SpatialGrid;
pub use herd::{find_chasing_near, HerdZombie, ZombieHorde};
pub use senses::{hears, Noise, Senses};
pub use specials::{SpecialConfig, SpecialDirective, SpecialKind, SpecialState};
pub use state::ZombieState;
pub use steering::ZombieSteering;
