//! Dev-only animation screenshot helpers (`--pose`, `--weapon`, `--cam-yaw`, `--block`, `--sprint`).
//!
//! All flags default to off so normal gameplay is untouched.

use engine::EngineArgs;
use glam::{Quat, Vec3};
use humanoid::{FullBodyOptions, HumanoidAnimator, LayerId, Locomotion, MaskId};
use pack::{ChannelKind, Clip, ModelPack, Skeleton};

use crate::play::{HandChange, PlayState};
use crate::player_visual::PlayerVisual;

/// Bone names treated as lower-body when classifying a clip as full-body vs upper-body.
const LOWER_BONES: &[&str] = &[
    "root",
    "hips",
    "pelvis",
    "thigh_L",
    "shin_L",
    "foot_L",
    "toe_L",
    "upper_leg_L",
    "lower_leg_L",
    "thigh_R",
    "shin_R",
    "foot_R",
    "toe_R",
    "upper_leg_R",
    "lower_leg_R",
];

/// Parsed `--pose=<clip>[@<seconds>]` request.
#[derive(Clone, Debug, PartialEq)]
pub struct PoseSpec {
    pub clip: String,
    pub time: Option<f32>,
}

/// Which mask/layer strategy a `--pose` clip uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoseLayer {
    UpperAction,
    FullBody,
}

/// Runtime state for animation screenshot dev flags.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimShotState {
    pub pose: Option<PoseSpec>,
    pub pose_applied: bool,
    pub pose_frozen: bool,
    pub weapon: Option<String>,
    pub cam_yaw_rad: Option<f32>,
    pub block: bool,
    pub sprint: bool,
}

impl AnimShotState {
    /// Parses `--pose`, `--weapon`, `--cam-yaw`, `--block`, and `--sprint` from CLI args.
    pub fn from_args(args: &EngineArgs) -> Self {
        let pose = args.value("--pose").and_then(parse_pose_flag);
        let weapon = args
            .value("--weapon")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let cam_yaw_rad = args
            .value("--cam-yaw")
            .and_then(|s| s.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite())
            .map(f32::to_radians);
        Self {
            pose,
            pose_applied: false,
            pose_frozen: false,
            weapon,
            cam_yaw_rad,
            block: args.flag("--block"),
            sprint: args.flag("--sprint"),
        }
    }

    /// True once `--pose` has been applied and the animator should stay frozen.
    pub fn is_pose_frozen(&self) -> bool {
        self.pose_frozen
    }

    /// Applies `--block` (right-mouse aim) and `--sprint` (with `--autowalk`) to player actions.
    pub fn apply_actions(&self, actions: &mut game::Actions, autowalk: bool) {
        if self.block {
            actions.aim = true;
        }
        if self.sprint {
            actions.sprint = true;
            if autowalk {
                actions.move_vec.y = -1.0;
            }
        }
    }

    /// Returns the orbited camera yaw when `--cam-yaw=<deg>` is set (`0` = behind, `90` = right, `180` = front).
    pub fn orbit_camera_yaw(&self, player_yaw: f32) -> Option<f32> {
        self.cam_yaw_rad.map(|offset| player_yaw + offset)
    }
}

/// Parses `<clip>[@<seconds>]`. Rejects empty clip names and non-numeric timestamps.
pub(crate) fn parse_pose_flag(raw: &str) -> Option<PoseSpec> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some((clip, time_raw)) = trimmed.split_once('@') {
        let clip = clip.trim();
        let time_str = time_raw.trim();
        if clip.is_empty() || time_str.is_empty() {
            return None;
        }
        let secs: f32 = time_str.parse().ok()?;
        if !secs.is_finite() {
            return None;
        }
        Some(PoseSpec {
            clip: clip.to_string(),
            time: Some(secs.max(0.0)),
        })
    } else {
        Some(PoseSpec {
            clip: trimmed.to_string(),
            time: None,
        })
    }
}

/// Resolves the frozen timestamp for a clip (defaults to the midpoint `duration * 0.5`).
pub(crate) fn resolve_pose_time(requested: Option<f32>, duration: f32) -> f32 {
    let dur = if duration.is_finite() && duration > 0.0 {
        duration
    } else {
        0.0
    };
    match requested {
        Some(t) if t.is_finite() => t.clamp(0.0, dur),
        _ => dur * 0.5,
    }
}

/// Returns `true` when `clip` moves any lower-body bone away from rest pose.
/// Upper-body clips in `models.pack` key unmentioned lower bones once at rest pose.
pub(crate) fn is_full_body_clip(clip: &Clip, skeleton: Option<&Skeleton>) -> bool {
    let Some(skel) = skeleton else {
        return true;
    };
    for ch in &clip.channels {
        let Some(joint) = skel.joints.get(ch.joint as usize) else {
            continue;
        };
        if !LOWER_BONES.contains(&joint.name.as_str()) {
            continue;
        }
        match ch.kind {
            ChannelKind::Translation => {
                let rest = Vec3::from(joint.rest_translation);
                if ch
                    .values
                    .iter()
                    .any(|v| (Vec3::new(v[0], v[1], v[2]) - rest).length_squared() > 1e-6)
                {
                    return true;
                }
            }
            ChannelKind::Rotation => {
                let rest = Quat::from_array(joint.rest_rotation).normalize();
                if ch
                    .values
                    .iter()
                    .any(|v| Quat::from_array(*v).normalize().dot(rest).abs() < 1.0 - 1e-4)
                {
                    return true;
                }
            }
            ChannelKind::Scale => {
                let rest = Vec3::from(joint.rest_scale);
                if ch
                    .values
                    .iter()
                    .any(|v| (Vec3::new(v[0], v[1], v[2]) - rest).length_squared() > 1e-6)
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Looks up the `HandChange` to equip `--weapon=<id>` (`none`/`unarmed` clears the hand).
pub fn weapon_hand_change(play: &PlayState, weapon_id: &str) -> Option<HandChange> {
    if weapon_id.eq_ignore_ascii_case("none") || weapon_id.eq_ignore_ascii_case("unarmed") {
        return Some(HandChange::Weapon(None));
    }
    if let Some(def) = play.weapons.get(weapon_id).cloned() {
        return Some(HandChange::Weapon(Some(def)));
    }
    if play.items.get(weapon_id).is_some_and(|d| d.holdable) {
        return Some(HandChange::Tool(weapon_id.to_string()));
    }
    None
}

/// Applies `spec` to `animator` on the upper-body or full-body layer and advances to the frozen timestamp.
pub fn apply_pose(
    animator: &mut HumanoidAnimator,
    clip: &Clip,
    skeleton: Option<&Skeleton>,
    spec: &PoseSpec,
) -> Option<(PoseLayer, f32)> {
    let t = resolve_pose_time(spec.time, clip.duration);
    let layer = if is_full_body_clip(clip, skeleton) {
        PoseLayer::FullBody
    } else {
        PoseLayer::UpperAction
    };
    let opts = FullBodyOptions {
        speed: 1.0,
        looping: false,
        hold_last: true,
        fade_in: 0.0,
        fade_out: 0.0,
    };
    if !animator.play_full_body(&clip.name, opts) {
        return None;
    }
    let mask = match layer {
        PoseLayer::UpperAction => MaskId::Upper,
        PoseLayer::FullBody => MaskId::Full,
    };
    animator.layers_mut().get_mut(LayerId::FullBody).mask = mask;
    animator.update(t, Locomotion::default());
    Some((layer, t))
}

/// Applies `--pose` once after the player visual and weapon model have been set up.
pub fn apply_pending_pose(shot: &mut AnimShotState, pv: &mut PlayerVisual, pack: &ModelPack) {
    if shot.pose_applied {
        return;
    }
    shot.pose_applied = true;
    let Some(spec) = shot.pose.as_ref() else {
        return;
    };
    let Some(model) = pack.find("human_player") else {
        log::warn!("--pose: human_player model missing from pack");
        return;
    };
    let Some(clip) = model.clips.iter().find(|c| c.name == spec.clip) else {
        log::warn!("--pose: unknown clip '{}'", spec.clip);
        return;
    };
    if let Some((layer, t)) = apply_pose(&mut pv.animator, clip, model.skeleton.as_ref(), spec) {
        shot.pose_frozen = true;
        log::info!("--pose: applied '{}' ({layer:?}) frozen at {t:.3}s", spec.clip);
    } else {
        log::warn!("--pose: animator could not play '{}'", spec.clip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::anim::Rig;
    use glam::Mat4;
    use pack::{Channel, Interpolation, Joint};
    use std::f32::consts::{FRAC_PI_2, PI};

    fn make_joint(name: &str, parent: Option<u16>) -> Joint {
        Joint {
            name: name.into(),
            parent,
            inverse_bind: Mat4::IDENTITY.to_cols_array(),
            rest_translation: [0.0, 0.0, 0.0],
            rest_rotation: [0.0, 0.0, 0.0, 1.0],
            rest_scale: [1.0, 1.0, 1.0],
        }
    }

    fn test_skeleton() -> Skeleton {
        Skeleton {
            joints: vec![
                make_joint("root", None),      // 0
                make_joint("hips", Some(0)),   // 1
                make_joint("thigh_L", Some(1)),// 2
                make_joint("spine", Some(1)),  // 3
                make_joint("upper_arm_R", Some(3)), // 4
            ],
        }
    }

    fn rot_channel(joint: u16, times: Vec<f32>, angles_x: Vec<f32>) -> Channel {
        Channel {
            joint,
            kind: ChannelKind::Rotation,
            interpolation: Interpolation::Linear,
            times,
            values: angles_x
                .into_iter()
                .map(|a| Quat::from_rotation_x(a).to_array())
                .collect(),
        }
    }

    #[test]
    fn parses_pose_flag_with_and_without_time() {
        assert_eq!(
            parse_pose_flag("block_2h"),
            Some(PoseSpec {
                clip: "block_2h".into(),
                time: None,
            })
        );
        assert_eq!(
            parse_pose_flag("melee_2h_b@0.35"),
            Some(PoseSpec {
                clip: "melee_2h_b".into(),
                time: Some(0.35),
            })
        );
        assert_eq!(
            parse_pose_flag("eat@-0.5"),
            Some(PoseSpec {
                clip: "eat".into(),
                time: Some(0.0),
            })
        );
        assert_eq!(parse_pose_flag(""), None);
        assert_eq!(parse_pose_flag("@0.5"), None);
        assert_eq!(parse_pose_flag("eat@"), None);
        assert_eq!(parse_pose_flag("eat@nope"), None);
    }

    #[test]
    fn parses_anim_shot_flags_from_engine_args() {
        let empty = AnimShotState::from_args(&EngineArgs::parse(vec![]));
        assert_eq!(empty, AnimShotState::default());
        assert!(!empty.is_pose_frozen());

        let args = EngineArgs::parse(vec![
            "--shot=/tmp/x.png".into(),
            "--pose=death_back@0.8".into(),
            "--weapon=bat".into(),
            "--cam-yaw=90".into(),
            "--block".into(),
            "--sprint".into(),
        ]);
        let state = AnimShotState::from_args(&args);
        assert_eq!(
            state.pose,
            Some(PoseSpec {
                clip: "death_back".into(),
                time: Some(0.8),
            })
        );
        assert_eq!(state.weapon.as_deref(), Some("bat"));
        assert!((state.cam_yaw_rad.unwrap() - FRAC_PI_2).abs() < 1e-5);
        assert!(state.block);
        assert!(state.sprint);
    }

    #[test]
    fn upper_body_clip_uses_upper_mask_and_preserves_idle_legs() {
        let skel = test_skeleton();
        let rig = Rig::new(&skel);
        let idle = Clip {
            name: "idle".into(),
            duration: 1.0,
            channels: vec![rot_channel(2, vec![0.0, 1.0], vec![0.4, 0.4])],
        };
        let block_2h = Clip {
            name: "block_2h".into(),
            duration: 0.5,
            channels: vec![
                // Unkeyed lower bone exported at rest pose by Blender baker
                rot_channel(2, vec![0.0], vec![0.0]),
                rot_channel(4, vec![0.0, 0.5], vec![0.2, 1.0]),
            ],
        };
        assert!(!is_full_body_clip(&block_2h, Some(&skel)));

        let mut anim = HumanoidAnimator::new(&skel, &[idle, block_2h.clone()]);
        let spec = PoseSpec {
            clip: "block_2h".into(),
            time: None,
        };
        let applied = apply_pose(&mut anim, &block_2h, Some(&skel), &spec);
        assert_eq!(applied, Some((PoseLayer::UpperAction, 0.25)));
        assert_eq!(anim.layers().get(LayerId::FullBody).mask, MaskId::Upper);

        let mut skin = vec![Mat4::IDENTITY; rig.joint_count()];
        anim.write_skin(&rig, &mut skin);

        // Lower body (thigh_L = 2) keeps idle's 0.4 rad rotation, not block_2h's rest 0.0
        let thigh_rot = Quat::from_mat4(&rig.joint_model_matrix(&skin, 2));
        let expected_thigh = Quat::from_rotation_x(0.4);
        assert!(thigh_rot.dot(expected_thigh).abs() > 0.999);

        // Stepping with dt=0.0 keeps the pose completely frozen
        anim.update(0.0, Locomotion::default());
        let mut skin_after = vec![Mat4::IDENTITY; rig.joint_count()];
        anim.write_skin(&rig, &mut skin_after);
        assert_eq!(skin, skin_after);
    }

    #[test]
    fn full_body_clip_uses_full_mask_and_overrides_legs() {
        let skel = test_skeleton();
        let rig = Rig::new(&skel);
        let idle = Clip {
            name: "idle".into(),
            duration: 1.0,
            channels: vec![rot_channel(2, vec![0.0, 1.0], vec![0.4, 0.4])],
        };
        let death = Clip {
            name: "death_back".into(),
            duration: 1.3,
            channels: vec![
                rot_channel(1, vec![0.0, 1.3], vec![0.0, -1.2]),
                rot_channel(2, vec![0.0, 1.3], vec![0.0, 0.8]),
            ],
        };
        assert!(is_full_body_clip(&death, Some(&skel)));

        let mut anim = HumanoidAnimator::new(&skel, &[idle, death.clone()]);
        let spec = PoseSpec {
            clip: "death_back".into(),
            time: Some(1.3),
        };
        let applied = apply_pose(&mut anim, &death, Some(&skel), &spec);
        assert_eq!(applied, Some((PoseLayer::FullBody, 1.3)));
        assert_eq!(anim.layers().get(LayerId::FullBody).mask, MaskId::Full);

        let mut skin = vec![Mat4::IDENTITY; rig.joint_count()];
        anim.write_skin(&rig, &mut skin);
        let expected = Quat::from_rotation_x(-1.2) * Quat::from_rotation_x(0.8);
        let thigh_world = Quat::from_mat4(&rig.joint_model_matrix(&skin, 2));
        assert!(thigh_world.dot(expected).abs() > 0.999);
    }

    #[test]
    fn pose_defaults_to_middle_and_clamps_out_of_range() {
        assert_eq!(resolve_pose_time(None, 1.5), 0.75);
        assert_eq!(resolve_pose_time(Some(5.0), 1.5), 1.5);
        assert_eq!(resolve_pose_time(Some(-1.0), 1.5), 0.0);
        assert_eq!(resolve_pose_time(None, 0.0), 0.0);
    }

    #[test]
    fn camera_orbit_and_actions_match_conventions() {
        let mut shot = AnimShotState::from_args(&EngineArgs::parse(vec![
            "--cam-yaw=180".into(),
            "--block".into(),
            "--sprint".into(),
        ]));
        assert!((shot.orbit_camera_yaw(0.0).unwrap() - PI).abs() < 1e-5);
        shot.cam_yaw_rad = Some(90.0f32.to_radians());
        assert!((shot.orbit_camera_yaw(0.0).unwrap() - FRAC_PI_2).abs() < 1e-5);

        let mut actions = game::Actions::default();
        shot.apply_actions(&mut actions, true);
        assert!(actions.aim);
        assert!(actions.sprint);
        assert_eq!(actions.move_vec.y, -1.0);
    }
}
