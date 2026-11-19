//! Two-bone analytical inverse kinematics for humanoid limbs.
//!
//! Solves the left arm chain (`upper_arm_L` -> `forearm_L` -> `hand_L`)
//! so that `hand_L` meets the off-hand grip point on a two-handed weapon,
//! with the elbow bending down and outward.

use engine::anim::Rig;
use glam::{Mat3, Mat4, Quat, Vec3};

use crate::weapon::weapon_hand_offset;

/// Direction vector for the left elbow pole (bending down and outward to the left).
pub const LEFT_ELBOW_POLE: Vec3 = Vec3::new(-0.6, -0.8, 0.0);

/// Solves 2-bone analytical IK for a 2-segment limb (shoulder -> elbow -> wrist).
///
/// - `shoulder`: model-space position of the shoulder joint (`upper_arm_L`)
/// - `target`: desired model-space position of the end effector (`hand_L`)
/// - `l1`: length of upper segment (upper arm)
/// - `l2`: length of lower segment (forearm)
/// - `pole_dir`: direction hint for elbow bend (e.g. down and outward)
///
/// Returns the model-space position of the middle joint (elbow).
pub fn solve_two_bone_ik(
    shoulder: Vec3,
    target: Vec3,
    l1: f32,
    l2: f32,
    pole_dir: Vec3,
) -> Vec3 {
    let to_target = target - shoulder;
    let dist = to_target.length();
    if dist < 1e-4 {
        return shoulder + pole_dir.normalize_or_zero() * l1;
    }

    let min_dist = (l1 - l2).abs() + 1e-3;
    let max_dist = l1 + l2 - 1e-3;
    let d = dist.clamp(min_dist, max_dist);
    let target_dir = to_target / dist;

    // Law of cosines for angle at shoulder between target vector and upper arm:
    // l2^2 = l1^2 + d^2 - 2 * l1 * d * cos(alpha)
    let cos_alpha = ((l1 * l1 + d * d - l2 * l2) / (2.0 * l1 * d)).clamp(-1.0, 1.0);
    let sin_alpha = (1.0 - cos_alpha * cos_alpha).max(0.0).sqrt();

    // Construct bend plane using target_dir and pole_dir
    let mut normal = target_dir.cross(pole_dir);
    if normal.length_squared() < 1e-5 {
        normal = target_dir.cross(Vec3::Y);
        if normal.length_squared() < 1e-5 {
            normal = target_dir.cross(Vec3::X);
        }
    }
    let normal = normal.normalize();

    // In-plane bend vector perpendicular to target_dir
    let mut bend_dir = normal.cross(target_dir).normalize();
    if bend_dir.dot(pole_dir) < 0.0 {
        bend_dir = -bend_dir;
    }

    shoulder + (target_dir * cos_alpha + bend_dir * sin_alpha) * l1
}

/// Constructs a rotation quaternion aligning local +Y with `dir` and local +Z with `normal`.
pub fn bone_rotation_from_dir(dir: Vec3, normal: Vec3) -> Quat {
    let y_axis = dir.normalize();
    let mut z_axis = normal;
    if z_axis.length_squared() < 1e-5 {
        z_axis = Vec3::Z;
    }
    let z_axis = z_axis.normalize();
    let x_axis = y_axis.cross(z_axis).normalize();
    let z_axis = x_axis.cross(y_axis).normalize();
    let mat = Mat3::from_cols(x_axis, y_axis, z_axis);
    Quat::from_mat3(&mat).normalize()
}

/// Computes parent-first topological order for evaluating skeleton transforms.
pub fn parent_first_order(parents: &[Option<usize>]) -> Vec<usize> {
    let n = parents.len();
    let mut order = Vec::with_capacity(n);
    let mut placed = vec![false; n];
    while order.len() < n {
        let before = order.len();
        for i in 0..n {
            if !placed[i] && parents[i].is_none_or(|p| p < n && placed[p]) {
                placed[i] = true;
                order.push(i);
            }
        }
        if order.len() == before {
            order.extend((0..n).filter(|&i| !placed[i]));
            break;
        }
    }
    order
}

/// Computes model-space forward kinematics matrices for joints given local transforms.
pub fn forward_kinematics(
    parents: &[Option<usize>],
    order: &[usize],
    local: &[(Vec3, Quat, Vec3)],
    out: &mut [Mat4],
) {
    let n = parents.len().min(out.len());
    for &j in order {
        if j >= n {
            continue;
        }
        let (tr, rot, sc) = local[j];
        let m = Mat4::from_scale_rotation_translation(sc, rot, tr);
        out[j] = match parents[j] {
            Some(p) if p < n => out[p] * m,
            _ => m,
        };
    }
}

/// Solves 2-bone IK for the left arm so `hand_L` grips the weapon's off-hand grip point.
pub fn apply_two_hand_left_grip_ik(
    parents: &[Option<usize>],
    order: &[usize],
    local: &mut [(Vec3, Quat, Vec3)],
    rig: &Rig,
    hold_pos: Vec3,
    hold_rot: Vec3,
    off_hand_grip: Vec3,
) {
    let Some(hr_idx) = rig.joint_index("hand_R") else { return };
    let Some(sh_l_idx) = rig.joint_index("shoulder_L") else { return };
    let Some(ua_l_idx) = rig.joint_index("upper_arm_L") else { return };
    let Some(fa_l_idx) = rig.joint_index("forearm_L") else { return };
    let Some(hd_l_idx) = rig.joint_index("hand_L") else { return };

    let n = rig.joint_count();
    let mut global = vec![Mat4::IDENTITY; n];
    forward_kinematics(parents, order, local, &mut global);

    // Compute model-space target position for left hand from weapon in right hand
    let weapon_model = global[hr_idx] * weapon_hand_offset(hold_pos, hold_rot);
    let target_pos = weapon_model.transform_point3(off_hand_grip);

    // Left shoulder model-space transform and upper-arm position
    let m_sh = global[sh_l_idx];
    let shoulder_pos = m_sh.transform_point3(local[ua_l_idx].0);

    let l1 = local[fa_l_idx].0.length();
    let l2 = local[hd_l_idx].0.length();
    if l1 < 1e-3 || l2 < 1e-3 {
        return;
    }

    let elbow_pos = solve_two_bone_ik(shoulder_pos, target_pos, l1, l2, LEFT_ELBOW_POLE);

    let to_target = (target_pos - shoulder_pos).normalize_or_zero();
    let mut normal = to_target.cross(LEFT_ELBOW_POLE);
    if normal.length_squared() < 1e-5 {
        normal = to_target.cross(Vec3::Y);
    }
    let normal = normal.normalize();

    // Rotate upper_arm_L to point from shoulder to elbow
    let v1 = elbow_pos - shoulder_pos;
    let r_ua_model = bone_rotation_from_dir(v1, normal);
    let (_, r_sh_model, _) = m_sh.to_scale_rotation_translation();
    local[ua_l_idx].1 = (r_sh_model.inverse() * r_ua_model).normalize();

    // Rotate forearm_L to point from elbow to target
    let v2 = target_pos - elbow_pos;
    let r_fa_model = bone_rotation_from_dir(v2, normal);
    local[fa_l_idx].1 = (r_ua_model.inverse() * r_fa_model).normalize();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_bone_ik_reaches_target_distance() {
        let shoulder = Vec3::new(-0.19, 1.42, 0.0);
        let target = Vec3::new(0.10, 1.25, -0.30);
        let l1 = 0.27;
        let l2 = 0.25;

        let elbow = solve_two_bone_ik(shoulder, target, l1, l2, LEFT_ELBOW_POLE);
        let d1 = (elbow - shoulder).length();
        let d2 = (target - elbow).length();

        assert!((d1 - l1).abs() < 1e-4, "upper arm length {d1} != {l1}");
        assert!((d2 - l2).abs() < 1e-4, "forearm length {d2} != {l2}");
        // Elbow must bend down (-Y) and out (-X)
        assert!(elbow.y < shoulder.y, "elbow y {} >= shoulder y {}", elbow.y, shoulder.y);
        assert!(elbow.x < shoulder.x + 0.15, "elbow x should be outward");
    }
}
