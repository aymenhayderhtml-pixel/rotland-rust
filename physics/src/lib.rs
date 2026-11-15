//! # physics
//!
//! Rapier3D-based collision and character mover for Rotland.
//!
//! Provides `PhysicsWorld` for managing static terrain chunks, props, and scene queries,
//! and `CharacterMover` for kinematic character movement (`move_and_slide`) matching
//! Godot's `CharacterBody3D` and `PlayerController.cs`.

mod mover;
mod types;
mod world;

pub use mover::CharacterMover;
pub use types::{Hit, MoveResult, GROUP_CHARACTER, GROUP_WORLD};
pub use world::{PhysicsWorld, CHUNK_CELL, CHUNK_CELLS, CHUNK_SAMPLES, CHUNK_SIZE};

// Re-export Rapier collider handles and math types for convenience
pub use rapier3d::geometry::ColliderHandle;
