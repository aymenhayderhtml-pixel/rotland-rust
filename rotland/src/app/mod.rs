#![allow(clippy::too_many_arguments)] // each frame step takes the state groups it touches as separate borrows
//! Application orchestration and frame pipeline modules.

pub mod minimap;
mod minimap_cache;
mod minimap_draw;
mod minimap_metro;

pub mod anim_shot;
pub mod bench;
pub mod frame_actors;
mod frame_actors_combat;
mod frame_actors_move;
pub mod frame_audio;
pub mod frame_camp;
pub mod frame_cars;
pub mod frame_guild;
pub mod frame_input;
pub mod frame_water;
pub mod frame_sim;
mod frame_sim_actors;
mod frame_sim_player;
mod frame_sim_zombies;
pub mod frame_siege;
pub mod frame_team;
mod frame_team_revive;
mod frame_team_view;
pub mod frame_ui;
pub mod frame_warmth;
mod metro_flags;
mod metro_flags_r41;
pub mod frame_world;
pub mod render_frame;
pub mod save_load;
#[cfg(test)]
pub mod spikes;
pub mod start_world;
pub mod startup;
pub mod test_flags;
pub mod state;
pub mod stats;

pub use state::*;
