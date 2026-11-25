//! Direction helpers for Godot-style yaw angles.
//! Godot coordinate convention: yaw forward = (-sin(yaw), 0, -cos(yaw)).
//! Facing east (+X) is yaw -PI/2.

use glam::Vec3;

/// Forward unit vector for a given yaw angle in radians.
///
/// Under Godot coordinates:
/// - yaw 0 faces forward along -Z: `(0, 0, -1)`
/// - yaw -PI/2 faces east along +X: `(1, 0, 0)`
#[inline]
pub fn yaw_forward(yaw: f32) -> Vec3 {
    let (sy, cy) = yaw.sin_cos();
    Vec3::new(-sy, 0.0, -cy)
}

/// Right unit vector perpendicular to `yaw_forward(yaw)`.
///
/// Corresponds to `forward.cross(Vec3::Y)`:
/// - yaw 0 facing -Z has right vector +X: `(1, 0, 0)`
/// - yaw -PI/2 facing +X has right vector +Z: `(0, 0, 1)`
#[inline]
pub fn yaw_right(yaw: f32) -> Vec3 {
    let (sy, cy) = yaw.sin_cos();
    Vec3::new(cy, 0.0, -sy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, PI};

    #[test]
    fn test_yaw_forward_cardinal() {
        // yaw 0 -> (0, 0, -1)
        let f0 = yaw_forward(0.0);
        assert!((f0.x - 0.0).abs() < 1e-6);
        assert!((f0.y - 0.0).abs() < 1e-6);
        assert!((f0.z - (-1.0)).abs() < 1e-6);

        // yaw -PI/2 -> (+1, 0, 0)
        let f_east = yaw_forward(-FRAC_PI_2);
        assert!((f_east.x - 1.0).abs() < 1e-6);
        assert!((f_east.y - 0.0).abs() < 1e-6);
        assert!(f_east.z.abs() < 1e-6);

        // yaw PI/2 -> (-1, 0, 0) (west)
        let f_west = yaw_forward(FRAC_PI_2);
        assert!((f_west.x - (-1.0)).abs() < 1e-6);
        assert!(f_west.y.abs() < 1e-6);
        assert!(f_west.z.abs() < 1e-6);

        // yaw PI or -PI -> (0, 0, +1) (north)
        let f_north = yaw_forward(PI);
        assert!(f_north.x.abs() < 1e-6);
        assert!(f_north.y.abs() < 1e-6);
        assert!((f_north.z - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_yaw_right_cardinal() {
        // yaw 0 -> (1, 0, 0)
        let r0 = yaw_right(0.0);
        assert!((r0.x - 1.0).abs() < 1e-6);
        assert!(r0.y.abs() < 1e-6);
        assert!(r0.z.abs() < 1e-6);

        // yaw -PI/2 -> (0, 0, 1)
        let r_east = yaw_right(-FRAC_PI_2);
        assert!(r_east.x.abs() < 1e-6);
        assert!(r_east.y.abs() < 1e-6);
        assert!((r_east.z - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_orthogonality() {
        for deg in [0.0, 30.0, 45.0, 90.0, 135.0, 180.0, 270.0] {
            let rad = (deg as f32).to_radians();
            let f = yaw_forward(rad);
            let r = yaw_right(rad);
            assert!((f.length() - 1.0).abs() < 1e-6);
            assert!((r.length() - 1.0).abs() < 1e-6);
            assert!(f.dot(r).abs() < 1e-6);
        }
    }
}
