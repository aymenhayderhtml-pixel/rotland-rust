use glam::{IVec2, Vec3};
use rapier3d::prelude::*;

use crate::types::{Hit, GROUP_CHARACTER, GROUP_WORLD};

/// Dimensions of world chunks, matching `world::WorldGenerator`.
pub const CHUNK_SIZE: f32 = 64.0;
pub const CHUNK_CELL: f32 = 2.0;
pub const CHUNK_SAMPLES: usize = 33;
pub const CHUNK_CELLS: usize = CHUNK_SAMPLES - 1;

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
            .collision_groups(
                InteractionGroups::none()
                    .with_memberships(GROUP_WORLD)
                    .with_filter(GROUP_CHARACTER | GROUP_WORLD),
            )
            .build();

        let handle = self.raw.insert_collider(collider, None);
        self.update_broad_phase();
        handle
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
        if colliders.is_empty() {
            return ColliderHandle::invalid();
        }

        let mut parts = Vec::new();
        for points in colliders {
            if points.len() < 4 {
                continue;
            }
            let scaled: Vec<Vector> = points
                .iter()
                .map(|p| Vector::new(p[0] * scale, p[1] * scale, p[2] * scale))
                .collect();
            if let Some(shape) = SharedShape::convex_hull(&scaled) {
                parts.push((Pose::IDENTITY, shape));
            }
        }

        if parts.is_empty() {
            return ColliderHandle::invalid();
        }

        let shared_shape = if parts.len() == 1 {
            parts.pop().unwrap().1
        } else {
            SharedShape::compound(parts)
        };

        let translation = Vector::new(position.x, position.y, position.z);
        let rotation = Rotation::from_scaled_axis(Vector::Y * yaw);
        let pose = Pose::from_parts(translation, rotation);

        let collider = ColliderBuilder::new(shared_shape)
            .position(pose)
            .collision_groups(
                InteractionGroups::none()
                    .with_memberships(GROUP_WORLD)
                    .with_filter(GROUP_CHARACTER | GROUP_WORLD),
            )
            .build();

        let handle = self.raw.insert_collider(collider, None);
        self.update_broad_phase();
        handle
    }

    /// Remove a collider by handle. Returns true if removed, false if handle was invalid.
    pub fn remove(&mut self, handle: ColliderHandle) -> bool {
        if handle == ColliderHandle::invalid() {
            return false;
        }
        let removed = self.raw.remove_collider(handle).is_some();
        if removed {
            self.update_broad_phase();
        }
        removed
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
        let ray = Ray::new(
            Vector::new(origin.x, origin.y, origin.z),
            Vector::new(dir.x, dir.y, dir.z),
        );
        let filter = QueryFilter::default().groups(
            InteractionGroups::none()
                .with_memberships(GROUP_CHARACTER | GROUP_WORLD)
                .with_filter(GROUP_WORLD),
        );

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
