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

use std::sync::atomic::{AtomicPtr, Ordering};

static ACTIVE_INSTANCE: AtomicPtr<BuildModeState> = AtomicPtr::new(std::ptr::null_mut());

/// Sets the active in-memory BuildModeState pointer from app state.
pub fn set_active_instance(ptr: *mut BuildModeState) {
    ACTIVE_INSTANCE.store(ptr, Ordering::SeqCst);
}

/// Executes a closure with mutable access to the active build mode state.
/// Compatibility accessor for render_frame.rs until Claude wires &mut play.build_mode directly.
pub fn with_state<R>(f: impl FnOnce(&mut BuildModeState) -> R) -> R {
    let ptr = ACTIVE_INSTANCE.load(Ordering::SeqCst);
    if !ptr.is_null() {
        // SAFETY: Main thread game loop only.
        let state = unsafe { &mut *ptr };
        f(state)
    } else {
        let mut fallback = BuildModeState::new();
        f(&mut fallback)
    }
}
