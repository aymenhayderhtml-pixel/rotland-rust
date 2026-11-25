//! Placement checking logic and rules for base building.

use std::collections::HashMap;

use glam::{IVec2, Vec2, Vec3};

use super::def::{BuildConfig, SnapTarget};
use super::state::{is_edge_part, BaseState};

/// Reasons why a part placement is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invalid {
    UnknownPart,
    OverlappingPart,
    NeedsFloorBelow,
    SlopeTooSteep,
    Blocked,
    TooFarFromCamp,
    MaxFloorsExceeded,
}

impl std::fmt::Display for Invalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownPart => write!(f, "Unknown building part"),
            Self::OverlappingPart => write!(f, "Overlapping another part"),
            Self::NeedsFloorBelow => write!(f, "Needs a floor below"),
            Self::SlopeTooSteep => write!(f, "Ground slope is too steep"),
            Self::Blocked => write!(f, "Location is blocked"),
            Self::TooFarFromCamp => write!(f, "Too far from camp"),
            Self::MaxFloorsExceeded => write!(f, "Maximum floor height reached"),
        }
    }
}

impl std::error::Error for Invalid {}

/// Alias for Invalid as placement error.
pub type PlacementError = Invalid;

/// Successful placement plan output.
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    pub part: String,
    pub cell: IVec2,
    pub floor: u32,
    pub rot: u8,
    pub position: Vec3,
    pub yaw: f32,
    pub hp: f32,
    pub cost: HashMap<String, u32>,
}

impl Placement {
    /// World position and yaw transform.
    pub fn world_transform(&self) -> (Vec3, f32) {
        (self.position, self.yaw)
    }
}

/// Converts a 90-degree rotation index (0..4) into Godot yaw angle in radians.
/// Rule 7: forward = (-sin(yaw), 0, -cos(yaw)). Facing East (+X) is -PI/2.
pub fn rot_to_yaw(rot: u8) -> f32 {
    match rot % 4 {
        0 => 0.0,
        1 => -std::f32::consts::FRAC_PI_2,
        2 => std::f32::consts::PI,
        3 => std::f32::consts::FRAC_PI_2,
        _ => unreachable!(),
    }
}

/// Returns the neighbor cell reached across the edge specified by rot.
pub fn neighbor_cell(cell: IVec2, rot: u8) -> IVec2 {
    match rot % 4 {
        0 => IVec2::new(cell.x, cell.y - 1), // North (-Z)
        1 => IVec2::new(cell.x + 1, cell.y), // East (+X)
        2 => IVec2::new(cell.x, cell.y + 1), // South (+Z)
        3 => IVec2::new(cell.x - 1, cell.y), // West (-X)
        _ => unreachable!(),
    }
}

/// Canonical undirected edge key connecting two adjacent cells at a floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanonicalEdge {
    pub floor: u32,
    pub cell_a: IVec2,
    pub cell_b: IVec2,
}

/// Returns the canonical undirected edge between `cell` and its neighbor via `rot`.
pub fn canonical_edge(floor: u32, cell: IVec2, rot: u8) -> CanonicalEdge {
    let neighbor = neighbor_cell(cell, rot);
    let (cell_a, cell_b) = if (cell.x, cell.y) <= (neighbor.x, neighbor.y) {
        (cell, neighbor)
    } else {
        (neighbor, cell)
    };
    CanonicalEdge { floor, cell_a, cell_b }
}

/// Planner for placement validation.
pub struct BuildPlan;

impl BuildPlan {
    /// Checks if a part can be placed at the given grid cell, floor, and rotation.
    #[allow(clippy::too_many_arguments)]
    pub fn check(
        state: &BaseState,
        cfg: &BuildConfig,
        part: &str,
        cell: IVec2,
        floor: u32,
        rot: u8,
        ground: impl Fn(f32, f32) -> f32,
        blocked: impl Fn(Vec2, Vec2) -> bool,
    ) -> Result<Placement, Invalid> {
        let def = cfg.get(part).ok_or(Invalid::UnknownPart)?;

        // 1. Max floors check
        if floor >= cfg.max_floors {
            return Err(Invalid::MaxFloorsExceeded);
        }

        // Cell center in world coordinates (X, Z)
        let cell_center = Vec2::new(
            cell.x as f32 * cfg.grid_size,
            cell.y as f32 * cfg.grid_size,
        );

        // 2. Camp radius check (if onlyNearCamp is enabled)
        if cfg.only_near_camp {
            let dist = cell_center.distance(cfg.camp_pos_vec2());
            if dist > cfg.camp_radius {
                return Err(Invalid::TooFarFromCamp);
            }
        }

        // 3. Blocked check (closure check against POI / lot footprints)
        let half_size = cfg.grid_size * 0.5;
        let min_xz = cell_center - Vec2::splat(half_size);
        let max_xz = cell_center + Vec2::splat(half_size);
        if blocked(min_xz, max_xz) {
            return Err(Invalid::Blocked);
        }

        // 4. Overlapping part check
        let is_edge = def.snap == SnapTarget::Edge || is_edge_part(part);
        if is_edge {
            if state.has_edge_part_at(floor, cell, rot) {
                return Err(Invalid::OverlappingPart);
            }
        } else if def.is_floor {
            if state.has_floor_at(cell, floor) {
                return Err(Invalid::OverlappingPart);
            }
        } else if def.is_roof {
            if state.has_roof_at(cell, floor) {
                return Err(Invalid::OverlappingPart);
            }
        } else if state.has_object_at(cell, floor) {
            return Err(Invalid::OverlappingPart);
        }

        // 5. Floor below support requirement (upper floors and roofs)
        if floor > 0 {
            if is_edge {
                let neighbor = neighbor_cell(cell, rot);
                let supported = state.has_floor_at(cell, floor - 1)
                    || state.has_floor_at(neighbor, floor - 1)
                    || state.has_edge_part_at(floor - 1, cell, rot);
                if !supported {
                    return Err(Invalid::NeedsFloorBelow);
                }
            } else if !state.has_floor_at(cell, floor - 1) {
                return Err(Invalid::NeedsFloorBelow);
            }
        }

        if def.is_roof {
            let has_support = if floor == 0 {
                state.has_floor_at(cell, 0)
            } else {
                state.has_floor_at(cell, floor) || state.has_floor_at(cell, floor - 1)
            };
            if !has_support {
                return Err(Invalid::NeedsFloorBelow);
            }
        }

        // 6. Slope check (on ground / floor 0)
        let ground_center = ground(cell_center.x, cell_center.y);
        if floor == 0 {
            let h0 = ground_center;
            let h1 = ground(cell_center.x - half_size, cell_center.y - half_size);
            let h2 = ground(cell_center.x + half_size, cell_center.y - half_size);
            let h3 = ground(cell_center.x + half_size, cell_center.y + half_size);
            let h4 = ground(cell_center.x - half_size, cell_center.y + half_size);

            let min_h = h0.min(h1).min(h2).min(h3).min(h4);
            let max_h = h0.max(h1).max(h2).max(h3).max(h4);

            if max_h - min_h > cfg.max_slope {
                return Err(Invalid::SlopeTooSteep);
            }
        }

        // 7. Calculate final transform position and yaw
        let yaw = rot_to_yaw(rot);
        let mut pos_x = cell_center.x;
        let mut pos_z = cell_center.y;
        let mut pos_y = ground_center + floor as f32 * cfg.floor_height;

        if is_edge {
            let (sy, cy) = yaw.sin_cos();
            // forward = (-sy, 0, -cy)
            pos_x += -sy * half_size;
            pos_z += -cy * half_size;
        } else if def.is_roof {
            pos_y += cfg.floor_height;
        }

        Ok(Placement {
            part: part.to_string(),
            cell,
            floor,
            rot: rot % 4,
            position: Vec3::new(pos_x, pos_y, pos_z),
            yaw,
            hp: def.hp,
            cost: def.cost.clone(),
        })
    }
}
