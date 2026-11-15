//! A crowd of animated zombies for the R0 speed test. No real AI yet: they walk straight at a target.

use engine::anim::Rig;
use engine::render::{GpuMesh, SkinInstance, SkinnedBatch};
use glam::{Mat4, Vec3};

struct Zombie {
    pos: Vec3,
    yaw: f32,
    phase: f32,
}

struct Kind {
    mesh: GpuMesh,
    rig: Rig,
    clip: pack::Clip,
    zombies: Vec<Zombie>,
}

#[derive(Default)]
pub struct Crowd {
    kinds: Vec<Kind>,
}

/// Shamble speed in m/s (zombies.json walkers are slow).
const SPEED: f32 = 1.1;
const STOP_AT: f32 = 1.6;

impl Crowd {
    /// `count` zombies of the three looks on a ring `min..max` metres around `center` (like BenchmarkZombies).
    pub fn spawn(device: &wgpu::Device, pack: &pack::ModelPack, center: Vec3, count: usize, min: f32, max: f32, height: &dyn Fn(f32, f32) -> f32) -> Self {
        let mut kinds = Vec::new();
        for (k, name) in ["zombie_01", "zombie_02", "zombie_03"].iter().enumerate() {
            let Some(m) = pack.find(name) else { continue };
            let (Some(skel), Some(clip)) = (&m.skeleton, m.clips.iter().find(|c| c.name == "zombie_shamble")) else { continue };
            let zombies = (k..count)
                .step_by(3)
                .map(|i| {
                    let angle = std::f32::consts::TAU * i as f32 / count as f32 + ((i * 7919) % 100) as f32 * 0.004 - 0.2;
                    let dist = min + (max - min) * (((i * 104729) % 1000) as f32 / 1000.0);
                    let (x, z) = (center.x + angle.cos() * dist, center.z + angle.sin() * dist);
                    Zombie { pos: Vec3::new(x, height(x, z), z), yaw: 0.0, phase: i as f32 * 0.37 }
                })
                .collect();
            kinds.push(Kind { mesh: GpuMesh::new(device, name, &m.vertices), rig: Rig::new(skel), clip: clip.clone(), zombies });
        }
        Self { kinds }
    }

    pub fn count(&self) -> usize {
        self.kinds.iter().map(|k| k.zombies.len()).sum()
    }

    /// Everyone walks toward `target`, stays on the ground, and faces where it goes.
    pub fn update(&mut self, dt: f32, target: Vec3, height: &dyn Fn(f32, f32) -> f32) {
        for z in self.kinds.iter_mut().flat_map(|k| k.zombies.iter_mut()) {
            let mut to = target - z.pos;
            to.y = 0.0;
            let d = to.length();
            if d > STOP_AT {
                let step = to / d * SPEED * dt;
                z.pos += step;
                z.pos.y = height(z.pos.x, z.pos.z);
            }
            // Models face +Z
            z.yaw = to.x.atan2(to.z);
        }
    }

    /// Adds this frame's skin matrices to `bones` and returns one instanced batch per look.
    pub fn pose(&self, time: f32, bones: &mut Vec<Mat4>) -> Vec<SkinnedBatch<'_>> {
        let mut batches = Vec::new();
        for kind in &self.kinds {
            let joints = kind.rig.joint_count();
            let mut instances = Vec::with_capacity(kind.zombies.len());
            for z in &kind.zombies {
                let base = bones.len();
                bones.resize(base + joints, Mat4::IDENTITY);
                kind.rig.sample(&kind.clip, time + z.phase, &mut bones[base..]);
                instances.push(SkinInstance::new(Mat4::from_translation(z.pos) * Mat4::from_rotation_y(z.yaw), base as u32));
            }
            batches.push(SkinnedBatch { mesh: &kind.mesh, instances });
        }
        batches
    }
}
