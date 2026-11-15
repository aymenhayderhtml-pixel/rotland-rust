//! Works out which flow-field cells a walker can stand in, by asking the physics world about each cell centre.
//! Kept apart from the search so the search stays plain grid code.

use glam::Vec3;
use physics::PhysicsWorld;

/// Steepest floor a walker climbs. Steeper cells count as walls for the flow field.
pub const MAX_FLOOR_ANGLE_DEG: f32 = 46.0;

/// The band above the floor that must be clear: from just above the ground up to 1.5 m,
/// which covers knee height (about 0.5 m) and the body, so low props and walls both block.
const BODY_START: f32 = 0.1;
const BODY_HEIGHT: f32 = 1.4;

/// The floor is found by a ray cast upward from this far below the terrain. Casting down from
/// above would hit tree canopies and roofs first and miss the ground under them.
const BELOW_TERRAIN: f32 = 100.0;
const RAY_LENGTH: f32 = 400.0;

/// World (x, z) of the centre of cell (i, j) of a grid whose lowest cell is `min` (in cell units).
pub fn cell_centre(min: (i32, i32), i: usize, j: usize, cell: f32) -> (f32, f32) {
    (
        ((min.0 + i as i32) as f32 + 0.5) * cell,
        ((min.1 + j as i32) as f32 + 0.5) * cell,
    )
}

/// Fills `blocked` (row-major, `side * side` cells, index = j * side + i) for the grid.
pub fn scan_blocked(
    physics: &PhysicsWorld,
    min: (i32, i32),
    cell: f32,
    side: usize,
    blocked: &mut [bool],
) {
    let min_up = MAX_FLOOR_ANGLE_DEG.to_radians().cos();
    for j in 0..side {
        for i in 0..side {
            let (x, z) = cell_centre(min, i, j, cell);
            blocked[j * side + i] = cell_blocked(physics, x, z, min_up);
        }
    }
}

/// True if a walker cannot stand at (x, z): no ground, a slope too steep, or something solid
/// in the body band above the ground.
fn cell_blocked(physics: &PhysicsWorld, x: f32, z: f32, min_up: f32) -> bool {
    let Some(floor) = physics.raycast(Vec3::new(x, -BELOW_TERRAIN, z), Vec3::Y, RAY_LENGTH) else {
        return true;
    };
    // The normal's vertical part is the slope test; abs() because the ray arrives from below.
    if floor.normal.y.abs() < min_up {
        return true;
    }
    // A prop whose base sits on this cell starts inside its own collider, so the ray reports a hit at 0.
    let band_start = Vec3::new(x, floor.point.y + BODY_START, z);
    physics.raycast(band_start, Vec3::Y, BODY_HEIGHT).is_some()
}
