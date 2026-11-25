use std::path::Path;
use serde::Deserialize;

use crate::items::inventory::Inventory;
use crate::items::registry::ItemRegistry;

/// One line of data/start_inventory.json (port of scripts/Items/StartItemEntry.cs).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct StartItemEntry {
    pub id: String,
    pub count: u32,
}

#[derive(Debug)]
pub enum StartInventoryLoadError {
    Io(std::io::Error),
    Parse(serde_json::Error),
    Empty,
}

impl std::fmt::Display for StartInventoryLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartInventoryLoadError::Io(e) => write!(f, "could not read start inventory file: {e}"),
            StartInventoryLoadError::Parse(e) => write!(f, "could not parse start inventory file: {e}"),
            StartInventoryLoadError::Empty => write!(f, "start inventory file is empty"),
        }
    }
}

impl std::error::Error for StartInventoryLoadError {}

/// Loads starting item entries from a JSON file.
pub fn load_start_inventory_entries(path: &Path) -> Result<Vec<StartItemEntry>, StartInventoryLoadError> {
    let text = std::fs::read_to_string(path).map_err(StartInventoryLoadError::Io)?;
    let entries: Vec<StartItemEntry> =
        serde_json::from_str(&text).map_err(StartInventoryLoadError::Parse)?;
    if entries.is_empty() {
        return Err(StartInventoryLoadError::Empty);
    }
    Ok(entries)
}

/// Attempts to create a new Inventory pre-populated from `data/start_inventory.json`.
pub fn try_start_inventory(registry: &ItemRegistry) -> Result<Inventory, StartInventoryLoadError> {
    let path = paths::repo_path("data/start_inventory.json");
    start_inventory_from_path(registry, &path)
}

/// Creates a new Inventory pre-populated with starting items from data/start_inventory.json.
/// If loading fails, logs an error and returns an empty inventory so the game never crashes.
pub fn start_inventory(registry: &ItemRegistry) -> Inventory {
    match try_start_inventory(registry) {
        Ok(inv) => inv,
        Err(err) => {
            eprintln!("failed to load start inventory: {err}; starting with empty inventory");
            Inventory::from_registry(registry.clone())
        }
    }
}

/// Creates a new Inventory pre-populated from the specified start_inventory.json path.
pub fn start_inventory_from_path(
    registry: &ItemRegistry,
    path: &Path,
) -> Result<Inventory, StartInventoryLoadError> {
    let entries = load_start_inventory_entries(path)?;
    let mut inventory = Inventory::from_registry(registry.clone());
    for entry in &entries {
        inventory.add(&entry.id, entry.count);
    }
    inventory.clear_changed();
    inventory.set_changed();
    Ok(inventory)
}
