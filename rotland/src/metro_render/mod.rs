//! New Halden placeholder towers: near meshes per loaded chunk, their colliders, and the far
//! skyline batch. Geometry is pure (`boxes.rs`); only `near.rs` and `skyline.rs` touch the GPU.
//!
//! Wiring entry points: `near::near_tower_mesh`, `hulls::tower_hulls`, `skyline::SkylineBatch`.

pub mod boxes;
pub mod hulls;
pub mod near;
pub mod skyline;

#[cfg(test)]
mod tests;
