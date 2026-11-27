//! Placed models (trees, rocks, bushes, grass, buildings) drawn as instances, with Godot's distance LODs
//! and shadow rules (ScatterConfig.RangeFor, ChunkProps.ShouldCastShadow). Models load from the pack on
//! first use. Static instances use joint 0 of the bone list, which is always the identity.

mod batch_scratch;
mod library;
mod model;
mod rules;

pub(crate) mod lod;

pub use batch_scratch::recycle_batches;
pub use library::PropLibrary;
pub use model::Placed;
