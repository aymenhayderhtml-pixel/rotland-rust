//! Loudness and left/right placement for a sound at a point, heard from the camera. Pure math so it is
//! tested without a sound card.

use glam::Vec3;

/// Volume change in decibels for a sound `distance` metres away that fades out at `max_distance`
/// (close to Godot's inverse-distance fade with a hard cut at MaxDistance). None = too far to hear.
pub fn distance_db(distance: f32, max_distance: f32) -> Option<f32> {
    if distance >= max_distance {
        return None;
    }
    let near = 2.0_f32.min(max_distance * 0.5);
    if distance <= near {
        return Some(0.0);
    }
    // Linear gain from 1 at `near` to 0 at the edge, squared for a softer tail
    let t = 1.0 - (distance - near) / (max_distance - near);
    Some(20.0 * (t * t).max(1e-3).log10())
}

/// -1 = left speaker, 1 = right speaker, from where the sound is relative to the camera's facing.
pub fn pan(listener: Vec3, forward: Vec3, at: Vec3) -> f32 {
    let right = forward.cross(Vec3::Y).normalize_or_zero();
    let to = (at - listener).normalize_or_zero();
    (to.dot(right) * 0.8).clamp(-1.0, 1.0)
}

/// Linear volume (0..1) to decibels, for the settings sliders.
pub fn linear_to_db(v: f32) -> f32 {
    if v <= 0.001 { -60.0 } else { 20.0 * v.log10() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn near_is_full_far_is_silent() {
        assert_eq!(distance_db(1.0, 60.0), Some(0.0));
        assert!(distance_db(60.0, 60.0).is_none());
        let mid = distance_db(30.0, 60.0).unwrap();
        assert!(mid < -3.0 && mid > -20.0, "{mid}");
    }

    #[test]
    fn right_side_pans_right() {
        let p = pan(Vec3::ZERO, Vec3::NEG_Z, Vec3::new(5.0, 0.0, 0.0));
        assert!(p > 0.5);
        assert!(pan(Vec3::ZERO, Vec3::NEG_Z, Vec3::new(0.0, 0.0, -5.0)).abs() < 1e-5);
    }
}
