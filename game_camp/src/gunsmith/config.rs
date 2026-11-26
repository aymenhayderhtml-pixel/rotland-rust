use std::collections::BTreeMap;
use std::path::Path;

use game_core::items::ItemRegistry;
use serde::{Deserialize, Serialize};

use crate::economy::StockEntry;
use crate::gunsmith::stock::ShopKind;

fn default_pack_size() -> u32 {
    1
}

/// Price in scrap for one item (or one pack/bundle of `pack_size` units) in `data/gunsmith.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GunsmithPriceEntry {
    pub price: u32,
    #[serde(default = "default_pack_size")]
    pub pack_size: u32,
}

/// One line of the camp gunsmith's stock, unlocked at `tier` (0..=3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampStockEntry {
    pub id: String,
    pub qty: u32,
    #[serde(default)]
    pub tier: u8,
}

/// Stock configuration for one outer-town gun shop.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TownShopConfig {
    pub name: String,
    #[serde(default)]
    pub flavour: String,
    pub stock: Vec<StockEntry>,
}

#[derive(Debug)]
pub enum GunsmithLoadError {
    Io(std::io::Error),
    Parse(serde_json::Error),
    Validation(String),
}

impl std::fmt::Display for GunsmithLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "could not read gunsmith config: {e}"),
            Self::Parse(e) => write!(f, "could not parse gunsmith config: {e}"),
            Self::Validation(e) => write!(f, "invalid gunsmith config: {e}"),
        }
    }
}

impl std::error::Error for GunsmithLoadError {}

/// Shape of `data/gunsmith.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GunsmithConfig {
    pub scrap_item: String,
    pub sell_rate: f32,
    pub repair_rate: f32,
    pub fit_labour: u32,
    pub restock_hour: u32,
    pub interact_range: f32,
    pub scrap_dollar_value: u32,
    pub max_camp_tier: u8,
    pub prices: BTreeMap<String, GunsmithPriceEntry>,
    pub camp_stock: Vec<CampStockEntry>,
    pub towns: BTreeMap<String, TownShopConfig>,
}

pub type GunsmithConfigData = GunsmithConfig;

impl Default for GunsmithConfig {
    fn default() -> Self {
        Self {
            scrap_item: "scrap_metal".to_string(),
            // data: gunsmith.json sellRate
            sell_rate: 0.4,
            // data: gunsmith.json repairRate
            repair_rate: 0.5,
            // data: gunsmith.json fitLabour
            fit_labour: 5,
            // data: gunsmith.json restockHour
            restock_hour: 6,
            // data: gunsmith.json interactRange
            interact_range: 4.0,
            // data: gunsmith.json scrapDollarValue
            scrap_dollar_value: 4,
            // data: gunsmith.json maxCampTier
            max_camp_tier: 3,
            prices: BTreeMap::new(),
            camp_stock: Vec::new(),
            towns: BTreeMap::new(),
        }
    }
}

impl GunsmithConfig {
    /// Loads `data/gunsmith.json` from the repository path.
    pub fn from_repo() -> Result<Self, GunsmithLoadError> {
        let path = paths::repo_path("data/gunsmith.json");
        Self::load(&path)
    }

    /// Reads gunsmith configuration from `path` and validates it.
    pub fn load(path: &Path) -> Result<Self, GunsmithLoadError> {
        let text = std::fs::read_to_string(path).map_err(GunsmithLoadError::Io)?;
        Self::from_json(&text)
    }

    /// Parses JSON text and validates configuration fields.
    pub fn from_json(text: &str) -> Result<Self, GunsmithLoadError> {
        let cfg: Self = serde_json::from_str(text).map_err(GunsmithLoadError::Parse)?;
        cfg.validate(None)?;
        Ok(cfg)
    }

    /// Validates numeric ranges, price coverage, and optional `ItemRegistry` membership.
    pub fn validate(&self, items: Option<&ItemRegistry>) -> Result<(), GunsmithLoadError> {
        // data: gunsmith.json restockHour
        let ok = !self.scrap_item.is_empty()
            && self.sell_rate > 0.0
            && self.sell_rate <= 1.0
            && self.repair_rate > 0.0
            && self.repair_rate <= 2.0
            && self.restock_hour < 24
            && self.interact_range > 0.0
            && self.scrap_dollar_value > 0;
        if !ok {
            return Err(GunsmithLoadError::Validation(
                "gunsmith config has a value out of range".to_string(),
            ));
        }

        for (id, entry) in &self.prices {
            if entry.price == 0 || entry.pack_size == 0 {
                return Err(GunsmithLoadError::Validation(format!(
                    "price entry '{id}' must have positive price and packSize"
                )));
            }
        }

        for entry in &self.camp_stock {
            if entry.tier > self.max_camp_tier {
                return Err(GunsmithLoadError::Validation(format!(
                    "camp stock '{}' has tier {} > maxCampTier {}",
                    entry.id, entry.tier, self.max_camp_tier
                )));
            }
            if !self.prices.contains_key(&entry.id) {
                return Err(GunsmithLoadError::Validation(format!(
                    "camp stock '{}' is missing a price in gunsmith.json",
                    entry.id
                )));
            }
        }

        for (town_key, town) in &self.towns {
            for entry in &town.stock {
                if !self.prices.contains_key(&entry.id) {
                    return Err(GunsmithLoadError::Validation(format!(
                        "town '{town_key}' stock '{}' is missing a price in gunsmith.json",
                        entry.id
                    )));
                }
            }
        }

        if let Some(registry) = items {
            if !registry.contains(&self.scrap_item) {
                return Err(GunsmithLoadError::Validation(format!(
                    "scrapItem '{}' is not in items.json",
                    self.scrap_item
                )));
            }
            for id in self.prices.keys() {
                if !registry.contains(id) {
                    return Err(GunsmithLoadError::Validation(format!(
                        "gunsmith price '{id}' is not in items.json"
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn price_entry(&self, id: &str) -> Option<&GunsmithPriceEntry> {
        self.prices.get(id)
    }

    pub fn buy_price(&self, id: &str) -> Option<u32> {
        self.prices.get(id).map(|e| e.price)
    }

    pub fn pack_size(&self, id: &str) -> u32 {
        self.prices.get(id).map_or(1, |e| e.pack_size.max(1))
    }

    pub fn shop_name(&self, kind: &ShopKind) -> String {
        match kind {
            ShopKind::Camp { .. } => "Camp Gunsmith".to_string(),
            ShopKind::Town { key } => self
                .towns
                .get(key)
                .map_or_else(|| format!("{key} Gun Shop"), |t| t.name.clone()),
        }
    }

    /// Returns the required camp tier if `kind` carries `id` (0 for towns), or `None` if not offered.
    pub fn required_tier(&self, kind: &ShopKind, id: &str) -> Option<u8> {
        match kind {
            ShopKind::Camp { .. } => self
                .camp_stock
                .iter()
                .find(|e| e.id == id)
                .map(|e| e.tier),
            ShopKind::Town { key } => self
                .towns
                .get(key)
                .and_then(|t| t.stock.iter().find(|e| e.id == id).map(|_| 0)),
        }
    }
}
