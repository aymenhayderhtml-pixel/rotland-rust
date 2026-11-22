//! Base building structures save data (SaveId: "base").

use serde::{Deserialize, Serialize};

use super::player::Vec3Data;

/// One placed building part in a save slot.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BasePartSaveData {
    pub id: u32,
    pub part: String,
    pub cell_x: i32,
    pub cell_z: i32,
    pub floor: u32,
    pub rot: u8,
    pub hp: f32,
    pub door_open: bool,
    pub position: Vec3Data,
    pub yaw: f32,
}

/// Base building system save data.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BaseSaveData {
    pub next_id: u32,
    #[serde(default)]
    pub parts: Vec<BasePartSaveData>,
}

impl BaseSaveData {
    pub fn new(next_id: u32, parts: Vec<BasePartSaveData>) -> Self {
        Self { next_id, parts }
    }
}
