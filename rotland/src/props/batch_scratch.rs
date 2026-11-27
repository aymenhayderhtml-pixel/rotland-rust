//! Reusable instance buffers for the per-frame batches, so the frame does not allocate them again.

use std::cell::RefCell;
use std::collections::VecDeque;

use engine::render::{SkinInstance, SkinnedBatch};

const MAX_POOLED_BATCH_VECS: usize = 256; // new: PERF R30.5
const LARGE_BATCH_THRESHOLD: usize = 64; // new: PERF R30.5

#[derive(Default)]
pub(super) struct BatchScratch {
    pub(super) main: Vec<Vec<SkinInstance>>,
    pub(super) shadow: Vec<Vec<SkinInstance>>,
    pub(super) last_main_len: Vec<usize>,
    pub(super) last_shadow_len: Vec<usize>,
    pub(super) pool: VecDeque<Vec<SkinInstance>>,
}

impl BatchScratch {
    pub(super) fn take_vec(&mut self, last_len: usize) -> Vec<SkinInstance> {
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

    pub(super) fn recycle_vec(&mut self, mut v: Vec<SkinInstance>) {
        if v.capacity() > 0 && self.pool.len() < MAX_POOLED_BATCH_VECS {
            v.clear();
            self.pool.push_back(v);
        }
    }
}

thread_local! {
    pub(super) static BATCH_SCRATCH: RefCell<BatchScratch> = RefCell::new(BatchScratch::default());
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

#[cfg(test)]
mod tests {
    use super::*;

    use engine::render::GpuMesh;
    use glam::Mat4;

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
