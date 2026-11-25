//! Placed models (trees, rocks, bushes, grass, buildings) drawn as instances, with Godot's distance LODs
//! and shadow rules (ScatterConfig.RangeFor, ChunkProps.ShouldCastShadow). Models load from the pack on
//! first use. Static instances use joint 0 of the bone list, which is always the identity.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

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

/// Cross-fade band width (10 m) around each LOD switch distance.
pub const FADE: f32 = 10.0;

/// Return type of `lod_fade`: up to 2 active LOD levels with their dither fade factors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LodFades {
    items: [(usize, f32, bool); 2],
    len: usize,
}

impl LodFades {
    fn empty() -> Self {
        Self { items: [(0, 0.0, false); 2], len: 0 }
    }

    fn one(lod: usize, fade: f32, complementary: bool) -> Self {
        Self { items: [(lod, fade, complementary), (0, 0.0, false)], len: 1 }
    }

    fn two(first: (usize, f32, bool), second: (usize, f32, bool)) -> Self {
        Self { items: [first, second], len: 2 }
    }

    pub fn as_slice(&self) -> &[(usize, f32, bool)] {
        &self.items[..self.len]
    }
}

impl std::ops::Deref for LodFades {
    type Target = [(usize, f32, bool)];
    fn deref(&self) -> &Self::Target {
        &self.items[..self.len]
    }
}

impl<'a> IntoIterator for &'a LodFades {
    type Item = &'a (usize, f32, bool);
    type IntoIter = std::slice::Iter<'a, (usize, f32, bool)>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl IntoIterator for LodFades {
    type Item = (usize, f32, bool);
    type IntoIter = std::iter::Take<std::array::IntoIter<(usize, f32, bool), 2>>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter().take(self.len)
    }
}

/// Pure helper: returns the active LOD(s) and dither fade values at `dist`.
/// Outside fade bands: exactly one LOD at fade 1.0.
/// In a fade band around an LOD boundary: two LODs whose fades sum to 1.0.
/// Ends of the band match the old hard switch.
pub fn lod_fade(dist: f32, ends: &[f32], band: f32) -> LodFades {
    if ends.is_empty() {
        return LodFades::empty();
    }
    let last_end = ends[ends.len() - 1];
    if dist >= last_end {
        return LodFades::empty();
    }
    let half = band * 0.5;
    for (i, &switch_dist) in ends.iter().enumerate().take(ends.len().saturating_sub(1)) {
        let start = switch_dist - half;
        let end = switch_dist + half;
        if dist < start {
            return LodFades::one(i, 1.0, false);
        }
        if dist <= end {
            let t = if band > 0.0 { ((dist - start) / band).clamp(0.0, 1.0) } else { 1.0 };
            if t <= 0.0 {
                return LodFades::one(i, 1.0, false);
            }
            if t >= 1.0 {
                return LodFades::one(i + 1, 1.0, false);
            }
            return LodFades::two((i, 1.0 - t, false), (i + 1, t, true));
        }
    }
    LodFades::one(ends.len() - 1, 1.0, false)
}

pub struct PropModel {
    /// Full detail, then the simpler versions (if the model has them).
    lods: Vec<GpuMesh>,
    /// Where each LOD stops being used (the last one is the visibility range).
    ends: Vec<f32>,
    /// Dither cross-fade band width (0.0 disables dither-fade so the fast non-discard pipeline is used).
    fade_band: f32,
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

fn is_ground(id: &str) -> bool {
    ["grass_", "flowers_", "fern", "mushrooms"].iter().any(|p| id.starts_with(p))
}

fn range_for(id: &str) -> f32 {
    let starts = |p: &str| id.starts_with(p);
    if starts("tree_") || starts("pine_") {
        TREE
    } else if starts("rock_") {
        ROCK
    } else if starts("bush_") {
        BUSH
    } else if is_ground(id) {
        GROUND
    } else {
        PROP
    }
}

fn casts_shadow(id: &str) -> bool {
    !is_ground(id) && !id.starts_with("rock_") && !id.starts_with("bush_")
}

const MAX_POOLED_BATCH_VECS: usize = 256; // new: PERF R30.5
const LARGE_BATCH_THRESHOLD: usize = 64; // new: PERF R30.5

#[derive(Default)]
struct BatchScratch {
    main: Vec<Vec<SkinInstance>>,
    shadow: Vec<Vec<SkinInstance>>,
    last_main_len: Vec<usize>,
    last_shadow_len: Vec<usize>,
    pool: VecDeque<Vec<SkinInstance>>,
}

impl BatchScratch {
    fn take_vec(&mut self, last_len: usize) -> Vec<SkinInstance> {
        let mut v = if last_len >= LARGE_BATCH_THRESHOLD {
            self.pool.pop_back()
        } else {
            self.pool.pop_front()
        }
        .unwrap_or_default();
        v.clear();
        let target_cap = last_len.max(4); // new: PERF R30.5
        if v.capacity() < target_cap {
            v.reserve(target_cap - v.capacity());
        }
        v
    }

    fn recycle_vec(&mut self, mut v: Vec<SkinInstance>) {
        if v.capacity() > 0 && self.pool.len() < MAX_POOLED_BATCH_VECS {
            v.clear();
            self.pool.push_back(v);
        }
    }
}

thread_local! {
    static BATCH_SCRATCH: RefCell<BatchScratch> = RefCell::new(BatchScratch::default());
}

/// Returns `SkinnedBatch` instance buffers to the thread-local pool for reuse next frame.
pub fn recycle_batches(main: Vec<SkinnedBatch<'_>>, shadow: Vec<SkinnedBatch<'_>>) {
    BATCH_SCRATCH.with(|cell| {
        let mut s = cell.borrow_mut();
        for b in main {
            s.recycle_vec(b.instances);
        }
        for b in shadow {
            s.recycle_vec(b.instances);
        }
        s.pool.make_contiguous().sort_unstable_by_key(|v| v.capacity());
    });
}

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
        let slot = load_model(device, pack, id, building).map(|m| {
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
                let dist = p.centre.distance(camera.position);
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
                // Like Godot: only the near versions cast shadows, and only inside the shadow distance.
                // During fade, only the nearer LOD casts the shadow (no double shadows).
                let nearer_lod = fades[0].0;
                if m.casts_shadow && nearer_lod < 2 && dist < shadow_distance + m.radius {
                    let slot = base_slot + nearer_lod;
                    if scratch.shadow[slot].is_empty() && scratch.shadow[slot].capacity() == 0 {
                        let last_len = scratch.last_shadow_len[slot];
                        let v = scratch.take_vec(last_len);
                        scratch.shadow[slot] = v;
                    }
                    scratch.shadow[slot].push(SkinInstance::new(p.matrix, 0));
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
    let fade_band = if !building && is_ground(id) { 0.0 } else { FADE }; // new: PERF R30.5
    let radius = (Vec3::from(full.bounds_max) - Vec3::from(full.bounds_min)).length() * 0.5 * 1.4;
    let center_y = (full.bounds_min[1] + full.bounds_max[1]) * 0.5;
    Some(PropModel { lods, ends, fade_band, casts_shadow: building || casts_shadow(id), radius, center_y })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_fade_outside_bands() {
        let ends = [70.0, 220.0];
        // Closer than the first fade band (65..75 m)
        let near = lod_fade(30.0, &ends, FADE);
        assert_eq!(near.len(), 1);
        assert_eq!(near[0], (0, 1.0, false));
        assert_eq!(near.as_slice(), &[(0, 1.0, false)]);

        // Farther than the fade band (65..75 m), before max range
        let far = lod_fade(120.0, &ends, FADE);
        assert_eq!(far.len(), 1);
        assert_eq!(far[0], (1, 1.0, false));

        // Beyond max range
        let beyond = lod_fade(220.0, &ends, FADE);
        assert!(beyond.is_empty());
        let way_beyond = lod_fade(300.0, &ends, FADE);
        assert!(way_beyond.is_empty());
    }

    #[test]
    fn test_lod_fade_in_band_sums_to_one() {
        let ends = [70.0, 220.0];
        // Sample points across the 65..75 m band
        for step in 1..10 {
            let dist = 65.0 + step as f32 * 1.0;
            let res = lod_fade(dist, &ends, FADE);
            assert_eq!(res.len(), 2, "dist {dist} should have 2 LODs in fade band");
            let (lod0, fade0, comp0) = res[0];
            let (lod1, fade1, comp1) = res[1];
            assert_eq!(lod0, 0);
            assert_eq!(lod1, 1);
            assert!(!comp0, "outgoing LOD should not be complementary");
            assert!(comp1, "incoming LOD should be complementary");
            assert!((fade0 + fade1 - 1.0).abs() < 1e-6, "fades must sum to 1.0 at {dist}");
        }

        // At midpoint 70.0 m, fades must be 0.5 each
        let mid = lod_fade(70.0, &ends, FADE);
        assert_eq!(mid.len(), 2);
        assert_eq!(mid[0], (0, 0.5, false));
        assert_eq!(mid[1], (1, 0.5, true));
    }

    #[test]
    fn test_lod_fade_ends_match_old_hard_switch() {
        let ends = [70.0, 220.0];
        // Lower end of band: 65.0 m matches old switch (LOD 0 at 1.0)
        let lower = lod_fade(65.0, &ends, FADE);
        assert_eq!(lower.len(), 1);
        assert_eq!(lower[0], (0, 1.0, false));

        // Upper end of band: 75.0 m matches old switch (LOD 1 at 1.0)
        let upper = lod_fade(75.0, &ends, FADE);
        assert_eq!(upper.len(), 1);
        assert_eq!(upper[0], (1, 1.0, false));
    }

    #[test]
    fn test_lod_fade_multi_lod_tree() {
        let ends = [TREE_LOD1, TREE_LOD2, TREE]; // [45.0, 95.0, 170.0]
        // Below band 0 (40..50)
        assert_eq!(&lod_fade(20.0, &ends, FADE)[..], &[(0, 1.0, false)]);
        // In band 0 (45.0)
        let b0 = lod_fade(45.0, &ends, FADE);
        assert_eq!(b0.len(), 2);
        assert_eq!(b0[0], (0, 0.5, false));
        assert_eq!(b0[1], (1, 0.5, true));
        // Between band 0 and band 1 (50..90)
        assert_eq!(&lod_fade(70.0, &ends, FADE)[..], &[(1, 1.0, false)]);
        // In band 1 (95.0)
        let b1 = lod_fade(95.0, &ends, FADE);
        assert_eq!(b1.len(), 2);
        assert_eq!(b1[0], (1, 0.5, false));
        assert_eq!(b1[1], (2, 0.5, true));
        // Past band 1 (100..170)
        assert_eq!(&lod_fade(120.0, &ends, FADE)[..], &[(2, 1.0, false)]);
        // Beyond visibility
        assert!(lod_fade(175.0, &ends, FADE).is_empty());
    }

    #[test]
    fn test_lod_fade_single_lod() {
        let ends = [80.0];
        assert_eq!(&lod_fade(40.0, &ends, FADE)[..], &[(0, 1.0, false)]);
        assert!(lod_fade(85.0, &ends, FADE).is_empty());
    }

    #[test]
    fn test_lod_fade_empty_ends() {
        assert!(lod_fade(10.0, &[], FADE).is_empty());
    }

    #[test]
    fn test_lod_batch_splitting() {
        use engine::render::{split_batches, split_instances};

        let dummy = std::mem::MaybeUninit::<GpuMesh>::uninit();
        let mesh_ptr: &'static GpuMesh = unsafe { &*dummy.as_ptr() };

        let s1 = SkinInstance::new(Mat4::IDENTITY, 0);
        let s2 = SkinInstance::with_fade(Mat4::IDENTITY, 0, 1.0, false);
        let f1 = SkinInstance::with_fade(Mat4::IDENTITY, 0, 0.7, false);
        let f2 = SkinInstance::with_fade(Mat4::IDENTITY, 0, 0.3, true);

        assert!(!s1.is_fading());
        assert!(!s2.is_fading());
        assert!(f1.is_fading());
        assert!(f2.is_fading());

        let (normal, fading) = split_instances(&[s1, f1, s2, f2]);
        assert_eq!(normal.len(), 2);
        assert_eq!(fading.len(), 2);
        assert_eq!(normal[0].fade(), (1.0, false));
        assert_eq!(normal[1].fade(), (1.0, false));
        assert_eq!(fading[0].fade(), (0.7, false));
        assert_eq!(fading[1].fade(), (0.3, true));

        let batch = SkinnedBatch { mesh: mesh_ptr, instances: vec![s1, f1, s2, f2] };
        let (n_batches, f_batches) = split_batches(&[batch]);
        assert_eq!(n_batches.len(), 1);
        assert_eq!(f_batches.len(), 1);
        assert_eq!(n_batches[0].instances.len(), 2);
        assert_eq!(f_batches[0].instances.len(), 2);
    }

    #[test]
    fn test_lod_fade_zero_band_never_fades() {
        let ends = [28.0, 65.0];
        for d in [0.0, 23.0, 28.0, 33.0, 64.9] {
            let res = lod_fade(d, &ends, 0.0);
            assert_eq!(res.len(), 1);
            let (_, fade, comp) = res[0];
            assert_eq!(fade, 1.0);
            assert!(!comp);
        }
        assert!(lod_fade(65.0, &ends, 0.0).is_empty());
    }

    #[test]
    fn test_ground_and_shadow_rules() {
        assert!(is_ground("grass_tuft_a"));
        assert!(is_ground("flowers_white"));
        assert!(is_ground("fern"));
        assert!(is_ground("mushrooms"));
        assert!(!is_ground("bush_01"));
        assert!(!is_ground("tree_pine_01"));

        assert!(!casts_shadow("grass_tuft_a"));
        assert!(!casts_shadow("flowers_yellow"));
        assert!(!casts_shadow("fern"));
        assert!(!casts_shadow("mushrooms"));
        assert!(!casts_shadow("rock_01"));
        assert!(!casts_shadow("bush_01"));
        assert!(casts_shadow("tree_pine_01"));
    }

    #[test]
    fn test_batch_scratch_reuses_capacity() {
        let mut scratch = BatchScratch::default();
        let mut big = Vec::with_capacity(128);
        big.push(SkinInstance::new(Mat4::IDENTITY, 0));
        scratch.recycle_vec(big);
        let small = Vec::with_capacity(8);
        scratch.recycle_vec(small);
        scratch.pool.make_contiguous().sort_unstable_by_key(|v| v.capacity());

        let reused_big = scratch.take_vec(100);
        assert!(reused_big.is_empty());
        assert!(reused_big.capacity() >= 128);

        let reused_small = scratch.take_vec(2);
        assert!(reused_small.is_empty());
        assert!(reused_small.capacity() >= 8);
    }
}
