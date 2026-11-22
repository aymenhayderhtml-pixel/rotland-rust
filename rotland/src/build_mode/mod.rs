//! Player base building system: part picker, grid-snapped ghost preview,
//! placement validation, physics colliders, and egui build mode UI.

mod colliders;
mod logic;
mod render;
mod state;
mod types;
mod ui;

#[allow(unused_imports)]
pub use colliders::sync_colliders;
pub use logic::{try_place, try_remove, try_toggle_door, update_raycast};
pub use render::render_placed_and_ghost;
pub use state::BuildModeState;
#[allow(unused_imports)]
pub use types::{part_display_name, part_model_name, PARTS_ORDER};
#[allow(unused_imports)]
pub use ui::draw_build_mode_ui;
