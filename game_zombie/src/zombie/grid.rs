//! Spatial grid for fast, allocation-free neighbor queries of herd zombies.

use glam::Vec3;

use super::herd::HerdZombie;

#[derive(Clone, Copy, Debug)]
struct GridEntry {
    cell_x: i32,
    cell_z: i32,
    zombie: HerdZombie,
    next: Option<u32>,
}

/// A 2D spatial hash grid with fixed buckets and entry pooling.
///
/// Rebuilding the grid once per frame and querying nearby cells avoids
/// all heap allocations during the frame update loop.
#[derive(Clone, Debug)]
pub struct SpatialGrid {
    cell_size: f32,
    buckets: Vec<Option<u32>>,
    entries: Vec<GridEntry>,
}

impl SpatialGrid {
    /// Creates a spatial grid with the given cell size (meters) and bucket count.
    pub fn new(cell_size: f32, bucket_count: usize) -> Self {
        Self {
            cell_size: cell_size.max(0.1),
            buckets: vec![None; bucket_count.max(16)],
            entries: Vec::with_capacity(128),
        }
    }

    /// Clears entries and resets buckets without deallocating vector capacity.
    pub fn clear(&mut self) {
        self.buckets.fill(None);
        self.entries.clear();
    }

    fn hash_cell(&self, cx: i32, cz: i32) -> usize {
        let h = (cx as i64).wrapping_mul(73856093) ^ (cz as i64).wrapping_mul(19349663);
        (h.rem_euclid(self.buckets.len() as i64)) as usize
    }

    /// Inserts a zombie into the grid based on its XZ ground position.
    pub fn insert(&mut self, zombie: HerdZombie) {
        let cx = (zombie.position.x / self.cell_size).floor() as i32;
        let cz = (zombie.position.z / self.cell_size).floor() as i32;
        let bucket = self.hash_cell(cx, cz);
        let next = self.buckets[bucket];
        let idx = self.entries.len() as u32;
        self.entries.push(GridEntry {
            cell_x: cx,
            cell_z: cz,
            zombie,
            next,
        });
        self.buckets[bucket] = Some(idx);
    }

    /// Queries all zombies in the 3x3 cells surrounding `pos`, writing them into `out`.
    ///
    /// Clears `out` before writing. Does not allocate if `out` has sufficient capacity.
    pub fn query_nearby(&self, pos: Vec3, out: &mut Vec<HerdZombie>) {
        out.clear();
        let cx = (pos.x / self.cell_size).floor() as i32;
        let cz = (pos.z / self.cell_size).floor() as i32;
        for dx in -1..=1 {
            for dz in -1..=1 {
                let qx = cx + dx;
                let qz = cz + dz;
                let bucket = self.hash_cell(qx, qz);
                let mut curr = self.buckets[bucket];
                while let Some(idx) = curr {
                    let entry = &self.entries[idx as usize];
                    if entry.cell_x == qx && entry.cell_z == qz {
                        out.push(entry.zombie);
                    }
                    curr = entry.next;
                }
            }
        }
    }
}
