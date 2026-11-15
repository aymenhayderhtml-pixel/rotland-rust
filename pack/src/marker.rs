use serde::{Deserialize, Serialize};

/// Named empty node exported from Blender (e.g. `loot_1`, `spawn_1`, `grip_L`, `door_1`).
///
/// Stores model-space transform (accumulated from the node hierarchy).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Marker {
    /// Name of the marker empty.
    pub name: String,
    /// Model-space translation [x, y, z].
    pub translation: [f32; 3],
    /// Model-space orientation quaternion [x, y, z, w].
    pub rotation: [f32; 4],
    /// Model-space scale [x, y, z].
    pub scale: [f32; 3],
}

/// A convex collision volume extracted from a `-convcolonly` or `-col` mesh node.
///
/// Vertices are stored in model space and passed directly to physics engine
/// convex hull builders (such as Rapier3D).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConvexCollider {
    /// Object name of the collision shape.
    pub name: String,
    /// Convex hull points in model space.
    pub points: Vec<[f32; 3]>,
}
