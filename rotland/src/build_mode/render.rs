//! Rendering integration: placed base parts and ghost preview model instances.

use physics::PhysicsWorld;

use super::colliders::sync_colliders;
use super::state::BuildModeState;
use super::types::part_model_name;
use crate::props::{Placed, PropLibrary};

/// Gathers all placed building parts and the ghost preview instance into the render list,
/// and synchronizes their physics colliders.
pub fn render_placed_and_ghost(
    state: &mut BuildModeState,
    device: &wgpu::Device,
    pack: &pack::ModelPack,
    library: &mut PropLibrary,
    placed_out: &mut Vec<Placed>,
    physics: &mut PhysicsWorld,
) {
    // 1. Keep physics colliders in sync with placed base parts
    sync_colliders(state, physics, pack);

    // 2. Render all placed base parts
    for part in &state.base.parts {
        let model = part_model_name(&part.part);
        if let Some(slot) = library.get(device, pack, &model, true) {
            placed_out.push(library.place(slot, part.position, part.yaw, 1.0));
        }
    }

    // 3. Render ghost preview model when in build mode
    if state.active
        && let Some(placement) = &state.current_placement
    {
        let model = part_model_name(&placement.part);
        if let Some(slot) = library.get(device, pack, &model, true) {
            placed_out.push(library.place(slot, placement.position, placement.yaw, 1.0));
        }
    }
}
