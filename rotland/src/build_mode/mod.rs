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

use std::sync::Mutex;

static GLOBAL_STATE: Mutex<Option<BuildModeState>> = Mutex::new(None);

/// Executes a closure with mutable access to the global build mode state.
pub fn with_state<R>(f: impl FnOnce(&mut BuildModeState) -> R) -> R {
    let mut lock = GLOBAL_STATE.lock().unwrap();
    let state = lock.get_or_insert_with(BuildModeState::new);
    f(state)
}

/// Returns whether build mode is currently active.
pub fn is_active() -> bool {
    with_state(|s| s.active)
}

/// Sets build mode active state.
pub fn set_active(active: bool) {
    with_state(|s| s.set_active(active));
}

/// Toggles build mode active state.
pub fn toggle() -> bool {
    with_state(|s| s.toggle())
}

/// Checks the `--build-test` command-line argument:
/// starts in build mode with 50 wood in the bag for screenshots.
pub fn check_build_test_flag(inventory: Option<&mut game::Inventory>) {
    with_state(|s| {
        if !s.build_test_initialized {
            let has_flag = std::env::args().any(|a| a == "--build-test");
            if has_flag {
                s.active = true;
                if let Some(inv) = inventory {
                    inv.add("wood", 50);
                }
                s.build_test_initialized = true;
            }
        }
    });
}
