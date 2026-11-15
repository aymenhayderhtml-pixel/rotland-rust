//! A crowd of animated zombies for the R0 speed test (no AI yet: they shamble on the spot).

use engine::anim::Rig;
use engine::render::{GpuMesh, SkinInstance, SkinnedBatch};
use glam::{Mat4, Vec3};

struct Kind {
    mesh: GpuMesh,
    rig: Rig,
    clip: pack::Clip,
    spots: Vec<(Vec3, f32, f32)>,
}

#[derive(Default)]
pub struct Crowd {
    kinds: Vec<Kind>,
    bones: Vec<Mat4>,
}

impl Crowd {
    /// `count` zombies of the three looks in a loose crowd around `center`, standing on the ground.
    pub fn load(device: &wgpu::Device, pack_path: &str, center: Vec3, count: usize, height: impl Fn(f32, f32) -> f32) -> Self {
        let models = match pack::ModelPack::load(pack_path) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("no model pack ({e}); run `cargo run --release -p baker`");
                return Self::default();
            }
        };
        let mut kinds = Vec::new();
        for (k, name) in ["zombie_01", "zombie_02", "zombie_03"].iter().enumerate() {
            let Some(m) = models.find(name) else { continue };
            let (Some(skel), Some(clip)) = (&m.skeleton, m.clips.iter().find(|c| c.name == "zombie_shamble")) else { continue };
            let spots = (k..count)
                .step_by(3)
                .map(|i| {
                    let a = i as f32 * 2.399;
                    let r = 2.0 + (i as f32).sqrt() * 2.2;
                    let (x, z) = (center.x + a.cos() * r, center.z + a.sin() * r);
                    (Vec3::new(x, height(x, z), z), a, i as f32 * 0.37)
                })
                .collect();
            kinds.push(Kind { mesh: GpuMesh::new(device, name, &m.vertices), rig: Rig::new(skel), clip: clip.clone(), spots });
        }
        Self { kinds, bones: Vec::new() }
    }

    pub fn count(&self) -> usize {
        self.kinds.iter().map(|k| k.spots.len()).sum()
    }

    /// This frame's poses: one instanced batch per look, and every instance's skin matrices.
    pub fn pose(&mut self, time: f32) -> (Vec<SkinnedBatch<'_>>, &[Mat4]) {
        self.bones.clear();
        let mut batches = Vec::new();
        for kind in &self.kinds {
            let joints = kind.rig.joint_count();
            let mut instances = Vec::with_capacity(kind.spots.len());
            for &(pos, yaw, phase) in &kind.spots {
                let base = self.bones.len();
                self.bones.resize(base + joints, Mat4::IDENTITY);
                kind.rig.sample(&kind.clip, time + phase, &mut self.bones[base..]);
                instances.push(SkinInstance::new(Mat4::from_translation(pos) * Mat4::from_rotation_y(yaw), base as u32));
            }
            batches.push(SkinnedBatch { mesh: &kind.mesh, instances });
        }
        (batches, &self.bones)
    }
}
