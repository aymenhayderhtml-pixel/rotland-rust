//! The model library: loads models from the pack once, places them, and builds each frame's batches
//! for the camera and the sun shadow.

use std::collections::HashMap;

use engine::render::frustum::Frustum;
use engine::render::{Camera, SkinInstance, SkinnedBatch};
use glam::{Mat4, Vec3};

use super::batch_scratch::BATCH_SCRATCH;
use super::lod::{lod_fade, shadow_lod_for_rule};
use super::model::{load_model, load_tower_model, Placed, PropModel};

const SMALL_PROP_SHADOW_DIST: f32 = 35.0; // new: PERF R34

/// Every model used so far, by name.
#[derive(Default)]
pub struct PropLibrary {
    models: Vec<PropModel>,
    index: HashMap<String, Option<usize>>,
    /// Starting flat LOD index for each model in `models`.
    first_slot: Vec<usize>,
    /// Flat LOD index -> (model index, lod index).
    lod_map: Vec<(usize, usize)>,
}

impl PropLibrary {
    /// The model's slot, loading it from the pack the first time. None if the pack does not have it.
    pub fn get(&mut self, device: &wgpu::Device, pack: &pack::ModelPack, id: &str, building: bool) -> Option<usize> {
        if let Some(slot) = self.index.get(id) {
            return *slot;
        }
        let model = load_model(device, pack, id, building);
        self.insert_model(id, model)
    }

    /// Loads a tower part model with a custom LOD0 switch distance and visibility range.
    /// Only LOD0 casts a sun shadow.
    pub fn get_tower(
        &mut self,
        device: &wgpu::Device,
        pack: &pack::ModelPack,
        id: &str,
        lod0_distance: f32,
        range: f32,
    ) -> Option<usize> {
        if let Some(slot) = self.index.get(id) {
            return *slot;
        }
        let model = load_tower_model(device, pack, id, lod0_distance, range);
        self.insert_model(id, model)
    }

    fn insert_model(&mut self, id: &str, model: Option<PropModel>) -> Option<usize> {
        let slot = model.map(|m| {
            let model_idx = self.models.len();
            let start = self.lod_map.len();
            self.first_slot.push(start);
            for lod_idx in 0..m.lods.len() {
                self.lod_map.push((model_idx, lod_idx));
            }
            self.models.push(m);
            model_idx
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
        sun_frustum: Option<&Frustum>,
    ) -> (Vec<SkinnedBatch<'a>>, Vec<SkinnedBatch<'a>>) {
        let frustum = Frustum::from_view_proj(&camera.view_proj(aspect));
        let lod_count = self.lod_map.len();
        BATCH_SCRATCH.with(|cell| {
            let mut scratch = cell.borrow_mut();
            let scratch = &mut *scratch;
            scratch.main.resize_with(lod_count, Vec::new);
            scratch.shadow.resize_with(lod_count, Vec::new);
            scratch.last_main_len.resize(lod_count, 0);
            scratch.last_shadow_len.resize(lod_count, 0);
            for v in &mut scratch.main {
                v.clear();
            }
            for v in &mut scratch.shadow {
                v.clear();
            }

            for p in placed {
                let m = &self.models[p.model];
                let dist_sq = p.centre.distance_squared(camera.position);
                if dist_sq >= m.max_range_sq {
                    continue;
                }
                let dist = dist_sq.sqrt();
                let fades = lod_fade(dist, &m.ends, m.fade_band);
                if fades.is_empty() {
                    continue;
                }
                let base_slot = self.first_slot[p.model];
                let r = Vec3::splat(m.radius);
                if frustum.intersects_aabb(p.centre - r, p.centre + r) {
                    for &(lod, fade, complementary) in &fades {
                        let slot = base_slot + lod;
                        if scratch.main[slot].is_empty() && scratch.main[slot].capacity() == 0 {
                            let last_len = scratch.last_main_len[slot];
                            let v = scratch.take_vec(last_len);
                            scratch.main[slot] = v;
                        }
                        scratch.main[slot].push(SkinInstance::with_fade(p.matrix, 0, fade, complementary));
                    }
                }
                // Only cast shadows inside the sun's orthographic box and within shadow distance.
                // Small props stop casting beyond SMALL_PROP_SHADOW_DIST; trees use LOD1 beyond 15 m.
                if let Some(sun) = sun_frustum {
                    let max_shadow = if m.is_small_prop {
                        shadow_distance.min(SMALL_PROP_SHADOW_DIST)
                    } else {
                        shadow_distance
                    };
                    if m.casts_shadow
                        && dist < max_shadow + m.radius
                        && sun.intersects_aabb(p.centre - r, p.centre + r)
                        && let Some(shadow_lod) = shadow_lod_for_rule(fades[0].0, m.lods.len(), dist, m.shadow_lod0_only)
                    {
                        let slot = base_slot + shadow_lod;
                        if scratch.shadow[slot].is_empty() && scratch.shadow[slot].capacity() == 0 {
                            let last_len = scratch.last_shadow_len[slot];
                            let v = scratch.take_vec(last_len);
                            scratch.shadow[slot] = v;
                        }
                        scratch.shadow[slot].push(SkinInstance::new(p.matrix, 0));
                    }
                }
            }

            let main_count = scratch.main.iter().filter(|v| !v.is_empty()).count();
            let shadow_count = scratch.shadow.iter().filter(|v| !v.is_empty()).count();
            let mut main_batches = Vec::with_capacity(main_count);
            let mut shadow_batches = Vec::with_capacity(shadow_count);

            for (i, &(model_idx, lod_idx)) in self.lod_map.iter().enumerate() {
                let mesh = &self.models[model_idx].lods[lod_idx];
                if !scratch.main[i].is_empty() {
                    let instances = std::mem::take(&mut scratch.main[i]);
                    scratch.last_main_len[i] = instances.len();
                    main_batches.push(SkinnedBatch { mesh, instances });
                } else {
                    scratch.last_main_len[i] = 0;
                    if scratch.main[i].capacity() > 0 {
                        let v = std::mem::take(&mut scratch.main[i]);
                        scratch.recycle_vec(v);
                    }
                }
                if !scratch.shadow[i].is_empty() {
                    let instances = std::mem::take(&mut scratch.shadow[i]);
                    scratch.last_shadow_len[i] = instances.len();
                    shadow_batches.push(SkinnedBatch { mesh, instances });
                } else {
                    scratch.last_shadow_len[i] = 0;
                    if scratch.shadow[i].capacity() > 0 {
                        let v = std::mem::take(&mut scratch.shadow[i]);
                        scratch.recycle_vec(v);
                    }
                }
            }

            (main_batches, shadow_batches)
        })
    }
}
