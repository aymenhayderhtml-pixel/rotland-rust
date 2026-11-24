//! Rendering and animation matrix pose generation for live zombies.

use engine::render::{SkinInstance, SkinnedBatch};
use game::zombie::AnimClip;
use glam::{Mat4, Vec3};

use super::types::resolved_clip_name;
use super::Zombies;

const DIE_HOLD_EPSILON: f32 = 0.01; // new: DECISIONS 2026-11-24

impl Zombies {
    /// Adds every zombie's skin matrices to `bones` and returns one instanced batch per look.
    pub fn pose(&self, camera: Vec3, bones: &mut Vec<Mat4>) -> Vec<SkinnedBatch<'_>> {
        let mut per_look: Vec<Vec<SkinInstance>> = vec![Vec::new(); self.looks.len()];
        let max_blend_dist_sq =
            self.anim_cfg.blend_max_distance * self.anim_cfg.blend_max_distance;
        for z in &self.list {
            let look = &self.looks[z.look];
            let name = resolved_clip_name(z, look);
            let Some(clip) = look.clip(name).or_else(|| look.clip("idle")) else {
                continue;
            };
            let t = if z.anim == AnimClip::Die {
                z.fade.time().min(clip.duration - DIE_HOLD_EPSILON).max(0.0)
            } else {
                z.fade.time()
            };
            let base = bones.len();
            bones.resize(base + look.rig.joint_count(), Mat4::IDENTITY);
            if z.position.distance_squared(camera) <= max_blend_dist_sq
                && let Some((prev_key, prev_t, weight)) = z.fade.blend()
                && let Some(prev_clip) = look.clip_by_key(prev_key)
            {
                look.rig
                    .sample_blend(prev_clip, prev_t, clip, t, weight, &mut bones[base..]);
            } else {
                look.rig.sample(clip, t, &mut bones[base..]);
            }
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
