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
    /// Body warmth is in the Freezing or Cold band (R32-05 `temp_cold` trigger).
    pub temp_cold: bool,
    /// Body warmth is in the Hot or Overheated band (R32-05 `temp_hot` trigger).
    pub temp_hot: bool,
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
            temp_cold: false,
            temp_hot: false,
        }
    }
}

impl HintContext {
    pub fn new() -> Self {
        Self::default()
    }
}
