use std::collections::{HashMap, HashSet};

use glam::IVec2;
use serde::{Deserialize, Serialize};

/// Tracks which crates have been looted across the world and per chunk (port of WorldState._looted).
/// Keeps looted state so a crate stays empty after unloading and loading again.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LootState {
    looted: HashSet<String>,
    by_chunk: HashMap<IVec2, HashSet<String>>,
}

impl LootState {
    pub fn new() -> Self {
        Self::default()
    }

    /// True if the crate with `crate_id` has been looted.
    pub fn is_looted(&self, crate_id: &str) -> bool {
        self.looted.contains(crate_id)
    }

    /// Marks a crate as looted. If the crate id encodes a chunk coordinate ("{x}_{z}_{idx}"),
    /// it also records it in that chunk's set.
    pub fn mark_looted(&mut self, crate_id: impl Into<String>) {
        let id = crate_id.into();
        if let Some(coord) = Self::parse_chunk_coord(&id) {
            self.by_chunk.entry(coord).or_default().insert(id.clone());
        }
        self.looted.insert(id);
    }

    /// Explicitly marks a crate as looted in a specific chunk.
    pub fn mark_chunk_looted(&mut self, coord: IVec2, crate_id: impl Into<String>) {
        let id = crate_id.into();
        self.by_chunk.entry(coord).or_default().insert(id.clone());
        self.looted.insert(id);
    }

    /// True if the crate is recorded as looted in the specified chunk or overall.
    pub fn is_chunk_looted(&self, coord: IVec2, crate_id: &str) -> bool {
        self.by_chunk
            .get(&coord)
            .is_some_and(|set| set.contains(crate_id))
            || self.looted.contains(crate_id)
    }

    /// Returns the set of crate ids looted in a chunk, if any.
    pub fn looted_in_chunk(&self, coord: IVec2) -> Option<&HashSet<String>> {
        self.by_chunk.get(&coord)
    }

    /// All looted crate ids.
    pub fn all_looted(&self) -> &HashSet<String> {
        &self.looted
    }

    /// Total number of looted crates.
    pub fn len(&self) -> usize {
        self.looted.len()
    }

    pub fn is_empty(&self) -> bool {
        self.looted.is_empty()
    }

    pub fn clear(&mut self) {
        self.looted.clear();
        self.by_chunk.clear();
    }

    /// Parses chunk coordinate "{x}_{z}_..." from a standard crate id.
    pub fn parse_chunk_coord(crate_id: &str) -> Option<IVec2> {
        let mut parts = crate_id.split('_');
        let x = parts.next()?.parse::<i32>().ok()?;
        let z = parts.next()?.parse::<i32>().ok()?;
        Some(IVec2::new(x, z))
    }
}
