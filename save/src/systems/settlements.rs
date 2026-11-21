use serde::{Deserialize, Serialize};

/// Upgrade in progress save data.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UpgradeInProgressSaveData {
    pub line: String,
    pub target_level: u8,
    pub hours_left: f32,
}

/// Camp upgrades save data.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CampUpgradesSaveData {
    pub walls_level: u8,
    pub guards_level: u8,
    pub workshop_level: u8,
    pub traders_level: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_progress: Option<UpgradeInProgressSaveData>,
    pub wall_hp: f32,
}

/// Settlement watcher data: visited settlement IDs and respawn settlement.
/// Port of `scripts/Settlement/SettlementWatcher.cs` (SaveId: "settlements").
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SettlementsSaveData {
    #[serde(default)]
    pub visited: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub respawn: Option<String>,
    #[serde(default, alias = "upgrades")]
    pub camp_upgrades: CampUpgradesSaveData,
}

impl SettlementsSaveData {
    pub fn new(visited: Vec<String>, respawn: Option<String>) -> Self {
        Self {
            visited,
            respawn,
            camp_upgrades: CampUpgradesSaveData::default(),
        }
    }
}
