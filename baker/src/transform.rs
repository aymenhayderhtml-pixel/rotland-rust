use std::collections::HashSet;

use glam::Mat4;
use pack::Marker;

/// Compute model-space transformation matrix for every node in the glTF document.
///
/// glTF nodes are organized in a parent-child scene hierarchy. By traversing
/// recursively from scene roots, we accumulate child transforms so that all
/// vertices and markers can be placed into model space.
pub fn compute_world_transforms(doc: &gltf::Document) -> Vec<Mat4> {
    let mut transforms = vec![Mat4::IDENTITY; doc.nodes().len()];
    let scene = doc.default_scene().or_else(|| doc.scenes().next());

    if let Some(scene) = scene {
        for root in scene.nodes() {
            compute_recursive(&root, Mat4::IDENTITY, &mut transforms);
        }
    } else {
        for node in doc.nodes() {
            transforms[node.index()] = Mat4::from_cols_array_2d(&node.transform().matrix());
        }
    }

    transforms
}

fn compute_recursive(node: &gltf::Node, parent_mat: Mat4, out: &mut [Mat4]) {
    let local = Mat4::from_cols_array_2d(&node.transform().matrix());
    let world = parent_mat * local;
    out[node.index()] = world;
    for child in node.children() {
        compute_recursive(&child, world, out);
    }
}

/// Collect node indices that belong to a skeleton or armature.
///
/// These include the skin's joint nodes and any armature container nodes
/// (such as `<model>_armature`), ensuring they are excluded from game marker empties.
pub fn collect_skeleton_nodes(doc: &gltf::Document) -> HashSet<usize> {
    let mut skeleton_nodes = HashSet::new();
    for skin in doc.skins() {
        for joint in skin.joints() {
            skeleton_nodes.insert(joint.index());
        }
    }

    // Armature container nodes in Blender exports (e.g. `zombie_01_armature`)
    // have no mesh and act as parents/containers for skeleton joints.
    for node in doc.nodes() {
        let name = node.name().unwrap_or("");
        if name.ends_with("_armature")
            || node
                .children()
                .any(|child| skeleton_nodes.contains(&child.index()))
        {
            skeleton_nodes.insert(node.index());
        }
    }

    skeleton_nodes
}

/// Extract model-space markers from empty nodes (nodes without meshes).
///
/// Blender empties (e.g. `loot_1`, `spawn_1`, `door_1`, `grip_L`) are preserved
/// with their model-space translation, rotation quaternion, and scale.
pub fn extract_markers(
    doc: &gltf::Document,
    world_transforms: &[Mat4],
    skeleton_nodes: &HashSet<usize>,
) -> Vec<Marker> {
    let mut markers = Vec::new();

    for node in doc.nodes() {
        // Skip nodes with meshes or nodes that are part of a skeleton/armature
        if node.mesh().is_some() || skeleton_nodes.contains(&node.index()) {
            continue;
        }

        let world_mat = world_transforms[node.index()];
        let (scale, rot, trans) = world_mat.to_scale_rotation_translation();

        markers.push(Marker {
            name: node.name().unwrap_or("").to_string(),
            translation: trans.to_array(),
            rotation: [rot.x, rot.y, rot.z, rot.w],
            scale: scale.to_array(),
        });
    }

    // Sort markers by name for deterministic pack output
    markers.sort_by(|a, b| a.name.cmp(&b.name));
    markers
}
