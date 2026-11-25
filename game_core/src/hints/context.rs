//! Environmental context passed to HintSystem::tick to evaluate dynamic triggers.
//!
//! Port of trigger evaluations in scripts/UI/HintSystem.cs and HintNpcDistance.cs.

use std::collections::{HashMap, HashSet};

/// Environmental and player context required to evaluate hint triggers.
#[derive(Debug, Clone)]
pub struct HintContext {
    pub nearest_zombie_dist: Option<f32>,
    pub nearest_crate_dist: Option<f32>,
    pub nearest_board_dist: Option<f32>,
    pub npc_distances: HashMap<String, f32>,
    pub needs: HashMap<String, f32>,
    pub hour: f32,
    pub bag_weight_ratio: Option<f32>,
    pub inventory_items: HashSet<String>,
    pub is_menu_open: bool,
    pub is_dead: bool,
    pub hints_wanted: bool,
}

impl Default for HintContext {
    fn default() -> Self {
        Self {
            nearest_zombie_dist: None,
            nearest_crate_dist: None,
            nearest_board_dist: None,
            npc_distances: HashMap::new(),
            needs: HashMap::new(),
            hour: 12.0,
            bag_weight_ratio: None,
            inventory_items: HashSet::new(),
            is_menu_open: false,
            is_dead: false,
            hints_wanted: true,
        }
    }
}

impl HintContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the distance to the nearest NPC with a given role.
    // Godot: HintNpcDistance.cs
    #[allow(dead_code)]
    fn with_npc_distance(mut self, role: impl Into<String>, distance: f32) -> Self {
        self.npc_distances.insert(role.into(), distance);
        self
    }

    /// Sets a player need value (e.g. "food", "water").
    #[allow(dead_code)]
    fn with_need(mut self, need_id: impl Into<String>, value: f32) -> Self {
        self.needs.insert(need_id.into(), value);
        self
    }

    /// Adds an item ID present in the player inventory.
    #[allow(dead_code)]
    fn with_item(mut self, item_id: impl Into<String>) -> Self {
        self.inventory_items.insert(item_id.into());
        self
    }
}
