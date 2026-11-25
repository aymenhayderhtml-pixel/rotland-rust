use glam::{IVec2, Vec3};
use rapier3d::prelude::*;

use crate::types::{Hit, GROUP_CHARACTER, GROUP_WORLD};

/// Dimensions of world chunks, matching `world::WorldGenerator`.
pub const CHUNK_SIZE: f32 = 64.0;
pub const CHUNK_CELL: f32 = 2.0;
pub const CHUNK_SAMPLES: usize = 33;
pub const CHUNK_CELLS: usize = CHUNK_SAMPLES - 1;

/// Collision groups for static world geometry (ground and static props).
/// Static-vs-static pairs are disabled so Rapier never tests them during broad/narrow phase.
fn world_collision_groups() -> InteractionGroups {
    InteractionGroups::none()
        .with_memberships(GROUP_WORLD)
        .with_filter(GROUP_CHARACTER)
}

fn world_pose(position: Vec3, yaw: f32) -> Pose {
    let translation = Vector::new(position.x, position.y, position.z);
    let rotation = Rotation::from_scaled_axis(Vector::Y * yaw);
    Pose::from_parts(translation, rotation)
}

fn push_convex_parts(
    parts: &mut Vec<(Pose, SharedShape)>,
    colliders: &[Vec<[f32; 3]>],
    pose: Pose,
    scale: f32,
) {
    for points in colliders {
        if points.len() < 4 {
            continue;
        }
        let scaled: Vec<Vector> = points
            .iter()
            .map(|p| Vector::new(p[0] * scale, p[1] * scale, p[2] * scale))
            .collect();
        if let Some(shape) = SharedShape::convex_hull(&scaled) {
            parts.push((pose, shape));
        }
    }
}

/// Physics simulation world managing static terrain, static props/buildings, and scene queries.
pub struct PhysicsWorld {
    /// Internal Rapier physics world bundle.
    pub raw: rapier3d::pipeline::PhysicsWorld,
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsWorld {
    /// Create a new, empty physics world.
    pub fn new() -> Self {
        Self {
            raw: rapier3d::pipeline::PhysicsWorld::default(),
        }
    }

    /// Refresh broad-phase and narrow-phase structures so queries immediately see changes.
    pub fn update_broad_phase(&mut self) {
        self.raw.detect_collisions(&(), &());
    }

    /// Add collision for one ground chunk using a trimesh matching the exact rendered triangles.
    ///
    /// The triangles are split `a-b-c` / `b-d-c` per 2m cell, exactly matching
    /// `world::WorldGenerator::build_chunk_ground`.
    pub fn add_ground_chunk(&mut self, coord: IVec2, heights: &[f32]) -> ColliderHandle {
        let handle = self.add_ground_chunk_deferred(coord, heights);
        self.update_broad_phase();
        handle
    }

    /// Add collision for one ground chunk without refreshing the broad-phase yet.
    /// Caller must call `update_broad_phase()` before running queries.
    pub fn add_ground_chunk_deferred(&mut self, coord: IVec2, heights: &[f32]) -> ColliderHandle {
        assert_eq!(
            heights.len(),
            CHUNK_SAMPLES * CHUNK_SAMPLES,
            "heights slice must have {} samples",
            CHUNK_SAMPLES * CHUNK_SAMPLES
        );

        let n = CHUNK_SAMPLES;
        let cells = CHUNK_CELLS;
        let ox = coord.x as f32 * CHUNK_SIZE;
        let oz = coord.y as f32 * CHUNK_SIZE;

        let mut vertices = Vec::with_capacity(n * n);
        for j in 0..n {
            for i in 0..n {
                let idx = j * n + i;
                let x = ox + i as f32 * CHUNK_CELL;
                let y = heights[idx];
                let z = oz + j as f32 * CHUNK_CELL;
                vertices.push(Vector::new(x, y, z));
            }
        }

        let mut indices = Vec::with_capacity(cells * cells * 2);
        for j in 0..cells {
            for i in 0..cells {
                let a = (j * n + i) as u32;
                let (b, c, d) = (a + 1, a + n as u32, a + n as u32 + 1);
                indices.push([a, b, c]);
                indices.push([b, d, c]);
            }
        }

        let shape = SharedShape::trimesh(vertices, indices)
            .expect("ground trimesh must be valid");

        let collider = ColliderBuilder::new(shape)
            .collision_groups(world_collision_groups())
            .build();

        self.raw.insert_collider(collider, None)
    }

    /// Add a placed static model (props, rocks, trees, buildings) with convex colliders.
    ///
    /// If there are multiple convex shapes, they are combined into a single compound collider.
    /// Returns the handle of the created collider, or an invalid handle if no colliders exist.
    pub fn add_static(
        &mut self,
        colliders: &[Vec<[f32; 3]>],
        position: Vec3,
        yaw: f32,
        scale: f32,
    ) -> ColliderHandle {
        self.add_static_tagged(colliders, position, yaw, scale, 0)
    }

    /// Like `add_static`, without refreshing the broad-phase yet.
    pub fn add_static_deferred(
        &mut self,
        colliders: &[Vec<[f32; 3]>],
        position: Vec3,
        yaw: f32,
        scale: f32,
    ) -> ColliderHandle {
        self.add_static_tagged_deferred(colliders, position, yaw, scale, 0)
    }

    /// Like `add_static`, with a tag (non-zero) that `raycast_except` can skip: a parked car's box
    /// must not be seen by that same car's wheel rays.
    pub fn add_static_tagged(
        &mut self,
        colliders: &[Vec<[f32; 3]>],
        position: Vec3,
        yaw: f32,
        scale: f32,
        tag: u128,
    ) -> ColliderHandle {
        let handle = self.add_static_tagged_deferred(colliders, position, yaw, scale, tag);
        if handle != ColliderHandle::invalid() {
            self.update_broad_phase();
        }
        handle
    }

    /// Like `add_static_tagged`, without refreshing the broad-phase yet.
    /// Caller must call `update_broad_phase()` before running queries.
    pub fn add_static_tagged_deferred(
        &mut self,
        colliders: &[Vec<[f32; 3]>],
        position: Vec3,
        yaw: f32,
        scale: f32,
        tag: u128,
    ) -> ColliderHandle {
        if colliders.is_empty() {
            return ColliderHandle::invalid();
        }

        let mut parts = Vec::new();
        push_convex_parts(&mut parts, colliders, Pose::IDENTITY, scale);

        if parts.is_empty() {
            return ColliderHandle::invalid();
        }

        let shared_shape = if parts.len() == 1 {
            parts.pop().unwrap().1
        } else {
            SharedShape::compound(parts)
        };

        let collider = ColliderBuilder::new(shared_shape)
            .position(world_pose(position, yaw))
            .collision_groups(world_collision_groups())
            .user_data(tag)
            .build();

        self.raw.insert_collider(collider, None)
    }

    /// Combine multiple placed static models into a single compound collider, without refreshing
    /// the broad-phase yet.
    /// Caller must call `update_broad_phase()` before running queries.
    pub fn add_static_compound_deferred<'a, I>(&mut self, items: I) -> ColliderHandle
    where
        I: IntoIterator<Item = (&'a [Vec<[f32; 3]>], Vec3, f32, f32)>,
    {
        let mut parts = Vec::new();
        for (colliders, position, yaw, scale) in items {
            if colliders.is_empty() {
                continue;
            }
            push_convex_parts(&mut parts, colliders, world_pose(position, yaw), scale);
        }

        if parts.is_empty() {
            return ColliderHandle::invalid();
        }

        let shared_shape = SharedShape::compound(parts);
        let collider = ColliderBuilder::new(shared_shape)
            .collision_groups(world_collision_groups())
            .build();

        self.raw.insert_collider(collider, None)
    }

    /// Remove a collider by handle. Returns true if removed, false if handle was invalid.
    pub fn remove(&mut self, handle: ColliderHandle) -> bool {
        let removed = self.remove_deferred(handle);
        if removed {
            self.update_broad_phase();
        }
        removed
    }

    /// Remove a collider by handle without refreshing the broad-phase yet.
    /// Caller must call `update_broad_phase()` before running queries.
    pub fn remove_deferred(&mut self, handle: ColliderHandle) -> bool {
        if handle == ColliderHandle::invalid() {
            return false;
        }
        self.raw.remove_collider(handle).is_some()
    }

    /// Number of active colliders in the physics world.
    pub fn collider_count(&self) -> usize {
        self.raw.colliders.len()
    }

    /// True if there are no colliders in the world.
    pub fn is_empty(&self) -> bool {
        self.raw.colliders.is_empty()
    }

    /// Cast a ray down from above to find the ground height at (x, z).
    pub fn ground_height(&self, x: f32, z: f32) -> Option<f32> {
        let origin = Vec3::new(x, 1000.0, z);
        let dir = Vec3::NEG_Y;
        let hit = self.raycast(origin, dir, 2000.0)?;
        Some(hit.point.y)
    }

    /// General raycast against static world geometry.
    pub fn raycast(&self, origin: Vec3, dir: Vec3, max_distance: f32) -> Option<Hit> {
        self.raycast_except(origin, dir, max_distance, 0)
    }

    /// Raycast that ignores colliders added with this tag (0 ignores nothing).
    pub fn raycast_except(&self, origin: Vec3, dir: Vec3, max_distance: f32, skip_tag: u128) -> Option<Hit> {
        let ray = Ray::new(
            Vector::new(origin.x, origin.y, origin.z),
            Vector::new(dir.x, dir.y, dir.z),
        );
        let filter = QueryFilter::default().groups(
            InteractionGroups::none()
                .with_memberships(GROUP_CHARACTER | GROUP_WORLD)
                .with_filter(GROUP_WORLD),
        );
        let skip = |_: ColliderHandle, c: &Collider| c.user_data != skip_tag;
        let filter = if skip_tag == 0 { filter } else { filter.predicate(&skip) };

        let (handle, intersection) =
            self.raw
                .query_pipeline_with_filter(filter)
                .cast_ray_and_get_normal(&ray, max_distance, true)?;

        let toi = intersection.time_of_impact;
        let hit_point = origin + dir * toi;
        let hit_normal = Vec3::new(
            intersection.normal.x,
            intersection.normal.y,
            intersection.normal.z,
        );

        let _ = handle;
        Some(Hit {
            point: hit_point,
            normal: hit_normal.normalize_or_zero(),
            distance: toi,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deferred_colliders_match_immediate_after_broad_phase() {
        let mut world = PhysicsWorld::new();
        let heights = vec![2.0_f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
        let ground = world.add_ground_chunk_deferred(IVec2::ZERO, &heights);
        let box_pts = vec![
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, -1.0],
            [1.0, 0.0, 1.0],
            [-1.0, 0.0, 1.0],
            [-1.0, 3.0, -1.0],
            [1.0, 3.0, -1.0],
            [1.0, 3.0, 1.0],
            [-1.0, 3.0, 1.0],
        ];
        let box_h = world.add_static_deferred(&[box_pts], Vec3::new(10.0, 2.0, 10.0), 0.0, 1.0);
        world.update_broad_phase();

        let h_ground = world.ground_height(4.0, 4.0);
        assert!(h_ground.is_some_and(|y| (y - 2.0).abs() < 1e-3));
        let h_box = world.ground_height(10.0, 10.0);
        assert!(h_box.is_some_and(|y| (y - 5.0).abs() < 1e-3));

        assert!(world.remove_deferred(box_h));
        assert!(world.remove_deferred(ground));
        world.update_broad_phase();
        assert!(world.is_empty());
    }

    #[test]
    fn compound_static_colliders_hit_each_part_and_remove_together() {
        let mut world = PhysicsWorld::new();
        let box_pts = vec![
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, -1.0],
            [1.0, 0.0, 1.0],
            [-1.0, 0.0, 1.0],
            [-1.0, 3.0, -1.0],
            [1.0, 3.0, -1.0],
            [1.0, 3.0, 1.0],
            [-1.0, 3.0, 1.0],
        ];
        let shapes = [box_pts];
        let items = [
            (shapes.as_slice(), Vec3::new(10.0, 2.0, 10.0), 0.0, 1.0),
            (
                shapes.as_slice(),
                Vec3::new(25.0, 4.0, 15.0),
                std::f32::consts::FRAC_PI_2,
                2.0,
            ),
        ];
        let compound_h = world.add_static_compound_deferred(items);
        assert_ne!(compound_h, ColliderHandle::invalid());
        assert_eq!(world.collider_count(), 1);
        world.update_broad_phase();

        let h1 = world.ground_height(10.0, 10.0);
        assert!(h1.is_some_and(|y| (y - 5.0).abs() < 1e-3));
        let h2 = world.ground_height(25.0, 15.0);
        assert!(h2.is_some_and(|y| (y - 10.0).abs() < 1e-3));
        assert!(world.ground_height(17.0, 12.0).is_none());

        assert!(world.remove_deferred(compound_h));
        world.update_broad_phase();
        assert!(world.is_empty());
    }

    #[test]
    fn static_world_colliders_disable_static_vs_static_filter() {
        let mut world = PhysicsWorld::new();
        let heights = vec![0.0_f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
        let ground = world.add_ground_chunk_deferred(IVec2::ZERO, &heights);
        let box_pts = vec![
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, -1.0],
            [1.0, 0.0, 1.0],
            [-1.0, 0.0, 1.0],
            [-1.0, 1.0, -1.0],
            [1.0, 1.0, -1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
        ];
        let prop = world.add_static_deferred(&[box_pts], Vec3::new(5.0, 0.0, 5.0), 0.0, 1.0);

        for h in [ground, prop] {
            let c = world.raw.colliders.get(h).unwrap();
            assert_eq!(c.collision_groups(), world_collision_groups());
            assert_eq!(c.active_events(), ActiveEvents::empty());
        }
    }
}
