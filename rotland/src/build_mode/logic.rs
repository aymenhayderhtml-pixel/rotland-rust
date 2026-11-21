//! Core placement checks, grid snapping, and interaction logic for build mode.

use engine::render::Camera;
use game::building::BuildPlan;
use game::Inventory;
use glam::{IVec2, Vec2, Vec3};
use physics::PhysicsWorld;

use super::state::BuildModeState;
use super::types::part_display_name;

/// Reach distance for aiming at and placing base parts.
const BUILD_REACH: f32 = 12.0;

/// Updates the ghost preview snap position and validation for the current frame.
pub fn update_raycast(
    state: &mut BuildModeState,
    camera: &Camera,
    physics: &PhysicsWorld,
) {
    if !state.active {
        state.current_placement = None;
        state.last_error = None;
        state.aimed_part_id = None;
        return;
    }

    let eye = camera.position;
    let dir = camera.forward();

    // Check aimed placed part for removal / door interaction
    state.aimed_part_id = state.base.raycast_part(eye, dir, BUILD_REACH);

    // Raycast against physics world to find ground or structure hit point
    let hit_point = if let Some(hit) = physics.raycast(eye, dir, BUILD_REACH) {
        hit.point
    } else {
        // Fallback: point in front of camera
        eye + dir * 5.0
    };

    let grid_size = state.config.grid_size;
    let floor_height = state.config.floor_height;

    let cell_x = (hit_point.x / grid_size).round() as i32;
    let cell_z = (hit_point.z / grid_size).round() as i32;
    let cell = IVec2::new(cell_x, cell_z);

    // Ground height closure using downward raycast from high above
    let ground_fn = |x: f32, z: f32| -> f32 {
        physics
            .raycast(Vec3::new(x, 100.0, z), Vec3::NEG_Y, 200.0)
            .map(|h| h.point.y)
            .unwrap_or(0.0)
    };

    let ground_center = ground_fn(cell_x as f32 * grid_size, cell_z as f32 * grid_size);
    let floor = if floor_height > 0.0 && hit_point.y > ground_center + floor_height * 0.5 {
        ((hit_point.y - ground_center) / floor_height).round().max(0.0) as u32
    } else {
        0
    };

    let part = state.selected_part();
    let rot = state.rotation;

    let blocked_fn = |_min: Vec2, _max: Vec2| -> bool { false };

    let result = BuildPlan::check(
        &state.base,
        &state.config,
        part,
        cell,
        floor,
        rot,
        ground_fn,
        blocked_fn,
    );

    match result {
        Ok(placement) => {
            state.current_placement = Some(placement);
            state.last_error = None;
        }
        Err(err) => {
            // Calculate a candidate position for drawing the red invalid preview box
            let yaw = game::building::rot_to_yaw(rot);
            let mut pos = Vec3::new(
                cell_x as f32 * grid_size,
                ground_center + floor as f32 * floor_height,
                cell_z as f32 * grid_size,
            );
            if game::building::is_edge_part(part) {
                let (sy, cy) = yaw.sin_cos();
                pos.x += -sy * (grid_size * 0.5);
                pos.z += -cy * (grid_size * 0.5);
            }
            state.current_placement = Some(game::building::Placement {
                part: part.to_string(),
                cell,
                floor,
                rot,
                position: pos,
                yaw,
                hp: 100.0,
                cost: std::collections::HashMap::new(),
            });
            state.last_error = Some(err);
        }
    }
}

/// Attempts to place the currently selected building part, deducting cost from inventory.
pub fn try_place(state: &mut BuildModeState, inventory: Option<&mut Inventory>) -> bool {
    if !state.active {
        return false;
    }
    if state.last_error.is_some() {
        return false;
    }
    let Some(placement) = state.current_placement.clone() else {
        return false;
    };

    let Some(def) = state.config.get(&placement.part).cloned() else {
        return false;
    };

    if let Some(inv) = inventory {
        // Verify inventory has required materials
        for (item, qty) in &def.cost {
            if inv.count(item) < *qty {
                state.toast = Some(format!("Not enough {item}"));
                return false;
            }
        }
        // Deduct materials from bag
        for (item, qty) in &def.cost {
            inv.remove(item, *qty);
        }
    }

    let part_name = placement.part.clone();
    state.base.place(placement);
    state.toast = Some(format!("Placed {}", part_display_name(&part_name)));
    true
}

/// Attempts to remove the aimed building part, refunding materials back to inventory.
pub fn try_remove(state: &mut BuildModeState, inventory: Option<&mut Inventory>) -> bool {
    if !state.active {
        return false;
    }
    let Some(aimed_id) = state.aimed_part_id else {
        return false;
    };

    if let Some(refund) = state.base.remove(&state.config, aimed_id) {
        if let Some(inv) = inventory {
            for (item, count) in refund {
                inv.add(&item, count);
            }
        }
        state.toast = Some("Removed part (+refund)".into());
        state.aimed_part_id = None;
        return true;
    }
    false
}

/// Attempts to toggle the door of the aimed building part.
pub fn try_toggle_door(state: &mut BuildModeState) -> bool {
    let Some(aimed_id) = state.aimed_part_id else {
        return false;
    };

    let is_door = state
        .base
        .get(aimed_id)
        .map(|p| p.part == "wall_door")
        .unwrap_or(false);

    if is_door {
        let is_open = state.base.toggle_door(aimed_id);
        state.toast = Some(if is_open {
            "Door opened".into()
        } else {
            "Door closed".into()
        });
        return true;
    }
    false
}
