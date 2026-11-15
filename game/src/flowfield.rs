//! Flow field: one shared search from the target (the player) gives every chasing zombie the way to
//! walk around trees, rocks and buildings. One rebuild serves all zombies; there is no path per zombie.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use glam::Vec3;
use physics::PhysicsWorld;

use crate::flowfield_scan::{cell_centre, scan_blocked};

/// Step costs as integers (14 is about 10 × √2), so the search never needs floats.
const STRAIGHT: u32 = 10;
const DIAGONAL: u32 = 14;
const UNREACHABLE: u32 = u32::MAX;

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
    blocked: Vec<bool>,
    dist: Vec<u32>,
    heap: BinaryHeap<Reverse<(u32, u32)>>,
    target: Vec3,
    target_idx: usize,
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
            blocked: vec![false; count],
            dist: vec![UNREACHABLE; count],
            heap: BinaryHeap::with_capacity(count),
            target: Vec3::ZERO,
            target_idx: 0,
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
        scan_blocked(physics, self.min, self.cell, self.side, &mut self.blocked);
        self.search();
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

    /// One Dijkstra from the target cell over the open cells.
    fn search(&mut self) {
        self.dist.fill(UNREACHABLE);
        self.heap.clear();
        self.dist[self.target_idx] = 0;
        self.heap.push(Reverse((0, self.target_idx as u32)));
        while let Some(Reverse((d, u))) = self.heap.pop() {
            let u = u as usize;
            if d > self.dist[u] {
                continue;
            }
            let (i, j) = (u % self.side, u / self.side);
            for &(di, dj, cost) in &STEPS {
                if let Some(v) = self.neighbour(i, j, (di, dj)) {
                    let nd = d + cost;
                    if nd < self.dist[v] {
                        self.dist[v] = nd;
                        self.heap.push(Reverse((nd, v as u32)));
                    }
                }
            }
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
        if self.blocked[nj * self.side + ni] {
            return None;
        }
        if step.0 != 0 && step.1 != 0
            && (self.blocked[j * self.side + ni] || self.blocked[nj * self.side + i])
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
