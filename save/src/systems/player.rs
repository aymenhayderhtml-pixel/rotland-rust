use serde::{Deserialize, Serialize};

/// 3D vector matching Godot's `{ "x": float, "y": float, "z": float }`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec3Data {
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default)]
    pub z: f32,
}

impl Vec3Data {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

fn default_player_health() -> f32 {
    100.0
}

/// Player position, angles, health, and spawn point.
/// Port of `scripts/Player/PlayerSave.cs` (SaveId: "player").
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayerSaveData {
    #[serde(default)]
    pub position: Vec3Data,
    #[serde(default, rename = "bodyYaw")]
    pub body_yaw: f32,
    #[serde(default, rename = "cameraYaw")]
    pub camera_yaw: f32,
    #[serde(default, rename = "cameraPitch")]
    pub camera_pitch: f32,
    #[serde(default = "default_player_health")]
    pub health: f32,
    #[serde(default)]
    pub spawn: Vec3Data,
}

impl Default for PlayerSaveData {
    fn default() -> Self {
        Self {
            position: Vec3Data::ZERO,
            body_yaw: 0.0,
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            health: 100.0,
            spawn: Vec3Data::ZERO,
        }
    }
}
