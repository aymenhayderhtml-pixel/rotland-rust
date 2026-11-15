//! Player visual representation: rigged body mesh, humanoid animation state,
//! and static weapon model attached to the right hand.

use engine::anim::Rig;
use engine::render::{GpuMesh, SkinInstance, SkinnedBatch};
use glam::{Mat4, Vec3};
use humanoid::{weapon_world_matrix_named, HumanoidAnimator, Locomotion};
use pack::ModelPack;

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
    pub fn load(
        device: &wgpu::Device,
        pack: &ModelPack,
        char_name: &str,
        weapon_model: Option<&str>,
    ) -> Self {
        let char_model = pack
            .find(char_name)
            .unwrap_or_else(|| panic!("character model '{char_name}' not found in pack"));
        let skeleton = char_model
            .skeleton
            .as_ref()
            .unwrap_or_else(|| panic!("character model '{char_name}' has no skeleton"));
        let rig = Rig::new(skeleton);
        let hand_joint = rig.joint_index("hand_R").unwrap_or(0);
        let body_mesh = GpuMesh::new(device, char_name, &char_model.vertices);
        let animator = HumanoidAnimator::new(skeleton, &char_model.clips);

        let (weapon_mesh, weapon_name) = match weapon_model {
            Some(wname) => {
                let wmodel = pack
                    .find(wname)
                    .unwrap_or_else(|| panic!("weapon model '{wname}' not found in pack"));
                (
                    Some(GpuMesh::new(device, wname, &wmodel.vertices)),
                    Some(wname.to_string()),
                )
            }
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
        match weapon_model.and_then(|w| pack.find(w).map(|m| (w, m))) {
            Some((name, model)) => {
                self.weapon_mesh = Some(GpuMesh::new(device, name, &model.vertices));
                self.weapon_name = Some(name.to_string());
            }
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
