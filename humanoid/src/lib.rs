//! Humanoid animation logic and weapon attachment for Rotland.
//!
//! Provides the `HumanoidAnimator` state machine for characters (player, NPCs, zombies)
//! and matrix attachment calculations for weapons held in the right hand.

pub mod action;
pub mod animator;
pub mod blend;
pub mod ik;
pub mod swing;
pub mod weapon;

pub use action::ActionState;
pub use animator::HumanoidAnimator;
pub use blend::{
    locomotion_weights, tempo_for, Locomotion, LocomotionWeights, ACTION_FADE_IN, ACTION_FADE_OUT,
    BLEND_RATE, RUN_BLEND, RUN_CLIP_SPEED, UPPER_BONES, WALK_BLEND, WALK_CLIP_SPEED,
};
pub use ik::{apply_two_hand_left_grip_ik, solve_two_bone_ik, LEFT_ELBOW_POLE};
pub use swing::{
    apply_two_hand_swing_pose, evaluate_two_hand_swing, swing_head_roll_deg, swing_spine_pitch_deg,
    swing_spine_yaw_deg, SwingPose,
};
pub use weapon::{
    is_two_handed, off_hand_grip_for, weapon_hand_offset, weapon_hold_for, weapon_world_matrix,
    weapon_world_matrix_named, HAND_GRIP_OFFSET, HAND_GRIP_ROT_DEG, REST_POSE_DEG,
    TWO_HANDED_WEAPONS,
};

#[cfg(test)]
mod tests {
    use super::*;
    use engine::anim::Rig;
    use glam::Mat4;
    use pack::{Clip, Joint, ModelPack, Skeleton};

    fn test_skeleton() -> Skeleton {
        let make_joint = |name: &str, parent: Option<u16>| Joint {
            name: name.into(),
            parent,
            inverse_bind: Mat4::IDENTITY.to_cols_array(),
            rest_translation: [0.0, 0.0, 0.0],
            rest_rotation: [0.0, 0.0, 0.0, 1.0],
            rest_scale: [1.0, 1.0, 1.0],
        };
        Skeleton {
            joints: vec![
                make_joint("root", None),
                make_joint("hips", Some(0)),
                make_joint("spine", Some(1)),
                make_joint("chest", Some(2)),
                make_joint("shoulder_R", Some(3)),
                make_joint("hand_R", Some(4)),
            ],
        }
    }

    #[test]
    fn one_shot_action_progression_and_end() {
        let skel = test_skeleton();
        let clip = Clip {
            name: "melee_2h".into(),
            duration: 1.0,
            channels: vec![],
        };
        let mut animator = HumanoidAnimator::new(&skel, &[clip]);

        animator.play_action("melee_2h", 1.0);
        assert!(animator.is_action_active());
        assert_eq!(animator.current_action(), Some("melee_2h"));

        // At t = 0.0, weight is 0
        let act = animator.action_state().unwrap();
        assert_eq!(act.weight(), 0.0);

        // Advance 0.05 s: fade in complete
        animator.update(0.05, Locomotion::default());
        let act = animator.action_state().unwrap();
        assert!((act.weight() - 1.0).abs() < 1e-4);

        // Advance to t = 0.5: still full weight
        animator.update(0.45, Locomotion::default());
        let act = animator.action_state().unwrap();
        assert!((act.weight() - 1.0).abs() < 1e-4);
        assert!(animator.is_action_active());

        // Advance to t = 0.925: time left is 0.075 < 0.15 fade out, weight is 0.5
        animator.update(0.425, Locomotion::default());
        let act = animator.action_state().unwrap();
        assert!((act.weight() - 0.5).abs() < 1e-4);

        // Advance past 1.0 s: action finishes
        animator.update(0.1, Locomotion::default());
        assert!(!animator.is_action_active());
        assert_eq!(animator.current_action(), None);
    }

    #[test]
    fn print_pack_clips_for_human_player() {
        let pack_path = paths::repo_path("assets/models.pack");
        if let Ok(pack) = ModelPack::load(&pack_path) {
            if let Some(player) = pack.find("human_player") {
                println!("=== human_player clips in models.pack ({} total) ===", player.clips.len());
                let mut names: Vec<_> = player.clips.iter().map(|c| &c.name).collect();
                names.sort();
                for name in &names {
                    println!(" - {name}");
                }
                assert!(!player.clips.is_empty());
                assert!(names.contains(&&"idle".to_string()));
                assert!(names.contains(&&"walk".to_string()));
                assert!(names.contains(&&"run".to_string()));
                assert!(names.contains(&&"melee_2h".to_string()));
            } else {
                panic!("human_player not found in pack");
            }
        }
    }

    #[test]
    fn weapon_stays_attached_during_locomotion() {
        let skel = test_skeleton();
        let rig = Rig::new(&skel);
        let hand_idx = rig.joint_index("hand_R").expect("hand_R in test skeleton");
        let mut animator = HumanoidAnimator::new(&skel, &[]);

        let char_world = Mat4::from_translation(glam::Vec3::new(10.0, 2.0, -5.0))
            * Mat4::from_rotation_y(1.23);

        for speed in [0.0, 2.0, 4.0, 6.0, 8.0] {
            animator.update(0.016, Locomotion { speed, on_floor: true, crouching: false });
            let mut bones = vec![Mat4::IDENTITY; rig.joint_count()];
            animator.write_skin(&rig, &mut bones);

            let weapon_world = weapon_world_matrix_named(char_world, &rig, &bones, hand_idx, "bat");
            let hand_model = rig.joint_model_matrix(&bones, hand_idx);
            let hand_world = char_world * hand_model;

            let hand_pos = hand_world.transform_point3(glam::Vec3::ZERO);
            let weapon_pos = weapon_world.transform_point3(glam::Vec3::ZERO);

            assert!(!hand_pos.is_nan());
            assert!(!weapon_pos.is_nan());
            let dist = (weapon_pos - hand_pos).length();
            // Hand grip offset is 0.06m (6cm), so distance must be ~0.06m
            assert!((dist - 0.06).abs() < 1e-4, "dist {dist} != 0.06 at speed {speed}");
        }
    }
}

