//! Rendering and animation matrix pose generation for live zombies.

use engine::render::{SkinInstance, SkinnedBatch};
use game::zombie::AnimClip;
use glam::Mat4;

use super::types::clip_name;
use super::Zombies;

impl Zombies {
    /// Adds every zombie's skin matrices to `bones` and returns one instanced batch per look.
    pub fn pose(&self, bones: &mut Vec<Mat4>) -> Vec<SkinnedBatch<'_>> {
        let mut per_look: Vec<Vec<SkinInstance>> = vec![Vec::new(); self.looks.len()];
        for z in &self.list {
            let look = &self.looks[z.look];
            let name = clip_name(z);
            let Some(clip) = look.clip(name).or_else(|| look.clip("idle")) else {
                continue;
            };
            let t = if z.anim == AnimClip::Die {
                z.anim_time.min(clip.duration - 0.01).max(0.0)
            } else {
                z.anim_time
            };
            let base = bones.len();
            bones.resize(base + look.rig.joint_count(), Mat4::IDENTITY);
            look.rig.sample(clip, t, &mut bones[base..]);
            let model = Mat4::from_translation(z.position) * Mat4::from_rotation_y(z.yaw);
            per_look[z.look].push(SkinInstance::new(model, base as u32));
        }
        per_look
            .into_iter()
            .zip(&self.looks)
            .filter(|(v, _)| !v.is_empty())
            .map(|(instances, look)| SkinnedBatch {
                mesh: &look.mesh,
                instances,
            })
            .collect()
    }
}
