//! Terrain height under the highway: the road surface (base height smoothed along the road, the
//! last stretch blended into the flat city ground) and the flatten weight that eases the land
//! around it into that surface.

use std::collections::HashMap;

use glam::Vec2;

use crate::math::{lerp, smooth_step};
use crate::metro::config::MetroConfig;
use crate::metro::highway::Highway;

/// Lookup grid cell size in metres (same 64 m grid the roads use).
const GRID_CELL_M: f32 = 64.0; // new: DECISIONS 2026-11-26 (lookup grid, matches roads)

/// Road surface height along a planned highway, with a grid for fast lookups.
#[derive(Debug, Clone)]
pub struct HighwayHeightField {
    pub points: Vec<Vec2>,
    pub heights: Vec<f32>,
    half_width: f32,
    shoulder_m: f32,
    flatten_edge_m: f32,
    /// Outermost distance any value depends on: half width plus the wider of shoulder and edge.
    reach: f32,
    cells: HashMap<(i32, i32), Vec<usize>>,
}

impl HighwayHeightField {
    /// Builds the surface. `base_height` is the raw terrain height (town interiors at their
    /// average, as for the roads). Heights are smoothed over `heightSmoothM` of arc length, then
    /// the last `cityBlendM` blend into `site_ground_y` so the highway meets the flat city.
    pub fn build(
        highway: &Highway,
        site_ground_y: f32,
        base_height: &dyn Fn(f32, f32) -> f32,
        cfg: &MetroConfig,
    ) -> Self {
        let pts = &highway.points;
        let arc = cumulative_arc(pts);
        let raw: Vec<f32> = pts.iter().map(|p| base_height(p.x, p.y)).collect();
        let window = cfg.highway.height_smooth_m; // data: metro.json highway.heightSmoothM
        let blend = cfg.highway.city_blend_m; // data: metro.json highway.cityBlendM
        let total = arc.last().copied().unwrap_or(0.0);

        let heights: Vec<f32> = (0..pts.len())
            .map(|i| {
                let (mut sum, mut count) = (0.0_f32, 0usize);
                for j in 0..pts.len() {
                    if (arc[j] - arc[i]).abs() <= window {
                        sum += raw[j];
                        count += 1;
                    }
                }
                let smoothed = sum / (count.max(1) as f32);
                let remaining = total - arc[i];
                if blend > 0.0 && remaining < blend {
                    // new: DECISIONS 2026-11-26 (blend factor 0 at the blend start, 1 at the city edge)
                    let u = (1.0 - remaining / blend).clamp(0.0, 1.0);
                    lerp(smoothed, site_ground_y, smooth_step(0.0, 1.0, u))
                } else {
                    smoothed
                }
            })
            .collect();

        let half_width = highway.width * 0.5; // new: DECISIONS 2026-11-26 (half of the road width)
        let shoulder_m = cfg.highway.shoulder_width; // data: metro.json highway.shoulderWidth
        let flatten_edge_m = cfg.highway.flatten_edge_m; // data: metro.json highway.flattenEdgeM
        let reach = half_width + shoulder_m.max(flatten_edge_m);
        let cells = build_cells(pts, reach);
        Self {
            points: pts.clone(),
            heights,
            half_width,
            shoulder_m,
            flatten_edge_m,
            reach,
            cells,
        }
    }

    /// Road surface height at `p` when `p` is on the highway or its shoulder, else `None`.
    pub fn height_at(&self, p: Vec2) -> Option<f32> {
        let (h, d) = self.sample(p)?;
        (d <= self.half_width + self.shoulder_m).then_some(h)
    }

    /// Terrain height at `p` after the highway's flattening: the road surface on the road, easing
    /// back to `raw` over `flattenEdgeM`. Returns `raw` away from the highway.
    pub fn flatten(&self, raw: f32, p: Vec2) -> f32 {
        let Some((h, d)) = self.sample(p) else {
            return raw;
        };
        let weight = if d <= self.half_width {
            1.0
        } else {
            1.0 - smooth_step(self.half_width, self.half_width + self.flatten_edge_m, d)
        };
        lerp(raw, h, weight)
    }

    /// 1 on the highway surface, easing to 0 over `flattenEdgeM` (0 away from the highway).
    /// Used to paint the asphalt.
    pub fn surface_weight(&self, p: Vec2) -> f32 {
        let Some((_, d)) = self.sample(p) else {
            return 0.0;
        };
        if d <= self.half_width {
            1.0
        } else {
            1.0 - smooth_step(self.half_width, self.half_width + self.flatten_edge_m, d)
        }
    }

    /// Height and distance to the nearest highway segment, within `reach`.
    fn sample(&self, p: Vec2) -> Option<(f32, f32)> {
        let key = cell_key(p);
        let candidates = self.cells.get(&key)?;
        let mut best: Option<(f32, f32)> = None;
        for &i in candidates {
            let (a, b) = (self.points[i], self.points[i + 1]);
            let ab = b - a;
            let t = ((p - a).dot(ab) / ab.length_squared().max(f32::EPSILON)).clamp(0.0, 1.0);
            let d = p.distance(a + ab * t);
            if d <= self.reach && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((lerp(self.heights[i], self.heights[i + 1], t), d));
            }
        }
        best
    }
}

/// Free wrapper: the road surface height at `p`, or `None` off the highway.
pub fn highway_height_at(field: &HighwayHeightField, p: Vec2) -> Option<f32> {
    field.height_at(p)
}

fn cumulative_arc(pts: &[Vec2]) -> Vec<f32> {
    let mut arc = Vec::with_capacity(pts.len());
    let mut total = 0.0_f32;
    arc.push(0.0);
    for pair in pts.windows(2) {
        total += pair[0].distance(pair[1]);
        arc.push(total);
    }
    arc
}

fn cell_key(p: Vec2) -> (i32, i32) {
    ((p.x / GRID_CELL_M).floor() as i32, (p.y / GRID_CELL_M).floor() as i32)
}

/// Registers each segment in every grid cell its reach-padded box touches.
fn build_cells(pts: &[Vec2], reach: f32) -> HashMap<(i32, i32), Vec<usize>> {
    let mut cells: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    for i in 0..pts.len().saturating_sub(1) {
        let (a, b) = (pts[i], pts[i + 1]);
        let lo = cell_key(a.min(b) - Vec2::splat(reach));
        let hi = cell_key(a.max(b) + Vec2::splat(reach));
        for cz in lo.1..=hi.1 {
            for cx in lo.0..=hi.0 {
                cells.entry((cx, cz)).or_default().push(i);
            }
        }
    }
    cells
}
