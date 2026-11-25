//! Zombie damage logic for parked and occupied cars.
//!
//! When zombies in Chase or Attack state get within reach of a car,
//! they deal damage to the car's condition until it is wrecked.

use glam::{Quat, Vec3};

/// A car target bounding box for zombie reach and damage calculation.
#[derive(Clone, Debug, PartialEq)]
pub struct CarDamageBox {
    pub id: String,
    pub position: Vec3,
    pub rotation: Quat,
    pub half_size: Vec3,
    pub is_wrecked: bool,
}

impl CarDamageBox {
    pub fn new(
        id: impl Into<String>,
        position: Vec3,
        rotation: Quat,
        half_size: Vec3,
        is_wrecked: bool,
    ) -> Self {
        Self {
            id: id.into(),
            position,
            rotation,
            half_size,
            is_wrecked,
        }
    }
}

/// Checks if a zombie position is within attack reach of a car.
///
/// Uses the car's oriented horizontal bounding box (half_size.x = half_width,
/// half_size.z = half_length). A zombie is within reach if its horizontal distance
/// to the box edge is <= `reach` and vertical offset is within reasonable bounds.
pub fn is_zombie_in_reach(
    car_pos: Vec3,
    car_rot: Quat,
    half_size: Vec3,
    reach: f32,
    zombie_pos: Vec3,
) -> bool {
    let local = car_rot.inverse() * (zombie_pos - car_pos);
    let dx = (local.x.abs() - half_size.x).max(0.0);
    let dz = (local.z.abs() - half_size.z).max(0.0);
    let horizontal_dist = (dx * dx + dz * dz).sqrt();
    let vertical_dist = local.y;
    // Car origin sits on the ground between axles; car height is ~half_size.y * 2.0.
    // Zombie origin is at feet. Allow from below ground (-reach) up to roof + reach.
    let vertical_ok = vertical_dist >= -reach && vertical_dist <= half_size.y * 2.0 + reach;
    horizontal_dist <= reach && vertical_ok
}

/// Counts how many attacking/chasing zombies are within reach of a car.
pub fn count_zombies_in_reach(
    car_pos: Vec3,
    car_rot: Quat,
    half_size: Vec3,
    reach: f32,
    zombies: &[Vec3],
) -> usize {
    zombies
        .iter()
        .filter(|&&z_pos| is_zombie_in_reach(car_pos, car_rot, half_size, reach, z_pos))
        .count()
}

/// Pure function calculating zombie damage per car.
///
/// Takes a slice of cars, zombie positions, damage per hit, hit interval in seconds,
/// attack reach, and delta time `dt`.
/// Returns a list of `(car_id, damage)` pairs.
///
/// - Wrecked cars receive 0.0 damage.
/// - Zombies out of reach contribute 0.0 damage.
/// - Each zombie in reach deals `(hit_damage / hit_seconds) * dt` damage.
pub fn calculate_zombie_damage(
    cars: &[CarDamageBox],
    zombies: &[Vec3],
    hit_damage: f32,
    hit_seconds: f32,
    reach: f32,
    dt: f32,
) -> Vec<(String, f32)> {
    if hit_damage <= 0.0 || hit_seconds <= 0.0 || dt <= 0.0 {
        return cars.iter().map(|c| (c.id.clone(), 0.0)).collect();
    }

    let damage_rate = hit_damage / hit_seconds;
    cars.iter()
        .map(|car| {
            if car.is_wrecked {
                (car.id.clone(), 0.0)
            } else {
                let count = count_zombies_in_reach(
                    car.position,
                    car.rotation,
                    car.half_size,
                    reach,
                    zombies,
                );
                let damage = count as f32 * damage_rate * dt;
                (car.id.clone(), damage)
            }
        })
        .collect()
}
