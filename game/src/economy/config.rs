use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::items::ItemRegistry;

/// One line of the trader's full stock in data/economy.json.
/// Port of `scripts/Economy/StockEntry.cs`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockEntry {
    pub id: String,
    pub qty: u32,
}

impl StockEntry {
    pub fn new(id: impl Into<String>, qty: u32) -> Self {
        Self {
            id: id.into(),
            qty,
        }
    }
}

#[derive(Debug)]
pub enum EconomyLoadError {
    Io(std::io::Error),
    Parse(serde_json::Error),
    Validation(String),
}

impl std::fmt::Display for EconomyLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "could not read economy config: {e}"),
            Self::Parse(e) => write!(f, "could not parse economy config: {e}"),
            Self::Validation(e) => write!(f, "invalid economy config: {e}"),
        }
    }
}

impl std::error::Error for EconomyLoadError {}

/// Shape of data/economy.json.
/// Port of `scripts/Economy/EconomyConfigData.cs` and `EconomyConfig.cs`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct EconomyConfigData {
    pub start_dollars: i32,
    /// The trader pays value x SellRate (rounded down, at least $1).
    pub sell_rate: f32,
    /// You pay value x BuyRate (rounded down, at least $1).
    pub buy_rate: f32,
    /// Hour of the day (06:00) when the trader refills to full stock, once per in-game day.
    pub restock_hour: u32,
    pub bank_slots: usize,
    /// Metres. A shop or bank screen closes when the player walks further than this from the NPC.
    pub interact_range: f32,
    pub stock: Vec<StockEntry>,
}

impl Default for EconomyConfigData {
    fn default() -> Self {
        Self {
            start_dollars: 20,
            sell_rate: 0.5,
            buy_rate: 1.0,
            restock_hour: 6,
            bank_slots: 40,
            interact_range: 4.0,
            stock: Vec::new(),
        }
    }
}

pub type EconomyConfig = EconomyConfigData;

impl EconomyConfigData {
    /// Loads data/economy.json from the default repository path.
    /// // Godot: EconomyConfig.cs Get
    pub fn from_repo() -> Result<Self, EconomyLoadError> {
        let path = paths::repo_path("data/economy.json");
        Self::load(&path)
    }

    /// Reads economy configuration from the given file path.
    pub fn load(path: &Path) -> Result<Self, EconomyLoadError> {
        let text = std::fs::read_to_string(path).map_err(EconomyLoadError::Io)?;
        Self::from_json(&text)
    }

    /// Parses JSON text into an EconomyConfigData and validates all fields.
    pub fn from_json(text: &str) -> Result<Self, EconomyLoadError> {
        let config: Self = serde_json::from_str(text).map_err(EconomyLoadError::Parse)?;
        config.validate(None)?;
        Ok(config)
    }

    /// Loads and validates configuration against an explicit ItemRegistry.
    pub fn load_with_registry(path: &Path, items: &ItemRegistry) -> Result<Self, EconomyLoadError> {
        let text = std::fs::read_to_string(path).map_err(EconomyLoadError::Io)?;
        let config: Self = serde_json::from_str(&text).map_err(EconomyLoadError::Parse)?;
        config.validate(Some(items))?;
        Ok(config)
    }

    /// Validates fields and verifies stock items exist if an ItemRegistry is provided.
    /// // Godot: EconomyConfig.cs Validate
    pub fn validate(&self, items: Option<&ItemRegistry>) -> Result<(), EconomyLoadError> {
        let ok = self.start_dollars >= 0
            && self.sell_rate > 0.0
            && self.sell_rate <= 1.0
            && self.buy_rate > 0.0
            && self.restock_hour < 24
            && self.bank_slots > 0
            && self.interact_range > 0.0;
        if !ok {
            return Err(EconomyLoadError::Validation(
                "economy config has a value out of range".to_string(),
            ));
        }

        if let Some(registry) = items {
            for entry in &self.stock {
                if !registry.contains(&entry.id) {
                    return Err(EconomyLoadError::Validation(format!(
                        "economy config stocks '{}', which is not in items.json",
                        entry.id
                    )));
                }
            }
        }

        Ok(())
    }
}
