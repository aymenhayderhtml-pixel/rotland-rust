//! Hired teammates on screen: each is drawn with an existing NPC model and its own clip clock, the same way
//! the camp people and the dogs are drawn (one instanced batch per model).

use std::collections::HashMap;

use engine::anim::Rig;
use engine::render::{GpuMesh, SkinInstance, SkinnedBatch};
use game::guild::companion::CompanionAnim;
use game::guild::RecruitRole;
use glam::Mat4;

use crate::app::frame_team::TeamFigure;

/// Fighter and guard use the guard person (npc_guard in tools/blender/characters/build_characters.py).
const GUARD_MODEL: &str = "npc_guard";
/// The scavenger uses the scout person (npc_scout in tools/blender/characters/build_characters.py).
const SCOUT_MODEL: &str = "npc_scout";
const MODELS: [&str; 2] = [GUARD_MODEL, SCOUT_MODEL];
/// Clips that play once and hold their last pose.
const ONCE: [&str; 1] = ["die"];

/// Which person model a recruit role is drawn with.
pub fn model_for_role(role: RecruitRole) -> &'static str {
    match role {
        RecruitRole::Scavenger => SCOUT_MODEL,
        RecruitRole::Fighter | RecruitRole::Guard => GUARD_MODEL,
    }
}

/// Which NPC clip plays for an animation state; a missing clip falls back to idle.
fn clip_for(anim: CompanionAnim) -> &'static str {
    match anim {
        CompanionAnim::Idle => "idle",
        CompanionAnim::Walk => "walk",
        CompanionAnim::Run => "run",
        CompanionAnim::Attack => "gun_bash",
        CompanionAnim::Downed => "die",
    }
}

struct Look {
    mesh: GpuMesh,
    rig: Rig,
    clips: Vec<pack::Clip>,
}

pub struct CompanionModels {
    looks: Vec<Look>,
    look_index: HashMap<&'static str, usize>,
    /// Per teammate: the clip playing and how long it has played.
    clocks: HashMap<String, (&'static str, f32)>,
}

impl CompanionModels {
    pub fn new(device: &wgpu::Device, pack: &pack::ModelPack) -> Self {
        let mut looks = Vec::new();
        let mut look_index = HashMap::new();
        for name in MODELS {
            let Some(m) = pack.find(name) else {
                log::warn!("teammate model {name} is not in the pack");
                continue;
            };
            let Some(skel) = m.skeleton.as_ref() else { continue };
            look_index.insert(name, looks.len());
            looks.push(Look {
                mesh: GpuMesh::new(device, name, &m.vertices),
                rig: Rig::new(skel),
                clips: m.clips.clone(),
            });
        }
        Self { looks, look_index, clocks: HashMap::new() }
    }

    /// Moves each teammate's clip clock on by `dt`, restarting it when the clip changes.
    pub fn advance(&mut self, dt: f32, figures: &[TeamFigure]) {
        for f in figures {
            let clip = clip_for(f.draw.anim);
            match self.clocks.get_mut(&f.draw.id) {
                Some(entry) => {
                    if entry.0 != clip {
                        *entry = (clip, 0.0);
                    }
                    entry.1 += dt;
                }
                None => {
                    self.clocks.insert(f.draw.id.clone(), (clip, dt));
                }
            }
        }
        self.clocks.retain(|id, _| figures.iter().any(|f| f.draw.id == *id));
    }

    /// Adds every teammate's skin matrices to `bones` and returns one batch per model.
    pub fn pose(&self, figures: &[TeamFigure], bones: &mut Vec<Mat4>) -> Vec<SkinnedBatch<'_>> {
        let mut per_look: Vec<Vec<SkinInstance>> = vec![Vec::new(); self.looks.len()];
        for f in figures {
            let Some(&li) = self.look_index.get(f.model) else { continue };
            let look = &self.looks[li];
            let wanted = clip_for(f.draw.anim);
            let Some(clip) = look
                .clips
                .iter()
                .find(|c| c.name == wanted)
                .or_else(|| look.clips.iter().find(|c| c.name == "idle"))
            else {
                continue;
            };
            let t = self.clocks.get(&f.draw.id).map_or(0.0, |c| c.1);
            let t = if ONCE.contains(&clip.name.as_str()) { t.min(clip.duration - 0.01).max(0.0) } else { t };
            let base = bones.len();
            bones.resize(base + look.rig.joint_count(), Mat4::IDENTITY);
            look.rig.sample(clip, t, &mut bones[base..]);
            let model = Mat4::from_translation(f.draw.position) * Mat4::from_rotation_y(f.draw.yaw);
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
