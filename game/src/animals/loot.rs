//! What a butchered animal gives (item id and count).
// Godot: AnimalLoot.cs

use super::config::AnimalSpeciesData;

pub const MEAT_ITEM: &str = "raw_meat";
pub const HIDE_ITEM: &str = "hide";
pub const ANTLER_ITEM: &str = "antler";

pub struct AnimalLoot;

impl AnimalLoot {
    // Godot: AnimalLoot.cs Roll
    pub fn roll(species: &AnimalSpeciesData, buck: bool, extra_meat: u32) -> Vec<(String, u32)> {
        let mut yields = Vec::new();
        let meat = species.meat + extra_meat;
        if meat > 0 {
            yields.push((MEAT_ITEM.to_string(), meat));
        }
        if species.hide > 0 {
            yields.push((HIDE_ITEM.to_string(), species.hide));
        }
        if buck && species.antler > 0 {
            yields.push((ANTLER_ITEM.to_string(), species.antler));
        }
        yields
    }
}
