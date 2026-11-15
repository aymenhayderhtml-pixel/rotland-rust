//! # game
//!
//! Rotland gameplay logic: player controller, camera rig, input actions, and stamina.
//! Pure logic crate with no direct wgpu dependencies.

pub mod camera_rig;
pub mod input;
pub mod player;
pub mod zombie;

pub use camera_rig::*;
pub use input::{Actions, InputBindings};
pub use player::*;
