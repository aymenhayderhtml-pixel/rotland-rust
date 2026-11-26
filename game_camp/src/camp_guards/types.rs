//! Types for camp guards and shooting frame output.

use glam::Vec3;
use serde::{Deserialize, Serialize};

use game_core::damage::DamageInfo;

/// Position and orientation of a guard standing at their post.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GuardPost {
    pub id: String,
    pub position: Vec3,
    pub yaw: f32,
}

impl GuardPost {
    pub fn new(id: impl Into<String>, position: Vec3, yaw: f32) -> Self {
        Self {
            id: id.into(),
            position,
            yaw,
        }
    }
}

/// Potential target for camp guards (usually an active zombie).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GuardTarget {
    pub id: u64,
    pub position: Vec3,
    pub alive: bool,
}

impl GuardTarget {
    pub fn new(id: u64, position: Vec3, alive: bool) -> Self {
        Self {
            id,
            position,
            alive,
        }
    }
}

/// One rifle shot fired by a guard at a target.
#[derive(Clone, Debug, PartialEq)]
pub struct GuardShot {
    pub guard_id: String,
    pub target_id: u64,
    pub from: Vec3,
    pub to: Vec3,
    pub damage: DamageInfo,
}

/// Per-frame output of camp guard simulation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GuardFrame {
    pub shots: Vec<GuardShot>,
    pub yaws: Vec<(String, f32)>,
    pub sounds: Vec<(String, Vec3)>,
}

/// Guard rifle constants and helpers (port of scripts/Settlement/GuardRifle.cs).
pub struct GuardRifle;

impl GuardRifle {
    pub const READY_POSE: &'static str = "rifle_aim";
    pub const FIRE_ACTION: &'static str = "rifle_fire";
    pub const SHOT_SOUND: &'static str = "rifle_shot";
    pub const DAMAGE_TYPE: &'static str = "bullet";
    pub const CHEST_HEIGHT: f32 = 1.2;
}
