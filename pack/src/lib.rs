//! Rotland 3D model pack binary format.
//!
//! Provides the shared data structures serialized by `baker` and loaded
//! directly into GPU memory by `rotland`. All geometry is stored as flat,
//! non-indexed triangle lists with palette-sampled vertex colors.

pub mod marker;
pub mod model;
pub mod skeleton;
pub mod vertex;

pub use marker::{ConvexCollider, Marker};
pub use model::{Model, ModelPack, PACK_VERSION};
pub use skeleton::{Channel, ChannelKind, Clip, Interpolation, Joint, Skeleton};
pub use vertex::PackVertex;
