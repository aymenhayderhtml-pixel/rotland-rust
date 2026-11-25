use glam::{Vec2, Vec3};

use crate::rivers::River;
use crate::rivers::path::vertex_tangent;

// new: DECISIONS 2026-11-25
const HALF: f32 = 0.5;
// new: DECISIONS 2026-11-25
const VERTS_PER_TRIANGLE: usize = 3;

/// An indexed river water surface triangle mesh matching `LakeWaterMesh` format.
#[derive(Debug, Clone, PartialEq)]
pub struct RiverWaterMesh {
    pub vertices: Vec<Vec3>,
    pub flow_dirs: Vec<Vec2>,
    pub indices: Vec<[u32; 3]>,
}

impl RiverWaterMesh {
    pub fn triangle_count(&self) -> usize {
        self.indices.len()
    }

    /// Converts the indexed mesh into a non-indexed triangle list of 3D vertices.
    pub fn to_triangle_list(&self) -> Vec<Vec3> {
        let mut list = Vec::with_capacity(self.indices.len() * VERTS_PER_TRIANGLE);
        for tri in &self.indices {
            list.push(self.vertices[tri[0] as usize]);
            list.push(self.vertices[tri[1] as usize]);
            list.push(self.vertices[tri[2] as usize]);
        }
        list
    }

    /// Converts the indexed mesh into a non-indexed triangle list with per-vertex flow direction.
    pub fn to_triangle_list_with_flow(&self) -> Vec<(Vec3, Vec2)> {
        let mut list = Vec::with_capacity(self.indices.len() * VERTS_PER_TRIANGLE);
        for tri in &self.indices {
            for &idx in tri {
                let i = idx as usize;
                list.push((self.vertices[i], self.flow_dirs[i]));
            }
        }
        list
    }
}

/// Builds a quad-strip water mesh for all river segments within `radius` of `center`.
pub fn build_river_water_mesh_near(
    rivers: &[River],
    bank_overlap_m: f32,
    center: Vec2,
    radius: f32,
) -> RiverWaterMesh {
    let mut vertices = Vec::new();
    let mut flow_dirs = Vec::new();
    let mut indices = Vec::new();

    if radius <= 0.0 {
        return RiverWaterMesh { vertices, flow_dirs, indices };
    }

    for river in rivers {
        if river.points.len() < 2 {
            continue;
        }
        let mut prev_pair: Option<(u32, u32)> = None;

        for seg_idx in 0..(river.points.len() - 1) {
            let p0 = river.points[seg_idx];
            let p1 = river.points[seg_idx + 1];
            let max_half = river.widths[seg_idx].max(river.widths[seg_idx + 1]) * HALF
                + bank_overlap_m;

            if !segment_within_radius(p0, p1, center, radius + max_half) {
                prev_pair = None;
                continue;
            }

            let (l0, r0) = match prev_pair {
                Some(pair) => pair,
                None => push_cross_section(river, seg_idx, bank_overlap_m, &mut vertices, &mut flow_dirs),
            };
            let (l1, r1) = push_cross_section(river, seg_idx + 1, bank_overlap_m, &mut vertices, &mut flow_dirs);

            // Winding produces +Y upward normals in right-handed Y-up space, matching lake/mesh.rs.
            indices.push([l0, r0, r1]);
            indices.push([l0, r1, l1]);
            prev_pair = Some((l1, r1));
        }
    }

    RiverWaterMesh { vertices, flow_dirs, indices }
}

fn push_cross_section(
    river: &River,
    idx: usize,
    bank_overlap_m: f32,
    vertices: &mut Vec<Vec3>,
    flow_dirs: &mut Vec<Vec2>,
) -> (u32, u32) {
    let p = river.points[idx];
    let y = river.water_levels[idx];
    let half_extent = river.widths[idx] * HALF + bank_overlap_m;
    let fwd = vertex_tangent(&river.points, idx);
    let right = Vec2::new(-fwd.y, fwd.x);
    let left_pos = p - right * half_extent;
    let right_pos = p + right * half_extent;

    let left_idx = vertices.len() as u32;
    vertices.push(Vec3::new(left_pos.x, y, left_pos.y));
    flow_dirs.push(fwd);
    let right_idx = vertices.len() as u32;
    vertices.push(Vec3::new(right_pos.x, y, right_pos.y));
    flow_dirs.push(fwd);
    (left_idx, right_idx)
}

fn segment_within_radius(p0: Vec2, p1: Vec2, center: Vec2, max_dist: f32) -> bool {
    let ab = p1 - p0;
    let len_sq = ab.length_squared();
    let t = if len_sq > f32::EPSILON {
        ((center - p0).dot(ab) / len_sq).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let closest = p0 + ab * t;
    center.distance_squared(closest) <= max_dist * max_dist
}
