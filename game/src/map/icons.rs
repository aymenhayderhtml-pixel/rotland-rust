//! Map icon kinds and color constants.
//!
//! Port of Godot C# scripts/UI/MapIcons.cs.

use serde::{Deserialize, Serialize};

// Godot: MapIcons.cs
/// Visual icon kinds rendered on the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapIconKind {
    Player,
    Camp,
    Building,
    Bounty,
    Death,
    Waypoint,
}

// Godot: MapIcons.cs color constants (RGBA in 0..255)
pub const OUTLINE_COLOR: [u8; 4] = [0, 0, 0, 230];
pub const CAMP_COLOR: [u8; 4] = [178, 255, 178, 255];
pub const BUILDING_COLOR: [u8; 4] = [242, 204, 115, 255];
pub const DIM_COLOR: [u8; 4] = [115, 115, 115, 255];
pub const BOUNTY_COLOR: [u8; 4] = [242, 51, 51, 255];
pub const SKULL_COLOR: [u8; 4] = [242, 237, 224, 255];
pub const WAYPOINT_COLOR: [u8; 4] = [255, 204, 51, 255];
pub const PLAYER_COLOR: [u8; 4] = [255, 255, 255, 255];
