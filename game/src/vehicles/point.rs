use glam::Vec3;
use serde::{Deserialize, Serialize};

/// What an interaction point on a car does (port of scripts/Vehicles/CarPointKind.cs).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CarPointKind {
    Door,
    Trunk,
    Fuel,
}

/// An interactable spot on a car in local or world space (port of scripts/Vehicles/CarPoint.cs).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarPoint {
    pub kind: CarPointKind,
    pub position: Vec3,
}

impl CarPoint {
    pub fn new(kind: CarPointKind, position: Vec3) -> Self {
        Self { kind, position }
    }
}
