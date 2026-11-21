//! Base state representation, mutation, support tracking, and serialization.

use std::collections::HashMap;

use glam::{IVec2, Vec3};
use serde::{Deserialize, Serialize};

use super::def::BuildConfig;
use super::plan::{canonical_edge, neighbor_cell, rot_to_yaw, Placement};

/// Helper to identify edge-placed parts (walls, doors, windows, half walls).
pub fn is_edge_part(part_name: &str) -> bool {
    matches!(
        part_name,
        "wall" | "wall_window" | "wall_door" | "half_wall"
    )
}

/// One placed building part instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlacedPart {
    #[serde(default)]
    pub id: u32,
    #[serde(default)]
    pub part: String,
    #[serde(default)]
    pub cell: IVec2,
    #[serde(default)]
    pub floor: u32,
    #[serde(default)]
    pub rot: u8,
    #[serde(default)]
    pub hp: f32,
    #[serde(default)]
    pub door_open: bool,
    #[serde(default)]
    pub position: Vec3,
    #[serde(default)]
    pub yaw: f32,
}

impl Default for PlacedPart {
    fn default() -> Self {
        Self {
            id: 0,
            part: String::new(),
            cell: IVec2::ZERO,
            floor: 0,
            rot: 0,
            hp: 0.0,
            door_open: false,
            position: Vec3::ZERO,
            yaw: 0.0,
        }
    }
}

/// Serialized state of player-built base parts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BaseState {
    #[serde(default)]
    pub next_id: u32,
    #[serde(default)]
    pub parts: Vec<PlacedPart>,
}

impl BaseState {
    /// Number of placed parts in the base.
    pub fn len(&self) -> usize {
        self.parts.len()
    }

    /// True if no parts have been placed.
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// Look up a placed part by ID.
    pub fn get(&self, part_id: u32) -> Option<&PlacedPart> {
        self.parts.iter().find(|p| p.id == part_id)
    }

    /// Look up a mutable placed part by ID.
    pub fn get_mut(&mut self, part_id: u32) -> Option<&mut PlacedPart> {
        self.parts.iter_mut().find(|p| p.id == part_id)
    }

    /// Places a checked placement into the state and assigns it a unique ID.
    pub fn place(&mut self, placement: Placement) -> u32 {
        let id = if self.next_id == 0 { 1 } else { self.next_id };
        self.next_id = id + 1;
        let placed = PlacedPart {
            id,
            part: placement.part,
            cell: placement.cell,
            floor: placement.floor,
            rot: placement.rot,
            hp: placement.hp,
            door_open: false,
            position: placement.position,
            yaw: placement.yaw,
        };
        self.parts.push(placed);
        id
    }

    /// Directly inserts a PlacedPart, updating next_id if needed.
    pub fn place_part(&mut self, mut part: PlacedPart) -> u32 {
        if part.id == 0 {
            let id = if self.next_id == 0 { 1 } else { self.next_id };
            self.next_id = id + 1;
            part.id = id;
        } else if part.id >= self.next_id {
            self.next_id = part.id + 1;
        }
        if part.position == Vec3::ZERO && part.yaw == 0.0 && (part.cell != IVec2::ZERO || part.floor != 0 || part.rot != 0) {
            let half = 1.0;
            let yaw = rot_to_yaw(part.rot);
            let mut pos_x = part.cell.x as f32 * 2.0;
            let mut pos_z = part.cell.y as f32 * 2.0;
            let mut pos_y = part.floor as f32 * 3.0;
            if is_edge_part(&part.part) {
                let (sy, cy) = yaw.sin_cos();
                pos_x += -sy * half;
                pos_z += -cy * half;
            } else if part.part == "roof" {
                pos_y += 3.0;
            }
            part.position = Vec3::new(pos_x, pos_y, pos_z);
            part.yaw = yaw;
        }
        let id = part.id;
        self.parts.push(part);
        id
    }

    /// Removes a placed part by ID, returns refunded materials, and collapses unsupported parts.
    pub fn remove(&mut self, cfg: &BuildConfig, part_id: u32) -> Option<HashMap<String, u32>> {
        let idx = self.parts.iter().position(|p| p.id == part_id)?;
        let part = self.parts.remove(idx);

        let mut refund = HashMap::new();
        if let Some(def) = cfg.get(&part.part) {
            for (item, count) in &def.cost {
                let refunded_count = (*count as f32 * cfg.refund_share).floor() as u32;
                if refunded_count > 0 {
                    refund.insert(item.clone(), refunded_count);
                }
            }
        }

        self.collapse_unsupported();
        Some(refund)
    }

    /// Removes a part without refund calculation, collapsing unsupported parts.
    pub fn remove_simple(&mut self, part_id: u32) -> bool {
        let Some(idx) = self.parts.iter().position(|p| p.id == part_id) else {
            return false;
        };
        self.parts.remove(idx);
        self.collapse_unsupported();
        true
    }

    /// Inflicts damage on a placed part. Destroys it if HP reaches 0 and collapses unsupported parts.
    /// Returns true if destroyed, false otherwise.
    pub fn damage(&mut self, part_id: u32, amount: f32) -> bool {
        let Some(part) = self.parts.iter_mut().find(|p| p.id == part_id) else {
            return false;
        };

        if part.hp <= amount {
            self.parts.retain(|p| p.id != part_id);
            self.collapse_unsupported();
            true
        } else {
            part.hp -= amount;
            false
        }
    }

    /// Repairs a placed part back to its maximum HP.
    pub fn repair(&mut self, cfg: &BuildConfig, part_id: u32) -> bool {
        let Some(part) = self.parts.iter_mut().find(|p| p.id == part_id) else {
            return false;
        };
        let Some(def) = cfg.get(&part.part) else {
            return false;
        };
        part.hp = def.hp;
        true
    }

    /// Repairs a placed part by a specific amount, capped at maximum HP.
    pub fn repair_amount(&mut self, cfg: &BuildConfig, part_id: u32, amount: f32) -> bool {
        let Some(part) = self.parts.iter_mut().find(|p| p.id == part_id) else {
            return false;
        };
        let Some(def) = cfg.get(&part.part) else {
            return false;
        };
        part.hp = (part.hp + amount).min(def.hp);
        true
    }

    /// Toggles door open/closed state. Returns the new open state, or false if not found.
    pub fn toggle_door(&mut self, part_id: u32) -> bool {
        let Some(part) = self.parts.iter_mut().find(|p| p.id == part_id) else {
            return false;
        };
        part.door_open = !part.door_open;
        part.door_open
    }

    /// Checks if a door is currently open.
    pub fn is_door_open(&self, part_id: u32) -> bool {
        self.get(part_id).map(|p| p.door_open).unwrap_or(false)
    }

    /// Finds all placed parts within a given 3D radius.
    pub fn parts_in_radius(&self, p: Vec3, r: f32) -> Vec<&PlacedPart> {
        self.parts
            .iter()
            .filter(|part| (part.position - p).length() <= r)
            .collect()
    }

    /// Finds all placed part IDs within a given 3D radius.
    pub fn parts_in_radius_ids(&self, p: Vec3, r: f32) -> Vec<u32> {
        self.parts
            .iter()
            .filter(|part| (part.position - p).length() <= r)
            .map(|part| part.id)
            .collect()
    }

    /// Returns the world transform (position, yaw) for drawing and colliders.
    pub fn world_transform(&self, part: &PlacedPart) -> (Vec3, f32) {
        (part.position, part.yaw)
    }

    /// Returns the world transform (position, yaw) for a part ID.
    pub fn world_transform_id(&self, part_id: u32) -> Option<(Vec3, f32)> {
        self.get(part_id).map(|p| (p.position, p.yaw))
    }

    /// Checks whether a floor exists at the given cell and floor index.
    pub fn has_floor_at(&self, cell: IVec2, floor: u32) -> bool {
        self.parts.iter().any(|p| p.cell == cell && p.floor == floor && p.part == "floor")
    }

    /// Checks whether a roof exists at the given cell and floor index.
    pub fn has_roof_at(&self, cell: IVec2, floor: u32) -> bool {
        self.parts.iter().any(|p| p.cell == cell && p.floor == floor && p.part == "roof")
    }

    /// Checks whether an object (furniture/non-floor/non-roof/non-edge) exists at cell and floor.
    pub fn has_object_at(&self, cell: IVec2, floor: u32) -> bool {
        self.parts.iter().any(|p| {
            p.cell == cell && p.floor == floor && p.part != "floor" && p.part != "roof" && !is_edge_part(&p.part)
        })
    }

    /// Checks whether an edge part exists along the canonical edge of (floor, cell, rot).
    pub fn has_edge_part_at(&self, floor: u32, cell: IVec2, rot: u8) -> bool {
        let target = canonical_edge(floor, cell, rot);
        self.parts.iter().any(|p| {
            p.floor == floor && is_edge_part(&p.part) && canonical_edge(p.floor, p.cell, p.rot) == target
        })
    }

    /// Checks whether a placed part currently has valid structural support.
    pub fn is_supported(&self, part: &PlacedPart) -> bool {
        if part.part == "floor" {
            if part.floor == 0 {
                return true;
            }
            return self.has_floor_at(part.cell, part.floor - 1);
        }
        if part.part == "roof" {
            return self.has_floor_at(part.cell, part.floor)
                || (part.floor > 0 && self.has_floor_at(part.cell, part.floor - 1));
        }
        if is_edge_part(&part.part) {
            if part.floor == 0 {
                return true;
            }
            let neighbor = neighbor_cell(part.cell, part.rot);
            return self.has_floor_at(part.cell, part.floor)
                || self.has_floor_at(neighbor, part.floor)
                || self.has_floor_at(part.cell, part.floor - 1)
                || self.has_floor_at(neighbor, part.floor - 1)
                || self.has_edge_part_at(part.floor - 1, part.cell, part.rot);
        }

        // Cell objects (bed, workbench, storage_box, stairs, light, spikes, barricade)
        if part.floor > 0 {
            return self.has_floor_at(part.cell, part.floor);
        }
        if part.part == "spikes" || part.part == "barricade" {
            return true;
        }
        self.has_floor_at(part.cell, 0)
    }

    /// Iteratively destroys all placed parts that have lost their support.
    pub fn collapse_unsupported(&mut self) -> Vec<u32> {
        let mut collapsed = Vec::new();
        loop {
            let mut to_remove = Vec::new();
            for part in &self.parts {
                if !self.is_supported(part) {
                    to_remove.push(part.id);
                }
            }
            if to_remove.is_empty() {
                break;
            }
            for id in &to_remove {
                self.parts.retain(|p| p.id != *id);
                collapsed.push(*id);
            }
        }
        collapsed
    }

    /// Checks whether a placed part blocks zombie movement. Open doors do not block.
    pub fn blocks_zombies(&self, cfg: &BuildConfig, part_id: u32) -> bool {
        let Some(part) = self.get(part_id) else {
            return false;
        };
        if part.door_open {
            return false;
        }
        cfg.get(&part.part).map(|d| d.blocks_zombies).unwrap_or(false)
    }
}
