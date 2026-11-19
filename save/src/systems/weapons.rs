use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Weapon holder data: equipped weapon and magazine counts for all guns used.
/// Port of `scripts/Combat/WeaponHolder.cs` (SaveId: "weapons").
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WeaponsSaveData {
    #[serde(default)]
    pub equipped: String,
    #[serde(default)]
    pub magazines: HashMap<String, i32>,
}

impl WeaponsSaveData {
    pub fn new(equipped: impl Into<String>, magazines: HashMap<String, i32>) -> Self {
        Self {
            equipped: equipped.into(),
            magazines,
        }
    }
}
