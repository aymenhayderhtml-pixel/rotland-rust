//! # game
//!
//! Rotland gameplay logic: player controller, camera rig, input actions, and stamina.
//! Pure logic crate with no direct wgpu dependencies.

pub mod camera_rig;
pub mod flowfield;
mod flowfield_scan;
pub mod input;
pub mod player;

pub use camera_rig::*;
pub use flowfield::FlowField;
pub use input::{Actions, InputBindings};
pub use player::*;
