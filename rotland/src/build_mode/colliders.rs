//! Physics collider synchronization for player-placed base building parts.

use std::collections::HashMap;

use physics::PhysicsWorld;

use super::state::BuildModeState;
use super::types::part_model_name;

/// Fallback box collider points when pack model does not specify collision shapes.
fn fallback_collider_shape(part_name: &str) -> Vec<[f32; 3]> {
    let (hx, hy, hz) = if game::building::is_edge_part(part_name) {
        (1.0, 1.5, 0.1)
    } else {
        (1.0, 0.05, 1.0)
    };
    vec![
        [-hx, 0.0, -hz],
        [hx, 0.0, -hz],
        [hx, 0.0, hz],
        [-hx, 0.0, hz],
        [-hx, hy * 2.0, -hz],
        [hx, hy * 2.0, -hz],
        [hx, hy * 2.0, hz],
        [-hx, hy * 2.0, hz],
    ]
}

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
    let mut changed = false;

    for part in &state.base.parts {
        active_ids.insert(part.id, part.door_open);

        // Open doors do not block passage
        if part.door_open {
            if let Some(handle) = state.part_colliders.remove(&part.id) {
                physics.remove(handle);
                changed = true;
            }
            continue;
        }

        // If part doesn't have an active collider, build one from its pack collision shapes or fallback
        if let std::collections::hash_map::Entry::Vacant(entry) = state.part_colliders.entry(part.id) {
            let model_name = part_model_name(&part.part);
            let pack_shapes: Vec<Vec<[f32; 3]>> = shapes_cache
                .entry(model_name.clone())
                .or_insert_with(|| {
                    pack.find(&model_name)
                        .map(|m| m.colliders.iter().map(|c| c.points.clone()).collect())
                        .unwrap_or_default()
                })
                .clone();

            let shapes = if !pack_shapes.is_empty() {
                pack_shapes
            } else {
                vec![fallback_collider_shape(&part.part)]
            };

            let handle = physics.add_static(&shapes, part.position, part.yaw, 1.0);
            entry.insert(handle);
            changed = true;
        }
    }

    // Clean up colliders for parts that were removed or collapsed
    let stale_ids: Vec<u32> = state
        .part_colliders
        .keys()
        .copied()
        .filter(|id| !active_ids.contains_key(id))
        .collect();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_colliders_add_and_door_toggle() {
        let mut state = BuildModeState::new();
        let mut physics = PhysicsWorld::new();
        let pack = pack::ModelPack::new(Vec::new());

        state.base.parts.push(game::building::PlacedPart {
            id: 1,
            part: "wall".into(),
            cell: glam::IVec2::ZERO,
            floor: 0,
            rot: 0,
            hp: 150.0,
            door_open: false,
            position: glam::Vec3::ZERO,
            yaw: 0.0,
        });

        // 1. Initial sync adds collider
        sync_colliders(&mut state, &mut physics, &pack);
        assert_eq!(state.part_colliders.len(), 1);

        // 2. Toggling door open removes collider
        state.base.parts[0].door_open = true;
        sync_colliders(&mut state, &mut physics, &pack);
        assert_eq!(state.part_colliders.len(), 0);

        // 3. Closing door re-adds collider
        state.base.parts[0].door_open = false;
        sync_colliders(&mut state, &mut physics, &pack);
        assert_eq!(state.part_colliders.len(), 1);

        // 4. Removing part cleans up collider
        state.base.parts.clear();
        sync_colliders(&mut state, &mut physics, &pack);
        assert_eq!(state.part_colliders.len(), 0);
    }
}
