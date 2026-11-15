use std::path::Path;

use pack::Model;

use crate::mesh::extract_geometry;
use crate::skeleton::extract_skeleton_and_clips;
use crate::transform::{collect_skeleton_nodes, compute_world_transforms, extract_markers};

/// Bake a single Blender `.glb` file into a compact GPU-ready `Model`.
pub fn bake_glb(path: &Path, name: String, folder: String) -> Result<Model, String> {
    let (doc, buffers, images) = gltf::import(path)
        .map_err(|e| format!("failed to import glb at {}: {e}", path.display()))?;

    let world_transforms = compute_world_transforms(&doc);
    let skeleton_nodes = collect_skeleton_nodes(&doc);
    let markers = extract_markers(&doc, &world_transforms, &skeleton_nodes);
    let geometry = extract_geometry(&doc, &buffers, &images, &world_transforms);
    let (skeleton, clips) = extract_skeleton_and_clips(&doc, &buffers);

    Ok(Model {
        name,
        folder,
        vertices: geometry.vertices,
        bounds_min: geometry.bounds_min,
        bounds_max: geometry.bounds_max,
        markers,
        colliders: geometry.colliders,
        skeleton,
        clips,
    })
}
