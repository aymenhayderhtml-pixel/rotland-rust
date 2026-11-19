#![allow(clippy::too_many_arguments)] // each frame step takes the state groups it touches as separate borrows
//! Application orchestration and frame pipeline modules.

pub mod minimap;
mod minimap_cache;
mod minimap_draw;

pub mod frame_actors;
pub mod frame_audio;
pub mod frame_camp;
pub mod frame_cars;
pub mod frame_input;
pub mod frame_sim;
pub mod frame_ui;
pub mod frame_world;
pub mod render_frame;
pub mod save_load;
pub mod start_world;
pub mod startup;
pub mod state;
pub mod stats;

pub use state::*;
