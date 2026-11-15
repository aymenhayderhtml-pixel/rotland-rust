use glam::Vec3;
use rapier3d::control::{
    CharacterAutostep, CharacterLength, KinematicCharacterController,
};
use rapier3d::parry::query::details::ShapeCastOptions;
use rapier3d::prelude::*;

use crate::types::{MoveResult, GROUP_CHARACTER, GROUP_WORLD};
use crate::world::PhysicsWorld;

/// Kinematic capsule controller providing Godot-like `move_and_slide` behaviour.
///
/// Designed to match Godot 4's `CharacterBody3D` settings from `Player.tscn` and `PlayerController.cs`.
#[derive(Clone, Copy, Debug)]
pub struct CharacterMover {
    /// Capsule cylinder radius in meters (from `Player.tscn`: 0.35).
    pub radius: f32,
    /// Total capsule height from head to feet in meters (from `Player.tscn`: 1.8).
    pub height: f32,
    /// Maximum walkable slope angle in radians (from `Player.tscn`: 46 deg = ~0.80285 rad).
    pub max_floor_angle: f32,
    /// Maximum distance in meters to snap down to slopes when walking down (from `Player.tscn`: 0.3).
    pub floor_snap_length: f32,
    /// Maximum obstacle / stair step height to climb automatically (from `PlayerController.cs`: 0.35).
    pub max_step_height: f32,
    /// Small gap preserved between the character shape and obstacles for numerical stability.
    pub offset: f32,
}

impl Default for CharacterMover {
    fn default() -> Self {
        Self {
            radius: 0.35,
            height: 1.8,
            max_floor_angle: 0.8028514, // 46 degrees, matching Player.tscn / PlayerController.cs
            floor_snap_length: 0.3,
            max_step_height: 0.35,
            offset: 0.01,
        }
    }
}

impl CharacterMover {
    /// Create a new CharacterMover with custom capsule radius and height.
    pub fn new(radius: f32, height: f32) -> Self {
        Self {
            radius,
            height,
            ..Default::default()
        }
    }

    /// Moves the character along `velocity * dt`, sliding along walls and climbing walkable slopes.
    ///
    /// The character's origin is at its feet (y = 0 in local space).
    pub fn move_and_slide(
        &self,
        world: &PhysicsWorld,
        position: Vec3,
        mut velocity: Vec3,
        dt: f32,
    ) -> MoveResult {
        if dt <= 0.0 {
            return MoveResult {
                position,
                velocity,
                on_floor: false,
                floor_normal: Vec3::ZERO,
            };
        }

        let half_height = (self.height - 2.0 * self.radius) * 0.5;
        let capsule = SharedShape::capsule_y(half_height.max(0.001), self.radius);
        let center_y = position.y + self.height * 0.5;
        let character_pos = Pose::from_translation(Vector::new(position.x, center_y, position.z));

        let filter = QueryFilter::default().groups(
            InteractionGroups::none()
                .with_memberships(GROUP_CHARACTER)
                .with_filter(GROUP_WORLD),
        );
        let query_pipeline = world.raw.query_pipeline_with_filter(filter);

        let kcc = KinematicCharacterController {
            up: Vector::Y,
            offset: CharacterLength::Absolute(self.offset),
            slide: true,
            autostep: if self.max_step_height > 0.0 {
                Some(CharacterAutostep {
                    max_height: CharacterLength::Absolute(self.max_step_height),
                    min_width: CharacterLength::Absolute(self.radius),
                    include_dynamic_bodies: false,
                })
            } else {
                None
            },
            max_slope_climb_angle: self.max_floor_angle,
            min_slope_slide_angle: self.max_floor_angle,
            snap_to_ground: if self.floor_snap_length > 0.0 {
                Some(CharacterLength::Absolute(self.floor_snap_length))
            } else {
                None
            },
            ..Default::default()
        };

        let desired_translation = Vector::new(velocity.x, velocity.y, velocity.z) * dt;
        let mut floor_normal: Option<Vec3> = None;
        let mut wall_normals: Vec<Vec3> = Vec::new();

        let mvt = kcc.move_shape(
            dt,
            &query_pipeline,
            capsule.as_ref(),
            &character_pos,
            desired_translation,
            |collision| {
                let n = Vec3::new(
                    collision.hit.normal2.x,
                    collision.hit.normal2.y,
                    collision.hit.normal2.z,
                );
                if n.y >= self.max_floor_angle.cos() {
                    floor_normal = Some(n.normalize_or_zero());
                } else if n.length_squared() > 1e-4 {
                    wall_normals.push(n.normalize());
                }
            },
        );

        let on_floor = mvt.grounded;
        let final_floor_normal = if on_floor {
            if let Some(fn_norm) = floor_normal {
                fn_norm
            } else {
                // Downward probe to sample the floor normal if grounded without explicit manifold collision
                let new_center = character_pos.translation + mvt.translation;
                let new_pos = Pose::from_translation(new_center);
                if let Some((_handle, hit)) = query_pipeline.cast_shape(
                    &new_pos,
                    -Vector::Y,
                    capsule.as_ref(),
                    ShapeCastOptions {
                        target_distance: self.offset,
                        stop_at_penetration: false,
                        max_time_of_impact: self.floor_snap_length.max(0.1) + self.offset,
                        compute_impact_geometry_on_penetration: true,
                    },
                ) {
                    let n = Vec3::new(hit.normal2.x, hit.normal2.y, hit.normal2.z);
                    if n.y >= self.max_floor_angle.cos() {
                        n.normalize_or_zero()
                    } else {
                        Vec3::Y
                    }
                } else {
                    Vec3::Y
                }
            }
        } else {
            Vec3::ZERO
        };

        // Slide velocity along any walls hit during the frame
        for wn in wall_normals {
            let dot = velocity.dot(wn);
            if dot < 0.0 {
                velocity -= wn * dot;
            }
        }

        // Zero vertical velocity when standing on or landing on the floor (like Godot PlayerController.cs)
        if on_floor && velocity.y < 0.0 {
            velocity.y = 0.0;
        }

        let new_position = position
            + Vec3::new(
                mvt.translation.x,
                mvt.translation.y,
                mvt.translation.z,
            );

        MoveResult {
            position: new_position,
            velocity,
            on_floor,
            floor_normal: final_floor_normal,
        }
    }
}
