use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::farming::def::CropDef;

fn default_till_range() -> f32 { 1.6 }
fn default_till_spacing() -> f32 { 2.2 }
fn default_water_window_hours() -> f32 { 24.0 }
fn default_dry_hours_to_die() -> f32 { 72.0 }
fn default_can_capacity() -> u32 { 5 }
fn default_trample_per_pass() -> f32 { 0.1 }
fn default_trample_half_size() -> f32 { 1.1 }
fn default_seed_back_chance() -> f32 { 0.5 }
fn default_yield_bonus_levels() -> Vec<u32> { vec![3, 6] }
fn default_camp_plots() -> Vec<[f32; 2]> {
    vec![[-10.0, 4.0], [-10.0, 8.0], [-13.0, 4.0], [-13.0, 8.0]]
}

/// Farming XP gained for each player action.
// Godot: FarmConfigData.cs FarmXpData
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FarmXpData {
    #[serde(default = "default_xp_till")]
    pub till: f32,
    #[serde(default = "default_xp_plant")]
    pub plant: f32,
    #[serde(default = "default_xp_water")]
    pub water: f32,
    #[serde(default = "default_xp_harvest")]
    pub harvest: f32,
}

fn default_xp_till() -> f32 { 2.0 }
fn default_xp_plant() -> f32 { 3.0 }
fn default_xp_water() -> f32 { 1.0 }
fn default_xp_harvest() -> f32 { 10.0 }

impl Default for FarmXpData {
    fn default() -> Self {
        Self {
            till: default_xp_till(),
            plant: default_xp_plant(),
            water: default_xp_water(),
            harvest: default_xp_harvest(),
        }
    }
}

/// Shape of data/farming.json: rules parameters, camp beds, and crop types.
// Godot: FarmConfigData.cs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FarmConfigData {
    /// Metres ahead of the player where a new bed is tilled.
    #[serde(default = "default_till_range")]
    pub till_range: f32,
    /// Two beds must have their centres at least this far apart (beds are 2 m square).
    #[serde(default = "default_till_spacing")]
    pub till_spacing: f32,
    /// A watering keeps a bed growing for this many in-game hours.
    #[serde(default = "default_water_window_hours")]
    pub water_window_hours: f32,
    /// A crop with no water for this many hours (3 dry days) dies.
    #[serde(default = "default_dry_hours_to_die")]
    pub dry_hours_to_die: f32,
    /// Waters the can holds when full.
    #[serde(default = "default_can_capacity")]
    pub can_capacity: u32,
    /// Chance per pass that a zombie walking over a field bed destroys its crop.
    #[serde(default = "default_trample_per_pass")]
    pub trample_per_pass: f32,
    /// Half the side of a bed in metres: a zombie inside this square walks over it.
    #[serde(default = "default_trample_half_size")]
    pub trample_half_size: f32,
    /// Chance that a harvest gives one seed back.
    #[serde(default = "default_seed_back_chance")]
    pub seed_back_chance: f32,
    /// Farming levels that each add one crop to a harvest.
    #[serde(default = "default_yield_bonus_levels")]
    pub yield_bonus_levels: Vec<u32>,
    #[serde(default)]
    pub xp: FarmXpData,
    /// Camp bed positions as [x, z] in camp-local metres (the camp centre is 0, 0).
    #[serde(default = "default_camp_plots")]
    pub camp_plots: Vec<[f32; 2]>,
    #[serde(default)]
    pub crops: Vec<CropDef>,
}

impl Default for FarmConfigData {
    fn default() -> Self {
        Self {
            till_range: default_till_range(),
            till_spacing: default_till_spacing(),
            water_window_hours: default_water_window_hours(),
            dry_hours_to_die: default_dry_hours_to_die(),
            can_capacity: default_can_capacity(),
            trample_per_pass: default_trample_per_pass(),
            trample_half_size: default_trample_half_size(),
            seed_back_chance: default_seed_back_chance(),
            yield_bonus_levels: default_yield_bonus_levels(),
            xp: FarmXpData::default(),
            camp_plots: default_camp_plots(),
            crops: Vec::new(),
        }
    }
}

impl FarmConfigData {
    /// The crop with this id, or None when unknown.
    // Godot: FarmConfig.cs TryCrop
    pub fn try_crop(&self, id: &str) -> Option<&CropDef> {
        self.crops.iter().find(|c| c.id == id)
    }

    /// The crop that seeds of this item plant, or None.
    // Godot: FarmConfig.cs CropForSeed
    pub fn crop_for_seed(&self, seed_id: &str) -> Option<&CropDef> {
        self.crops.iter().find(|c| c.seed == seed_id)
    }
}

#[derive(Debug)]
pub enum FarmLoadError {
    Io(std::io::Error),
    Parse(serde_json::Error),
    Validation(String),
}

impl std::fmt::Display for FarmLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "could not read farming file: {e}"),
            Self::Parse(e) => write!(f, "could not parse farming file: {e}"),
            Self::Validation(e) => write!(f, "invalid farming configuration: {e}"),
        }
    }
}

impl std::error::Error for FarmLoadError {}

/// Loader and validator for data/farming.json.
// Godot: FarmConfig.cs
pub struct FarmConfig;

impl FarmConfig {
    /// Loads and validates configuration from the repo's data/farming.json.
    // Godot: FarmConfig.cs Get
    pub fn from_repo() -> Result<FarmConfigData, FarmLoadError> {
        let path = paths::repo_path("data/farming.json");
        Self::load(&path)
    }

    /// Reads and validates a farming.json file at `path`.
    pub fn load(path: &Path) -> Result<FarmConfigData, FarmLoadError> {
        let text = std::fs::read_to_string(path).map_err(FarmLoadError::Io)?;
        Self::from_json(&text)
    }

    /// Parses farming.json text and validates the configuration.
    pub fn from_json(text: &str) -> Result<FarmConfigData, FarmLoadError> {
        let data: FarmConfigData = serde_json::from_str(text).map_err(FarmLoadError::Parse)?;
        Self::validate(&data)?;
        Ok(data)
    }

    /// Validates farming configuration ranges and crop values.
    // Godot: FarmConfig.cs Check
    pub fn validate(data: &FarmConfigData) -> Result<(), FarmLoadError> {
        let ok = data.till_range > 0.0
            && data.till_spacing > 2.0
            && data.water_window_hours > 0.0
            && data.dry_hours_to_die > data.water_window_hours
            && data.can_capacity > 0
            && data.trample_per_pass >= 0.0
            && data.trample_per_pass <= 1.0
            && data.trample_half_size > 0.0
            && data.seed_back_chance >= 0.0
            && data.seed_back_chance <= 1.0
            && !data.crops.is_empty();

        if !ok {
            return Err(FarmLoadError::Validation(
                "data/farming.json has a value out of range".to_string(),
            ));
        }

        for spot in &data.camp_plots {
            if spot[0].abs() > 16.0 || spot[1].abs() > 16.0 {
                return Err(FarmLoadError::Validation(
                    "data/farming.json camp plot out of range".to_string(),
                ));
            }
        }

        for crop in &data.crops {
            if crop.id.is_empty()
                || crop.yield_count == 0
                || crop.grow_days <= 0.0
                || crop.model.is_empty()
            {
                return Err(FarmLoadError::Validation(format!(
                    "data/farming.json: crop '{}' has a bad value",
                    crop.id
                )));
            }
        }

        Ok(())
    }
}
