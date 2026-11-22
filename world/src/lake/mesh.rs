use std::collections::HashMap;

use glam::{Vec2, Vec3};

use crate::lake::layout::LakeLayout;

/// A watertight water surface triangle mesh.
#[derive(Debug, Clone, PartialEq)]
pub struct LakeWaterMesh {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<[u32; 3]>,
}

impl LakeWaterMesh {
    pub fn triangle_count(&self) -> usize {
        self.indices.len()
    }

    /// Converts the indexed mesh into a non-indexed triangle list of 3D vertices.
    pub fn to_triangle_list(&self) -> Vec<Vec3> {
        let mut list = Vec::with_capacity(self.indices.len() * 3);
        for tri in &self.indices {
            list.push(self.vertices[tri[0] as usize]);
            list.push(self.vertices[tri[1] as usize]);
            list.push(self.vertices[tri[2] as usize]);
        }
        list
    }
}

/// Finds the point along the line segment between `p0` and `p1` where shore_distance is 0.
fn find_shore_crossing(lake: &LakeLayout, p0: Vec2, p1: Vec2) -> Vec2 {
    let d0 = lake.shore_distance(p0);
    let mut t_wet = if d0 < 0.0 { 0.0f32 } else { 1.0f32 };
    let mut t_dry = if d0 < 0.0 { 1.0f32 } else { 0.0f32 };

    // 10 bisection iterations to pin the shoreline crossing within centimeters
    for _ in 0..10 {
        let t_mid = 0.5 * (t_wet + t_dry);
        let p_mid = p0 + t_mid * (p1 - p0);
        if lake.shore_distance(p_mid) < 0.0 {
            t_wet = t_mid;
        } else {
            t_dry = t_mid;
        }
    }
    let t = 0.5 * (t_wet + t_dry);
    p0 + t * (p1 - p0)
}

/// Generates a watertight water mesh clipped smoothly to the lake shoreline.
pub fn build_lake_water_mesh(lake: &LakeLayout) -> LakeWaterMesh {
    let (min, max) = lake.water_bounds();
    let cell_size = lake.config.water_grid_cell;
    let cols = ((max.x - min.x) / cell_size).ceil() as i32;
    let rows = ((max.y - min.y) / cell_size).ceil() as i32;

    let node_pos = |i: i32, j: i32| -> Vec2 {
        Vec2::new(min.x + i as f32 * cell_size, min.y + j as f32 * cell_size)
    };

    let total_nodes = ((cols + 1) * (rows + 1)) as usize;
    let mut dists = Vec::with_capacity(total_nodes);
    for j in 0..=rows {
        for i in 0..=cols {
            dists.push(lake.shore_distance(node_pos(i, j)));
        }
    }

    let node_dist = |i: i32, j: i32| -> f32 {
        dists[(j * (cols + 1) + i) as usize]
    };

    let mut vertices: Vec<Vec3> = Vec::new();
    let mut indices: Vec<[u32; 3]> = Vec::new();

    let mut node_indices: HashMap<(i32, i32), u32> = HashMap::new();
    let mut h_crossings: HashMap<(i32, i32), u32> = HashMap::new();
    let mut v_crossings: HashMap<(i32, i32), u32> = HashMap::new();

    let mut get_node_vertex = |i: i32, j: i32, verts: &mut Vec<Vec3>| -> u32 {
        *node_indices.entry((i, j)).or_insert_with(|| {
            let p = node_pos(i, j);
            let idx = verts.len() as u32;
            verts.push(Vec3::new(p.x, lake.water_level, p.y));
            idx
        })
    };

    let mut get_h_crossing = |i: i32, j: i32, verts: &mut Vec<Vec3>| -> u32 {
        *h_crossings.entry((i, j)).or_insert_with(|| {
            let p0 = node_pos(i, j);
            let p1 = node_pos(i + 1, j);
            let pt = find_shore_crossing(lake, p0, p1);
            let idx = verts.len() as u32;
            verts.push(Vec3::new(pt.x, lake.water_level, pt.y));
            idx
        })
    };

    let mut get_v_crossing = |i: i32, j: i32, verts: &mut Vec<Vec3>| -> u32 {
        *v_crossings.entry((i, j)).or_insert_with(|| {
            let p0 = node_pos(i, j);
            let p1 = node_pos(i, j + 1);
            let pt = find_shore_crossing(lake, p0, p1);
            let idx = verts.len() as u32;
            verts.push(Vec3::new(pt.x, lake.water_level, pt.y));
            idx
        })
    };

    for j in 0..rows {
        for i in 0..cols {
            let d0 = node_dist(i, j);
            let d1 = node_dist(i, j + 1);
            let d2 = node_dist(i + 1, j + 1);
            let d3 = node_dist(i + 1, j);

            let w0 = d0 < 0.0;
            let w1 = d1 < 0.0;
            let w2 = d2 < 0.0;
            let w3 = d3 < 0.0;

            if !w0 && !w1 && !w2 && !w3 {
                continue;
            }

            if w0 && w1 && w2 && w3 {
                let v0 = get_node_vertex(i, j, &mut vertices);
                let v1 = get_node_vertex(i, j + 1, &mut vertices);
                let v2 = get_node_vertex(i + 1, j + 1, &mut vertices);
                let v3 = get_node_vertex(i + 1, j, &mut vertices);
                indices.push([v0, v1, v2]);
                indices.push([v0, v2, v3]);
                continue;
            }

            // Diagonal saddle cases
            if w0 && w2 && !w1 && !w3 {
                let v0 = get_node_vertex(i, j, &mut vertices);
                let e0 = get_v_crossing(i, j, &mut vertices);
                let e3 = get_h_crossing(i, j, &mut vertices);
                indices.push([v0, e0, e3]);

                let v2 = get_node_vertex(i + 1, j + 1, &mut vertices);
                let e2 = get_v_crossing(i + 1, j, &mut vertices);
                let e1 = get_h_crossing(i, j + 1, &mut vertices);
                indices.push([v2, e2, e1]);
                continue;
            }

            if w1 && w3 && !w0 && !w2 {
                let v1 = get_node_vertex(i, j + 1, &mut vertices);
                let e1 = get_h_crossing(i, j + 1, &mut vertices);
                let e0 = get_v_crossing(i, j, &mut vertices);
                indices.push([v1, e1, e0]);

                let v3 = get_node_vertex(i + 1, j, &mut vertices);
                let e3 = get_h_crossing(i, j, &mut vertices);
                let e2 = get_v_crossing(i + 1, j, &mut vertices);
                indices.push([v3, e3, e2]);
                continue;
            }

            // Marching perimeter walk in counter-clockwise order
            let mut poly = Vec::new();

            if w0 {
                poly.push(get_node_vertex(i, j, &mut vertices));
            }
            if w0 != w1 {
                poly.push(get_v_crossing(i, j, &mut vertices));
            }

            if w1 {
                poly.push(get_node_vertex(i, j + 1, &mut vertices));
            }
            if w1 != w2 {
                poly.push(get_h_crossing(i, j + 1, &mut vertices));
            }

            if w2 {
                poly.push(get_node_vertex(i + 1, j + 1, &mut vertices));
            }
            if w2 != w3 {
                poly.push(get_v_crossing(i + 1, j, &mut vertices));
            }

            if w3 {
                poly.push(get_node_vertex(i + 1, j, &mut vertices));
            }
            if w3 != w0 {
                poly.push(get_h_crossing(i, j, &mut vertices));
            }

            poly.dedup();
            if poly.len() > 1 && poly.first() == poly.last() {
                poly.pop();
            }

            if poly.len() >= 3 {
                let root = poly[0];
                for k in 1..(poly.len() - 1) {
                    indices.push([root, poly[k], poly[k + 1]]);
                }
            }
        }
    }

    LakeWaterMesh { vertices, indices }
}
