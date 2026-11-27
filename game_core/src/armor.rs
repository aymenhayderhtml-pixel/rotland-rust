use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

use crate::items::Inventory;

/// One wearable armor item entry in `data/armor.json`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArmorItemDef {
    pub slot: String,
    pub reduce: f32,
}

/// Armor configuration loaded from `data/armor.json`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ArmorConfig {
    pub max_reduce: f32,
    pub items: BTreeMap<String, ArmorItemDef>,
}

impl Default for ArmorConfig {
    fn default() -> Self {
        Self {
            // data: armor.json maxReduce
            max_reduce: 0.5,
            items: BTreeMap::new(),
        }
    }
}

impl ArmorConfig {
    /// Loads `data/armor.json` from the repository root.
    pub fn from_repo() -> Result<Self, String> {
        Self::load(&paths::repo_path("data/armor.json"))
    }

    /// Reads an armor JSON file at `path`.
    pub fn load(path: &Path) -> Result<Self, String> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&raw)
    }

    /// Parses armor configuration from JSON text.
    pub fn from_json(raw: &str) -> Result<Self, String> {
        serde_json::from_str(raw).map_err(|e| format!("invalid armor.json: {e}"))
    }
}

/// Sums the best `reduce` per armor slot carried in `inventory`, capped at `cfg.max_reduce`.
pub fn armor_reduction(inventory: &Inventory, cfg: &ArmorConfig) -> f32 {
    let mut best_by_slot: BTreeMap<&str, f32> = BTreeMap::new();
    for stack in inventory.slots().iter().flatten() {
        if stack.count == 0 {
            continue;
        }
        if let Some(def) = cfg.items.get(&stack.item_id) {
            let reduce = def.reduce.max(0.0);
            let entry = best_by_slot.entry(def.slot.as_str()).or_insert(0.0);
            if reduce > *entry {
                *entry = reduce;
            }
        }
    }
    let total: f32 = best_by_slot.values().sum();
    let cap = cfg.max_reduce.clamp(0.0, 1.0);
    total.clamp(0.0, cap)
}

/// Applies armor damage reduction to a raw bite damage `d`: `d * (1 - reduction)`.
pub fn apply_armor_damage(damage: f32, reduction: f32) -> f32 {
    damage.max(0.0) * (1.0 - reduction.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cfg() -> ArmorConfig {
        ArmorConfig::from_json(
            r#"{
                "maxReduce": 0.5,
                "items": {
                    "body_armor": { "slot": "body", "reduce": 0.35 },
                    "army_helmet": { "slot": "head", "reduce": 0.15 }
                }
            }"#,
        )
        .expect("valid sample armor json")
    }

    #[test]
    fn best_per_slot_and_cap() {
        let cfg = sample_cfg();
        let mut inv = Inventory::new();
        assert_eq!(armor_reduction(&inv, &cfg), 0.0);

        inv.add("body_armor", 1);
        assert!((armor_reduction(&inv, &cfg) - 0.35).abs() < 1e-6);

        inv.add("body_armor", 1);
        assert!((armor_reduction(&inv, &cfg) - 0.35).abs() < 1e-6);

        inv.add("army_helmet", 1);
        assert!((armor_reduction(&inv, &cfg) - 0.5).abs() < 1e-6);
        assert!((apply_armor_damage(20.0, 0.5) - 10.0).abs() < 1e-6);
    }
}
