//! Static collision box management for parked cars.
//!
//! When cars are parked (!driven), a static box collider is added to the PhysicsWorld
//! so that both the player and zombies are blocked.
//! When a car is driven, its collider is removed so that its own wheel raycasts
//! do not collide with its body box. When the car moves or leaves the simulation,
//! its collider is moved or removed accordingly.

use std::collections::{HashMap, HashSet};
use glam::{Mat4, Vec3};
use physics::{ColliderHandle, PhysicsWorld};

/// Model-space box collider vertices for a car when models.pack is not available.
/// Dimensions: half_width = 0.8, sill = 0.2, roof = 1.4, half_length = 1.9.
pub fn default_car_box() -> Vec<[f32; 3]> {
    let hw = 0.8;
    let min_y = 0.2;
    let max_y = 1.4;
    let hl = 1.9;
    vec![
        [-hw, min_y, -hl],
        [hw, min_y, -hl],
        [-hw, max_y, -hl],
        [hw, max_y, -hl],
        [-hw, min_y, hl],
        [hw, min_y, hl],
        [-hw, max_y, hl],
        [hw, max_y, hl],
    ]
}

#[derive(Clone, Debug)]
struct CarSolidEntry {
    handle: ColliderHandle,
    position: Vec3,
    yaw: f32,
}

/// Manages static box colliders for parked (non-driven) cars in the PhysicsWorld.
#[derive(Clone, Debug)]
pub struct ParkedCarSolids {
    colliders: Vec<Vec<[f32; 3]>>,
    entries: HashMap<String, CarSolidEntry>,
}

impl Default for ParkedCarSolids {
    fn default() -> Self {
        Self::new()
    }
}

impl ParkedCarSolids {
    /// Creates a new manager with the default car box collider shape.
    pub fn new() -> Self {
        Self::with_colliders(vec![default_car_box()])
    }

    /// Creates a new manager with custom convex collider shapes.
    pub fn with_colliders(colliders: Vec<Vec<[f32; 3]>>) -> Self {
        Self {
            colliders,
            entries: HashMap::new(),
        }
    }

    /// Creates a manager by reading the collider shapes from the model pack
    /// (e.g. `car_hatchback_red` or `car_hatchback_blue`), falling back to the default box.
    pub fn from_pack(pack: &pack::ModelPack) -> Self {
        let colliders: Vec<Vec<[f32; 3]>> = pack
            .find("car_hatchback_red")
            .or_else(|| pack.find("car_hatchback_blue"))
            .map(|m| m.colliders.iter().map(|c| c.points.clone()).collect())
            .filter(|v: &Vec<Vec<[f32; 3]>>| !v.is_empty())
            .unwrap_or_else(|| vec![default_car_box()]);
        Self::with_colliders(colliders)
    }

    /// Synchronizes static colliders with the current cars list.
    ///
    /// - Parked (`!driven`) cars receive a static box collider.
    /// - Driven cars have their collider removed (so wheel rays never hit own box).
    /// - Cars whose transform changed are updated (remove + add).
    /// - Cars gone from the list have their colliders removed.
    pub fn sync(&mut self, physics: &mut PhysicsWorld, cars: &[(String, Mat4, bool)]) {
        let mut active_ids = HashSet::with_capacity(cars.len());

        for (id, world, driven) in cars {
            if *driven {
                // While driven, remove the solid box so wheel rays don't hit it
                if let Some(entry) = self.entries.remove(id) {
                    physics.remove(entry.handle);
                }
            } else {
                active_ids.insert(id.as_str());
                let (scale, rotation, translation) = world.to_scale_rotation_translation();
                let position = translation;
                let fwd = rotation * -Vec3::Z;
                let yaw = (-fwd.x).atan2(-fwd.z);
                let scale_val = if scale.x.abs() > 0.001 { scale.x } else { 1.0 };

                if let Some(entry) = self.entries.get(id) {
                    let moved = (entry.position - position).length_squared() > 1e-4
                        || (entry.yaw - yaw).abs() > 1e-4;
                    if moved {
                        let old_handle = entry.handle;
                        physics.remove(old_handle);
                        let handle = physics.add_static_tagged(&self.colliders, position, yaw, scale_val, game::car_tag(id));
                        self.entries.insert(
                            id.clone(),
                            CarSolidEntry {
                                handle,
                                position,
                                yaw,
                            },
                        );
                    }
                } else {
                    let handle = physics.add_static_tagged(&self.colliders, position, yaw, scale_val, game::car_tag(id));
                    self.entries.insert(
                        id.clone(),
                        CarSolidEntry {
                            handle,
                            position,
                            yaw,
                        },
                    );
                }
            }
        }

        // Remove any parked cars that are no longer present
        self.entries.retain(|id, entry| {
            if !active_ids.contains(id.as_str()) {
                physics.remove(entry.handle);
                false
            } else {
                true
            }
        });
    }

    /// Returns the number of currently active parked car colliders.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// True if a parked collider is active for the car with `id`.
    #[cfg(test)]
    pub fn contains(&self, id: &str) -> bool {
        self.entries.contains_key(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parked_car_solid_blocks_ray_and_clears_when_driven() {
        let mut physics = PhysicsWorld::new();
        let mut solids = ParkedCarSolids::new();

        let car_id = "car_hatchback_red_1".to_string();
        let parked_world = Mat4::from_translation(Vec3::new(10.0, 0.0, 20.0));

        // 1. Initially parked: sync adds a static collider
        let cars = vec![(car_id.clone(), parked_world, false)];
        solids.sync(&mut physics, &cars);
        assert_eq!(solids.count(), 1);
        assert!(solids.contains(&car_id));

        // Downward raycast aimed at the car roof (y = 1.4)
        let ray_origin = Vec3::new(10.0, 5.0, 20.0);
        let ray_dir = Vec3::NEG_Y;
        let hit = physics.raycast(ray_origin, ray_dir, 10.0);
        assert!(hit.is_some(), "Ray should hit the parked car's static box");
        let h = hit.unwrap();
        assert!((h.point.y - 1.4).abs() < 1e-3);
        assert!((h.distance - 3.6).abs() < 1e-3);

        // 2. Car becomes driven: sync removes the static collider
        let cars_driven = vec![(car_id.clone(), parked_world, true)];
        solids.sync(&mut physics, &cars_driven);
        assert_eq!(solids.count(), 0);
        assert!(!solids.contains(&car_id));

        // Raycast at the same location should now pass through (no hit)
        let hit_after = physics.raycast(ray_origin, ray_dir, 10.0);
        assert!(hit_after.is_none(), "Driven car collider must be removed, avoiding wheel ray hits");

        // 3. Car is parked again at a new position: moved collider
        let new_world = Mat4::from_translation(Vec3::new(30.0, 0.0, 50.0));
        let cars_moved = vec![(car_id.clone(), new_world, false)];
        solids.sync(&mut physics, &cars_moved);
        assert_eq!(solids.count(), 1);

        // Old position has no hit
        assert!(physics.raycast(ray_origin, ray_dir, 10.0).is_none());

        // New position has a hit
        let ray_new = Vec3::new(30.0, 5.0, 50.0);
        let hit_new = physics.raycast(ray_new, ray_dir, 10.0);
        assert!(hit_new.is_some(), "Moved car must be hit at its new location");

        // 4. Car despawned / gone from list
        solids.sync(&mut physics, &[]);
        assert_eq!(solids.count(), 0);
        assert!(physics.raycast(ray_new, ray_dir, 10.0).is_none());
    }

    #[test]
    fn test_horizontal_ray_hits_parked_car_box() {
        let mut physics = PhysicsWorld::new();
        let mut solids = ParkedCarSolids::new();

        let car_id = "test_car".to_string();
        let world = Mat4::from_translation(Vec3::ZERO);
        solids.sync(&mut physics, &[(car_id, world, false)]);

        // Horizontal ray along +X at half-height (y = 0.8), starting at x = -5.0
        let ray_origin = Vec3::new(-5.0, 0.8, 0.0);
        let ray_dir = Vec3::X;
        let hit = physics.raycast(ray_origin, ray_dir, 10.0);
        assert!(hit.is_some());
        let h = hit.unwrap();
        // Box half-width is 0.8, so it should hit at x = -0.8
        assert!((h.point.x - (-0.8)).abs() < 1e-3);
        assert!((h.distance - 4.2).abs() < 1e-3);
    }
}
