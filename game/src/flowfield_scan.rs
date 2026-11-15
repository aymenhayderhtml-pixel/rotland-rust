//! Works out which flow-field cells a walker can stand in, by asking the physics world about each cell centre.
//! Kept apart from the search so the search stays plain grid code.

use std::collections::HashMap;
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
/// Trees, rocks and buildings never move, so each world cell's answer is remembered in `cache` and
/// only new cells cost rays (~32,000 rays per full scan before, a few hundred per rebuild after).
/// Cells with no ground yet (their chunk has not loaded) are not remembered, so they are checked again.
pub fn scan_blocked(
    physics: &PhysicsWorld,
    min: (i32, i32),
    cell: f32,
    side: usize,
    blocked: &mut [bool],
    cache: &mut HashMap<(i32, i32), bool>,
) {
    if cache.len() > CACHE_LIMIT {
        cache.clear();
    }
    let min_up = MAX_FLOOR_ANGLE_DEG.to_radians().cos();
    for j in 0..side {
        for i in 0..side {
            let key = (min.0 + i as i32, min.1 + j as i32);
            blocked[j * side + i] = match cache.get(&key) {
                Some(&b) => b,
                None => {
                    let (x, z) = cell_centre(min, i, j, cell);
                    match cell_blocked(physics, x, z, min_up) {
                        Some(b) => {
                            cache.insert(key, b);
                            b
                        }
                        None => true,
                    }
                }
            };
        }
    }
}

/// About 40 full grids of cells; past this the memory is simply dropped and rebuilt as needed.
const CACHE_LIMIT: usize = 650_000;

/// Some(true) if a walker cannot stand at (x, z): a slope too steep, or something solid in the body
/// band above the ground. None when there is no ground there (yet).
fn cell_blocked(physics: &PhysicsWorld, x: f32, z: f32, min_up: f32) -> Option<bool> {
    let floor = physics.raycast(Vec3::new(x, -BELOW_TERRAIN, z), Vec3::Y, RAY_LENGTH)?;
    // The normal's vertical part is the slope test; abs() because the ray arrives from below.
    if floor.normal.y.abs() < min_up {
        return Some(true);
    }
    // A prop whose base sits on this cell starts inside its own collider, so the ray reports a hit at 0.
    let band_start = Vec3::new(x, floor.point.y + BODY_START, z);
    Some(physics.raycast(band_start, Vec3::Y, BODY_HEIGHT).is_some())
}
