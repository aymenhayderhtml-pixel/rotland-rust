//! Street grid for herd routes: walkable street edges, districts and grid/world conversion.

use std::collections::HashMap;

use glam::Vec2;

use crate::metro::herds::LEN_EPS;
use crate::metro::layout::{MetroDistrict, MetroLayout, MetroStreet};

pub(super) type GridPt = (i32, i32);

#[derive(Clone, Copy)]
pub(super) struct GridEdge {
    pub(super) to: GridPt,
    pub(super) avenue: bool,
    pub(super) zone: MetroDistrict,
}

pub(super) struct StreetGrid<'a> {
    layout: &'a MetroLayout,
    half: f32,
    bs: f32,
    pub(super) adj: HashMap<GridPt, Vec<GridEdge>>,
}

impl<'a> StreetGrid<'a> {
    pub(super) fn from_layout(layout: &'a MetroLayout, blocked_world: &[Vec2]) -> Option<Self> {
        let first_block = layout.blocks.first()?;
        let bs = (first_block.rect.1.x - first_block.rect.0.x).abs();
        if bs <= LEN_EPS {
            return None;
        }
        let half = layout.site.half_size;
        let n = ((half * 2.0) / bs).round() as i32;
        let blocked_local: Vec<Vec2> = blocked_world.iter().map(|&p| layout.site.to_local(p)).collect();
        let mut adj: HashMap<GridPt, Vec<GridEdge>> = HashMap::new();

        for s in &layout.streets {
            if is_blocked_street(s, &blocked_local) {
                continue;
            }
            let p0 = to_grid(s.a, half, bs);
            let p1 = to_grid(s.b, half, bs);
            // Keep routes inside the city grid so they never walk along outer perimeter dead-ends.
            if !is_interior(p0, n) || !is_interior(p1, n) {
                continue;
            }
            let Some(zone) = segment_district(layout, p0, p1) else {
                continue;
            };
            adj.entry(p0).or_default().push(GridEdge { to: p1, avenue: s.avenue, zone });
            adj.entry(p1).or_default().push(GridEdge { to: p0, avenue: s.avenue, zone });
        }
        Some(Self { layout, half, bs, adj })
    }

    pub(super) fn edges_for_district(&self, zone: MetroDistrict) -> Vec<(GridPt, GridEdge)> {
        let mut out = Vec::new();
        let mut keys: Vec<GridPt> = self.adj.keys().copied().collect();
        keys.sort();
        for u in keys {
            for &e in &self.adj[&u] {
                if e.zone == zone && u < e.to {
                    out.push((u, e));
                }
            }
        }
        out
    }

    pub(super) fn to_world(&self, p: GridPt) -> Vec2 {
        let local = Vec2::new(-self.half + p.0 as f32 * self.bs, -self.half + p.1 as f32 * self.bs);
        self.layout.site.to_world(local)
    }

    pub(super) fn has_edge(&self, a: GridPt, b: GridPt, zone: MetroDistrict) -> bool {
        self.adj.get(&a).is_some_and(|list| list.iter().any(|e| e.to == b && e.zone == zone))
    }
}

fn to_grid(local: Vec2, half: f32, bs: f32) -> GridPt {
    (((local.x + half) / bs).round() as i32, ((local.y + half) / bs).round() as i32)
}

fn is_interior(p: GridPt, n: i32) -> bool {
    p.0 > 0 && p.0 < n && p.1 > 0 && p.1 < n
}

fn is_blocked_street(s: &MetroStreet, blocked_local: &[Vec2]) -> bool {
    let half_w = s.width * 0.5 + 1.0; // new: DECISIONS 2026-11-27 (street half-width + margin)
    blocked_local.iter().any(|&p| s.contains_point(p) || s.closest_point(p).distance(p) <= half_w)
}

/// Returns the district if both adjacent blocks exist, neither is reserved, and both share the district.
fn segment_district(layout: &MetroLayout, a: GridPt, b: GridPt) -> Option<MetroDistrict> {
    let (b0, b1) = if a.1 == b.1 {
        let ix = a.0.min(b.0);
        (layout.block_at(ix, a.1 - 1)?, layout.block_at(ix, a.1)?)
    } else {
        let iz = a.1.min(b.1);
        (layout.block_at(a.0 - 1, iz)?, layout.block_at(a.0, iz)?)
    };
    if b0.reserved.is_some() || b1.reserved.is_some() || b0.district != b1.district {
        return None;
    }
    Some(b0.district)
}
