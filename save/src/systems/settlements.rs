use serde::{Deserialize, Serialize};

/// Settlement watcher data: visited settlement IDs and respawn settlement.
/// Port of `scripts/Settlement/SettlementWatcher.cs` (SaveId: "settlements").
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SettlementsSaveData {
    #[serde(default)]
    pub visited: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub respawn: Option<String>,
}

impl SettlementsSaveData {
    pub fn new(visited: Vec<String>, respawn: Option<String>) -> Self {
        Self { visited, respawn }
    }
}
