use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::error::SaveError;
use crate::systems::{
    DifficultySaveData, InventorySaveData, PlayerSaveData, SettlementsSaveData, StatsSaveData,
    TimeSaveData, WalletSaveData, WeaponsSaveData, WorldSaveData, WorldStateSaveData,
};

pub const FORMAT_VERSION: i32 = 1;

/// Formats the current system time as a UTC ISO 8601 string (e.g. "2026-11-16T08:00:00Z").
pub fn utc_now_iso8601() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format_utc_timestamp(duration.as_secs())
}

/// Converts Unix seconds to an ISO 8601 UTC date-time string.
pub fn format_utc_timestamp(total_seconds: u64) -> String {
    let seconds = (total_seconds % 60) as u32;
    let minutes = ((total_seconds / 60) % 60) as u32;
    let hours = ((total_seconds / 3600) % 24) as u32;
    let days = (total_seconds / 86400) as i64;

    // Howard Hinnant's civil date algorithm
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

/// The systems map containing known typed systems and preserving unknown systems as `serde_json::Value`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Systems {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player: Option<PlayerSaveData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stats: Option<StatsSaveData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inventory: Option<InventorySaveData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<TimeSaveData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world: Option<WorldSaveData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worldstate: Option<WorldStateSaveData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<DifficultySaveData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallet: Option<WalletSaveData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapons: Option<WeaponsSaveData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settlements: Option<SettlementsSaveData>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Systems {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a Systems struct with all typed systems populated with default values.
    pub fn new_populated(seed: i32) -> Self {
        Self {
            player: Some(PlayerSaveData::default()),
            stats: Some(StatsSaveData::default()),
            inventory: Some(InventorySaveData::default()),
            time: Some(TimeSaveData::default()),
            world: Some(WorldSaveData { seed }),
            worldstate: Some(WorldStateSaveData::default()),
            difficulty: Some(DifficultySaveData::default()),
            wallet: Some(WalletSaveData::default()),
            weapons: Some(WeaponsSaveData::default()),
            settlements: Some(SettlementsSaveData::default()),
            extra: HashMap::new(),
        }
    }
}

fn default_version() -> i32 {
    FORMAT_VERSION
}

/// Root Godot save file structure: `{ "version": 1, "savedAtUtc": "...", "systems": { ... } }`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SaveFile {
    #[serde(default = "default_version")]
    pub version: i32,
    #[serde(rename = "savedAtUtc", default)]
    pub saved_at_utc: String,
    #[serde(default)]
    pub systems: Systems,
}

impl Default for SaveFile {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveFile {
    pub fn new() -> Self {
        Self {
            version: FORMAT_VERSION,
            saved_at_utc: utc_now_iso8601(),
            systems: Systems::default(),
        }
    }

    pub fn new_populated(seed: i32) -> Self {
        Self {
            version: FORMAT_VERSION,
            saved_at_utc: utc_now_iso8601(),
            systems: Systems::new_populated(seed),
        }
    }

    pub fn from_json_str(s: &str) -> Result<Self, SaveError> {
        let save: Self = serde_json::from_str(s)?;
        if save.version != FORMAT_VERSION {
            return Err(SaveError::UnsupportedVersion(save.version));
        }
        Ok(save)
    }

    pub fn to_json_pretty(&self) -> Result<String, SaveError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn to_json_string(&self) -> Result<String, SaveError> {
        Ok(serde_json::to_string(self)?)
    }
}
