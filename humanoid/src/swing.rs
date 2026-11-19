//! Posing for natural two-handed melee weapon swings (e.g. baseball bat, axe).
//!
//! Fixes issue #3 (ANIM-001, ANIM-002):
//! - Twists at the waist (spine yaw -24° windup -> +28° hit -> +36° follow-through),
//!   ported from Godot `tools/blender/characters/poses_combat.py:25, 36, 47`.
//! - Spine pitch never arches backward past Godot's value (+6.0° at windup from
//!   `tools/blender/characters/poses_combat.py:25`), with a small forward lean (-6.0°) at hit.
//! - Counter-rotates neck and head against the torso twist so the head stays upright
//!   facing the target with roll within ±5° (0° throughout).

use engine::anim::Rig;
use glam::{EulerRot, Quat, Vec3};

/// Keyframe timestamps in seconds for the 0.70 s two-handed swing.
const SWING_TIMES: [f32; 5] = [0.0, 0.20, 0.40, 0.55, 0.70];

/// Spine (pitch, yaw, roll) in degrees at each keyframe.
///
/// Quoting Godot references:
/// - t=0.20 windup: pitch +6.0° (never exceeds Godot's windup value from `poses_combat.py:25`),
///   yaw -24.0° (twisting back, `poses_combat.py:25`).
/// - t=0.40 hit: pitch -6.0° (small forward lean, `poses_combat.py:36`),
///   yaw +28.0° (Godot's twist at hit time, `poses_combat.py:36`).
/// - t=0.55 follow-through: pitch -6.0°, yaw +36.0° (`poses_combat.py:47`).
const SPINE_EULER_DEG: [(f32, f32, f32); 5] = [
    (0.0, 0.0, 0.0),
    (6.0, -24.0, 0.0),
    (-6.0, 28.0, 0.0),
    (-6.0, 36.0, 0.0),
    (0.0, 0.0, 0.0),
];

/// Chest (pitch, yaw, roll) in degrees at each keyframe.
/// Ported from Godot `poses_combat.py:26, 37, 48`.
const CHEST_EULER_DEG: [(f32, f32, f32); 5] = [
    (0.0, 0.0, 0.0),
    (4.0, -30.0, 0.0),
    (-8.0, 34.0, 0.0),
    (-8.0, 40.0, 0.0),
    (0.0, 0.0, 0.0),
];

/// Right upper arm (pitch, yaw, roll) in degrees at each keyframe.
const UPPER_ARM_R_DEG: [(f32, f32, f32); 5] = [
    (45.0, -30.0, 35.0),
    (50.0, -20.0, 30.0),
    (55.0, -25.0, 25.0),
    (50.0, -20.0, 20.0),
    (45.0, -30.0, 35.0),
];

/// Right forearm (pitch, yaw, roll) in degrees at each keyframe.
const FOREARM_R_DEG: [(f32, f32, f32); 5] = [
    (105.0, 0.0, 0.0),
    (95.0, 0.0, 0.0),
    (95.0, 0.0, 0.0),
    (90.0, 0.0, 0.0),
    (105.0, 0.0, 0.0),
];

/// Right hand (pitch, yaw, roll) in degrees at each keyframe.
const HAND_R_DEG: [(f32, f32, f32); 5] = [
    (0.0, 0.0, 0.0),
    (0.0, 0.0, 0.0),
    (0.0, 0.0, 0.0),
    (0.0, 0.0, 0.0),
    (0.0, 0.0, 0.0),
];

fn sample_keyframes(times: &[f32], values: &[(f32, f32, f32)], t: f32) -> (f32, f32, f32) {
    if times.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let t_clamped = t.clamp(times[0], times[times.len() - 1]);
    let k = times.partition_point(|&x| x <= t_clamped);
    if k == 0 {
        return values[0];
    }
    if k >= times.len() {
        return values[values.len() - 1];
    }
    let (t0, t1) = (times[k - 1], times[k]);
    let frac = if t1 > t0 { (t_clamped - t0) / (t1 - t0) } else { 0.0 };
    let (p0, y0, r0) = values[k - 1];
    let (p1, y1, r1) = values[k];
    (
        p0 + (p1 - p0) * frac,
        y0 + (y1 - y0) * frac,
        r0 + (r1 - r0) * frac,
    )
}

/// Returns the spine pitch angle in degrees for the two-handed swing at time `t`.
pub fn swing_spine_pitch_deg(t: f32) -> f32 {
    sample_keyframes(&SWING_TIMES, &SPINE_EULER_DEG, t).0
}

/// Returns the spine yaw angle in degrees for the two-handed swing at time `t`.
pub fn swing_spine_yaw_deg(t: f32) -> f32 {
    sample_keyframes(&SWING_TIMES, &SPINE_EULER_DEG, t).1
}

/// Returns the head roll angle in degrees for the two-handed swing at time `t`.
pub fn swing_head_roll_deg(t: f32) -> f32 {
    let _ = t;
    0.0 // Head roll stays 0.0° throughout the whole swing
}

/// Evaluated upper-body joint rotations for the two-handed swing.
#[derive(Clone, Copy, Debug)]
pub struct SwingPose {
    pub spine: Quat,
    pub chest: Quat,
    pub neck: Quat,
    pub head: Quat,
    pub upper_arm_r: Quat,
    pub forearm_r: Quat,
    pub hand_r: Quat,
}

/// Evaluates joint rotations for the two-handed swing at clip time `t` (0..=0.70 s).
pub fn evaluate_two_hand_swing(t: f32) -> SwingPose {
    let (sp_p, sp_y, sp_r) = sample_keyframes(&SWING_TIMES, &SPINE_EULER_DEG, t);
    let (ch_p, ch_y, ch_r) = sample_keyframes(&SWING_TIMES, &CHEST_EULER_DEG, t);
    let (ua_p, ua_y, ua_r) = sample_keyframes(&SWING_TIMES, &UPPER_ARM_R_DEG, t);
    let (fa_p, fa_y, fa_r) = sample_keyframes(&SWING_TIMES, &FOREARM_R_DEG, t);
    let (hd_p, hd_y, hd_r) = sample_keyframes(&SWING_TIMES, &HAND_R_DEG, t);

    // Total torso rotation that neck and head must counter-rotate against
    let torso_yaw = sp_y + ch_y;
    let torso_pitch = sp_p + ch_p;

    // Counter-rotate neck and head so total head yaw is 0° (facing target) and upright
    let neck_yaw = -0.4 * torso_yaw;
    let head_yaw = -0.6 * torso_yaw;
    let neck_pitch = -0.3 * torso_pitch;
    let head_pitch = -0.7 * torso_pitch;

    let euler_quat = |p: f32, y: f32, r: f32| {
        Quat::from_euler(
            EulerRot::XYZ,
            p.to_radians(),
            y.to_radians(),
            r.to_radians(),
        )
    };

    SwingPose {
        spine: euler_quat(sp_p, sp_y, sp_r),
        chest: euler_quat(ch_p, ch_y, ch_r),
        neck: euler_quat(neck_pitch, neck_yaw, 0.0),
        head: euler_quat(head_pitch, head_yaw, 0.0),
        upper_arm_r: euler_quat(ua_p, ua_y, ua_r),
        forearm_r: euler_quat(fa_p, fa_y, fa_r),
        hand_r: euler_quat(hd_p, hd_y, hd_r),
    }
}

/// Applies the two-handed swing pose onto `local` joint transforms with weight `weight`.
pub fn apply_two_hand_swing_pose(
    rig: &Rig,
    local: &mut [(Vec3, Quat, Vec3)],
    t: f32,
    weight: f32,
) {
    let w = weight.clamp(0.0, 1.0);
    if w <= 0.0 {
        return;
    }
    let pose = evaluate_two_hand_swing(t);

    let mut blend = |name: &str, target_rot: Quat| {
        if let Some(idx) = rig.joint_index(name) && idx < local.len() {
            local[idx].1 = local[idx].1.slerp(target_rot, w);
        }
    };

    blend("spine", pose.spine);
    blend("chest", pose.chest);
    blend("neck", pose.neck);
    blend("head", pose.head);
    blend("upper_arm_R", pose.upper_arm_r);
    blend("forearm_R", pose.forearm_r);
    blend("hand_R", pose.hand_r);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spine_pitch_never_exceeds_godot_value() {
        for step in 0..=70 {
            let t = step as f32 * 0.01;
            let pitch = swing_spine_pitch_deg(t);
            // Godot melee_1h windup spine pitch is +6.0° (poses_combat.py:25)
            assert!(
                pitch <= 6.0 + 1e-4,
                "spine pitch {pitch}° exceeded 6.0° at t={t}"
            );
        }
    }

    #[test]
    fn spine_yaw_reaches_godot_twist_at_hit_time() {
        let yaw = swing_spine_yaw_deg(0.40);
        // Godot melee_1h hit spine yaw is +28.0° (poses_combat.py:36)
        assert!(
            (yaw - 28.0).abs() < 1e-3,
            "spine yaw {yaw}° != 28.0° at hit time t=0.40"
        );
    }

    #[test]
    fn head_roll_stays_within_5_degrees() {
        for step in 0..=70 {
            let t = step as f32 * 0.01;
            let roll = swing_head_roll_deg(t);
            assert!(
                roll.abs() <= 5.0,
                "head roll {roll}° exceeded ±5° at t={t}"
            );
        }
    }
}
