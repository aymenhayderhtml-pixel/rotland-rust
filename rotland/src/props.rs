//! Placed models (trees, rocks, bushes, grass, buildings) drawn as instances, with Godot's distance LODs
//! and shadow rules (ScatterConfig.RangeFor, ChunkProps.ShouldCastShadow). Models load from the pack on
//! first use. Static instances use joint 0 of the bone list, which is always the identity.

use std::collections::HashMap;

use engine::render::frustum::Frustum;
use engine::render::{Camera, GpuMesh, SkinInstance, SkinnedBatch};
use glam::{Mat4, Vec3};

/// Distances from data/scatter.json "visibility".
const TREE: f32 = 170.0;
const TREE_LOD1: f32 = 45.0;
const TREE_LOD2: f32 = 95.0;
const ROCK: f32 = 80.0;
const BUSH: f32 = 65.0;
const BUSH_LOD1: f32 = 28.0;
const GROUND: f32 = 35.0;
const PROP: f32 = 120.0;
/// Buildings stay visible to the fog line.
const BUILDING: f32 = 220.0;
/// Buildings switch to their simple model here (was the bush distance, 28 m, which popped up close).
const BUILDING_LOD1: f32 = 70.0;

pub struct PropModel {
    /// Full detail, then the simpler versions (if the model has them).
    lods: Vec<GpuMesh>,
    /// Where each LOD stops being used (the last one is the visibility range).
    ends: Vec<f32>,
    casts_shadow: bool,
    radius: f32,
    /// Height of the model's middle above its origin (origins sit on the ground).
    center_y: f32,
}

/// One placed model: which model, its world matrix, and the centre of its bounds.
#[derive(Clone, Copy)]
pub struct Placed {
    pub model: usize,
    pub matrix: Mat4,
    pub centre: Vec3,
}

fn range_for(id: &str) -> f32 {
    let starts = |p: &str| id.starts_with(p);
    if starts("tree_") || starts("pine_") {
        TREE
    } else if starts("rock_") {
        ROCK
    } else if starts("bush_") {
        BUSH
    } else if starts("grass_") || starts("flowers_") || starts("fern") || starts("mushrooms") {
        GROUND
    } else {
        PROP
    }
}

fn casts_shadow(id: &str) -> bool {
    !["grass_", "flowers_", "fern", "mushrooms", "rock_", "bush_"].iter().any(|p| id.starts_with(p))
}

/// Every model used so far, by name.
#[derive(Default)]
pub struct PropLibrary {
    models: Vec<PropModel>,
    index: HashMap<String, Option<usize>>,
}

impl PropLibrary {
    /// The model's slot, loading it from the pack the first time. None if the pack does not have it.
    pub fn get(&mut self, device: &wgpu::Device, pack: &pack::ModelPack, id: &str, building: bool) -> Option<usize> {
        if let Some(slot) = self.index.get(id) {
            return *slot;
        }
        let slot = load_model(device, pack, id, building).map(|m| {
            self.models.push(m);
            self.models.len() - 1
        });
        if slot.is_none() {
            log::warn!("model {id} is not in the pack");
        }
        self.index.insert(id.to_string(), slot);
        slot
    }

    /// Places model `slot` at `position`, turned by `yaw`, scaled by `scale` (Godot's Basis.FromEuler(0, yaw, 0).Scaled).
    pub fn place(&self, slot: usize, position: Vec3, yaw: f32, scale: f32) -> Placed {
        let m = &self.models[slot];
        let matrix = Mat4::from_translation(position) * Mat4::from_rotation_y(yaw) * Mat4::from_scale(Vec3::splat(scale));
        Placed { model: slot, matrix, centre: position + Vec3::Y * m.center_y * scale }
    }

    /// This frame's batches for the camera and for the sun shadow (one per model and LOD that has instances).
    pub fn batches<'a>(
        &'a self,
        placed: impl Iterator<Item = &'a Placed>,
        camera: &Camera,
        aspect: f32,
        shadow_distance: f32,
    ) -> (Vec<SkinnedBatch<'a>>, Vec<SkinnedBatch<'a>>) {
        let frustum = Frustum::from_view_proj(&camera.view_proj(aspect));
        let mut first_slot = Vec::with_capacity(self.models.len());
        let mut lod_count = 0;
        for m in &self.models {
            first_slot.push(lod_count);
            lod_count += m.lods.len();
        }
        let mut main: Vec<Vec<SkinInstance>> = vec![Vec::new(); lod_count];
        let mut shadow: Vec<Vec<SkinInstance>> = vec![Vec::new(); lod_count];
        for p in placed {
            let m = &self.models[p.model];
            let dist = p.centre.distance(camera.position);
            let Some(lod) = m.ends.iter().position(|&end| dist < end) else { continue };
            let inst = SkinInstance::new(p.matrix, 0);
            let r = Vec3::splat(m.radius);
            if frustum.intersects_aabb(p.centre - r, p.centre + r) {
                main[first_slot[p.model] + lod].push(inst);
            }
            // Like Godot: only the near versions cast shadows, and only inside the shadow distance
            if m.casts_shadow && lod < 2 && dist < shadow_distance + m.radius {
                shadow[first_slot[p.model] + lod].push(inst);
            }
        }
        let meshes: Vec<&GpuMesh> = self.models.iter().flat_map(|m| m.lods.iter()).collect();
        let to_batches = |lists: Vec<Vec<SkinInstance>>| {
            lists.into_iter().enumerate().filter(|(_, v)| !v.is_empty()).map(|(i, instances)| SkinnedBatch { mesh: meshes[i], instances }).collect()
        };
        (to_batches(main), to_batches(shadow))
    }
}

fn load_model(device: &wgpu::Device, pack: &pack::ModelPack, id: &str, building: bool) -> Option<PropModel> {
    let full = pack.find(id)?;
    let mut lods = vec![GpuMesh::new(device, id, &full.vertices)];
    for k in 1..=2 {
        if let Some(m) = pack.find(&format!("{id}_lod{k}")) {
            lods.push(GpuMesh::new(device, id, &m.vertices));
        }
    }
    let range = if building { BUILDING } else { range_for(id) };
    let ends = match lods.len() {
        3 => vec![TREE_LOD1, TREE_LOD2, range],
        2 if building => vec![BUILDING_LOD1, range],
        2 => vec![BUSH_LOD1, range],
        _ => vec![range],
    };
    let radius = (Vec3::from(full.bounds_max) - Vec3::from(full.bounds_min)).length() * 0.5 * 1.4;
    let center_y = (full.bounds_min[1] + full.bounds_max[1]) * 0.5;
    Some(PropModel { lods, ends, casts_shadow: building || casts_shadow(id), radius, center_y })
}
