use glam::Mat4;
use pack::{Channel, ChannelKind, Clip, Interpolation, Joint, Skeleton};

/// Extract skeleton hierarchy and animation clips from glTF document.
pub fn extract_skeleton_and_clips(
    doc: &gltf::Document,
    buffers: &[gltf::buffer::Data],
) -> (Option<Skeleton>, Vec<Clip>) {
    let Some(skin) = doc.skins().next() else {
        return (None, Vec::new());
    };

    let skin_joints: Vec<gltf::Node> = skin.joints().collect();
    if skin_joints.is_empty() {
        return (None, Vec::new());
    }

    // Read inverse bind matrices (IBM)
    let ibm_matrices: Vec<[f32; 16]> = {
        let reader = skin.reader(|b| Some(&buffers[b.index()]));
        if let Some(iter) = reader.read_inverse_bind_matrices() {
            iter.map(|mat| {
                let mut flat = [0.0f32; 16];
                for col in 0..4 {
                    for row in 0..4 {
                        flat[col * 4 + row] = mat[col][row];
                    }
                }
                flat
            })
            .collect()
        } else {
            vec![Mat4::IDENTITY.to_cols_array(); skin_joints.len()]
        }
    };

    // Build joint hierarchy
    let mut joints = Vec::with_capacity(skin_joints.len());
    for (j_idx, joint_node) in skin_joints.iter().enumerate() {
        let parent = find_parent_joint_index(doc, joint_node.index(), &skin_joints);
        let (rest_trans, rest_rot, rest_scale) = joint_node.transform().decomposed();
        let inverse_bind = ibm_matrices
            .get(j_idx)
            .copied()
            .unwrap_or_else(|| Mat4::IDENTITY.to_cols_array());

        joints.push(Joint {
            name: joint_node.name().unwrap_or("").to_string(),
            parent,
            inverse_bind,
            rest_translation: rest_trans,
            rest_rotation: rest_rot,
            rest_scale,
        });
    }

    // Extract animation clips
    let mut clips = Vec::new();
    for anim in doc.animations() {
        let name = anim.name().unwrap_or("default").to_string();
        let mut channels = Vec::new();

        for ch in anim.channels() {
            let target_node = ch.target().node();
            let Some(joint_idx) = skin_joints
                .iter()
                .position(|j| j.index() == target_node.index())
            else {
                continue;
            };

            let kind = match ch.target().property() {
                gltf::animation::Property::Translation => ChannelKind::Translation,
                gltf::animation::Property::Rotation => ChannelKind::Rotation,
                gltf::animation::Property::Scale => ChannelKind::Scale,
                _ => continue,
            };

            let sampler = ch.sampler();
            let interpolation = match sampler.interpolation() {
                gltf::animation::Interpolation::Step => Interpolation::Step,
                _ => Interpolation::Linear,
            };

            let reader = ch.reader(|b| Some(&buffers[b.index()]));
            let Some(times_iter) = reader.read_inputs() else {
                continue;
            };
            let times: Vec<f32> = times_iter.collect();

            let Some(outputs) = reader.read_outputs() else {
                continue;
            };
            let values: Vec<[f32; 4]> = match outputs {
                gltf::animation::util::ReadOutputs::Translations(iter) => {
                    iter.map(|[x, y, z]| [x, y, z, 0.0]).collect()
                }
                gltf::animation::util::ReadOutputs::Scales(iter) => {
                    iter.map(|[x, y, z]| [x, y, z, 0.0]).collect()
                }
                gltf::animation::util::ReadOutputs::Rotations(rotations) => {
                    rotations.into_f32().collect()
                }
                _ => continue,
            };

            channels.push(Channel {
                joint: joint_idx as u16,
                kind,
                interpolation,
                times,
                values,
            });
        }

        let duration = channels
            .iter()
            .flat_map(|c| c.times.iter().copied())
            .fold(0.0f32, f32::max);

        clips.push(Clip {
            name,
            duration,
            channels,
        });
    }

    (Some(Skeleton { joints }), clips)
}

/// Find the parent joint index in `skin_joints` for a given joint node index.
///
/// Returns `None` if the node has no parent, or if its parent is not part
/// of the skin (e.g. an armature container or scene root).
fn find_parent_joint_index(
    doc: &gltf::Document,
    joint_node_index: usize,
    skin_joints: &[gltf::Node],
) -> Option<u16> {
    for potential_parent in doc.nodes() {
        for child in potential_parent.children() {
            if child.index() == joint_node_index {
                return skin_joints
                    .iter()
                    .position(|j| j.index() == potential_parent.index())
                    .map(|pos| pos as u16);
            }
        }
    }
    None
}
