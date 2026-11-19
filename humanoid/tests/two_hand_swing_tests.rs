//! Integration tests for two-handed bat swing mechanics (R10-03, GitHub issue #3).
//!
//! Verifies:
//! 1. Left hand stays within 6 cm of the bat off-hand grip at start, middle, and hit time.
//! 2. Spine pitch never exceeds Godot windup value (+6.0°), spine yaw reaches +28.0° at hit.
//! 3. Head roll stays within ±5° throughout the whole swing.
//! 4. One-handed weapons (knife, pistol) leave the left arm pose unchanged from unequipped.

use engine::anim::Rig;
use glam::{Mat4, Vec3};
use humanoid::{
    is_two_handed, off_hand_grip_for, swing_head_roll_deg, swing_spine_pitch_deg,
    swing_spine_yaw_deg, weapon_world_matrix, HumanoidAnimator, Locomotion,
};
use pack::{Clip, Joint, Skeleton};

/// Builds a test humanoid skeleton matching `human_player.glb` proportions.
fn humanoid_test_skeleton() -> Skeleton {
    let make_joint = |name: &str, parent: Option<u16>, trans: [f32; 3]| Joint {
        name: name.into(),
        parent,
        inverse_bind: Mat4::IDENTITY.to_cols_array(),
        rest_translation: trans,
        rest_rotation: [0.0, 0.0, 0.0, 1.0],
        rest_scale: [1.0, 1.0, 1.0],
    };
    Skeleton {
        joints: vec![
            make_joint("root", None, [0.0, 0.0, 0.0]),
            make_joint("hips", Some(0), [0.0, 0.90, 0.0]),
            make_joint("spine", Some(1), [0.0, 0.14, 0.0]),
            make_joint("chest", Some(2), [0.0, 0.18, 0.0]),
            make_joint("neck", Some(3), [0.0, 0.24, 0.0]),
            make_joint("head", Some(4), [0.0, 0.08, 0.0]),
            make_joint("shoulder_R", Some(3), [0.08, 0.20, 0.0]),
            make_joint("upper_arm_R", Some(6), [0.0, 0.11, 0.0]),
            make_joint("forearm_R", Some(7), [0.0, 0.27, 0.0]),
            make_joint("hand_R", Some(8), [0.0, 0.25, 0.0]),
            make_joint("shoulder_L", Some(3), [-0.08, 0.20, 0.0]),
            make_joint("upper_arm_L", Some(10), [0.0, 0.11, 0.0]),
            make_joint("forearm_L", Some(11), [0.0, 0.27, 0.0]),
            make_joint("hand_L", Some(12), [0.0, 0.25, 0.0]),
        ],
    }
}

#[test]
fn left_hand_within_6cm_of_bat_grip_at_start_middle_and_hit() {
    let skel = humanoid_test_skeleton();
    let rig = Rig::new(&skel);
    let clip = Clip {
        name: "melee_2h".into(),
        duration: 0.70,
        channels: vec![],
    };

    let mut animator = HumanoidAnimator::new(&skel, &[clip]);
    animator.set_weapon(Some("bat"));
    animator.play_action("melee_2h", 1.0);

    let hr_idx = rig.joint_index("hand_R").expect("hand_R in skeleton");
    let hl_idx = rig.joint_index("hand_L").expect("hand_L in skeleton");
    let off_hand_grip = off_hand_grip_for("bat").expect("bat has off_hand_grip");
    let (hold_pos, hold_rot) = humanoid::weapon_hold_for("bat");

    let char_world = Mat4::from_translation(Vec3::new(2.5, 0.0, -1.0)) * Mat4::from_rotation_y(0.5);

    // Test start (t = 0.0), middle (t = 0.20 windup), and hit time (t = 0.40)
    let checkpoints = [(0.0, "start"), (0.20, "middle"), (0.40, "hit")];
    let mut current_time = 0.0;

    for (t, label) in checkpoints {
        let dt = t - current_time;
        if dt > 0.0 {
            animator.update(dt, Locomotion::default());
            current_time = t;
        }

        let mut skin = vec![Mat4::IDENTITY; rig.joint_count()];
        animator.write_skin(&rig, &mut skin);

        let weapon_world = weapon_world_matrix(char_world, &rig, &skin, hr_idx, hold_pos, hold_rot);
        let grip_world = weapon_world.transform_point3(off_hand_grip);

        let hand_l_model = rig.joint_model_matrix(&skin, hl_idx);
        let hand_l_world = char_world * hand_l_model;
        let hand_l_pos = hand_l_world.transform_point3(Vec3::ZERO);

        let dist = (hand_l_pos - grip_world).length();
        assert!(
            dist <= 0.06,
            "left hand distance to bat grip at {label} (t={t:.2} s) was {dist:.4} m (> 0.06 m)"
        );
    }
}

#[test]
fn spine_pitch_never_exceeds_godot_and_yaw_reaches_godot_twist_at_hit() {
    // Godot windup spine pitch in melee_1h is +6.0° (poses_combat.py:25).
    // The swing must never lean backward beyond this value.
    for step in 0..=70 {
        let t = step as f32 * 0.01;
        let pitch = swing_spine_pitch_deg(t);
        assert!(
            pitch <= 6.0 + 1e-4,
            "spine pitch {pitch:.2}° exceeded Godot max of +6.0° at t={t:.2} s"
        );
    }

    // Godot hit spine yaw is +28.0° (poses_combat.py:36).
    let yaw_at_hit = swing_spine_yaw_deg(0.40);
    assert!(
        (yaw_at_hit - 28.0).abs() < 1e-3,
        "spine yaw at hit time {yaw_at_hit:.2}° did not reach Godot twist (+28.0°)"
    );
}

#[test]
fn head_roll_stays_within_5_degrees_throughout_swing() {
    for step in 0..=70 {
        let t = step as f32 * 0.01;
        let roll = swing_head_roll_deg(t);
        assert!(
            roll.abs() <= 5.0,
            "head roll {roll:.2}° exceeded ±5° limit at t={t:.2} s"
        );
    }
}

#[test]
fn one_handed_weapons_left_arm_pose_is_unchanged() {
    let skel = humanoid_test_skeleton();
    let rig = Rig::new(&skel);

    assert!(!is_two_handed("knife"));
    assert!(!is_two_handed("pistol"));
    assert_eq!(off_hand_grip_for("knife"), None);
    assert_eq!(off_hand_grip_for("pistol"), None);

    let mut anim_unarmed = HumanoidAnimator::new(&skel, &[]);
    let mut anim_knife = HumanoidAnimator::new(&skel, &[]);
    let mut anim_pistol = HumanoidAnimator::new(&skel, &[]);

    anim_knife.set_weapon(Some("knife"));
    anim_pistol.set_weapon(Some("pistol"));

    let loco = Locomotion {
        speed: 2.0,
        on_floor: true,
        crouching: false,
    };
    anim_unarmed.update(0.1, loco);
    anim_knife.update(0.1, loco);
    anim_pistol.update(0.1, loco);

    let mut skin_unarmed = vec![Mat4::IDENTITY; rig.joint_count()];
    let mut skin_knife = vec![Mat4::IDENTITY; rig.joint_count()];
    let mut skin_pistol = vec![Mat4::IDENTITY; rig.joint_count()];

    anim_unarmed.write_skin(&rig, &mut skin_unarmed);
    anim_knife.write_skin(&rig, &mut skin_knife);
    anim_pistol.write_skin(&rig, &mut skin_pistol);

    let ua_l_idx = rig.joint_index("upper_arm_L").unwrap();
    let fa_l_idx = rig.joint_index("forearm_L").unwrap();
    let hd_l_idx = rig.joint_index("hand_L").unwrap();

    for &idx in &[ua_l_idx, fa_l_idx, hd_l_idx] {
        let diff_knife = (skin_unarmed[idx] - skin_knife[idx]).abs();
        let max_err_knife = diff_knife
            .to_cols_array()
            .iter()
            .cloned()
            .fold(0.0f32, f32::max);
        assert!(
            max_err_knife < 1e-5,
            "knife changed left arm joint {idx}: error {max_err_knife}"
        );

        let diff_pistol = (skin_unarmed[idx] - skin_pistol[idx]).abs();
        let max_err_pistol = diff_pistol
            .to_cols_array()
            .iter()
            .cloned()
            .fold(0.0f32, f32::max);
        assert!(
            max_err_pistol < 1e-5,
            "pistol changed left arm joint {idx}: error {max_err_pistol}"
        );
    }
}
