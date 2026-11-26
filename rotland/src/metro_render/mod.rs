//! New Halden towers: instanced kit parts per loaded chunk (`parts.rs`, `near.rs`),
//! colliders (`hulls.rs`), and the far skyline batch (`skyline.rs`, `boxes.rs`).
//!
//! Wiring entry points: `near::place_chunk_towers`, `hulls::tower_hulls`, `hulls::tower_bases`,
//! `skyline::SkylineBatch`.

pub mod boxes;
pub mod curb_mesh;
pub mod hulls;
pub mod near;
pub mod parts;
pub mod skyline;

#[cfg(test)]
mod tests;
