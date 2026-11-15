use serde::{Deserialize, Serialize};

/// Hierarchical bone skeleton for skinned models.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Skeleton {
    /// List of joints. Joint index corresponds directly to `PackVertex.bone`.
    pub joints: Vec<Joint>,
}

/// A single bone joint in a skeleton.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Joint {
    /// Bone name, e.g. "root", "hips", "head".
    pub name: String,
    /// Parent joint index within `Skeleton.joints`, or `None` for the root joint.
    pub parent: Option<u16>,
    /// 4x4 inverse bind matrix (column-major order).
    pub inverse_bind: [f32; 16],
    /// Rest pose local translation [x, y, z].
    pub rest_translation: [f32; 3],
    /// Rest pose local rotation quaternion [x, y, z, w].
    pub rest_rotation: [f32; 4],
    /// Rest pose local scale [x, y, z].
    pub rest_scale: [f32; 3],
}

/// Skeletal animation clip.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    /// Clip name, e.g. "walk", "idle", "zombie_shamble".
    pub name: String,
    /// Duration of the clip in seconds.
    pub duration: f32,
    /// Animation tracks animating individual joints.
    pub channels: Vec<Channel>,
}

/// A single animated property track targeting a joint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Channel {
    /// Target joint index in `Skeleton.joints`.
    pub joint: u16,
    /// Target transformation component.
    pub kind: ChannelKind,
    /// Keyframe interpolation mode.
    pub interpolation: Interpolation,
    /// Keyframe timestamps in seconds.
    pub times: Vec<f32>,
    /// Keyframe values. Translations and scales have [x, y, z, 0.0]; rotations have [x, y, z, w].
    pub values: Vec<[f32; 4]>,
}

/// Target transformation type for an animation track.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelKind {
    Translation,
    Rotation,
    Scale,
}

/// Interpolation curve between animation keyframes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Interpolation {
    Step,
    Linear,
}
