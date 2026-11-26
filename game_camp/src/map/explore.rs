//! Explored-area memory: which 8m map cells the player has seen.
//! Cells are grouped by the 64m chunk they lie in: one 64-bit mask per chunk (8 by 8 cells).
//!
//! Port of Godot C# scripts/World/MapExplore.cs.

use std::collections::{HashMap, HashSet};
use glam::{IVec2, Vec2};
use serde::{Deserialize, Serialize};

// Godot: MapExplore.cs CellSize
pub const CELL_SIZE: f32 = 8.0;

// Godot: MapExplore.cs CellsPerChunk
pub const CELLS_PER_CHUNK: i32 = 8;

/// Which 8m map cells the player has seen.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MapExplore {
    masks: HashMap<IVec2, u64>,
    #[serde(skip)]
    dirty: HashSet<IVec2>,
}

impl MapExplore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Converts world XZ coordinates into an 8m map cell coordinate.
    // Godot: MapExplore.cs CellOf
    #[inline]
    pub fn cell_of(xz: Vec2) -> IVec2 {
        IVec2::new((xz.x / CELL_SIZE).floor() as i32, (xz.y / CELL_SIZE).floor() as i32)
    }

    /// Converts a cell coordinate into its enclosing 64m chunk coordinate.
    // Godot: MapExplore.cs ChunkOfCell
    #[inline]
    pub fn chunk_of_cell(cell: IVec2) -> IVec2 {
        IVec2::new(cell.x.div_euclid(CELLS_PER_CHUNK), cell.y.div_euclid(CELLS_PER_CHUNK))
    }

    /// True if the given cell has been revealed.
    // Godot: MapExplore.cs IsRevealed
    pub fn is_revealed(&self, cell: IVec2) -> bool {
        let chunk = Self::chunk_of_cell(cell);
        self.masks.get(&chunk).is_some_and(|&mask| (mask & Self::bit_of(cell)) != 0)
    }

    /// The 64-bit mask for a chunk (8x8 cells).
    // Godot: MapExplore.cs MaskOf
    pub fn mask_of(&self, chunk: IVec2) -> u64 {
        self.masks.get(&chunk).copied().unwrap_or(0)
    }

    /// Chunks with at least one seen cell.
    // Godot: MapExplore.cs Chunks
    pub fn chunks(&self) -> impl Iterator<Item = IVec2> + '_ {
        self.masks.keys().copied()
    }

    /// Returns the number of explored chunks.
    pub fn chunk_count(&self) -> usize {
        self.masks.len()
    }

    /// Chunks whose picture must be rebuilt.
    // Godot: MapExplore.cs IsDirty
    pub fn is_dirty(&self, chunk: IVec2) -> bool {
        self.dirty.contains(&chunk)
    }

    /// Clears the dirty flag for a chunk.
    // Godot: MapExplore.cs ClearDirty
    pub fn clear_dirty(&mut self, chunk: IVec2) {
        self.dirty.remove(&chunk);
    }

    /// Marks every cell whose centre lies within radius of the point. Returns how many were new.
    // Godot: MapExplore.cs Reveal
    pub fn reveal(&mut self, centre: Vec2, radius: f32) -> usize {
        let middle = Self::cell_of(centre);
        let reach = (radius / CELL_SIZE).ceil() as i32;
        let radius_squared = radius * radius;
        let mut added = 0;

        for dz in -reach..=reach {
            for dx in -reach..=reach {
                let cell = IVec2::new(middle.x + dx, middle.y + dz);
                let cell_centre = (cell.as_vec2() + Vec2::splat(0.5)) * CELL_SIZE;
                if (cell_centre - centre).length_squared() <= radius_squared && self.set_revealed(cell) {
                    added += 1;
                }
            }
        }
        added
    }

    /// Saves each chunk's mask as 16 hex digits, keyed "x_z".
    // Godot: MapExplore.cs ToDictionary
    pub fn to_save(&self) -> serde_json::Value {
        let mut map = serde_json::Map::with_capacity(self.masks.len());
        for (chunk, mask) in &self.masks {
            map.insert(format!("{}_{}", chunk.x, chunk.y), serde_json::Value::String(format!("{:016x}", mask)));
        }
        serde_json::Value::Object(map)
    }

    /// Replaces the seen cells with the saved ones. Bad entries are skipped. Every chunk is redrawn once.
    // Godot: MapExplore.cs Load
    pub fn from_save(&mut self, data: &serde_json::Value) {
        self.masks.clear();
        self.dirty.clear();

        if let Some(obj) = data.as_object() {
            for (key, val) in obj {
                let mut parts = key.split('_');
                if let (Some(x_str), Some(z_str)) = (parts.next(), parts.next())
                    && parts.next().is_none()
                    && let Ok(x) = x_str.parse::<i32>()
                    && let Ok(z) = z_str.parse::<i32>()
                    && let Some(hex_str) = val.as_str()
                    && let Ok(mask) = u64::from_str_radix(hex_str, 16)
                    && mask != 0
                {
                    let chunk = IVec2::new(x, z);
                    self.masks.insert(chunk, mask);
                    self.dirty.insert(chunk);
                }
            }
        }
    }

    fn set_revealed(&mut self, cell: IVec2) -> bool {
        let chunk = Self::chunk_of_cell(cell);
        let mask = self.masks.entry(chunk).or_insert(0);
        let bit = Self::bit_of(cell);
        if (*mask & bit) != 0 {
            return false;
        }
        *mask |= bit;
        self.dirty.insert(chunk);
        true
    }

    #[inline]
    fn bit_of(cell: IVec2) -> u64 {
        let lx = cell.x.rem_euclid(CELLS_PER_CHUNK) as u64;
        let ly = cell.y.rem_euclid(CELLS_PER_CHUNK) as u64;
        1u64 << (lx + ly * CELLS_PER_CHUNK as u64)
    }
}
