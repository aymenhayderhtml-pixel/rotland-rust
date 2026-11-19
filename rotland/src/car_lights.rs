//! Car headlights: produces spot lights for vehicle headlights illuminating the road ahead.
//!
//! Port of `scripts/Vehicles/CarLights.cs` and the vehicle model contract (`docs/VEHICLE_CONTRACT.md`).
//! The two headlights sit at `headlight_L` and `headlight_R` empties on the vehicle model,
//! pointing forward (-Z in Godot coordinates) and slightly tilted down (-Y) so the beam
//! lights the road surface ahead of the vehicle.

use engine::render::SpotLight;
use game::VehicleConfigData;
use glam::{Mat4, Vec3};

/// Headlight range in metres (off-road night visibility; upgraded from CarLights.cs Range = 25f).
pub const HEADLIGHT_RANGE: f32 = 38.0;

/// Headlight cone half-angle in degrees (upgraded from CarLights.cs ConeDegrees = 30f).
pub const HEADLIGHT_CONE_DEG: f32 = 34.0;

/// Headlight light energy / brightness (upgraded from CarLights.cs Energy = 2.5f).
pub const HEADLIGHT_ENERGY: f32 = 3.2;

/// Warm white light colour (sRGB) for vehicle headlights.
pub const HEADLIGHT_COLOR: Vec3 = Vec3::new(1.0, 0.98, 0.90);

/// Default contract position for headlight_L on hatchback model (docs/VEHICLE_CONTRACT.md).
#[cfg(test)]
pub const DEFAULT_HEADLIGHT_L: Vec3 = Vec3::new(-0.60, 0.80, -1.98);

/// Default contract position for headlight_R on hatchback model (docs/VEHICLE_CONTRACT.md).
#[cfg(test)]
pub const DEFAULT_HEADLIGHT_R: Vec3 = Vec3::new(0.60, 0.80, -1.98);

/// Points forward (-Z) and tilted down onto the road (-Y).
const LOCAL_DIR: Vec3 = Vec3::new(0.0, -0.14, -0.990_15);

/// Computes the left and right spot lights for car headlights in world space.
///
/// When `lights_on` is false, returns `[None, None]`.
/// When `lights_on` is true, returns `[Some(left_spot), Some(right_spot)]` with positions
/// transformed by `world` and directions pointing forward-down in world space.
pub fn headlight_spots(
    world: Mat4,
    left_marker: Vec3,
    right_marker: Vec3,
    lights_on: bool,
    _cfg: &VehicleConfigData,
) -> [Option<SpotLight>; 2] {
    if !lights_on {
        return [None, None];
    }

    let left_pos = world.transform_point3(left_marker);
    let right_pos = world.transform_point3(right_marker);
    let dir = world.transform_vector3(LOCAL_DIR).normalize_or_zero();

    let left_spot = SpotLight {
        position: left_pos,
        direction: dir,
        range: HEADLIGHT_RANGE,
        angle_deg: HEADLIGHT_CONE_DEG,
        color: HEADLIGHT_COLOR,
        energy: HEADLIGHT_ENERGY,
    };

    let right_spot = SpotLight {
        position: right_pos,
        direction: dir,
        range: HEADLIGHT_RANGE,
        angle_deg: HEADLIGHT_CONE_DEG,
        color: HEADLIGHT_COLOR,
        energy: HEADLIGHT_ENERGY,
    };

    [Some(left_spot), Some(right_spot)]
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_lights_off_returns_none() {
        let cfg = VehicleConfigData::default();
        let spots = headlight_spots(Mat4::IDENTITY, DEFAULT_HEADLIGHT_L, DEFAULT_HEADLIGHT_R, false, &cfg);
        assert_eq!(spots, [None, None]);
    }

    #[test]
    fn test_identity_matrix_positions_and_direction() {
        let cfg = VehicleConfigData::default();
        let spots = headlight_spots(Mat4::IDENTITY, DEFAULT_HEADLIGHT_L, DEFAULT_HEADLIGHT_R, true, &cfg);

        let left = spots[0].expect("left headlight must be Some");
        let right = spots[1].expect("right headlight must be Some");

        // Positions match markers under identity matrix
        assert_eq!(left.position, DEFAULT_HEADLIGHT_L);
        assert_eq!(right.position, DEFAULT_HEADLIGHT_R);

        // Parameters upgraded for off-road night visibility
        assert_eq!(left.range, 38.0);
        assert_eq!(left.angle_deg, 34.0);
        assert_eq!(left.energy, 3.2);
        assert_eq!(left.color, Vec3::new(1.0, 0.98, 0.90));

        assert_eq!(right.range, 38.0);
        assert_eq!(right.angle_deg, 34.0);
        assert_eq!(right.energy, 3.2);
        assert_eq!(right.color, Vec3::new(1.0, 0.98, 0.90));

        // Direction tilts down onto the road while pointing forward (-Z)
        let expected_dir = Vec3::new(0.0, -0.14, -0.990_15).normalize();
        assert!((left.direction - expected_dir).length() < 1e-6);
        assert!((right.direction - expected_dir).length() < 1e-6);
        assert!(left.direction.y < 0.0); // tilted down onto the road
        assert!(left.direction.z < 0.0); // pointing forward
    }

    #[test]
    fn test_translation_follows_world_matrix() {
        let cfg = VehicleConfigData::default();
        let offset = Vec3::new(10.0, 5.0, -20.0);
        let world = Mat4::from_translation(offset);

        let spots = headlight_spots(world, DEFAULT_HEADLIGHT_L, DEFAULT_HEADLIGHT_R, true, &cfg);
        let left = spots[0].unwrap();
        let right = spots[1].unwrap();

        // Traced hand calculation:
        // Left: (-0.60 + 10.0, 0.80 + 5.0, -1.98 - 20.0) = (9.40, 5.80, -21.98)
        // Right: (0.60 + 10.0, 0.80 + 5.0, -1.98 - 20.0) = (10.60, 5.80, -21.98)
        assert!((left.position - Vec3::new(9.40, 5.80, -21.98)).length() < 1e-5);
        assert!((right.position - Vec3::new(10.60, 5.80, -21.98)).length() < 1e-5);

        // Direction is unaffected by pure translation
        let expected_dir = Vec3::new(0.0, -0.14, -0.990_15).normalize();
        assert!((left.direction - expected_dir).length() < 1e-6);
    }

    #[test]
    fn test_rotation_follows_car_yaw() {
        let cfg = VehicleConfigData::default();
        // 180 degrees turn around Y (PI): facing opposite direction (+Z forward instead of -Z)
        let world = Mat4::from_rotation_y(PI);

        let spots = headlight_spots(world, DEFAULT_HEADLIGHT_L, DEFAULT_HEADLIGHT_R, true, &cfg);
        let left = spots[0].unwrap();
        let right = spots[1].unwrap();

        // 180-deg rotation flips X and Z:
        // Left: (-0.60, 0.80, -1.98) -> (0.60, 0.80, 1.98)
        // Right: (0.60, 0.80, -1.98) -> (-0.60, 0.80, 1.98)
        assert!((left.position - Vec3::new(0.60, 0.80, 1.98)).length() < 1e-5);
        assert!((right.position - Vec3::new(-0.60, 0.80, 1.98)).length() < 1e-5);

        // Direction flipped in Z (+Z), still pitched down (-Y):
        let expected_dir = Vec3::new(0.0, -0.14, 0.990_15).normalize();
        assert!((left.direction - expected_dir).length() < 1e-5);
        assert!((right.direction - expected_dir).length() < 1e-5);
    }
}
