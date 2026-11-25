//! The Hunting skill's effects on animals.
// Godot: AnimalHunting.cs

use game_core::items::Inventory;
use super::config::{AnimalButcherData, AnimalSpeciesData};

pub struct AnimalHunting;

impl AnimalHunting {
    // Godot: AnimalHunting.cs DamageMultiplier
    pub fn damage_multiplier(is_animal: bool, hunting_damage_bonus: f32) -> f32 {
        if !is_animal {
            1.0
        } else {
            1.0 + hunting_damage_bonus
        }
    }

    // Godot: AnimalHunting.cs StealthBonus
    pub fn stealth_bonus(hunting_stealth_bonus: f32) -> f32 {
        hunting_stealth_bonus
    }

    // Godot: AnimalHunting.cs ExtraMeat
    pub fn extra_meat(hunting_level: u32, butcher: &AnimalButcherData) -> u32 {
        let mut extra = 0;
        for &level in &butcher.extra_meat_levels {
            if hunting_level >= level {
                extra += 1;
            }
        }
        extra
    }

    // Godot: AnimalHunting.cs ButcherSeconds
    pub fn butcher_seconds(bag: &Inventory, butcher: &AnimalButcherData) -> f32 {
        if Self::has_tool(bag, butcher) {
            butcher.seconds / butcher.tool_speed_factor
        } else {
            butcher.seconds
        }
    }

    // Godot: AnimalHunting.cs HasTool
    pub fn has_tool(bag: &Inventory, butcher: &AnimalButcherData) -> bool {
        let slots = butcher.tool_slots.min(bag.slot_count());
        for i in 0..slots {
            if let Some(stack) = bag.slot(i)
                && butcher.tool_ids.iter().any(|id| id == &stack.item_id) {
                    return true;
                }
        }
        false
    }

    // Godot: AnimalHunting.cs AwardKill
    #[allow(dead_code)]
    fn award_kill(species: &AnimalSpeciesData) -> u32 {
        species.xp
    }
}
