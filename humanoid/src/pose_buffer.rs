//! Reusable pose buffers and masked override / additive pose blending math.

use glam::{Quat, Vec3};

/// Local joint transform `(translation, rotation, scale)`.
pub type LocalTransform = (Vec3, Quat, Vec3);

const IDENTITY_TRANSFORM: LocalTransform = (Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);

/// Fixed pool of reusable local-pose buffers to avoid per-frame allocations.
#[derive(Clone, Debug)]
pub struct PosePool {
    buffers: Vec<Vec<LocalTransform>>,
    joint_count: usize,
}

impl PosePool {
    /// Allocates `buffers` (at least 1) pose buffers sized for `joint_count` joints.
    pub fn new(joint_count: usize, buffers: usize) -> Self {
        let count = buffers.max(1); // new: DECISIONS 2026-11-23
        Self {
            buffers: (0..count).map(|_| vec![IDENTITY_TRANSFORM; joint_count]).collect(),
            joint_count,
        }
    }

    /// Resizes all buffers when the target skeleton's joint count changes.
    pub fn resize(&mut self, joint_count: usize) {
        if self.joint_count == joint_count {
            return;
        }
        for buf in &mut self.buffers {
            buf.resize(joint_count, IDENTITY_TRANSFORM);
        }
        self.joint_count = joint_count;
    }

    /// Returns a mutable slice to buffer `index`.
    pub fn get_mut(&mut self, index: usize) -> &mut [LocalTransform] {
        &mut self.buffers[index]
    }

    /// Returns two disjoint mutable pose slices `(first, second)`.
    pub fn pair_mut(
        &mut self,
        first: usize,
        second: usize,
    ) -> (&mut [LocalTransform], &mut [LocalTransform]) {
        assert_ne!(first, second, "PosePool::pair_mut requires distinct buffer indices");
        if first < second {
            let (left, right) = self.buffers.split_at_mut(second);
            (&mut left[first], &mut right[0])
        } else {
            let (left, right) = self.buffers.split_at_mut(first);
            (&mut right[0], &mut left[second])
        }
    }

    /// Returns three disjoint mutable pose slices for multi-buffer blending.
    pub fn triple_mut(
        &mut self,
    ) -> (&mut [LocalTransform], &mut [LocalTransform], &mut [LocalTransform]) {
        let [a, b, c, ..] = self.buffers.as_mut_slice() else {
            panic!("PosePool::triple_mut requires at least 3 buffers");
        };
        (a.as_mut_slice(), b.as_mut_slice(), c.as_mut_slice())
    }
}

/// Blends `over` onto `base` with `weight * mask[i]`, taking the shortest quaternion arc.
pub fn blend_pose_masked(
    base: &mut [LocalTransform],
    over: &[LocalTransform],
    weight: f32,
    mask: &[f32],
) {
    let w = weight.clamp(0.0, 1.0); // new: DECISIONS 2026-11-23
    if w <= 0.0 {
        return;
    }
    let n = base.len().min(over.len()).min(mask.len());
    for i in 0..n {
        let mw = (w * mask[i]).clamp(0.0, 1.0);
        if mw <= 0.0 {
            continue;
        }
        if mw >= 1.0 {
            base[i].0 = over[i].0;
            base[i].1 = over[i].1.normalize();
            base[i].2 = over[i].2;
        } else {
            let a = base[i].1.normalize();
            let mut b = over[i].1.normalize();
            if a.dot(b) < 0.0 {
                b = -b;
            }
            base[i].0 = base[i].0.lerp(over[i].0, mw);
            base[i].1 = a.slerp(b, mw).normalize();
            base[i].2 = base[i].2.lerp(over[i].2, mw);
        }
    }
}

/// Adds the difference `(delta - rest)` onto `base` scaled by `weight * mask[i]`.
pub fn add_pose_masked(
    base: &mut [LocalTransform],
    delta: &[LocalTransform],
    rest: &[LocalTransform],
    weight: f32,
    mask: &[f32],
) {
    let w = weight.clamp(0.0, 1.0); // new: DECISIONS 2026-11-23
    if w <= 0.0 {
        return;
    }
    let n = base.len().min(delta.len()).min(rest.len()).min(mask.len());
    for i in 0..n {
        let mw = (w * mask[i]).clamp(0.0, 1.0);
        if mw <= 0.0 {
            continue;
        }
        base[i].0 += (delta[i].0 - rest[i].0) * mw;
        let rest_q = rest[i].1.normalize();
        let delta_q = delta[i].1.normalize();
        let mut dq = rest_q.conjugate() * delta_q;
        if dq.w < 0.0 {
            dq = -dq;
        }
        let step_q = Quat::IDENTITY.slerp(dq.normalize(), mw);
        base[i].1 = (base[i].1.normalize() * step_q).normalize();
        base[i].2 += (delta[i].2 - rest[i].2) * mw;
    }
}
