use glam::Vec3;
use rapier3d::prelude::Group;

/// Collision group for static world objects (terrain chunks, rocks, trees, buildings).
pub const GROUP_WORLD: Group = Group::GROUP_1;

/// Collision group for characters (player, zombies, NPCs). Characters do not collide with each other.
pub const GROUP_CHARACTER: Group = Group::GROUP_2;

/// Result of a raycast against the physics world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    /// Point in world space where the ray hit.
    pub point: Vec3,
    /// Surface normal at the hit point.
    pub normal: Vec3,
    /// Distance along the ray from the origin to the hit point.
    pub distance: f32,
}

/// Result of character movement after sliding against obstacles and floor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoveResult {
    /// New world-space character position (at feet).
    pub position: Vec3,
    /// Velocity updated to slide along walls/floor.
    pub velocity: Vec3,
    /// Whether the character is currently standing on a walkable floor.
    pub on_floor: bool,
    /// Surface normal of the floor under feet, or zero if in the air.
    pub floor_normal: Vec3,
}
