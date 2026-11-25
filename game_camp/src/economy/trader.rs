use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

use crate::economy::config::{EconomyConfigData, EconomyLoadError, StockEntry};

pub const TRADER_SAVE_ID: &str = "trader";

/// What the trader has in stock to sell.
/// Port of `scripts/Economy/TraderStock.cs` (SaveId: "trader").
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraderStock {
    qty: BTreeMap<String, u32>,
    order: Vec<String>,
    last_restock_day: i32,
    base_stock: Vec<StockEntry>,
}

impl TraderStock {
    /// Initializes trader stock from the given configuration, filling full stock initially.
    pub fn new(config: &EconomyConfigData) -> Self {
        Self::from_entries(&config.stock)
    }

    /// Loads the economy configuration from the repo and initializes trader stock.
    pub fn from_repo() -> Result<Self, EconomyLoadError> {
        let config = EconomyConfigData::from_repo()?;
        Ok(Self::new(&config))
    }

    /// Initializes trader stock from a list of stock entries.
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

    /// The in-game day of the last refill (0 before the first one).
    pub fn last_restock_day(&self) -> i32 {
        self.last_restock_day
    }

    /// Item ids in the order economy.json lists them.
    pub fn order(&self) -> &[String] {
        &self.order
    }

    /// How many of an item the trader holds in stock.
    /// // Godot: TraderStock.cs Qty
    pub fn qty(&self, id: &str) -> u32 {
        self.qty.get(id).copied().unwrap_or(0)
    }

    /// Takes units off the shelf. Returns true if the quantity was taken, false if insufficient.
    /// // Godot: TraderStock.cs Take
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

    /// Refills stock to full when the hour has reached restock_hour and day > last_restock_day.
    /// // Godot: TraderStock.cs Tick
    pub fn tick(&mut self, day: i32, hour: u32, restock_hour: u32) {
        if hour >= restock_hour && day > self.last_restock_day {
            self.fill_all();
            self.last_restock_day = day;
        }
    }

    /// Helper for tick using the restock_hour from EconomyConfigData.
    pub fn tick_with_config(&mut self, day: i32, hour: u32, config: &EconomyConfigData) {
        self.tick(day, hour, config.restock_hour);
    }

    /// Refills all item quantities back to their base stock counts.
    /// // Godot: TraderStock.cs FillAll
    pub fn fill_all(&mut self) {
        for entry in &self.base_stock {
            self.qty.insert(entry.id.clone(), entry.qty);
        }
    }

    /// Serializes trader stock using Godot save keys ("qty" object and "restockDay" integer).
    /// // Godot: TraderStock.cs Save
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

    /// Restores trader stock from Godot save data.
    /// // Godot: TraderStock.cs Load
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

    /// Creates a TraderStock instance from configuration and Godot save data.
    pub fn from_save_value(data: &serde_json::Value, config: &EconomyConfigData) -> Self {
        let mut stock = Self::new(config);
        stock.from_save(data);
        stock
    }
}
