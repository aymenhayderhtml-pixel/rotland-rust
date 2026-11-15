use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use world::BiomeId;

use crate::items::ItemRegistry;

#[derive(Debug)]
pub enum LootLoadError {
    Io(std::io::Error),
    Parse(serde_json::Error),
    Validation(String),
}

impl std::fmt::Display for LootLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "could not read loot tables file: {e}"),
            Self::Parse(e) => write!(f, "could not parse loot tables file: {e}"),
            Self::Validation(e) => write!(f, "invalid loot table definition: {e}"),
        }
    }
}

impl std::error::Error for LootLoadError {}

/// One weighted line of a loot table: an item and how many of it a roll can give (port of scripts/World/LootEntryData.cs).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LootEntryData {
    pub id: String,
    pub weight: i32,
    /// [min, max], both inclusive.
    #[serde(default = "default_range")]
    pub count: [i32; 2],
}

/// Dollars a crate may hold: a chance (0 to 1) and [min, max] dollars (port of scripts/World/LootCashData.cs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LootCashData {
    pub chance: f64,
    pub min: i32,
    pub max: i32,
}

fn default_range() -> [i32; 2] {
    [1, 1]
}

/// One loot table from data/loot_tables.json: how many rolls, and weighted entries (port of scripts/World/LootTableData.cs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LootTableData {
    /// [min, max] rolls per crate, both inclusive.
    #[serde(default = "default_range")]
    pub rolls: [i32; 2],
    #[serde(default)]
    pub entries: Vec<LootEntryData>,
    pub cash: Option<LootCashData>,
}

/// Registry of all loot tables loaded from data/loot_tables.json (port of scripts/World/LootTableRegistry.cs).
#[derive(Clone, Debug, Default)]
pub struct LootTableRegistry {
    tables: BTreeMap<String, LootTableData>,
}

impl LootTableRegistry {
    pub const FALLBACK_TABLE: &'static str = "generic";

    /// Loads loot tables from the default repository path `data/loot_tables.json`.
    pub fn from_repo() -> Result<Self, LootLoadError> {
        let path = paths::repo_path("data/loot_tables.json");
        Self::load(&path)
    }

    /// Reads and validates a loot_tables.json file at `path`.
    pub fn load(path: &Path) -> Result<Self, LootLoadError> {
        let text = std::fs::read_to_string(path).map_err(LootLoadError::Io)?;
        Self::from_json(&text)
    }

    /// Parses loot_tables.json text and validates every table.
    pub fn from_json(text: &str) -> Result<Self, LootLoadError> {
        let tables: BTreeMap<String, LootTableData> =
            serde_json::from_str(text).map_err(LootLoadError::Parse)?;

        if tables.is_empty() {
            return Err(LootLoadError::Validation("loot tables data is empty".to_string()));
        }

        if !tables.contains_key(Self::FALLBACK_TABLE) {
            return Err(LootLoadError::Validation(format!(
                "loot tables must have a '{}' table",
                Self::FALLBACK_TABLE
            )));
        }

        for (name, table) in &tables {
            Self::validate_table(name, table)?;
        }

        Ok(Self { tables })
    }

    fn validate_table(name: &str, table: &LootTableData) -> Result<(), LootLoadError> {
        if table.rolls[0] < 0 || table.rolls[0] > table.rolls[1] {
            return Err(LootLoadError::Validation(format!(
                "Loot table '{name}' needs rolls [min, max] with 0 <= min <= max"
            )));
        }
        if table.entries.is_empty() {
            return Err(LootLoadError::Validation(format!(
                "Loot table '{name}' has no entries"
            )));
        }
        for entry in &table.entries {
            if entry.weight <= 0 {
                return Err(LootLoadError::Validation(format!(
                    "Loot table '{name}' entry '{}' needs weight > 0",
                    entry.id
                )));
            }
            if entry.count[0] < 1 || entry.count[0] > entry.count[1] {
                return Err(LootLoadError::Validation(format!(
                    "Loot table '{name}' entry '{}' needs count [min, max] with 1 <= min <= max",
                    entry.id
                )));
            }
        }
        if let Some(cash) = &table.cash {
            if cash.chance < 0.0 || cash.chance > 1.0 || cash.min < 0 || cash.min > cash.max {
                return Err(LootLoadError::Validation(format!(
                    "Loot table '{name}' needs cash chance 0..1 and 0 <= min <= max"
                )));
            }
        }
        Ok(())
    }

    /// Validates that all items used by loot tables exist in the given ItemRegistry.
    pub fn validate_items(&self, items: &ItemRegistry) -> Result<(), LootLoadError> {
        for (name, table) in &self.tables {
            for entry in &table.entries {
                if !items.contains(&entry.id) {
                    return Err(LootLoadError::Validation(format!(
                        "Loot table '{name}' uses item '{}', which is not in items registry",
                        entry.id
                    )));
                }
            }
        }
        Ok(())
    }

    /// Returns the loot table for a biome, falling back to "generic".
    pub fn for_biome(&self, biome: BiomeId) -> &LootTableData {
        let name = match biome {
            BiomeId::Plains => "plains",
            BiomeId::Forest => "forest",
        };
        self.get(name).unwrap_or_else(|| {
            self.get(Self::FALLBACK_TABLE)
                .expect("generic table must exist")
        })
    }

    /// Looks up a loot table by name.
    pub fn get(&self, name: &str) -> Option<&LootTableData> {
        self.tables.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.tables.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &LootTableData)> {
        self.tables.iter()
    }
}
