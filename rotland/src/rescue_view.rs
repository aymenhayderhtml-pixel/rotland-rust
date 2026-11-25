//! Renders trapped world survivors using the `npc_survivor` model in the idle animation
//! within normal NPC view distance (`data/camp_life.json` `farRange`).

use engine::anim::Rig;
use engine::render::{GpuMesh, SkinInstance, SkinnedBatch};
use game::rescue_place::RescueSpot;
use glam::{Mat4, Vec3};

const SURVIVOR_MODEL: &str = "npc_survivor";
const IDLE_CLIP: &str = "idle";
const DEFAULT_VIEW_DISTANCE: f32 = 60.0; // data: camp_life.json farRange
const PHASE_STEP: f32 = 0.37; // new: DECISIONS 2026-11-25

struct Look {
    mesh: GpuMesh,
    rig: Rig,
    clips: Vec<pack::Clip>,
}

/// GPU mesh/rig cache and animation clock for unrescued world survivors.
pub struct RescueView {
    look: Option<Look>,
    time: f32,
    view_distance: f32,
}

impl RescueView {
    /// Prepares the `npc_survivor` mesh, skeleton, and idle clip from the model pack.
    pub fn new(device: &wgpu::Device, pack: &pack::ModelPack) -> Self {
        let view_distance = game::CampLifeConfig::load_default()
            .map_or(DEFAULT_VIEW_DISTANCE, |d| d.far_range);
        let look = pack.find(SURVIVOR_MODEL).and_then(|m| {
            let skel = m.skeleton.as_ref()?;
            Some(Look {
                mesh: GpuMesh::new(device, SURVIVOR_MODEL, &m.vertices),
                rig: Rig::new(skel),
                clips: m.clips.clone(),
            })
        });
        if look.is_none() {
            log::warn!("rescue survivor model {SURVIVOR_MODEL} is not in the pack");
        }
        Self {
            look,
            time: 0.0,
            view_distance,
        }
    }

    /// Advances the idle animation clock.
    pub fn update(&mut self, dt: f32) {
        self.time += dt;
    }

    /// Adds bone matrices for active survivors within NPC view distance and returns their batch.
    pub fn pose(&self, spots: &[RescueSpot], player: Vec3, bones: &mut Vec<Mat4>) -> Vec<SkinnedBatch<'_>> {
        let Some(look) = &self.look else {
            return Vec::new();
        };
        let Some(clip) = look.clips.iter().find(|c| c.name == IDLE_CLIP) else {
            return Vec::new();
        };
        if spots.is_empty() {
            return Vec::new();
        }
        let max_dist_sq = self.view_distance * self.view_distance;
        let mut instances = Vec::new();
        for (i, spot) in spots.iter().enumerate() {
            let dx = player.x - spot.position.x;
            let dz = player.z - spot.position.z;
            if dx * dx + dz * dz > max_dist_sq {
                continue;
            }
            let phase = (i as f32 * PHASE_STEP) % 1.0;
            let base = bones.len();
            bones.resize(base + look.rig.joint_count(), Mat4::IDENTITY);
            look.rig.sample(clip, self.time + phase, &mut bones[base..]);
            let model = Mat4::from_translation(spot.position) * Mat4::from_rotation_y(spot.yaw);
            instances.push(SkinInstance::new(model, base as u32));
        }
        if instances.is_empty() {
            Vec::new()
        } else {
            vec![SkinnedBatch {
                mesh: &look.mesh,
                instances,
            }]
        }
    }
}
