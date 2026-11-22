//! In-memory state of the build mode system.

use std::collections::HashMap;

use game::building::{BaseState, BuildConfig, Invalid, Placement};
use physics::ColliderHandle;

use super::types::PARTS_ORDER;

/// In-memory state of build mode, containing part selection, rotation, and player's base parts.
pub struct BuildModeState {
    pub active: bool,
    pub selected_index: usize,
    pub rotation: u8,
    pub base: BaseState,
    pub config: BuildConfig,
    pub current_placement: Option<Placement>,
    pub last_error: Option<Invalid>,
    pub aimed_part_id: Option<u32>,
    pub part_colliders: HashMap<u32, ColliderHandle>,
    pub build_test_initialized: bool,
    pub save_loaded: bool,
    pub toast: Option<String>,
}

impl Default for BuildModeState {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildModeState {
    /// Creates a new BuildModeState, loading the default configuration from data/build_parts.json.
    pub fn new() -> Self {
        let config = BuildConfig::load_default().unwrap_or_default();
        Self {
            active: false,
            selected_index: 0,
            rotation: 0,
            base: BaseState::default(),
            config,
            current_placement: None,
            last_error: None,
            aimed_part_id: None,
            part_colliders: HashMap::new(),
            build_test_initialized: false,
            save_loaded: false,
            toast: None,
        }
    }

    /// Resets the build mode state (e.g. on new game or load).
    pub fn reset(&mut self) {
        self.active = false;
        self.selected_index = 0;
        self.rotation = 0;
        self.base = BaseState::default();
        self.current_placement = None;
        self.last_error = None;
        self.aimed_part_id = None;
        self.part_colliders.clear();
        self.build_test_initialized = false;
        self.save_loaded = false;
        self.toast = None;
    }

    /// Serializes placed base parts to save format.
    pub fn to_save(&self) -> save::BaseSaveData {
        let parts = self
            .base
            .parts
            .iter()
            .map(|p| save::BasePartSaveData {
                id: p.id,
                part: p.part.clone(),
                cell_x: p.cell.x,
                cell_z: p.cell.y,
                floor: p.floor,
                rot: p.rot,
                hp: p.hp,
                door_open: p.door_open,
                position: save::Vec3Data::new(p.position.x, p.position.y, p.position.z),
                yaw: p.yaw,
            })
            .collect();
        save::BaseSaveData::new(self.base.next_id, parts)
    }

    /// Loads placed base parts from save format.
    pub fn load_from_save(&mut self, save_data: &save::BaseSaveData) {
        self.base.next_id = save_data.next_id;
        self.base.parts.clear();
        for p in &save_data.parts {
            self.base.parts.push(game::building::PlacedPart {
                id: p.id,
                part: p.part.clone(),
                cell: glam::IVec2::new(p.cell_x, p.cell_z),
                floor: p.floor,
                rot: p.rot,
                hp: p.hp,
                door_open: p.door_open,
                position: glam::Vec3::new(p.position.x, p.position.y, p.position.z),
                yaw: p.yaw,
            });
        }
        self.part_colliders.clear();
        self.save_loaded = true;
    }

    /// Checks the `--build-test` command-line argument:
    /// starts in build mode with 50 wood in the bag for screenshots.
    pub fn check_build_test_flag(&mut self, inventory: Option<&mut game::Inventory>) {
        if !self.build_test_initialized {
            let has_flag = std::env::args().any(|a| a == "--build-test");
            if has_flag {
                self.active = true;
                if let Some(inv) = inventory {
                    inv.add("wood", 50);
                }
                self.build_test_initialized = true;
            }
        }
    }

    /// Toggles build mode on or off.
    pub fn toggle(&mut self) -> bool {
        self.active = !self.active;
        if !self.active {
            self.current_placement = None;
            self.last_error = None;
            self.aimed_part_id = None;
        }
        self.active
    }

    /// Sets whether build mode is active.
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
        if !active {
            self.current_placement = None;
            self.last_error = None;
            self.aimed_part_id = None;
        }
    }

    /// Returns the currently selected part ID.
    pub fn selected_part(&self) -> &'static str {
        if PARTS_ORDER.is_empty() {
            "floor"
        } else {
            PARTS_ORDER[self.selected_index % PARTS_ORDER.len()]
        }
    }

    /// Cycles to the next available building part.
    pub fn cycle_next(&mut self) {
        if !PARTS_ORDER.is_empty() {
            self.selected_index = (self.selected_index + 1) % PARTS_ORDER.len();
        }
    }

    /// Cycles to the previous available building part.
    pub fn cycle_prev(&mut self) {
        if !PARTS_ORDER.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                PARTS_ORDER.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    /// Selects a specific slot index (e.g. from number keys 1..9).
    pub fn select_slot(&mut self, slot: usize) {
        if slot < PARTS_ORDER.len() {
            self.selected_index = slot;
        }
    }

    /// Rotates the active build part by 90 degrees clockwise (0..4).
    pub fn rotate_90(&mut self) {
        self.rotation = (self.rotation + 1) % 4;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::frame_input::build_mode::*;
    use game::Inventory;

    #[test]
    fn test_build_mode_state_cycling_and_rotation() {
        let mut state = BuildModeState::new();
        assert_eq!(state.selected_part(), "floor");
        assert_eq!(state.rotation, 0);

        state.rotate_90();
        assert_eq!(state.rotation, 1);
        state.rotate_90();
        state.rotate_90();
        state.rotate_90();
        assert_eq!(state.rotation, 0);

        state.cycle_next();
        assert_eq!(state.selected_part(), "wall");
        state.cycle_prev();
        assert_eq!(state.selected_part(), "floor");
        state.cycle_prev();
        assert_eq!(state.selected_part(), "barricade");

        state.select_slot(3);
        assert_eq!(state.selected_part(), "wall_door");
    }

    #[test]
    fn test_build_mode_toggle_and_reset() {
        let mut state = BuildModeState::new();
        assert!(!state.active);

        state.toggle();
        assert!(state.active);

        state.toggle();
        assert!(!state.active);
    }

    #[test]
    fn test_part_model_and_display_name() {
        assert_eq!(part_model_name("floor"), "base_floor");
        assert_eq!(part_model_name("wall_door"), "base_wall_door");
        assert_eq!(part_display_name("wall"), "Wall");
        assert_eq!(part_display_name("stairs"), "Stairs");
    }

    #[test]
    fn test_try_place_and_remove_with_inventory() {
        let mut state = BuildModeState::new();
        state.active = true;
        let mut inv = Inventory::default();
        inv.add("wood", 10);

        // Without placement set, try_place fails
        assert!(!try_place(&mut state, Some(&mut inv)));

        // Setup valid placement
        let placement = game::building::Placement {
            part: "wall".into(),
            cell: glam::IVec2::new(0, 0),
            floor: 0,
            rot: 0,
            position: glam::Vec3::new(0.0, 0.0, 0.0),
            yaw: 0.0,
            hp: 150.0,
            cost: [("wood".into(), 4)].into_iter().collect(),
        };
        state.current_placement = Some(placement);

        let placed = try_place(&mut state, Some(&mut inv));
        assert!(placed);
        assert_eq!(inv.count("wood"), 6); // 10 - 4
        assert_eq!(state.base.len(), 1);

        let placed_id = state.base.parts[0].id;
        state.aimed_part_id = Some(placed_id);

        let removed = try_remove(&mut state, Some(&mut inv));
        assert!(removed);
        assert_eq!(state.base.len(), 0);
        // Refund is 50% of 4 wood = 2 wood
        assert_eq!(inv.count("wood"), 8);
    }

    #[test]
    fn test_build_mode_save_load_roundtrip() {
        let mut state = BuildModeState::new();
        state.base.next_id = 4;
        state.base.parts.push(game::building::PlacedPart {
            id: 1,
            part: "floor".into(),
            cell: glam::IVec2::new(1, 2),
            floor: 0,
            rot: 0,
            hp: 100.0,
            door_open: false,
            position: glam::Vec3::new(2.0, 0.0, 4.0),
            yaw: 0.0,
        });
        state.base.parts.push(game::building::PlacedPart {
            id: 2,
            part: "wall".into(),
            cell: glam::IVec2::new(1, 2),
            floor: 0,
            rot: 1,
            hp: 150.0,
            door_open: false,
            position: glam::Vec3::new(3.0, 0.0, 4.0),
            yaw: -std::f32::consts::FRAC_PI_2,
        });
        state.base.parts.push(game::building::PlacedPart {
            id: 3,
            part: "wall_door".into(),
            cell: glam::IVec2::new(1, 3),
            floor: 0,
            rot: 2,
            hp: 120.0,
            door_open: true,
            position: glam::Vec3::new(2.0, 0.0, 6.0),
            yaw: -std::f32::consts::PI,
        });

        let saved = state.to_save();
        assert_eq!(saved.next_id, 4);
        assert_eq!(saved.parts.len(), 3);

        let mut loaded_state = BuildModeState::new();
        loaded_state.load_from_save(&saved);

        assert_eq!(loaded_state.base.next_id, 4);
        assert_eq!(loaded_state.base.len(), 3);
        assert_eq!(loaded_state.base.parts[0].part, "floor");
        assert_eq!(loaded_state.base.parts[0].cell, glam::IVec2::new(1, 2));
        assert_eq!(loaded_state.base.parts[1].part, "wall");
        assert_eq!(loaded_state.base.parts[1].rot, 1);
        assert_eq!(loaded_state.base.parts[2].part, "wall_door");
        assert!(loaded_state.base.parts[2].door_open);
    }
}
