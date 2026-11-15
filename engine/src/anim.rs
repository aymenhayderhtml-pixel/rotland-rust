//! Skeletal animation on the CPU: sample a clip at a time and produce one skin matrix per joint.
//! Rigid skinning (each vertex follows one joint), so the GPU only does one matrix per vertex.

use glam::{Mat4, Quat, Vec3};
use pack::{ChannelKind, Clip, Interpolation, Skeleton};

/// A skeleton prepared for fast sampling: joints in parent-first order and inverse binds as matrices.
pub struct Rig {
    order: Vec<usize>,
    parents: Vec<Option<usize>>,
    inverse_bind: Vec<Mat4>,
    rest: Vec<(Vec3, Quat, Vec3)>,
    names: Vec<String>,
}

impl Rig {
    pub fn new(skeleton: &Skeleton) -> Self {
        let n = skeleton.joints.len();
        let parents: Vec<Option<usize>> = skeleton.joints.iter().map(|j| j.parent.map(|p| p as usize)).collect();
        // Parent-first order, whatever order the file lists them in
        let mut order = Vec::with_capacity(n);
        let mut placed = vec![false; n];
        while order.len() < n {
            let before = order.len();
            for i in 0..n {
                if !placed[i] && parents[i].is_none_or(|p| placed[p]) {
                    placed[i] = true;
                    order.push(i);
                }
            }
            assert!(order.len() > before, "skeleton has a parent loop");
        }
        Self {
            order,
            parents,
            inverse_bind: skeleton.joints.iter().map(|j| Mat4::from_cols_array(&j.inverse_bind)).collect(),
            rest: skeleton
                .joints
                .iter()
                .map(|j| (Vec3::from(j.rest_translation), Quat::from_array(j.rest_rotation).normalize(), Vec3::from(j.rest_scale)))
                .collect(),
            names: skeleton.joints.iter().map(|j| j.name.clone()).collect(),
        }
    }

    pub fn joint_count(&self) -> usize {
        self.inverse_bind.len()
    }

    /// Writes the skin matrices of `clip` at `time` (looped) into `out` (one per joint).
    pub fn sample(&self, clip: &Clip, time: f32, out: &mut [Mat4]) {
        let local = self.local_pose(clip, time);
        self.skin(&local, out);
    }

    /// Like `sample`, but mixes two clips: `weight` 0 = all `a`, 1 = all `b` (walk to run by speed, and so on).
    pub fn sample_blend(&self, a: &Clip, time_a: f32, b: &Clip, time_b: f32, weight: f32, out: &mut [Mat4]) {
        let w = weight.clamp(0.0, 1.0);
        if w <= 0.0 {
            return self.sample(a, time_a, out);
        }
        if w >= 1.0 {
            return self.sample(b, time_b, out);
        }
        let mut pa = self.local_pose(a, time_a);
        let pb = self.local_pose(b, time_b);
        for (x, y) in pa.iter_mut().zip(pb) {
            x.0 = x.0.lerp(y.0, w);
            x.1 = x.1.slerp(y.1, w);
            x.2 = x.2.lerp(y.2, w);
        }
        self.skin(&pa, out);
    }

    /// Each joint's own translation, rotation and scale for `clip` at `time` (looped).
    pub fn local_pose(&self, clip: &Clip, time: f32) -> Vec<(Vec3, Quat, Vec3)> {
        let n = self.joint_count();
        let t = if clip.duration > 0.0 { time.rem_euclid(clip.duration) } else { 0.0 };
        let mut local = self.rest.clone();
        for ch in &clip.channels {
            let j = ch.joint as usize;
            if j >= n {
                continue;
            }
            let v = sample_channel(ch.interpolation, ch.kind, &ch.times, &ch.values, t);
            match ch.kind {
                ChannelKind::Translation => local[j].0 = Vec3::new(v[0], v[1], v[2]),
                ChannelKind::Rotation => local[j].1 = Quat::from_array(v).normalize(),
                ChannelKind::Scale => local[j].2 = Vec3::new(v[0], v[1], v[2]),
            }
        }
        local
    }

    /// Turns a local pose into skin matrices (parents first, then the inverse bind).
    pub fn skin(&self, local: &[(Vec3, Quat, Vec3)], out: &mut [Mat4]) {
        let n = self.joint_count();
        let mut global = [Mat4::IDENTITY; 256];
        for &j in &self.order {
            let (tr, rot, sc) = local[j];
            let m = Mat4::from_scale_rotation_translation(sc, rot, tr);
            global[j] = match self.parents[j] {
                Some(p) => global[p] * m,
                None => m,
            };
        }
        for j in 0..n.min(out.len()) {
            out[j] = global[j] * self.inverse_bind[j];
        }
    }

    /// World-space matrix of joint `name` in the pose (for things held in a hand: weapons, the flashlight).
    pub fn joint_index(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|n| n == name)
    }

    /// Model-space transform of joint `j` given skin matrices from `sample`: skin * bind.
    pub fn joint_model_matrix(&self, skin: &[Mat4], j: usize) -> Mat4 {
        skin[j] * self.inverse_bind[j].inverse()
    }
}

fn sample_channel(interp: Interpolation, kind: ChannelKind, times: &[f32], values: &[[f32; 4]], t: f32) -> [f32; 4] {
    if times.is_empty() {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let k = times.partition_point(|&x| x <= t);
    if k == 0 {
        return values[0];
    }
    if k >= times.len() || interp == Interpolation::Step {
        return values[k - 1];
    }
    let (t0, t1) = (times[k - 1], times[k]);
    let f = if t1 > t0 { (t - t0) / (t1 - t0) } else { 0.0 };
    let (a, b) = (values[k - 1], values[k]);
    if kind == ChannelKind::Rotation {
        return Quat::from_array(a).slerp(Quat::from_array(b), f).to_array();
    }
    [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f, a[2] + (b[2] - a[2]) * f, 0.0]
}

#[cfg(test)]
mod tests {
    use super::*;
    use pack::{Channel, Joint};

    fn two_joint_skeleton() -> Skeleton {
        let joint = |name: &str, parent: Option<u16>, y: f32| Joint {
            name: name.into(),
            parent,
            inverse_bind: Mat4::from_translation(Vec3::new(0.0, -y, 0.0)).to_cols_array(),
            rest_translation: [0.0, if parent.is_some() { 1.0 } else { 0.0 }, 0.0],
            rest_rotation: [0.0, 0.0, 0.0, 1.0],
            rest_scale: [1.0, 1.0, 1.0],
        };
        // Child listed first on purpose: the rig must still evaluate parents first
        Skeleton { joints: vec![joint("child", Some(1), 1.0), joint("root", None, 0.0)] }
    }

    #[test]
    fn rest_pose_gives_identity_skin() {
        let rig = Rig::new(&two_joint_skeleton());
        let clip = Clip { name: "none".into(), duration: 1.0, channels: vec![] };
        let mut out = [Mat4::ZERO; 2];
        rig.sample(&clip, 0.3, &mut out);
        for m in out {
            assert!(m.abs_diff_eq(Mat4::IDENTITY, 1e-5));
        }
    }

    #[test]
    fn blending_half_way_lands_in_the_middle() {
        let rig = Rig::new(&two_joint_skeleton());
        let lift = |y: f32| Clip {
            name: "lift".into(),
            duration: 1.0,
            channels: vec![Channel {
                joint: 1,
                kind: ChannelKind::Translation,
                interpolation: Interpolation::Step,
                times: vec![0.0],
                values: vec![[0.0, y, 0.0, 0.0]],
            }],
        };
        let mut out = [Mat4::ZERO; 2];
        rig.sample_blend(&lift(0.0), 0.0, &lift(2.0), 0.0, 0.5, &mut out);
        let p = out[0].transform_point3(Vec3::new(0.0, 1.0, 0.0));
        assert!((p.y - 2.0).abs() < 1e-5, "half of a 2 m lift is 1 m, got {}", p.y - 1.0);
        assert_eq!(rig.joint_index("child"), Some(0));
    }

    #[test]
    fn moving_the_root_moves_the_child() {
        let rig = Rig::new(&two_joint_skeleton());
        let clip = Clip {
            name: "lift".into(),
            duration: 2.0,
            channels: vec![Channel {
                joint: 1,
                kind: ChannelKind::Translation,
                interpolation: Interpolation::Linear,
                times: vec![0.0, 2.0],
                values: vec![[0.0, 0.0, 0.0, 0.0], [0.0, 2.0, 0.0, 0.0]],
            }],
        };
        let mut out = [Mat4::ZERO; 2];
        rig.sample(&clip, 1.0, &mut out);
        let p = out[0].transform_point3(Vec3::new(0.0, 1.0, 0.0));
        assert!((p.y - 2.0).abs() < 1e-5, "child vertex at bind height 1 should rise by 1, got {}", p.y);
    }
}
