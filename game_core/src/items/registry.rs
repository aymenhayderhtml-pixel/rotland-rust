use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use crate::items::def::{ItemCategory, ItemDef};
use crate::weapons::WeaponRegistry;

static SHARED_REGISTRY: OnceLock<Arc<ItemRegistry>> = OnceLock::new();

#[derive(Debug)]
pub enum ItemLoadError {
    Io(std::io::Error),
    Parse(serde_json::Error),
    Validation(String),
}

impl std::fmt::Display for ItemLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemLoadError::Io(e) => write!(f, "could not read items file: {e}"),
            ItemLoadError::Parse(e) => write!(f, "could not parse items file: {e}"),
            ItemLoadError::Validation(e) => write!(f, "invalid item definition: {e}"),
        }
    }
}

impl std::error::Error for ItemLoadError {}

/// Registry of all item definitions from data/items.json (port of scripts/Items/ItemRegistry.cs).
/// Checks every entry on load so broken data fails early with a clear message.
#[derive(Clone, Debug, Default)]
pub struct ItemRegistry {
    defs: BTreeMap<String, ItemDef>,
}

impl ItemRegistry {
    /// Returns the process-wide cached `ItemRegistry`.
    /// On first call, loads from repo (`data/items.json`).
    /// If loading fails, logs an error and returns an empty registry so callers never panic.
    pub fn shared() -> Arc<Self> {
        SHARED_REGISTRY
            .get_or_init(|| match Self::from_repo() {
                Ok(reg) => Arc::new(reg),
                Err(err) => {
                    eprintln!("failed to load shared ItemRegistry: {err}; using empty fallback registry");
                    Arc::new(Self::default())
                }
            })
            .clone()
    }

    /// Loads from the default repo path `data/items.json`.
    pub fn from_repo() -> Result<Self, ItemLoadError> {
        let path = paths::repo_path("data/items.json");
        Self::load(&path)
    }

    /// Reads an items.json file at `path`. A missing or broken file is an error, never an empty registry.
    pub fn load(path: &Path) -> Result<Self, ItemLoadError> {
        let text = std::fs::read_to_string(path).map_err(ItemLoadError::Io)?;
        Self::from_json(&text)
    }

    /// Parses items.json text, sets item ids from keys, and validates every entry.
    pub fn from_json(text: &str) -> Result<Self, ItemLoadError> {
        let mut defs: BTreeMap<String, ItemDef> =
            serde_json::from_str(text).map_err(ItemLoadError::Parse)?;

        if defs.is_empty() {
            return Err(ItemLoadError::Validation("items data is empty".to_string()));
        }

        for (id, def) in defs.iter_mut() {
            def.id = id.clone();
            if def.max_stack < 1 {
                return Err(ItemLoadError::Validation(format!(
                    "Item '{id}' has maxStack {}, must be at least 1",
                    def.max_stack
                )));
            }
            if def.weight < 0.0 {
                return Err(ItemLoadError::Validation(format!(
                    "Item '{id}' has negative weight {}",
                    def.weight
                )));
            }
            if def.category == ItemCategory::Weapon && def.weapon_id.is_empty() {
                return Err(ItemLoadError::Validation(format!(
                    "Weapon item '{id}' has no weaponId"
                )));
            }
        }

        Ok(Self { defs })
    }

    /// Validates that all weapon items refer to valid weapons in the WeaponRegistry.
    pub fn validate_weapons(&self, weapons: &WeaponRegistry) -> Result<(), ItemLoadError> {
        for def in self.defs.values() {
            if def.category == ItemCategory::Weapon && weapons.get(&def.weapon_id).is_none() {
                return Err(ItemLoadError::Validation(format!(
                    "Item '{}' has weaponId '{}', which is not in weapons registry",
                    def.id, def.weapon_id
                )));
            }
        }
        Ok(())
    }

    /// Looks up an item definition by id.
    pub fn get(&self, id: &str) -> Option<&ItemDef> {
        self.defs.get(id)
    }

    /// Returns None for an unknown id instead of panicking.
    pub fn try_get(&self, id: &str) -> Option<&ItemDef> {
        self.get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.defs.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    /// Iterates over all item definitions in id order.
    pub fn iter(&self) -> impl Iterator<Item = &ItemDef> {
        self.defs.values()
    }

    /// Alias for iter().
    pub fn all(&self) -> impl Iterator<Item = &ItemDef> {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::Inventory;

    #[test]
    fn inventory_shared_registry_matches() {
        let inv = Inventory::new();
        let shared = ItemRegistry::shared();
        assert_eq!(inv.registry().len(), shared.len());
        assert_eq!(inv.slot_count(), crate::items::DEFAULT_SLOT_COUNT);
    }
}

