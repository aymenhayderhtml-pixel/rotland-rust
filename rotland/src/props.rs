//! Scattered props (trees, rocks, bushes, grass...) as instanced models, with Godot's distance LODs and
//! shadow rules (ScatterConfig.RangeFor, ChunkProps.ShouldCastShadow). Static instances use joint 0 of the
//! bone list, which is always the identity.

use std::collections::HashMap;

use engine::render::{Camera, GpuMesh, SkinInstance, SkinnedBatch};
use engine::render::frustum::Frustum;
use glam::{IVec2, Mat4, Vec3};
use rayon::prelude::*;
use world::{PropInstance, WorldGenerator};

/// Distances from data/scatter.json "visibility".
const TREE: f32 = 170.0;
const TREE_LOD1: f32 = 45.0;
const TREE_LOD2: f32 = 95.0;
const ROCK: f32 = 80.0;
const BUSH: f32 = 65.0;
const BUSH_LOD1: f32 = 28.0;
const GROUND: f32 = 35.0;
const PROP: f32 = 120.0;

struct PropModel {
    /// Full detail, then the simpler versions (if the model has them).
    lods: Vec<GpuMesh>,
    /// Where each LOD stops being used (the last one is the visibility range).
    ends: Vec<f32>,
    casts_shadow: bool,
    radius: f32,
    /// Height of the model's middle above its origin (origins sit on the ground).
    center_y: f32,
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

pub struct Props {
    models: Vec<PropModel>,
    /// (model index, world matrix, centre of its bounds) for every prop in the loaded chunks.
    instances: Vec<(usize, Mat4, Vec3)>,
}

impl Props {
    pub fn build(device: &wgpu::Device, pack: &pack::ModelPack, world_gen: &WorldGenerator, coords: &[IVec2]) -> Self {
        let per_chunk: Vec<Vec<PropInstance>> = coords.par_iter().map(|&c| world_gen.chunk_contents(c).1).collect();
        let mut index: HashMap<String, usize> = HashMap::new();
        let mut models = Vec::new();
        let mut instances = Vec::new();
        for prop in per_chunk.into_iter().flatten() {
            let slot = match index.get(&prop.model) {
                Some(&i) => i,
                None => {
                    let Some(model) = load_model(device, pack, &prop.model) else { continue };
                    models.push(model);
                    index.insert(prop.model.clone(), models.len() - 1);
                    models.len() - 1
                }
            };
            let m = Mat4::from_translation(prop.position) * Mat4::from_rotation_y(prop.yaw) * Mat4::from_scale(Vec3::splat(prop.scale));
            let lift = Vec3::Y * models[slot].center_y * prop.scale;
            instances.push((slot, m, prop.position + lift));
        }
        Self { models, instances }
    }

    pub fn count(&self) -> usize {
        self.instances.len()
    }

    /// This frame's batches for the camera and for the sun shadow (one per model and LOD that has instances).
    pub fn batches(&self, camera: &Camera, aspect: f32, shadow_distance: f32) -> (Vec<SkinnedBatch<'_>>, Vec<SkinnedBatch<'_>>) {
        let frustum = Frustum::from_view_proj(&camera.view_proj(aspect));
        let lod_count: usize = self.models.iter().map(|m| m.lods.len()).sum();
        let mut main: Vec<Vec<SkinInstance>> = vec![Vec::new(); lod_count];
        let mut shadow: Vec<Vec<SkinInstance>> = vec![Vec::new(); lod_count];
        let mut first_slot = Vec::with_capacity(self.models.len());
        let mut acc = 0;
        for m in &self.models {
            first_slot.push(acc);
            acc += m.lods.len();
        }
        for &(model, matrix, centre) in &self.instances {
            let m = &self.models[model];
            let dist = centre.distance(camera.position);
            let Some(lod) = m.ends.iter().position(|&end| dist < end) else { continue };
            let inst = SkinInstance::new(matrix, 0);
            let r = Vec3::splat(m.radius);
            if frustum.intersects_aabb(centre - r, centre + r) {
                main[first_slot[model] + lod].push(inst);
            }
            // Like Godot: only the near versions cast shadows, and only inside the shadow distance
            if m.casts_shadow && lod < 2 && dist < shadow_distance + m.radius {
                shadow[first_slot[model] + lod].push(inst);
            }
        }
        let meshes: Vec<&GpuMesh> = self.models.iter().flat_map(|m| m.lods.iter()).collect();
        let to_batches = |lists: Vec<Vec<SkinInstance>>| {
            lists.into_iter().enumerate().filter(|(_, v)| !v.is_empty()).map(|(i, instances)| SkinnedBatch { mesh: meshes[i], instances }).collect()
        };
        (to_batches(main), to_batches(shadow))
    }
}

fn load_model(device: &wgpu::Device, pack: &pack::ModelPack, id: &str) -> Option<PropModel> {
    let Some(full) = pack.find(id) else {
        log::warn!("prop model {id} is not in the pack");
        return None;
    };
    let mut lods = vec![GpuMesh::new(device, id, &full.vertices)];
    for k in 1..=2 {
        if let Some(m) = pack.find(&format!("{id}_lod{k}")) {
            lods.push(GpuMesh::new(device, id, &m.vertices));
        }
    }
    let range = range_for(id);
    let ends = match lods.len() {
        3 => vec![TREE_LOD1, TREE_LOD2, range],
        2 => vec![BUSH_LOD1, range],
        _ => vec![range],
    };
    let radius = (Vec3::from(full.bounds_max) - Vec3::from(full.bounds_min)).length() * 0.5 * 1.4;
    let center_y = (full.bounds_min[1] + full.bounds_max[1]) * 0.5;
    Some(PropModel { lods, ends, casts_shadow: casts_shadow(id), radius, center_y })
}
