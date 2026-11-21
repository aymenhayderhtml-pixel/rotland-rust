//! Type definitions and layout configuration for camp upgrades.

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// Upgrade levels for the camp lines (pure values so `world` does not depend on `game`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CampLevels {
    pub walls: u8,
    pub guards: u8,
    pub workshop: u8,
    pub traders: u8,
}

/// One tower corner position and local orientation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TowerCorner {
    pub x: f32,
    pub z: f32,
    #[serde(default)]
    pub yaw: f32,
}

// data: camp_layout.json wallRadius
const DEFAULT_WALL_RADIUS: f32 = 20.0;
// data: camp_layout.json wallSectionLength
const DEFAULT_WALL_SECTION_LENGTH: f32 = 4.0;
// data: camp_layout.json gateWidth
const DEFAULT_GATE_WIDTH: f32 = 8.0;
// data: camp_layout.json gateOpening
const DEFAULT_GATE_OPENING: f32 = 6.0;
// data: camp_layout.json towerRadius
const DEFAULT_TOWER_RADIUS: f32 = 17.5;
// data: camp_layout.json workshopOffset
const DEFAULT_WORKSHOP_OFFSET: [f32; 2] = [13.5, 3.5];
// data: camp_layout.json doctorOffset
const DEFAULT_DOCTOR_OFFSET: [f32; 2] = [-14.0, -13.5];
// data: camp_layout.json mechanicOffset
const DEFAULT_MECHANIC_OFFSET: [f32; 2] = [14.5, -12.0];

/// Configuration layout data loaded from `data/camp_layout.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampLayoutConfigData {
    pub wall_radius: f32,
    pub wall_section_length: f32,
    pub gate_width: f32,
    pub gate_opening: f32,
    pub tower_radius: f32,
    pub tower_corners: Vec<TowerCorner>,
    pub guard_tower_corners: HashMap<String, Vec<usize>>,
    pub workshop_offset: [f32; 2],
    #[serde(default)]
    pub workshop_yaw: f32,
    pub doctor_offset: [f32; 2],
    #[serde(default)]
    pub doctor_yaw: f32,
    pub mechanic_offset: [f32; 2],
    #[serde(default)]
    pub mechanic_yaw: f32,
}

impl Default for CampLayoutConfigData {
    fn default() -> Self {
        let mut guard_corners = HashMap::new();
        guard_corners.insert("1".to_string(), vec![1]);
        guard_corners.insert("2".to_string(), vec![1]);
        guard_corners.insert("3".to_string(), vec![1, 3]);

        Self {
            wall_radius: DEFAULT_WALL_RADIUS,
            wall_section_length: DEFAULT_WALL_SECTION_LENGTH,
            gate_width: DEFAULT_GATE_WIDTH,
            gate_opening: DEFAULT_GATE_OPENING,
            tower_radius: DEFAULT_TOWER_RADIUS,
            tower_corners: vec![
                TowerCorner { x: -DEFAULT_TOWER_RADIUS, z: -DEFAULT_TOWER_RADIUS, yaw: 0.0 },
                TowerCorner { x: DEFAULT_TOWER_RADIUS, z: -DEFAULT_TOWER_RADIUS, yaw: 0.0 },
                TowerCorner { x: DEFAULT_TOWER_RADIUS, z: DEFAULT_TOWER_RADIUS, yaw: std::f32::consts::PI },
                TowerCorner { x: -DEFAULT_TOWER_RADIUS, z: DEFAULT_TOWER_RADIUS, yaw: std::f32::consts::PI },
            ],
            guard_tower_corners: guard_corners,
            workshop_offset: DEFAULT_WORKSHOP_OFFSET,
            workshop_yaw: 0.0,
            doctor_offset: DEFAULT_DOCTOR_OFFSET,
            doctor_yaw: 0.0,
            mechanic_offset: DEFAULT_MECHANIC_OFFSET,
            mechanic_yaw: 0.0,
        }
    }
}

impl CampLayoutConfigData {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let data: Self = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(data)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn load_default() -> Self {
        let path = Path::new("data/camp_layout.json");
        if path.is_file() {
            Self::load_from_file(path).unwrap_or_default()
        } else {
            Self::default()
        }
    }
}
