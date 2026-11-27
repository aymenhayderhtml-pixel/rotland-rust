//! Aim field-of-view numbers and the per-weapon aim FOV rule.

use game::WeaponDef;

/// Camera aim FOVs from scripts/Player/CameraRig.cs.
pub const THIRD_PERSON_AIM_FOV: f32 = 55.0;
pub const FIRST_PERSON_AIM_FOV: f32 = 50.0;

/// Aim FOV for a weapon: scope FOV if scoped (including scoped crossbows), else 50 deg for 1st-person or 55 deg for 3rd-person.
/// Returns None for unscoped non-guns (melee, unscoped bows).
pub fn aim_fov(weapon: &WeaponDef, first_person: bool) -> Option<f32> {
    if let Some(scope) = &weapon.scope {
        return Some(scope.fov);
    }
    if !weapon.is_gun() {
        return None;
    }
    if first_person {
        Some(FIRST_PERSON_AIM_FOV)
    } else {
        Some(THIRD_PERSON_AIM_FOV)
    }
}
