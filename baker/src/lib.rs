//! Model asset baker library for Rotland.
//!
//! Transforms Blender `.glb` assets into compact `Model` and `ModelPack` data structures.

pub mod bake;
pub mod budget;
pub mod color;
pub mod mesh;
pub mod skeleton;
pub mod transform;

pub use bake::bake_glb;
pub use budget::{check_model_budget, Budgets};
