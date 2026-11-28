//! The camp's people on screen: the trader, banker, guards and survivors standing at their posts, and
//! the residents walking their daily routine (game::camp). Drawn like the zombies: one instanced batch
//! per model, each person with its own clip and time.

use std::collections::HashMap;

use engine::anim::Rig;
use engine::render::{GpuMesh, SkinInstance, SkinnedBatch};
use game::CampLifeDirector;
use glam::{Mat4, Vec3};

use crate::rooftop_npcs::RooftopFigure;

/// One person model: mesh, skeleton and clips.
struct Look {
    mesh: GpuMesh,
    rig: Rig,
    clips: Vec<pack::Clip>,
}

pub struct CampNpcs {
    pub director: CampLifeDirector,
    /// Guards turned toward a target this frame: id -> yaw.
    pub aim: HashMap<String, f32>,
    looks: Vec<Look>,
    look_index: HashMap<String, usize>,
    /// Per person: the clip playing and how long it has played.
    clocks: HashMap<String, (String, f32)>,
    /// The rooftop camp's people (R44), animated like the camp's own.
    rooftop: Vec<RooftopFigure>,
}

/// Something to draw this frame.
struct Figure {
    id: String,
    model: String,
    position: Vec3,
    yaw: f32,
    clip: String,
    /// A start offset so people with the same clip do not move in step.
    phase: f32,
}

impl CampNpcs {
    pub fn new(
        device: &wgpu::Device,
        pack: &pack::ModelPack,
        director: CampLifeDirector,
        rooftop: Vec<RooftopFigure>,
    ) -> Self {
        let mut looks = Vec::new();
        let mut look_index = HashMap::new();
        let names = director
            .fixed_npcs()
            .iter()
            .map(|n| n.model_id.clone())
            .chain(director.residents().iter().map(|r| r.render_output(false).model_name))
            .chain(rooftop.iter().map(|r| r.model.clone()));
        for name in names {
            if look_index.contains_key(&name) {
                continue;
            }
            let Some(m) = pack.find(&name) else {
                // The bounty board is part of the camp model itself, not a person
                log::debug!("camp model {name} is not in the pack");
                continue;
            };
            let Some(skel) = m.skeleton.as_ref() else { continue };
            look_index.insert(name.clone(), looks.len());
            looks.push(Look { mesh: GpuMesh::new(device, &name, &m.vertices), rig: Rig::new(skel), clips: m.clips.clone() });
        }
        Self { director, aim: HashMap::new(), looks, look_index, clocks: HashMap::new(), rooftop }
    }

    /// Runs the camp's routine and advances every person's animation clock.
    pub fn update(&mut self, dt: f32, hour: f32, player: Option<Vec3>) {
        self.director.tick(dt, hour, player);
        for f in self.figures(player) {
            let entry = self.clocks.entry(f.id).or_insert_with(|| (f.clip.clone(), f.phase));
            if entry.0 != f.clip {
                *entry = (f.clip, 0.0);
            }
            entry.1 += dt;
        }
    }

    fn figures(&self, player: Option<Vec3>) -> Vec<Figure> {
        let fixed = self.director.fixed_npcs().iter().filter(|n| !n.is_resident).map(|n| Figure {
            id: n.id.clone(),
            model: n.model_id.clone(),
            position: n.position,
            yaw: self.aim.get(&n.id).copied().unwrap_or(n.yaw),
            clip: n.animation.clone(),
            phase: n.idle_phase,
        });
        let residents = self.director.resident_outputs(player).into_iter().filter(|o| o.visible).map(|o| Figure {
            phase: (o.id.len() as f32 * 0.37) % 1.0,
            id: o.id,
            model: o.model_name,
            position: o.position,
            yaw: o.yaw,
            clip: o.animation,
        });
        let rooftop = self.rooftop.iter().map(|r| Figure {
            phase: (r.id.len() as f32 * 0.21) % 1.0,
            id: r.id.clone(),
            model: r.model.clone(),
            position: r.position,
            yaw: r.yaw,
            clip: r.clip.clone(),
        });
        fixed.chain(residents).chain(rooftop).collect()
    }

    /// Adds every person's skin matrices to `bones` and returns one batch per model.
    pub fn pose(&self, player: Option<Vec3>, bones: &mut Vec<Mat4>) -> Vec<SkinnedBatch<'_>> {
        let mut per_look: Vec<Vec<SkinInstance>> = vec![Vec::new(); self.looks.len()];
        for f in self.figures(player) {
            let Some(&li) = self.look_index.get(&f.model) else { continue };
            let look = &self.looks[li];
            let Some(clip) = look.clips.iter().find(|c| c.name == f.clip).or_else(|| look.clips.iter().find(|c| c.name == "idle")) else {
                continue;
            };
            let t = self.clocks.get(&f.id).map_or(f.phase, |c| c.1);
            let base = bones.len();
            bones.resize(base + look.rig.joint_count(), Mat4::IDENTITY);
            look.rig.sample(clip, t, &mut bones[base..]);
            let model = Mat4::from_translation(f.position) * Mat4::from_rotation_y(f.yaw);
            per_look[li].push(SkinInstance::new(model, base as u32));
        }
        per_look
            .into_iter()
            .zip(&self.looks)
            .filter(|(v, _)| !v.is_empty())
            .map(|(instances, look)| SkinnedBatch { mesh: &look.mesh, instances })
            .collect()
    }

    /// The person or board the player faces within reach: (id, role, prompt).
    pub fn facing(&self, player: Vec3, facing: Vec3) -> Option<(String, String, String)> {
        self.director.nearest_interactable(player, facing)
    }
}
