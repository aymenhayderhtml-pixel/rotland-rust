//! Physics collider synchronization for player-placed base building parts.

use std::collections::HashMap;

use physics::PhysicsWorld;

use super::state::BuildModeState;
use super::types::part_model_name;

/// Synchronizes physics colliders for all placed base parts with the physics world.
/// Automatically adds colliders for new parts, removes colliders for deleted parts,
/// and removes colliders when doors are open so entities can walk through.
pub fn sync_colliders(
    state: &mut BuildModeState,
    physics: &mut PhysicsWorld,
    pack: &pack::ModelPack,
) {
    let mut active_ids = HashMap::new();
    let mut shapes_cache: HashMap<String, Vec<Vec<[f32; 3]>>> = HashMap::new();

    for part in &state.base.parts {
        active_ids.insert(part.id, part.door_open);

        // Open doors do not block passage
        if part.door_open {
            if let Some(handle) = state.part_colliders.remove(&part.id) {
                physics.remove(handle);
            }
            continue;
        }

        // If part doesn't have an active collider, build one from its pack collision shapes
        if let std::collections::hash_map::Entry::Vacant(entry) = state.part_colliders.entry(part.id) {
            let model_name = part_model_name(&part.part);
            let shapes = shapes_cache.entry(model_name.clone()).or_insert_with(|| {
                pack.find(&model_name)
                    .map(|m| m.colliders.iter().map(|c| c.points.clone()).collect())
                    .unwrap_or_default()
            });

            if !shapes.is_empty() {
                let handle = physics.add_static(shapes, part.position, part.yaw, 1.0);
                entry.insert(handle);
            }
        }
    }

    // Clean up colliders for parts that were removed or collapsed
    let stale_ids: Vec<u32> = state
        .part_colliders
        .keys()
        .copied()
        .filter(|id| !active_ids.contains_key(id))
        .collect();

    let mut changed = !stale_ids.is_empty();
    for id in stale_ids {
        if let Some(handle) = state.part_colliders.remove(&id) {
            physics.remove(handle);
            changed = true;
        }
    }

    if changed {
        physics.update_broad_phase();
    }
}
