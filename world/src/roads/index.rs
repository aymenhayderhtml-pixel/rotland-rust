use std::collections::HashMap;

use glam::Vec2;

use crate::roads::{Road, RoadKind};

// new: DECISIONS 2026-11-25
const CELL_SIZE_M: f32 = 64.0;
// new: DECISIONS 2026-11-25
const MAX_QUERY_DISTANCE_M: f32 = 30.0;
// new: DECISIONS 2026-11-25
const MAX_DENSE_GRID_CELLS: usize = 262_144;
// new: DECISIONS 2026-11-25
const TIE_EPSILON_M: f32 = 1e-4;

#[derive(Debug, Clone, Copy)]
struct RoadSegment {
    a: Vec2,
    b: Vec2,
    kind: RoadKind,
}

#[derive(Debug, Clone)]
enum GridStorage {
    Empty,
    Dense {
        min_cx: i32,
        min_cz: i32,
        width: usize,
        height: usize,
        offsets: Vec<u32>,
        entries: Vec<u32>,
    },
    Sparse(HashMap<(i32, i32), Vec<u32>>),
}

/// Uniform 64 m grid spatial index for fast road distance queries.
#[derive(Debug, Clone)]
pub(crate) struct RoadSpatialIndex {
    segments: Vec<RoadSegment>,
    storage: GridStorage,
}

impl RoadSpatialIndex {
    pub(crate) fn build(roads: &[Road]) -> Self {
        let mut segments = Vec::new();
        for road in roads {
            for pair in road.points.windows(2) {
                segments.push(RoadSegment {
                    a: pair[0],
                    b: pair[1],
                    kind: road.kind,
                });
            }
        }

        if segments.is_empty() {
            return Self {
                segments,
                storage: GridStorage::Empty,
            };
        }

        let mut min_x = f32::INFINITY;
        let mut min_z = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for seg in &segments {
            min_x = min_x.min(seg.a.x).min(seg.b.x);
            min_z = min_z.min(seg.a.y).min(seg.b.y);
            max_x = max_x.max(seg.a.x).max(seg.b.x);
            max_z = max_z.max(seg.a.y).max(seg.b.y);
        }

        let min_cx = ((min_x - MAX_QUERY_DISTANCE_M) / CELL_SIZE_M).floor() as i32;
        let min_cz = ((min_z - MAX_QUERY_DISTANCE_M) / CELL_SIZE_M).floor() as i32;
        let max_cx = ((max_x + MAX_QUERY_DISTANCE_M) / CELL_SIZE_M).floor() as i32;
        let max_cz = ((max_z + MAX_QUERY_DISTANCE_M) / CELL_SIZE_M).floor() as i32;

        let width = (max_cx - min_cx + 1) as usize;
        let height = (max_cz - min_cz + 1) as usize;
        let cell_count = width.saturating_mul(height);

        let storage = if cell_count <= MAX_DENSE_GRID_CELLS {
            let mut buckets: Vec<Vec<u32>> = vec![Vec::new(); cell_count];
            for (seg_idx, seg) in segments.iter().enumerate() {
                let seg_id = seg_idx as u32;
                for_each_touching_cell(seg.a, seg.b, |cx, cz| {
                    if cx >= min_cx && cx <= max_cx && cz >= min_cz && cz <= max_cz {
                        let idx = (cz - min_cz) as usize * width + (cx - min_cx) as usize;
                        buckets[idx].push(seg_id);
                    }
                });
            }

            let total_entries: usize = buckets.iter().map(Vec::len).sum();
            let mut offsets = Vec::with_capacity(cell_count + 1);
            let mut entries = Vec::with_capacity(total_entries);
            offsets.push(0);
            for bucket in buckets {
                entries.extend_from_slice(&bucket);
                offsets.push(entries.len() as u32);
            }

            GridStorage::Dense {
                min_cx,
                min_cz,
                width,
                height,
                offsets,
                entries,
            }
        } else {
            let mut sparse: HashMap<(i32, i32), Vec<u32>> = HashMap::new();
            for (seg_idx, seg) in segments.iter().enumerate() {
                let seg_id = seg_idx as u32;
                for_each_touching_cell(seg.a, seg.b, |cx, cz| {
                    sparse.entry((cx, cz)).or_default().push(seg_id);
                });
            }
            GridStorage::Sparse(sparse)
        };

        Self { segments, storage }
    }

    pub(crate) fn distance_to_road(&self, p: Vec2) -> Option<(f32, RoadKind)> {
        let cx = (p.x / CELL_SIZE_M).floor() as i32;
        let cz = (p.y / CELL_SIZE_M).floor() as i32;
        let max_dist_sq = MAX_QUERY_DISTANCE_M * MAX_QUERY_DISTANCE_M;
        let mut best_dist_sq = max_dist_sq;
        let mut best_kind: Option<RoadKind> = None;

        match &self.storage {
            GridStorage::Empty => return None,
            GridStorage::Dense {
                min_cx,
                min_cz,
                width,
                height,
                offsets,
                entries,
            } => {
                let max_cx = *min_cx + *width as i32 - 1;
                let max_cz = *min_cz + *height as i32 - 1;
                if cx < *min_cx || cx > max_cx || cz < *min_cz || cz > max_cz {
                    return None;
                }

                for dz in -1..=1 {
                    let nz = cz + dz;
                    if nz < *min_cz || nz > max_cz {
                        continue;
                    }
                    let row_offset = (nz - *min_cz) as usize * *width;
                    for dx in -1..=1 {
                        let nx = cx + dx;
                        if nx < *min_cx || nx > max_cx {
                            continue;
                        }
                        let cell_idx = row_offset + (nx - *min_cx) as usize;
                        let start = offsets[cell_idx] as usize;
                        let end = offsets[cell_idx + 1] as usize;
                        for &seg_id in &entries[start..end] {
                            self.consider_segment(
                                p,
                                seg_id as usize,
                                &mut best_dist_sq,
                                &mut best_kind,
                            );
                        }
                    }
                }
            }
            GridStorage::Sparse(map) => {
                for dz in -1..=1 {
                    for dx in -1..=1 {
                        if let Some(bucket) = map.get(&(cx + dx, cz + dz)) {
                            for &seg_id in bucket {
                                self.consider_segment(
                                    p,
                                    seg_id as usize,
                                    &mut best_dist_sq,
                                    &mut best_kind,
                                );
                            }
                        }
                    }
                }
            }
        }

        best_kind.map(|kind| (best_dist_sq.sqrt(), kind))
    }

    #[inline]
    fn consider_segment(
        &self,
        p: Vec2,
        seg_idx: usize,
        best_dist_sq: &mut f32,
        best_kind: &mut Option<RoadKind>,
    ) {
        let seg = self.segments[seg_idx];
        let d_sq = point_segment_distance_sq(p, seg.a, seg.b);
        if d_sq > *best_dist_sq {
            return;
        }
        let is_better = match *best_kind {
            None => true,
            Some(prev_kind) => {
                let d = d_sq.sqrt();
                let prev_d = best_dist_sq.sqrt();
                d + TIE_EPSILON_M < prev_d
                    || ((d - prev_d).abs() <= TIE_EPSILON_M
                        && matches!(seg.kind, RoadKind::Highway)
                        && matches!(prev_kind, RoadKind::Trail))
            }
        };
        if is_better {
            *best_dist_sq = d_sq;
            *best_kind = Some(seg.kind);
        }
    }
}

fn for_each_touching_cell(a: Vec2, b: Vec2, mut f: impl FnMut(i32, i32)) {
    let min_cx = (a.x.min(b.x) / CELL_SIZE_M).floor() as i32;
    let max_cx = (a.x.max(b.x) / CELL_SIZE_M).floor() as i32;
    let min_cz = (a.y.min(b.y) / CELL_SIZE_M).floor() as i32;
    let max_cz = (a.y.max(b.y) / CELL_SIZE_M).floor() as i32;

    for cz in min_cz..=max_cz {
        for cx in min_cx..=max_cx {
            if segment_intersects_cell(a, b, cx, cz) {
                f(cx, cz);
            }
        }
    }
}

fn segment_intersects_cell(a: Vec2, b: Vec2, cx: i32, cz: i32) -> bool {
    let x0 = cx as f32 * CELL_SIZE_M;
    let x1 = x0 + CELL_SIZE_M;
    let z0 = cz as f32 * CELL_SIZE_M;
    let z1 = z0 + CELL_SIZE_M;

    let dx = b.x - a.x;
    let dz = b.y - a.y;
    let mut t0 = 0.0_f32;
    let mut t1 = 1.0_f32;

    let checks = [
        (-dx, a.x - x0),
        (dx, x1 - a.x),
        (-dz, a.y - z0),
        (dz, z1 - a.y),
    ];

    for (p, q) in checks {
        if p.abs() <= f32::EPSILON {
            if q < 0.0 {
                return false;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                if r > t1 {
                    return false;
                }
                if r > t0 {
                    t0 = r;
                }
            } else {
                if r < t0 {
                    return false;
                }
                if r < t1 {
                    t1 = r;
                }
            }
        }
    }

    t0 <= t1
}

#[inline]
fn point_segment_distance_sq(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq <= f32::EPSILON {
        return p.distance_squared(a);
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    let closest = a + ab * t;
    p.distance_squared(closest)
}
