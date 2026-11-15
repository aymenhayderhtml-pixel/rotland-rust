//! Locomotion thresholds, blend calculations and pose interpolation.
//!
//! Matches Godot's HumanoidAnimTree, HumanoidBlender and LocomotionProfile:
//! - Walk blend point at 4.0 m/s (natural clip speed 1.6 m/s)
//! - Run blend point at 7.0 m/s (natural clip speed 4.5 m/s)
//! - Blend rate 8.0 s^-1 exponential smoothing
//! - Action fade in 0.05 s, fade out 0.15 s

use glam::{Quat, Vec3};

/// Blend-line position of walk clip (PlayerController walk speed in m/s).
pub const WALK_BLEND: f32 = 4.0;
/// Natural forward speed of walk clip in m/s.
pub const WALK_CLIP_SPEED: f32 = 1.6;
/// Blend-line position of run clip (PlayerController sprint speed in m/s).
pub const RUN_BLEND: f32 = 7.0;
/// Natural forward speed of run clip in m/s.
pub const RUN_CLIP_SPEED: f32 = 4.5;
/// How quickly the blend values follow their targets (per second).
pub const BLEND_RATE: f32 = 8.0;
/// Blend-in duration of a one-shot action in seconds.
pub const ACTION_FADE_IN: f32 = 0.05;
/// Blend-out duration of a one-shot action in seconds.
pub const ACTION_FADE_OUT: f32 = 0.15;

/// Bones that upper-body layers (actions and aim) override.
/// Lower body (hips, legs, feet) continues running locomotion.
pub const UPPER_BONES: &[&str] = &[
    "spine",
    "chest",
    "neck",
    "head",
    "shoulder_L",
    "upper_arm_L",
    "forearm_L",
    "hand_L",
    "shoulder_R",
    "upper_arm_R",
    "forearm_R",
    "hand_R",
];

/// Per-frame movement state driving locomotion and posture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Locomotion {
    pub speed: f32,
    pub on_floor: bool,
    pub crouching: bool,
}

impl Default for Locomotion {
    fn default() -> Self {
        Self {
            speed: 0.0,
            on_floor: true,
            crouching: false,
        }
    }
}

/// Normalized weights of idle, walk, and run on the blend line.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocomotionWeights {
    pub idle: f32,
    pub walk: f32,
    pub run: f32,
}

/// Computes idle-walk-run blend weights given the current blend position.
pub fn locomotion_weights(blend_pos: f32) -> LocomotionWeights {
    let s = blend_pos.clamp(0.0, RUN_BLEND);
    if s <= WALK_BLEND {
        let w = s / WALK_BLEND;
        LocomotionWeights {
            idle: 1.0 - w,
            walk: w,
            run: 0.0,
        }
    } else {
        let w = (s - WALK_BLEND) / (RUN_BLEND - WALK_BLEND);
        LocomotionWeights {
            idle: 0.0,
            walk: 1.0 - w,
            run: w,
        }
    }
}

/// Playback speed multiplier so foot strides match the character's ground speed.
pub fn tempo_for(blend_pos: f32) -> f32 {
    if blend_pos < 0.05 {
        1.0
    } else {
        let natural = if blend_pos <= WALK_BLEND {
            WALK_CLIP_SPEED
        } else {
            let t = (blend_pos - WALK_BLEND) / (RUN_BLEND - WALK_BLEND);
            WALK_CLIP_SPEED + (RUN_CLIP_SPEED - WALK_CLIP_SPEED) * t
        };
        (blend_pos / natural).clamp(0.25, 4.0)
    }
}

/// Blends joint transforms `(translation, rotation, scale)` with optional bone filtering.
pub fn blend_local_poses(
    base: &mut [(Vec3, Quat, Vec3)],
    other: &[(Vec3, Quat, Vec3)],
    weight: f32,
    mask: Option<&[bool]>,
) {
    let w = weight.clamp(0.0, 1.0);
    if w <= 0.0 {
        return;
    }
    let n = base.len().min(other.len());
    for i in 0..n {
        if mask.is_some_and(|m| !m[i]) {
            continue;
        }
        if w >= 1.0 {
            base[i] = other[i];
        } else {
            base[i].0 = base[i].0.lerp(other[i].0, w);
            base[i].1 = base[i].1.slerp(other[i].1, w);
            base[i].2 = base[i].2.lerp(other[i].2, w);
        }
    }
}

/// Returns the update interval in seconds based on distance and screen visibility (HumanoidVisual.SetLod).
pub fn lod_interval(distance: f32, on_screen: bool) -> f32 {
    if !on_screen {
        if distance > 30.0 { 1.0 } else { 0.5 }
    } else if distance < 10.0 {
        0.0
    } else if distance < 25.0 {
        0.04
    } else {
        0.1
    }
}

/// Evaluates the base locomotion local pose (idle, walk, run blend).
pub fn sample_locomotion(
    rig: &engine::anim::Rig,
    idle: Option<&pack::Clip>,
    walk: Option<&pack::Clip>,
    run: Option<&pack::Clip>,
    blend_pos: f32,
    time_idle: f32,
    time_loco: f32,
) -> Vec<(Vec3, Quat, Vec3)> {
    let weights = locomotion_weights(blend_pos);
    if blend_pos <= WALK_BLEND {
        match (idle, walk) {
            (Some(ic), Some(wc)) => {
                let mut p = rig.local_pose(ic, time_idle);
                if weights.walk > 0.0 {
                    let wp = rig.local_pose(wc, time_loco);
                    blend_local_poses(&mut p, &wp, weights.walk, None);
                }
                p
            }
            (Some(ic), None) => rig.local_pose(ic, time_idle),
            (None, Some(wc)) => rig.local_pose(wc, time_loco),
            _ => rig.local_pose(&pack::Clip { name: "rest".into(), duration: 0.0, channels: vec![] }, 0.0),
        }
    } else {
        match (walk, run) {
            (Some(wc), Some(rc)) => {
                let mut p = rig.local_pose(wc, time_loco);
                if weights.run > 0.0 {
                    let rp = rig.local_pose(rc, time_loco);
                    blend_local_poses(&mut p, &rp, weights.run, None);
                }
                p
            }
            (Some(wc), None) => rig.local_pose(wc, time_loco),
            (None, Some(rc)) => rig.local_pose(rc, time_loco),
            _ => rig.local_pose(&pack::Clip { name: "rest".into(), duration: 0.0, channels: vec![] }, 0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blend_weights_at_cardinal_speeds() {
        let w0 = locomotion_weights(0.0);
        assert!((w0.idle - 1.0).abs() < 1e-5);
        assert!(w0.walk.abs() < 1e-5);
        assert!(w0.run.abs() < 1e-5);

        let w2 = locomotion_weights(2.0);
        assert!((w2.idle - 0.5).abs() < 1e-5);
        assert!((w2.walk - 0.5).abs() < 1e-5);
        assert!(w2.run.abs() < 1e-5);

        let w4 = locomotion_weights(4.0);
        assert!(w4.idle.abs() < 1e-5);
        assert!((w4.walk - 1.0).abs() < 1e-5);
        assert!(w4.run.abs() < 1e-5);

        let w55 = locomotion_weights(5.5);
        assert!(w55.idle.abs() < 1e-5);
        assert!((w55.walk - 0.5).abs() < 1e-5);
        assert!((w55.run - 0.5).abs() < 1e-5);

        let w7 = locomotion_weights(7.0);
        assert!(w7.idle.abs() < 1e-5);
        assert!(w7.walk.abs() < 1e-5);
        assert!((w7.run - 1.0).abs() < 1e-5);

        // Clamping above max
        let w9 = locomotion_weights(9.0);
        assert!(w9.idle.abs() < 1e-5);
        assert!(w9.walk.abs() < 1e-5);
        assert!((w9.run - 1.0).abs() < 1e-5);
    }

    #[test]
    fn tempo_matches_csharp_ranges() {
        assert_eq!(tempo_for(0.0), 1.0);
        assert_eq!(tempo_for(0.04), 1.0);
        // At 1.6 m/s walk speed, natural speed is 1.6, tempo is 1.0
        assert!((tempo_for(1.6) - 1.0).abs() < 1e-4);
        // At 4.0 m/s (WalkBlend), natural is 1.6, tempo is 4.0 / 1.6 = 2.5
        assert!((tempo_for(4.0) - 2.5).abs() < 1e-4);
        // At 7.0 m/s (RunBlend), natural is 4.5, tempo is 7.0 / 4.5
        assert!((tempo_for(7.0) - (7.0 / 4.5)).abs() < 1e-4);
    }
}
