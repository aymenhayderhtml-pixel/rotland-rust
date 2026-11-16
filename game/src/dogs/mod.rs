//! Dogs module for Rotland: companion pet dog, commands, combat, and stray encounters.
// Port of scripts/Dogs/

mod breeder;
mod combat;
mod config;
mod dog;
mod dog_update;
mod food;
mod growth;
mod manager;
mod manager_update;
mod mode;
mod motion;
mod save;
mod steering;
mod stray;
mod types;

pub use breeder::DogBreeder;
pub use combat::{DogBite, DogCombat};
pub use config::DogConfigData;
pub use dog::Dog;
pub use food::DogFood;
pub use growth::DogGrowth;
pub use manager::Dogs;
pub use mode::DogMode;
pub use motion::DogMotion;
pub use save::DOG_SAVE_ID;
pub use steering::DogSteering;
pub use stray::{DogPoiSite, Stray, StrayDogSpawner};
pub use types::{DogDrawState, DogFrame, DogUpdateContext};
