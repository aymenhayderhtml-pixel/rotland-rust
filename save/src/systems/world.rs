use serde::{Deserialize, Serialize};

/// World generator seed.
/// Port of `scripts/World/WorldRoot.cs` (SaveId: "world").
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSaveData {
    #[serde(default)]
    pub seed: i32,
}

impl WorldSaveData {
    pub fn new(seed: i32) -> Self {
        Self { seed }
    }
}
