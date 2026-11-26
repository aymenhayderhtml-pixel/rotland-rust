use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::economy::StockEntry;
use crate::gunsmith::config::{GunsmithConfig, GunsmithLoadError};

pub const GUNSMITH_SAVE_ID: &str = "gunsmith";

/// Which gunsmith shop the player is interacting with.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShopKind {
    Camp { tier: u8 },
    Town { key: String },
}

/// Per-shop stock tracking quantities and daily morning restock.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShopStock {
    qty: BTreeMap<String, u32>,
    order: Vec<String>,
    last_restock_day: i32,
    base_stock: Vec<StockEntry>,
}

impl ShopStock {
    pub fn from_entries(entries: &[StockEntry]) -> Self {
        let mut order = Vec::with_capacity(entries.len());
        let mut qty = BTreeMap::new();
        let mut base_stock = Vec::with_capacity(entries.len());

        for entry in entries {
            order.push(entry.id.clone());
            qty.insert(entry.id.clone(), entry.qty);
            base_stock.push(entry.clone());
        }

        Self {
            qty,
            order,
            last_restock_day: 0,
            base_stock,
        }
    }

    pub fn last_restock_day(&self) -> i32 {
        self.last_restock_day
    }

    pub fn order(&self) -> &[String] {
        &self.order
    }

    pub fn qty(&self, id: &str) -> u32 {
        self.qty.get(id).copied().unwrap_or(0)
    }

    pub fn take(&mut self, id: &str, count: u32) -> bool {
        if count == 0 {
            return true;
        }
        let current = self.qty(id);
        if current >= count {
            self.qty.insert(id.to_string(), current - count);
            true
        } else {
            false
        }
    }

    pub fn tick(&mut self, day: i32, hour: u32, restock_hour: u32) {
        if hour >= restock_hour && day > self.last_restock_day {
            self.fill_all();
            self.last_restock_day = day;
        }
    }

    pub fn fill_all(&mut self) {
        for entry in &self.base_stock {
            self.qty.insert(entry.id.clone(), entry.qty);
        }
    }

    pub fn to_save(&self) -> serde_json::Value {
        let mut qty_map = serde_json::Map::new();
        for id in &self.order {
            qty_map.insert(id.clone(), serde_json::Value::from(self.qty(id)));
        }
        serde_json::json!({
            "qty": serde_json::Value::Object(qty_map),
            "restockDay": self.last_restock_day
        })
    }

    pub fn from_save(&mut self, data: &serde_json::Value) {
        if self.order.is_empty() {
            self.fill_all();
        }
        self.last_restock_day = data
            .get("restockDay")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;

        if let Some(qty_obj) = data.get("qty").and_then(|v| v.as_object()) {
            for id in &self.order {
                if let Some(val) = qty_obj.get(id).and_then(|v| v.as_i64()) {
                    self.qty.insert(id.clone(), val.max(0) as u32);
                }
            }
        }
    }
}

/// Stock state for the camp gunsmith and all outer-town gun shops (SaveId: `"gunsmith"`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GunsmithStock {
    camp: ShopStock,
    towns: BTreeMap<String, ShopStock>,
}

impl GunsmithStock {
    /// Initializes stock for the camp gunsmith and every configured town shop.
    pub fn new(config: &GunsmithConfig) -> Self {
        let camp_entries: Vec<StockEntry> = config
            .camp_stock
            .iter()
            .map(|e| StockEntry::new(&e.id, e.qty))
            .collect();
        let camp = ShopStock::from_entries(&camp_entries);

        let mut towns = BTreeMap::new();
        for (key, town_cfg) in &config.towns {
            towns.insert(key.clone(), ShopStock::from_entries(&town_cfg.stock));
        }

        Self { camp, towns }
    }

    /// Loads `data/gunsmith.json` from repo and initializes all gunsmith stocks.
    pub fn from_repo() -> Result<Self, GunsmithLoadError> {
        let cfg = GunsmithConfig::from_repo()?;
        Ok(Self::new(&cfg))
    }

    pub fn shop(&self, kind: &ShopKind) -> Option<&ShopStock> {
        match kind {
            ShopKind::Camp { .. } => Some(&self.camp),
            ShopKind::Town { key } => self.towns.get(key),
        }
    }

    pub fn shop_mut(&mut self, kind: &ShopKind) -> Option<&mut ShopStock> {
        match kind {
            ShopKind::Camp { .. } => Some(&mut self.camp),
            ShopKind::Town { key } => self.towns.get_mut(key),
        }
    }

    pub fn qty(&self, kind: &ShopKind, id: &str) -> u32 {
        self.shop(kind).map_or(0, |s| s.qty(id))
    }

    pub fn take(&mut self, kind: &ShopKind, id: &str, count: u32) -> bool {
        self.shop_mut(kind)
            .is_some_and(|s| s.take(id, count))
    }

    /// Restocks all gunsmith shops once per day once `hour >= restock_hour`.
    pub fn tick(&mut self, day: i32, hour: u32, restock_hour: u32) {
        self.camp.tick(day, hour, restock_hour);
        for shop in self.towns.values_mut() {
            shop.tick(day, hour, restock_hour);
        }
    }

    pub fn tick_with_config(&mut self, day: i32, hour: u32, config: &GunsmithConfig) {
        self.tick(day, hour, config.restock_hour);
    }

    pub fn fill_all(&mut self) {
        self.camp.fill_all();
        for shop in self.towns.values_mut() {
            shop.fill_all();
        }
    }

    /// Serializes all gunsmith shops under `"camp"` and `"towns"`.
    pub fn to_save(&self) -> serde_json::Value {
        let mut towns_map = serde_json::Map::new();
        for (key, shop) in &self.towns {
            towns_map.insert(key.clone(), shop.to_save());
        }
        serde_json::json!({
            "camp": self.camp.to_save(),
            "towns": serde_json::Value::Object(towns_map),
        })
    }

    /// Restores stock for camp and any matching towns from save data.
    pub fn from_save(&mut self, data: &serde_json::Value) {
        if let Some(camp_val) = data.get("camp") {
            self.camp.from_save(camp_val);
        }
        if let Some(towns_obj) = data.get("towns").and_then(|v| v.as_object()) {
            for (key, shop) in &mut self.towns {
                if let Some(town_val) = towns_obj.get(key) {
                    shop.from_save(town_val);
                }
            }
        }
    }

    pub fn from_save_value(data: &serde_json::Value, config: &GunsmithConfig) -> Self {
        let mut stock = Self::new(config);
        stock.from_save(data);
        stock
    }
}
