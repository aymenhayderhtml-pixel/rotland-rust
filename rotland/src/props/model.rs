//! One loaded prop model (its LOD meshes and distances), and the placed copy of it.

use engine::render::GpuMesh;
use glam::{Mat4, Vec3};

use super::lod::FADE;
use super::rules::{casts_shadow, is_ground, range_for};

const TREE_PERF_LOD1: f32 = 35.0; // new: PERF R34
const TREE_PERF_LOD2: f32 = 75.0; // new: PERF R34
/// Buildings stay visible to the fog line.
const BUILDING: f32 = 220.0; // godot: ScatterConfig.cs
/// Buildings switch to their simple model here (was the bush distance, 28 m, which popped up close).
const BUILDING_LOD1: f32 = 70.0; // new: PERF R30.5
const BUSH_LOD1: f32 = 28.0; // data: scatter.json visibility.bushLod1

pub struct PropModel {
    /// Full detail, then the simpler versions (if the model has them).
    pub(super) lods: Vec<GpuMesh>,
    /// Where each LOD stops being used (the last one is the visibility range).
    pub(super) ends: Vec<f32>,
    /// Squared maximum visibility range for fast early-out before sqrt.
    pub(super) max_range_sq: f32, // new: PERF R34
    /// Dither cross-fade band width (0.0 disables dither-fade so the fast non-discard pipeline is used).
    pub(super) fade_band: f32,
    pub(super) casts_shadow: bool,
    pub(super) is_small_prop: bool, // new: PERF R34
    pub(super) shadow_lod0_only: bool,
    pub(super) radius: f32,
    /// Height of the model's middle above its origin (origins sit on the ground).
    pub(super) center_y: f32,
}

/// One placed model: which model, its world matrix, and the centre of its bounds.
#[derive(Clone, Copy)]
pub struct Placed {
    pub model: usize,
    pub matrix: Mat4,
    pub centre: Vec3,
}

pub(super) fn load_model(device: &wgpu::Device, pack: &pack::ModelPack, id: &str, building: bool) -> Option<PropModel> {
    let full = pack.find(id)?;
    let mut lods = vec![GpuMesh::new(device, id, &full.vertices)];
    for k in 1..=2 {
        if let Some(m) = pack.find(&format!("{id}_lod{k}")) {
            lods.push(GpuMesh::new(device, id, &m.vertices));
        }
    }
    let range = if building { BUILDING } else { range_for(id) };
    let ends = match lods.len() {
        3 => vec![TREE_PERF_LOD1, TREE_PERF_LOD2, range],
        2 if building => vec![BUILDING_LOD1, range],
        2 => vec![BUSH_LOD1, range],
        _ => vec![range],
    };
    let max_range_sq = range * range;
    let is_small_prop = !building && lods.len() < 3;
    let fade_band = if !building && is_ground(id) { 0.0 } else { FADE }; // new: PERF R30.5
    let radius = (Vec3::from(full.bounds_max) - Vec3::from(full.bounds_min)).length() * 0.5 * 1.4;
    let center_y = (full.bounds_min[1] + full.bounds_max[1]) * 0.5;
    Some(PropModel {
        lods,
        ends,
        max_range_sq,
        fade_band,
        casts_shadow: building || casts_shadow(id),
        is_small_prop,
        shadow_lod0_only: false,
        radius,
        center_y,
    })
}

pub(super) fn load_tower_model(
    device: &wgpu::Device,
    pack: &pack::ModelPack,
    id: &str,
    lod0_distance: f32,
    range: f32,
) -> Option<PropModel> {
    let full = pack.find(id)?;
    let mut lods = vec![GpuMesh::new(device, id, &full.vertices)];
    if let Some(m) = pack.find(&format!("{id}_lod1")) {
        lods.push(GpuMesh::new(device, id, &m.vertices));
    }
    let ends = match lods.len() {
        2 => vec![lod0_distance, range],
        _ => vec![range],
    };
    let radius = (Vec3::from(full.bounds_max) - Vec3::from(full.bounds_min)).length() * 0.5 * 1.4;
    let center_y = (full.bounds_min[1] + full.bounds_max[1]) * 0.5;
    Some(PropModel {
        lods,
        ends,
        max_range_sq: range * range,
        fade_band: FADE,
        casts_shadow: true,
        is_small_prop: false,
        shadow_lod0_only: true,
        radius,
        center_y,
    })
}
