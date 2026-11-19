//! Weapon attachment to humanoid right hand (hand_R).
//!
//! Replicates the node hierarchy in Godot's `WeaponMounts.cs` and `WeaponVisuals.cs`:
//! - Socket on hand_R
//! - `WeaponGrip` node: Position (0, 0.06, 0), RotationDegrees (90, 0, 0)
//! - `WeaponMount` node: RotationDegrees RestPose (-20, 0, 0)
//! - `WeaponHold` node: Position & Rotation from `data/weapons.json` (or aimHold)
//! - Weapon model whose origin sits at the handle grip.

use engine::anim::Rig;
use glam::{EulerRot, Mat4, Vec3};

/// Standard grip offset on `hand_R` from `WeaponMounts.HandGripOffset`.
pub const HAND_GRIP_OFFSET: Vec3 = Vec3::new(0.0, 0.06, 0.0);
/// Standard grip rotation in degrees from `WeaponMounts.HandGripRotation`.
pub const HAND_GRIP_ROT_DEG: Vec3 = Vec3::new(90.0, 0.0, 0.0);
/// Melee / weapon mount rest pose in degrees from `MeleeHandler.RestPose`.
pub const REST_POSE_DEG: Vec3 = Vec3::new(-20.0, 0.0, 0.0);

/// Local transformation matrix from `hand_R` joint space to weapon origin.
pub fn weapon_hand_offset(hold_pos: Vec3, hold_rot_deg: Vec3) -> Mat4 {
    let grip = Mat4::from_translation(HAND_GRIP_OFFSET)
        * Mat4::from_rotation_x(HAND_GRIP_ROT_DEG.x.to_radians());
    let mount = Mat4::from_rotation_x(REST_POSE_DEG.x.to_radians());
    let hold = Mat4::from_translation(hold_pos)
        * Mat4::from_euler(
            EulerRot::XYZ,
            hold_rot_deg.x.to_radians(),
            hold_rot_deg.y.to_radians(),
            hold_rot_deg.z.to_radians(),
        );
    grip * mount * hold
}

/// Returns the (position, rotation_deg) hold offset for a weapon id from `data/weapons.json`.
pub fn weapon_hold_for(weapon: &str) -> (Vec3, Vec3) {
    match weapon {
        "knife" | "pistol" | "shotgun" | "rifle" | "ak47" | "smg" => {
            (Vec3::ZERO, Vec3::new(90.0, 0.0, 0.0))
        }
        _ => (Vec3::ZERO, Vec3::ZERO),
    }
}

/// Two-handed weapon identifiers from Godot's `WeaponGripConvention.cs:19`.
pub const TWO_HANDED_WEAPONS: &[&str] = &[
    "bat", "axe", "crowbar", "katana", "shotgun", "rifle", "ak47", "smg",
];

/// Returns true if the weapon is two-handed according to Godot `WeaponGripConvention.cs:19`.
pub fn is_two_handed(weapon: &str) -> bool {
    TWO_HANDED_WEAPONS.contains(&weapon)
}

/// Returns the model-space off-hand grip offset (`grip_L`) for a two-handed weapon.
///
/// Ported from the `grip_L` marker empties exported in each weapon's `.glb` model:
/// - bat: (0, 0, 0.10) from `tools/blender/weapons/bat.py:27`
/// - axe: (0, 0, 0.20) from `tools/blender/weapons/axe.py:38`
/// - crowbar: (0, 0, 0.20) from `tools/blender/weapons/crowbar.py:26`
/// - katana: (0, 0, 0.05) from `tools/blender/weapons/katana.py:36`
/// - shotgun: (0, 0, -0.47) from `tools/blender/weapons/shotgun.py:35`
/// - rifle: (0, 0.03, -0.39) from `tools/blender/weapons/rifle.py:40`
/// - ak47: (0, -0.01, -0.30) from `tools/blender/weapons/ak47.py:53`
/// - smg: (0, -0.01, -0.18) from `tools/blender/weapons/smg.py:39`
pub fn off_hand_grip_for(weapon: &str) -> Option<Vec3> {
    match weapon {
        "bat" => Some(Vec3::new(0.0, 0.0, 0.10)),
        "axe" => Some(Vec3::new(0.0, 0.0, 0.20)),
        "crowbar" => Some(Vec3::new(0.0, 0.0, 0.20)),
        "katana" => Some(Vec3::new(0.0, 0.0, 0.05)),
        "shotgun" => Some(Vec3::new(0.0, 0.0, -0.47)),
        "rifle" => Some(Vec3::new(0.0, 0.03, -0.39)),
        "ak47" => Some(Vec3::new(0.0, -0.01, -0.30)),
        "smg" => Some(Vec3::new(0.0, -0.01, -0.18)),
        _ => None,
    }
}

/// Computes the world-space transform of a weapon model held in hand_R.
pub fn weapon_world_matrix(
    char_world: Mat4,
    rig: &Rig,
    skin: &[Mat4],
    hand_joint: usize,
    hold_pos: Vec3,
    hold_rot_deg: Vec3,
) -> Mat4 {
    let hand_model = rig.joint_model_matrix(skin, hand_joint);
    char_world * hand_model * weapon_hand_offset(hold_pos, hold_rot_deg)
}

/// Convenience function to compute weapon world matrix using default hold offsets.
pub fn weapon_world_matrix_named(
    char_world: Mat4,
    rig: &Rig,
    skin: &[Mat4],
    hand_joint: usize,
    weapon_name: &str,
) -> Mat4 {
    let (pos, rot) = weapon_hold_for(weapon_name);
    weapon_world_matrix(char_world, rig, skin, hand_joint, pos, rot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapon_hand_offset_is_finite_and_expected_distance() {
        let offset = weapon_hand_offset(Vec3::ZERO, Vec3::ZERO);
        let origin = offset.transform_point3(Vec3::ZERO);
        assert!(!origin.is_nan());
        // In hand space, the grip is translated by (0, 0.06, 0)
        assert!((origin - Vec3::new(0.0, 0.06, 0.0)).length() < 1e-5);
    }

    #[test]
    fn hold_offsets_for_known_weapons() {
        let (pos_bat, rot_bat) = weapon_hold_for("bat");
        assert_eq!(pos_bat, Vec3::ZERO);
        assert_eq!(rot_bat, Vec3::ZERO);

        let (pos_gun, rot_gun) = weapon_hold_for("shotgun");
        assert_eq!(pos_gun, Vec3::ZERO);
        assert_eq!(rot_gun, Vec3::new(90.0, 0.0, 0.0));
    }

    #[test]
    fn two_handed_weapons_and_off_hand_grip_points() {
        assert!(is_two_handed("bat"));
        assert!(is_two_handed("axe"));
        assert!(is_two_handed("crowbar"));
        assert!(is_two_handed("katana"));
        assert!(is_two_handed("shotgun"));
        assert!(is_two_handed("rifle"));
        assert!(is_two_handed("ak47"));
        assert!(is_two_handed("smg"));

        assert!(!is_two_handed("knife"));
        assert!(!is_two_handed("machete"));
        assert!(!is_two_handed("pistol"));

        assert_eq!(off_hand_grip_for("bat"), Some(Vec3::new(0.0, 0.0, 0.10)));
        assert_eq!(off_hand_grip_for("axe"), Some(Vec3::new(0.0, 0.0, 0.20)));
        assert_eq!(off_hand_grip_for("knife"), None);
        assert_eq!(off_hand_grip_for("pistol"), None);
    }
}

