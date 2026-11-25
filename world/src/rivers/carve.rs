use std::collections::{HashMap, HashSet};

use glam::Vec2;

use crate::math::{lerp, smooth_step};
use crate::rivers::{River, RiversData};

// new: DECISIONS 2026-11-25
const HALF: f32 = 0.5;
// new: DECISIONS 2026-11-25
const PARALLEL_EPS: f32 = 1e-6;
// data: rivers.json bank_m
const TWO_BANKS: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SegmentRef {
    pub river_idx: usize,
    pub seg_idx: usize,
}

/// Coarse spatial grid indexing padded river segment AABBs into square cells.
#[derive(Debug, Clone)]
pub(crate) struct SegmentGrid {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<SegmentRef>>,
}

impl SegmentGrid {
    pub(crate) fn build(rivers: &[River], bank_m: f32, cell_size: f32) -> Self {
        let mut cells: HashMap<(i32, i32), Vec<SegmentRef>> = HashMap::new();
        for (river_idx, river) in rivers.iter().enumerate() {
            if river.points.len() < 2 {
                continue;
            }
            for seg_idx in 0..(river.points.len() - 1) {
                let p0 = river.points[seg_idx];
                let p1 = river.points[seg_idx + 1];
                let max_half_w = river.widths[seg_idx].max(river.widths[seg_idx + 1]) * HALF;
                let pad = max_half_w + bank_m;

                let min_x = p0.x.min(p1.x) - pad;
                let max_x = p0.x.max(p1.x) + pad;
                let min_z = p0.y.min(p1.y) - pad;
                let max_z = p0.y.max(p1.y) + pad;

                let cx0 = cell_coord(min_x, cell_size);
                let cx1 = cell_coord(max_x, cell_size);
                let cz0 = cell_coord(min_z, cell_size);
                let cz1 = cell_coord(max_z, cell_size);

                let seg_ref = SegmentRef { river_idx, seg_idx };
                for cz in cz0..=cz1 {
                    for cx in cx0..=cx1 {
                        cells.entry((cx, cz)).or_default().push(seg_ref);
                    }
                }
            }
        }
        Self { cell_size, cells }
    }

    pub(crate) fn cell_segments(&self, pos: Vec2) -> &[SegmentRef] {
        let key = (
            cell_coord(pos.x, self.cell_size),
            cell_coord(pos.y, self.cell_size),
        );
        self.cells.get(&key).map(Vec::as_slice).unwrap_or(&[])
    }

    pub(crate) fn segments_in_aabb(&self, min: Vec2, max: Vec2) -> Vec<SegmentRef> {
        let cx0 = cell_coord(min.x, self.cell_size);
        let cx1 = cell_coord(max.x, self.cell_size);
        let cz0 = cell_coord(min.y, self.cell_size);
        let cz1 = cell_coord(max.y, self.cell_size);

        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for cz in cz0..=cz1 {
            for cx in cx0..=cx1 {
                if let Some(bucket) = self.cells.get(&(cx, cz)) {
                    for &seg in bucket {
                        if seen.insert(seg) {
                            out.push(seg);
                        }
                    }
                }
            }
        }
        out
    }
}

#[derive(Debug, Clone, Copy)]
struct RiverSample {
    dist: f32,
    half_width: f32,
    water_level: f32,
}

/// Computes carved bed height and blend weight (`1.0` in water, `0.0` at outer bank edge).
pub(crate) fn carve_point(
    rivers: &[River],
    grid: &SegmentGrid,
    data: &RiversData,
    pos: Vec2,
) -> Option<(f32, f32)> {
    let sample = closest_river_sample(rivers, grid, pos)?;
    let outer_radius = sample.half_width + data.bank_m;
    if sample.dist >= outer_radius {
        return None;
    }

    if sample.dist <= sample.half_width {
        let bed_min = sample.water_level - data.depth_m * data.bed_depth_factor;
        let t = smooth_step(0.0, sample.half_width, sample.dist);
        let bed = lerp(bed_min, sample.water_level, t);
        Some((bed, 1.0))
    } else {
        let t = smooth_step(sample.half_width, outer_radius, sample.dist);
        let blend = 1.0 - t;
        Some((sample.water_level, blend))
    }
}

/// Returns the water surface level when `pos` lies inside a river channel (`dist <= width/2`).
pub(crate) fn water_level_at(rivers: &[River], grid: &SegmentGrid, pos: Vec2) -> Option<f32> {
    let sample = closest_river_sample(rivers, grid, pos)?;
    if sample.dist <= sample.half_width {
        Some(sample.water_level)
    } else {
        None
    }
}

/// Finds deduplicated road-river intersections as `(position, road_heading_yaw_rad, bridge_span_m)`.
pub(crate) fn find_crossings(
    rivers: &[River],
    grid: &SegmentGrid,
    data: &RiversData,
    polylines: &[&[Vec2]],
) -> Vec<(Vec2, f32, f32)> {
    let merge_sq = data.crossing_merge_dist_m * data.crossing_merge_dist_m;
    let mut crossings: Vec<(Vec2, f32, f32)> = Vec::new();

    for polyline in polylines {
        if polyline.len() < 2 {
            continue;
        }
        for road_seg in polyline.windows(2) {
            let a = road_seg[0];
            let b = road_seg[1];
            let road_vec = b - a;
            let Some(road_dir) = road_vec.try_normalize() else {
                continue;
            };
            // Godot yaw convention: forward = (-sin yaw, 0, -cos yaw).
            let road_yaw = (-road_dir.x).atan2(-road_dir.y);

            let min = Vec2::new(a.x.min(b.x), a.y.min(b.y));
            let max = Vec2::new(a.x.max(b.x), a.y.max(b.y));
            let candidates = grid.segments_in_aabb(min, max);

            for seg in candidates {
                let river = &rivers[seg.river_idx];
                let c = river.points[seg.seg_idx];
                let d = river.points[seg.seg_idx + 1];

                if let Some((hit, u)) = segment_intersection(a, b, c, d) {
                    let already_found = crossings
                        .iter()
                        .any(|(existing, _, _)| existing.distance_squared(hit) <= merge_sq);
                    if already_found {
                        continue;
                    }
                    let width = lerp(river.widths[seg.seg_idx], river.widths[seg.seg_idx + 1], u);
                    let raw_span = width + TWO_BANKS * data.bank_m;
                    let span = snap_bridge_span(raw_span, &data.bridge_spans_m);
                    crossings.push((hit, road_yaw, span));
                }
            }
        }
    }

    crossings
}

fn closest_river_sample(rivers: &[River], grid: &SegmentGrid, pos: Vec2) -> Option<RiverSample> {
    let mut best: Option<RiverSample> = None;
    let mut best_edge_dist = f32::INFINITY;

    for &seg in grid.cell_segments(pos) {
        let river = &rivers[seg.river_idx];
        let p0 = river.points[seg.seg_idx];
        let p1 = river.points[seg.seg_idx + 1];
        let ab = p1 - p0;
        let len_sq = ab.length_squared();
        let t = if len_sq > f32::EPSILON {
            ((pos - p0).dot(ab) / len_sq).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let closest = p0 + ab * t;
        let dist = pos.distance(closest);
        let half_width = lerp(river.widths[seg.seg_idx], river.widths[seg.seg_idx + 1], t) * HALF;
        let edge_dist = dist - half_width;

        let is_better = match best {
            None => true,
            Some(prev) => {
                edge_dist < best_edge_dist
                    || ((edge_dist - best_edge_dist).abs() <= f32::EPSILON && dist < prev.dist)
            }
        };
        if is_better {
            let water_level = lerp(
                river.water_levels[seg.seg_idx],
                river.water_levels[seg.seg_idx + 1],
                t,
            );
            best_edge_dist = edge_dist;
            best = Some(RiverSample {
                dist,
                half_width,
                water_level,
            });
        }
    }

    best
}

fn segment_intersection(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> Option<(Vec2, f32)> {
    let r = b - a;
    let s = d - c;
    let denom = cross_2d(r, s);
    if denom.abs() <= PARALLEL_EPS {
        return None;
    }
    let q_minus_p = c - a;
    let t = cross_2d(q_minus_p, s) / denom;
    let u = cross_2d(q_minus_p, r) / denom;
    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        Some((a + r * t, u))
    } else {
        None
    }
}

fn snap_bridge_span(raw_span: f32, spans: &[f32]) -> f32 {
    let mut sorted = spans.to_vec();
    sorted.sort_by(f32::total_cmp);
    for &span in &sorted {
        if raw_span <= span {
            return span;
        }
    }
    sorted.last().copied().unwrap_or(raw_span)
}

#[inline]
fn cross_2d(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

#[inline]
fn cell_coord(coord: f32, cell_size: f32) -> i32 {
    (coord / cell_size).floor() as i32
}
