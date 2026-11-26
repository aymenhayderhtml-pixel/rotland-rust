use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

use crate::items::ItemStack;

/// Errors returned when loading `data/weapon_wear.json`.
#[derive(Debug)]
pub enum WearLoadError {
    Io(std::io::Error),
    Parse(serde_json::Error),
}

impl std::fmt::Display for WearLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WearLoadError::Io(e) => write!(f, "could not read weapon wear file: {e}"),
            WearLoadError::Parse(e) => write!(f, "could not parse weapon wear file: {e}"),
        }
    }
}

impl std::error::Error for WearLoadError {}

/// Per-weapon wear rates from `data/weapon_wear.json`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WeaponWearEntry {
    pub wear_per_shot: f32,
    pub wear_per_hit: f32,
}

/// Global and per-weapon wear settings loaded from `data/weapon_wear.json`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WearConfig {
    pub jam_below: f32,
    pub jam_chance_max: f32,
    pub spread_below: f32,
    pub spread_mult_max: f32,
    pub melee_damage_mult_at_zero: f32,
    pub weapons: BTreeMap<String, WeaponWearEntry>,
}

impl Default for WearConfig {
    fn default() -> Self {
        Self {
            // data: weapon_wear.json jamBelow
            jam_below: 0.3,
            // data: weapon_wear.json jamChanceMax
            jam_chance_max: 0.15,
            // data: weapon_wear.json spreadBelow
            spread_below: 0.5,
            // data: weapon_wear.json spreadMultMax
            spread_mult_max: 1.6,
            // data: weapon_wear.json meleeDamageMultAtZero
            melee_damage_mult_at_zero: 0.5,
            weapons: BTreeMap::new(),
        }
    }
}

impl WearConfig {
    /// Loads `data/weapon_wear.json` from the repository root.
    pub fn from_repo() -> Result<Self, WearLoadError> {
        let path = paths::repo_path("data/weapon_wear.json");
        Self::load(&path)
    }

    /// Reads a weapon wear JSON file at `path`.
    pub fn load(path: &Path) -> Result<Self, WearLoadError> {
        let text = std::fs::read_to_string(path).map_err(WearLoadError::Io)?;
        Self::from_json(&text).map_err(WearLoadError::Parse)
    }

    /// Parses weapon wear configuration from JSON text.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Condition lost per shot for the given weapon id (0.0 if not configured).
    pub fn wear_per_shot(&self, weapon_id: &str) -> f32 {
        self.weapons
            .get(weapon_id)
            .map_or(0.0, |e| e.wear_per_shot.max(0.0))
    }

    /// Condition lost per melee hit for the given weapon id (0.0 if not configured).
    pub fn wear_per_hit(&self, weapon_id: &str) -> f32 {
        self.weapons
            .get(weapon_id)
            .map_or(0.0, |e| e.wear_per_hit.max(0.0))
    }
}

/// Applies one shot's wear to a weapon stack (guns and bows). Never drops below 0.0.
pub fn apply_shot_wear(stack: &mut ItemStack, cfg: &WearConfig) {
    let delta = cfg.wear_per_shot(&stack.item_id);
    if delta > 0.0 {
        let next = (stack.condition() - delta).clamp(0.0, 1.0);
        stack.wear = Some(next);
    }
}

/// Applies one melee hit's wear to a weapon stack. Never drops below 0.0.
pub fn apply_hit_wear(stack: &mut ItemStack, cfg: &WearConfig) {
    let delta = cfg.wear_per_hit(&stack.item_id);
    if delta > 0.0 {
        let next = (stack.condition() - delta).clamp(0.0, 1.0);
        stack.wear = Some(next);
    }
}

/// True when a weapon has reached zero condition and cannot fire until repaired.
pub fn is_broken(cond: f32) -> bool {
    cond <= 0.0
}

/// Jam probability in 0.0..=jam_chance_max for the given condition.
/// Returns 0.0 when `cond >= cfg.jam_below`.
pub fn jam_chance(cond: f32, cfg: &WearConfig) -> f32 {
    let cond = cond.clamp(0.0, 1.0);
    if cfg.jam_below <= 0.0 || cond >= cfg.jam_below {
        return 0.0;
    }
    let t = ((cfg.jam_below - cond) / cfg.jam_below).clamp(0.0, 1.0);
    t * cfg.jam_chance_max.max(0.0)
}

/// Rolls whether a shot jams at `cond` using an RNG closure returning `[0.0, 1.0)`.
pub fn jam_roll(cond: f32, cfg: &WearConfig, mut rng: impl FnMut() -> f32) -> bool {
    let chance = jam_chance(cond, cfg);
    chance > 0.0 && rng() < chance
}

/// Spread multiplier at `cond`: 1.0 at or above `spread_below`, rising smoothly to `spread_mult_max` at 0.0.
pub fn spread_mult(cond: f32, cfg: &WearConfig) -> f32 {
    let cond = cond.clamp(0.0, 1.0);
    if cfg.spread_below <= 0.0 || cond >= cfg.spread_below {
        return 1.0;
    }
    let t = ((cfg.spread_below - cond) / cfg.spread_below).clamp(0.0, 1.0);
    1.0 + t * (cfg.spread_mult_max - 1.0).max(0.0)
}

/// Melee damage multiplier at `cond`: 1.0 when brand new, scaling smoothly down to `melee_damage_mult_at_zero` at 0.0.
pub fn melee_damage_mult(cond: f32, cfg: &WearConfig) -> f32 {
    let cond = cond.clamp(0.0, 1.0);
    let min_mult = cfg.melee_damage_mult_at_zero.clamp(0.0, 1.0);
    min_mult + cond * (1.0 - min_mult)
}
