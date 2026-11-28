//! Rooftop trader stock from data/rooftop_trader.json (R44). Prices still come from economy.json.

use serde::{Deserialize, Serialize};

use game_core::items::ItemRegistry;

use crate::economy::config::StockEntry;
use crate::economy::trader::TraderStock;

/// Shape of data/rooftop_trader.json.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RooftopTraderConfig {
    pub map_label: String,
    pub stock: Vec<StockEntry>,
}

impl RooftopTraderConfig {
    /// Loads data/rooftop_trader.json from the default repository path.
    pub fn from_repo() -> Result<Self, String> {
        let path = paths::repo_path("data/rooftop_trader.json");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("could not read rooftop trader config: {e}"))?;
        Self::from_json(&text)
    }

    /// Parses rooftop trader JSON text.
    pub fn from_json(text: &str) -> Result<Self, String> {
        serde_json::from_str(text).map_err(|e| format!("could not parse rooftop trader config: {e}"))
    }

    /// Checks that every stocked item id exists in the item registry.
    pub fn validate(&self, items: &ItemRegistry) -> Result<(), String> {
        for entry in &self.stock {
            if !items.contains(&entry.id) {
                return Err(format!(
                    "rooftop trader stocks '{}', which is not in items.json",
                    entry.id
                ));
            }
        }
        Ok(())
    }

    /// The rooftop trader's shelf, filled to full stock.
    pub fn rooftop_stock(&self) -> TraderStock {
        TraderStock::from_entries(&self.stock)
    }
}
