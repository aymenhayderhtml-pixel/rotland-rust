//! Flow field: one shared search from the target (the player) gives every chasing zombie the way to
//! walk around trees, rocks and buildings. One rebuild serves all zombies; there is no path per zombie.

use glam::Vec3;
use physics::PhysicsWorld;

use crate::flowfield_scan::{cell_centre, scan_blocked, ScanGrid};

/// Step costs as integers (14 is about 10 × √2), so the search never needs floats.
const STRAIGHT: u32 = 10;
const DIAGONAL: u32 = 14;
const UNREACHABLE: u32 = u32::MAX;
/// Bucket queue ring size: every step costs less than this, so a step never lands in the bucket
/// being emptied (Dial's algorithm: Dijkstra with small whole-number costs, no heap).
const BUCKETS: usize = DIAGONAL as usize + 1;

/// The 8 steps from a cell: (column step, row step, cost).
const STEPS: [(isize, isize, u32); 8] = [
    (1, 0, STRAIGHT),
    (-1, 0, STRAIGHT),
    (0, 1, STRAIGHT),
    (0, -1, STRAIGHT),
    (1, 1, DIAGONAL),
    (1, -1, DIAGONAL),
    (-1, 1, DIAGONAL),
    (-1, -1, DIAGONAL),
];

/// A square grid of cells centred on the target, with each cell's walking distance to the target.
/// All buffers are allocated once in `new` and reused by every `rebuild`.
pub struct FlowField {
    cell: f32,
    half_size: usize,
    side: usize,
    /// Cell index (in whole cells) of the grid's lowest-x, lowest-z cell.
    min: (i32, i32),
    /// This rebuild's obstacle answers, and the previous rebuild's (swapped each rebuild).
    scan: ScanGrid,
    prev_scan: ScanGrid,
    dist: Vec<u32>,
    /// True once `dist` holds a search result.
    searched: bool,
    buckets: Vec<Vec<u32>>,
    target: Vec3,
    target_idx: usize,
    /// Remembered blocked/free answer per world cell (static obstacles), see scan_blocked.
    cache: std::collections::HashMap<(i32, i32), bool>,
}

impl Default for FlowField {
    /// 1 m cells, 128 m across: the size the zombies need.
    fn default() -> Self {
        Self::new(1.0, 64)
    }
}

impl FlowField {
    pub fn new(cell_size: f32, half_size: usize) -> Self {
        assert!(half_size > 0, "flow field needs at least one cell in each direction");
        let side = half_size * 2;
        let count = side * side;
        Self {
            cell: cell_size,
            half_size,
            side,
            min: (0, 0),
            scan: ScanGrid::new(count),
            prev_scan: ScanGrid::new(count),
            dist: vec![UNREACHABLE; count],
            searched: false,
            buckets: vec![Vec::new(); BUCKETS],
            target: Vec3::ZERO,
            target_idx: 0,
            cache: std::collections::HashMap::new(),
        }
    }

    /// Centres the grid on `target`, marks the blocked cells from the physics world and searches
    /// outward from the target's cell once.
    pub fn rebuild(&mut self, physics: &PhysicsWorld, target: Vec3) {
        let tx = (target.x / self.cell).floor() as i32;
        let tz = (target.z / self.cell).floor() as i32;
        let half = self.half_size as i32;
        self.min = (tx - half, tz - half);
        self.target = target;
        // The target's cell is always the grid's centre cell.
        self.target_idx = self.half_size * self.side + self.half_size;
        std::mem::swap(&mut self.scan, &mut self.prev_scan);
        let changed = scan_blocked(physics, self.min, self.cell, self.side, &self.prev_scan, &mut self.scan, &mut self.cache);
        // Same grid, same obstacles: the distances from the last search are still right.
        if changed || !self.searched {
            self.search();
            self.searched = true;
        }
    }

    /// Forgets the remembered obstacle answers. Call when something solid appears or disappears in a
    /// place already scanned (doors, barricades); loading and unloading chunks does not need it.
    pub fn forget(&mut self) {
        self.cache.clear();
        self.scan.min = None;
        self.prev_scan.min = None;
        self.searched = false;
    }

    /// Walking direction (flat, unit length) for a walker at `from`. None outside the grid, on a cell
    /// the target cannot be reached from, or exactly on the target (the caller then walks straight).
    pub fn direction(&self, from: Vec3) -> Option<Vec3> {
        let (i, j) = self.cell_at(from)?;
        let u = j * self.side + i;
        let here = self.dist[u];
        if here == UNREACHABLE {
            return None;
        }
        if u == self.target_idx {
            return flat_unit(self.target - from);
        }
        let mut best: Option<(u32, usize)> = None;
        for &(di, dj, _) in &STEPS {
            if let Some(v) = self.neighbour(i, j, (di, dj)) {
                let d = self.dist[v];
                if d < here && best.is_none_or(|(bd, _)| d < bd) {
                    best = Some((d, v));
                }
            }
        }
        let (_, v) = best?;
        let (cx, cz) = cell_centre(self.min, v % self.side, v / self.side, self.cell);
        flat_unit(Vec3::new(cx, 0.0, cz) - Vec3::new(from.x, 0.0, from.z))
    }

    /// One Dijkstra from the target cell over the open cells, with a ring of buckets instead of a
    /// heap (all step costs are small whole numbers): about 3x faster on 16k cells.
    fn search(&mut self) {
        self.dist.fill(UNREACHABLE);
        for b in &mut self.buckets {
            b.clear();
        }
        self.dist[self.target_idx] = 0;
        self.buckets[0].push(self.target_idx as u32);
        let mut pending = 1usize;
        let mut d = 0u32;
        while pending > 0 {
            let slot = d as usize % BUCKETS;
            let mut bucket = std::mem::take(&mut self.buckets[slot]);
            pending -= bucket.len();
            for &u in &bucket {
                let u = u as usize;
                if self.dist[u] != d {
                    continue;
                }
                let (i, j) = (u % self.side, u / self.side);
                for &(di, dj, cost) in &STEPS {
                    if let Some(v) = self.neighbour(i, j, (di, dj)) {
                        let nd = d + cost;
                        if nd < self.dist[v] {
                            self.dist[v] = nd;
                            self.buckets[nd as usize % BUCKETS].push(v as u32);
                            pending += 1;
                        }
                    }
                }
            }
            bucket.clear();
            self.buckets[slot] = bucket;
            d += 1;
        }
    }

    /// The cell one step from (i, j), if it is open. A diagonal step may not cut the corner of a
    /// blocked cell, or the walker would clip through a wall's end.
    fn neighbour(&self, i: usize, j: usize, step: (isize, isize)) -> Option<usize> {
        let ni = i.checked_add_signed(step.0)?;
        let nj = j.checked_add_signed(step.1)?;
        if ni >= self.side || nj >= self.side {
            return None;
        }
        if self.scan.blocked[nj * self.side + ni] {
            return None;
        }
        if step.0 != 0 && step.1 != 0
            && (self.scan.blocked[j * self.side + ni] || self.scan.blocked[nj * self.side + i])
        {
            return None;
        }
        Some(nj * self.side + ni)
    }

    /// The grid cell containing `p`, or None outside the grid.
    fn cell_at(&self, p: Vec3) -> Option<(usize, usize)> {
        let i = (p.x / self.cell).floor() as i32 - self.min.0;
        let j = (p.z / self.cell).floor() as i32 - self.min.1;
        let side = self.side as i32;
        if i < 0 || j < 0 || i >= side || j >= side {
            return None;
        }
        Some((i as usize, j as usize))
    }
}

/// The flat (XZ) unit vector of `v`, or None when `v` is (almost) zero.
fn flat_unit(v: Vec3) -> Option<Vec3> {
    let flat = Vec3::new(v.x, 0.0, v.z);
    let len = flat.length();
    if len < 1e-4 {
        None
    } else {
        Some(flat / len)
    }
}
