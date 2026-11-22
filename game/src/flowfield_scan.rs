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

/// The grid of the previous scan: its lowest cell plus its answers, so cells still inside the new
/// grid are copied instead of looked up again (standing still costs nothing).
pub struct ScanGrid {
    pub min: Option<(i32, i32)>,
    pub blocked: Vec<bool>,
    /// False where the cell had no ground yet (its chunk was not loaded): checked again next scan.
    pub known: Vec<bool>,
}

impl ScanGrid {
    pub fn new(count: usize) -> Self {
        Self { min: None, blocked: vec![false; count], known: vec![false; count] }
    }
}

/// Fills `out` (row-major, `side * side` cells, index = j * side + i) for the grid at `min`.
/// Trees, rocks and buildings never move: cells inside the previous grid `prev` are copied, other
/// cells come from `cache`, and only never-seen cells cost rays.
/// Cells with no ground yet (their chunk has not loaded) are not remembered, so they are checked again.
/// Returns false when the grid did not move and every cell was copied (the last search still holds).
pub fn scan_blocked(
    physics: &PhysicsWorld,
    min: (i32, i32),
    cell: f32,
    side: usize,
    prev: &ScanGrid,
    out: &mut ScanGrid,
    cache: &mut HashMap<(i32, i32), bool>,
) -> bool {
    if cache.len() > CACHE_LIMIT {
        cache.clear();
    }
    let min_up = MAX_FLOOR_ANGLE_DEG.to_radians().cos();
    let n = side as i32;
    let mut changed = prev.min != Some(min);
    for j in 0..side {
        for i in 0..side {
            let u = j * side + i;
            let (wx, wz) = (min.0 + i as i32, min.1 + j as i32);
            if let Some(pm) = prev.min {
                let (pi, pj) = (wx - pm.0, wz - pm.1);
                if pi >= 0 && pj >= 0 && pi < n && pj < n {
                    let pu = pj as usize * side + pi as usize;
                    if prev.known[pu] {
                        out.blocked[u] = prev.blocked[pu];
                        out.known[u] = true;
                        continue;
                    }
                }
            }
            changed = true;
            let answer = match cache.get(&(wx, wz)) {
                Some(&b) => Some(b),
                None => {
                    let (x, z) = cell_centre(min, i, j, cell);
                    let b = cell_blocked(physics, x, z, min_up);
                    if let Some(b) = b {
                        cache.insert((wx, wz), b);
                    }
                    b
                }
            };
            out.blocked[u] = answer.unwrap_or(true);
            out.known[u] = answer.is_some();
        }
    }
    out.min = Some(min);
    changed
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
