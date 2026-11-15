use glam::{Mat3, Mat4, Vec3};
use pack::{ConvexCollider, PackVertex};

use crate::color::{factor_to_srgb, sample_palette_nearest};

/// Result of extracting all drawable geometry and collision shapes from a glTF file.
pub struct ExtractedGeometry {
    pub vertices: Vec<PackVertex>,
    pub colliders: Vec<ConvexCollider>,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
}

/// Extract drawable vertices and convex colliders from document meshes.
pub fn extract_geometry(
    doc: &gltf::Document,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
    world_transforms: &[Mat4],
) -> ExtractedGeometry {
    let mut vertices = Vec::new();
    let mut colliders = Vec::new();

    // Traverse scene nodes in hierarchy order to preserve GLB triangle order
    let scene = doc.default_scene().or_else(|| doc.scenes().next());
    let mut node_indices = Vec::new();
    if let Some(scene) = scene {
        for root in scene.nodes() {
            collect_node_indices(&root, &mut node_indices);
        }
    } else {
        node_indices.extend(0..doc.nodes().len());
    }

    let all_nodes: Vec<gltf::Node> = doc.nodes().collect();

    for idx in node_indices {
        let node = &all_nodes[idx];
        let Some(mesh) = node.mesh() else { continue };
        let node_name = node.name().unwrap_or("");
        let world_mat = world_transforms[node.index()];

        // Collision meshes: name ends with "-convcolonly" or contains "-col"
        if node_name.ends_with("-convcolonly") || node_name.contains("-col") {
            let collider = extract_collider(&mesh, buffers, world_mat, node_name);
            colliders.push(collider);
            continue;
        }

        // Normal transformation matrix: inverse-transpose of 3x3 rotation/scale
        let normal_mat = Mat3::from_mat4(world_mat).inverse().transpose();

        for primitive in mesh.primitives() {
            let reader = primitive.reader(|b| Some(&buffers[b.index()]));

            let positions: Vec<[f32; 3]> = match reader.read_positions() {
                Some(iter) => iter.collect(),
                None => continue,
            };

            let normals: Vec<[f32; 3]> = match reader.read_normals() {
                Some(iter) => iter.collect(),
                None => vec![[0.0, 1.0, 0.0]; positions.len()],
            };

            let uvs: Vec<[f32; 2]> = match reader.read_tex_coords(0) {
                Some(iter) => iter.into_f32().collect(),
                None => vec![[0.0, 0.0]; positions.len()],
            };

            let joints: Vec<[u16; 4]> = match reader.read_joints(0) {
                Some(iter) => iter.into_u16().collect(),
                None => Vec::new(),
            };

            let weights: Vec<[f32; 4]> = match reader.read_weights(0) {
                Some(iter) => iter.into_f32().collect(),
                None => Vec::new(),
            };

            let indices: Vec<u32> = match reader.read_indices() {
                Some(iter) => iter.into_u32().collect(),
                None => (0..positions.len() as u32).collect(),
            };

            let pbr = primitive.material().pbr_metallic_roughness();
            let base_color_factor = pbr.base_color_factor();
            let texture_info = pbr.base_color_texture();

            for chunk in indices.as_chunks::<3>().0 {
                let (i0, i1, i2) = (chunk[0] as usize, chunk[1] as usize, chunk[2] as usize);

                let uv0 = uvs.get(i0).copied().unwrap_or([0.0, 0.0]);
                let uv1 = uvs.get(i1).copied().unwrap_or([0.0, 0.0]);
                let uv2 = uvs.get(i2).copied().unwrap_or([0.0, 0.0]);
                let u_centroid = (uv0[0] + uv1[0] + uv2[0]) / 3.0;
                let v_centroid = (uv0[1] + uv1[1] + uv2[1]) / 3.0;

                let triangle_color = if let Some(ref tex) = texture_info {
                    let img_idx = tex.texture().source().index();
                    if let Some(img) = images.get(img_idx) {
                        sample_palette_nearest(img, u_centroid, v_centroid, base_color_factor)
                    } else {
                        factor_to_srgb(base_color_factor)
                    }
                } else {
                    factor_to_srgb(base_color_factor)
                };

                for &idx in &[i0, i1, i2] {
                    let p_local = positions.get(idx).copied().unwrap_or([0.0; 3]);
                    let p_world = world_mat.transform_point3(Vec3::from(p_local)).to_array();

                    let n_local = normals.get(idx).copied().unwrap_or([0.0, 1.0, 0.0]);
                    let n_trans = normal_mat * Vec3::from(n_local);
                    let n_norm = n_trans.normalize_or_zero();
                    let normal = if n_norm == Vec3::ZERO {
                        [0.0, 1.0, 0.0]
                    } else {
                        n_norm.to_array()
                    };

                    let bone = if !joints.is_empty() && !weights.is_empty() {
                        let w = weights.get(idx).copied().unwrap_or([1.0, 0.0, 0.0, 0.0]);
                        let j = joints.get(idx).copied().unwrap_or([0, 0, 0, 0]);
                        let mut best_k = 0;
                        let mut max_w = -1.0f32;
                        for (k, &weight) in w.iter().enumerate() {
                            if weight > max_w {
                                max_w = weight;
                                best_k = k;
                            }
                        }
                        j[best_k] as u8
                    } else {
                        0
                    };

                    vertices.push(PackVertex {
                        position: p_world,
                        normal,
                        color: triangle_color,
                        bone,
                        _pad: [0; 3],
                    });
                }
            }
        }
    }

    let (bounds_min, bounds_max) = compute_bounds(&vertices);

    ExtractedGeometry {
        vertices,
        colliders,
        bounds_min,
        bounds_max,
    }
}

fn collect_node_indices(node: &gltf::Node, out: &mut Vec<usize>) {
    out.push(node.index());
    for child in node.children() {
        collect_node_indices(&child, out);
    }
}

fn extract_collider(
    mesh: &gltf::Mesh,
    buffers: &[gltf::buffer::Data],
    world_mat: Mat4,
    node_name: &str,
) -> ConvexCollider {
    let mut raw_points = Vec::new();
    for primitive in mesh.primitives() {
        let reader = primitive.reader(|b| Some(&buffers[b.index()]));
        if let Some(pos_iter) = reader.read_positions() {
            for p in pos_iter {
                raw_points.push(world_mat.transform_point3(Vec3::from(p)).to_array());
            }
        }
    }

    // Keep unique points for efficient convex hull computation
    let mut unique_points = Vec::new();
    for p in raw_points {
        if !unique_points.iter().any(|existing: &[f32; 3]| {
            (existing[0] - p[0]).abs() < 1e-4
                && (existing[1] - p[1]).abs() < 1e-4
                && (existing[2] - p[2]).abs() < 1e-4
        }) {
            unique_points.push(p);
        }
    }

    ConvexCollider {
        name: node_name.to_string(),
        points: unique_points,
    }
}

fn compute_bounds(vertices: &[PackVertex]) -> ([f32; 3], [f32; 3]) {
    if vertices.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }

    let mut min_p = Vec3::splat(f32::MAX);
    let mut max_p = Vec3::splat(f32::MIN);
    for v in vertices {
        let p = Vec3::from(v.position);
        min_p = min_p.min(p);
        max_p = max_p.max(p);
    }
    (min_p.to_array(), max_p.to_array())
}
