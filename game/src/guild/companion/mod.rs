//! Pure runtime AI and state for hired guild teammates following, guarding, and fighting.

mod config;
mod geometry;
mod squad;
mod targeting;
mod tick;
mod tuning;
mod types;

pub use config::CompanionConfig;
pub use squad::CompanionSquad;
pub use types::{CompanionAnim, CompanionDraw, CompanionEvent, CompanionTarget, CompanionWorld};
