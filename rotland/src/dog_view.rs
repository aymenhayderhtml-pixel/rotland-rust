//! The player's dog (and strays) on screen: game::dogs moves them, this draws them like the animals
//! (one instanced batch per model, each dog with its clip clock) and scales puppies.

use std::collections::HashMap;

use engine::anim::Rig;
use engine::render::{GpuMesh, SkinInstance, SkinnedBatch};
use game::{DogDrawState, DogFrame, DogUpdateContext, Dogs};
use glam::{Mat4, Vec3};
use physics::PhysicsWorld;

const MODELS: [&str; 2] = ["dog_tan", "dog_dark"];
/// Clips that play once and hold their last pose.
const ONCE: [&str; 1] = ["die"];

struct Look {
    mesh: GpuMesh,
    rig: Rig,
    clips: Vec<pack::Clip>,
}

pub struct DogView {
    pub sim: Dogs,
    looks: Vec<Look>,
    look_index: HashMap<String, usize>,
    /// Per dog: the clip playing and how long it has played.
    clocks: HashMap<u64, (String, f32)>,
    draws: Vec<DogDrawState>,
}

impl DogView {
    pub fn new(device: &wgpu::Device, pack: &pack::ModelPack) -> Result<Self, String> {
        let sim = Dogs::from_repo()?;
        let mut looks = Vec::new();
        let mut look_index = HashMap::new();
        for name in MODELS {
            let Some(m) = pack.find(name) else {
                log::warn!("dog model {name} is not in the pack");
                continue;
            };
            let Some(skel) = m.skeleton.as_ref() else { continue };
            look_index.insert(name.to_string(), looks.len());
            looks.push(Look { mesh: GpuMesh::new(device, name, &m.vertices), rig: Rig::new(skel), clips: m.clips.clone() });
        }
        Ok(Self { sim, looks, look_index, clocks: HashMap::new(), draws: Vec::new() })
    }

    pub fn update(&mut self, dt: f32, ctx: &DogUpdateContext, physics: &PhysicsWorld) -> DogFrame {
        let frame = self.sim.update(dt, ctx, physics);
        for d in &frame.draws {
            let entry = self.clocks.entry(d.id).or_insert_with(|| (d.clip.clone(), 0.0));
            if entry.0 != d.clip {
                *entry = (d.clip.clone(), 0.0);
            }
            entry.1 += dt;
        }
        self.draws = frame.draws.clone();
        frame
    }

    /// The player's own dog within reach in front: (id, name).
    pub fn own_dog_in_front(&self, player: Vec3, facing: Vec3) -> Option<(u64, String)> {
        let dog = self.sim.dog()?;
        if dog.is_dead() {
            return None;
        }
        let d = Vec3::new(dog.position().x - player.x, 0.0, dog.position().z - player.z);
        let flat = Vec3::new(facing.x, 0.0, facing.z).normalize_or_zero();
        (d.length() < 2.2 && (d.length() < 0.8 || d.normalize_or_zero().dot(flat) > 0.4)).then(|| (dog.id(), dog.name().to_string()))
    }

    /// Adds every dog's skin matrices to `bones` and returns one batch per model.
    pub fn pose(&self, bones: &mut Vec<Mat4>) -> Vec<SkinnedBatch<'_>> {
        let mut per_look: Vec<Vec<SkinInstance>> = vec![Vec::new(); self.looks.len()];
        for d in &self.draws {
            let Some(&li) = self.look_index.get(&d.model) else { continue };
            let look = &self.looks[li];
            let Some(clip) = look.clips.iter().find(|c| c.name == d.clip).or_else(|| look.clips.iter().find(|c| c.name == "idle")) else {
                continue;
            };
            let t = self.clocks.get(&d.id).map_or(0.0, |c| c.1);
            let t = if ONCE.contains(&clip.name.as_str()) { t.min(clip.duration - 0.01).max(0.0) } else { t };
            let base = bones.len();
            bones.resize(base + look.rig.joint_count(), Mat4::IDENTITY);
            look.rig.sample(clip, t, &mut bones[base..]);
            let model = Mat4::from_translation(d.position) * Mat4::from_rotation_y(d.yaw) * Mat4::from_scale(Vec3::splat(d.scale.max(0.1)));
            per_look[li].push(SkinInstance::new(model, base as u32));
        }
        per_look
            .into_iter()
            .zip(&self.looks)
            .filter(|(v, _)| !v.is_empty())
            .map(|(instances, look)| SkinnedBatch { mesh: &look.mesh, instances })
            .collect()
    }
}
