//! Player visual representation: rigged body mesh, humanoid animation state,
//! and static weapon model attached to the right hand.

use engine::anim::Rig;
use engine::render::{GpuMesh, SkinInstance, SkinnedBatch};
use glam::{Mat4, Vec3};
use humanoid::{weapon_world_matrix_named, HumanoidAnimator, Locomotion};
use pack::{Joint, ModelPack, PackVertex, Skeleton};

/// Pure helper: returns the requested model from `pack`, or falls back to `fallback_name` if missing.
/// Logs an error message if the primary model is missing.
pub fn pick_model_or_fallback<'a>(
    pack: &'a ModelPack,
    name: &str,
    fallback_name: Option<&str>,
) -> Option<&'a pack::Model> {
    if let Some(m) = pack.find(name) {
        return Some(m);
    }
    eprintln!("player_visual: model '{name}' not found in pack");
    if let Some(fb) = fallback_name {
        if let Some(m) = pack.find(fb) {
            eprintln!("player_visual: using fallback model '{fb}' for '{name}'");
            return Some(m);
        }
    }
    None
}

/// Creates a minimal fallback skeleton with a root joint and hand_R attachment joint.
pub fn fallback_skeleton() -> Skeleton {
    let make_joint = |name: &str, parent: Option<u16>| Joint {
        name: name.to_string(),
        parent,
        inverse_bind: Mat4::IDENTITY.to_cols_array(),
        rest_translation: [0.0, 0.0, 0.0],
        rest_rotation: [0.0, 0.0, 0.0, 1.0],
        rest_scale: [1.0, 1.0, 1.0],
    };
    Skeleton {
        joints: vec![
            make_joint("root", None),
            make_joint("hand_R", Some(0)),
        ],
    }
}

/// Generates a simple box mesh (12 triangles / 36 vertices) as a fallback when character geometry is missing.
pub fn fallback_box_vertices() -> Vec<PackVertex> {
    let min = [-0.25f32, 0.0, -0.25];
    let max = [0.25f32, 1.8, 0.25];
    let color = [180, 180, 180, 255];
    let mut v = Vec::with_capacity(36);

    let mut add_quad = |p1: [f32; 3], p2: [f32; 3], p3: [f32; 3], p4: [f32; 3], n: [f32; 3]| {
        v.push(PackVertex { position: p1, normal: n, color, bone: 0, _pad: [0; 3] });
        v.push(PackVertex { position: p2, normal: n, color, bone: 0, _pad: [0; 3] });
        v.push(PackVertex { position: p3, normal: n, color, bone: 0, _pad: [0; 3] });

        v.push(PackVertex { position: p1, normal: n, color, bone: 0, _pad: [0; 3] });
        v.push(PackVertex { position: p3, normal: n, color, bone: 0, _pad: [0; 3] });
        v.push(PackVertex { position: p4, normal: n, color, bone: 0, _pad: [0; 3] });
    };

    // Front (+Z)
    add_quad([min[0], min[1], max[2]], [max[0], min[1], max[2]], [max[0], max[1], max[2]], [min[0], max[1], max[2]], [0.0, 0.0, 1.0]);
    // Back (-Z)
    add_quad([max[0], min[1], min[2]], [min[0], min[1], min[2]], [min[0], max[1], min[2]], [max[0], max[1], min[2]], [0.0, 0.0, -1.0]);
    // Left (-X)
    add_quad([min[0], min[1], min[2]], [min[0], min[1], max[2]], [min[0], max[1], max[2]], [min[0], max[1], min[2]], [-1.0, 0.0, 0.0]);
    // Right (+X)
    add_quad([max[0], min[1], max[2]], [max[0], min[1], min[2]], [max[0], max[1], min[2]], [max[0], max[1], max[2]], [1.0, 0.0, 0.0]);
    // Top (+Y)
    add_quad([min[0], max[1], max[2]], [max[0], max[1], max[2]], [max[0], max[1], min[2]], [min[0], max[1], min[2]], [0.0, 1.0, 0.0]);
    // Bottom (-Y)
    add_quad([min[0], min[1], min[2]], [max[0], min[1], min[2]], [max[0], min[1], max[2]], [min[0], min[1], max[2]], [0.0, -1.0, 0.0]);

    v
}

/// Posed humanoid character mesh with optional weapon attachment.
pub struct PlayerVisual {
    body_mesh: GpuMesh,
    rig: Rig,
    pub animator: HumanoidAnimator,
    weapon_mesh: Option<GpuMesh>,
    weapon_name: Option<String>,
    hand_joint: usize,
}

impl PlayerVisual {
    /// Loads a rigged humanoid character model and optional weapon from `pack`.
    /// Missing models or skeletons log an error and use a fallback (box mesh / skip weapon).
    pub fn load(
        device: &wgpu::Device,
        pack: &ModelPack,
        char_name: &str,
        weapon_model: Option<&str>,
    ) -> Self {
        let (body_mesh, rig, animator) = match pack.find(char_name) {
            Some(char_model) => match &char_model.skeleton {
                Some(skeleton) => {
                    let rig = Rig::new(skeleton);
                    let body_mesh = GpuMesh::new(device, char_name, &char_model.vertices);
                    let animator = HumanoidAnimator::new(skeleton, &char_model.clips);
                    (body_mesh, rig, animator)
                }
                None => {
                    eprintln!("player_visual: character model '{char_name}' has no skeleton; using fallback box mesh");
                    let skel = fallback_skeleton();
                    let rig = Rig::new(&skel);
                    let animator = HumanoidAnimator::new(&skel, &[]);
                    let box_verts = fallback_box_vertices();
                    let body_mesh = GpuMesh::new(device, "fallback_box", &box_verts);
                    (body_mesh, rig, animator)
                }
            },
            None => {
                eprintln!("player_visual: character model '{char_name}' not found in pack; using fallback box mesh");
                let skel = fallback_skeleton();
                let rig = Rig::new(&skel);
                let animator = HumanoidAnimator::new(&skel, &[]);
                let box_verts = fallback_box_vertices();
                let body_mesh = GpuMesh::new(device, "fallback_box", &box_verts);
                (body_mesh, rig, animator)
            }
        };

        let hand_joint = rig.joint_index("hand_R").unwrap_or(0);

        let (weapon_mesh, weapon_name) = match weapon_model {
            Some(wname) => match pack.find(wname) {
                Some(wmodel) => (
                    Some(GpuMesh::new(device, wname, &wmodel.vertices)),
                    Some(wname.to_string()),
                ),
                None => {
                    eprintln!("player_visual: weapon model '{wname}' not found in pack; skipping drawing weapon");
                    (None, None)
                }
            },
            None => (None, None),
        };

        Self {
            body_mesh,
            rig,
            animator,
            weapon_mesh,
            weapon_name,
            hand_joint,
        }
    }

    /// Poses the character, writes skin matrices to `bones`, and returns the body and weapon batches.
    /// The weapon uses static instance bone_base 0 (identity matrix).
    pub fn pose(
        &mut self,
        dt: f32,
        pos: Vec3,
        yaw: f32,
        locomotion: Locomotion,
        bones: &mut Vec<Mat4>,
    ) -> (SkinnedBatch<'_>, Option<SkinnedBatch<'_>>) {
        self.animator.update(dt, locomotion);

        let base = bones.len();
        let joints = self.rig.joint_count();
        bones.resize(base + joints, Mat4::IDENTITY);
        self.animator.write_skin(&self.rig, &mut bones[base..]);

        let char_world = Mat4::from_translation(pos) * Mat4::from_rotation_y(yaw);
        let body_batch = SkinnedBatch {
            mesh: &self.body_mesh,
            instances: vec![SkinInstance::new(char_world, base as u32)],
        };

        let weapon_batch = self.weapon_mesh.as_ref().map(|wmesh| {
            let name = self.weapon_name.as_deref().unwrap_or("");
            let weapon_world = weapon_world_matrix_named(
                char_world,
                &self.rig,
                &bones[base..],
                self.hand_joint,
                name,
            );
            SkinnedBatch {
                mesh: wmesh,
                instances: vec![SkinInstance::new(weapon_world, 0)],
            }
        });

        (body_batch, weapon_batch)
    }

    /// Swaps the model held in the right hand (None: empty hand).
    #[allow(dead_code)] // unused by the humanoid_view example, which includes this file
    pub fn set_weapon(&mut self, device: &wgpu::Device, pack: &ModelPack, weapon_model: Option<&str>) {
        if weapon_model == self.weapon_name.as_deref() {
            return;
        }
        match weapon_model {
            Some(wname) => match pack.find(wname) {
                Some(model) => {
                    self.weapon_mesh = Some(GpuMesh::new(device, wname, &model.vertices));
                    self.weapon_name = Some(wname.to_string());
                }
                None => {
                    eprintln!("player_visual: weapon model '{wname}' not found in pack; skipping drawing weapon");
                    self.weapon_mesh = None;
                    self.weapon_name = None;
                }
            },
            None => {
                self.weapon_mesh = None;
                self.weapon_name = None;
            }
        }
    }

    #[allow(dead_code)]
    pub fn rig(&self) -> &Rig {
        &self.rig
    }

    #[allow(dead_code)]
    pub fn hand_joint(&self) -> usize {
        self.hand_joint
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pack::Model;

    #[test]
    fn player_visual_picks_fallback_when_mesh_missing() {
        let fallback_model = Model {
            name: "box".into(),
            folder: "characters".into(),
            vertices: vec![],
            bounds_min: [0.0; 3],
            bounds_max: [1.0; 3],
            markers: vec![],
            colliders: vec![],
            skeleton: None,
            clips: vec![],
        };
        let pack = ModelPack::new(vec![fallback_model]);

        let picked = pick_model_or_fallback(&pack, "missing_mesh", Some("box"));
        assert!(picked.is_some());
        assert_eq!(picked.unwrap().name, "box");

        let picked_primary = pick_model_or_fallback(&pack, "box", Some("other"));
        assert!(picked_primary.is_some());
        assert_eq!(picked_primary.unwrap().name, "box");

        let picked_none = pick_model_or_fallback(&pack, "missing_mesh", Some("missing_fallback"));
        assert!(picked_none.is_none());
    }
}

