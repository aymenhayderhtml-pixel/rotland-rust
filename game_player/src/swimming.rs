//! Swimming and water movement mechanics.
//!
//! Zone classification from the lake:
//! - Shallow water: slows the player (speed multiplier 0.6).
//! - Deep water: swimming (speed multiplier 0.45), no sprint, no jump, stamina drain,
//!   drowning damage when stamina is exhausted, floating at water surface, hands lowered.

use std::path::Path;

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use world::{LakeLayout, LakeZone};

/// Configuration for swimming and water movement.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SwimmingConfig {
    pub shallow_speed_multiplier: f32,
    pub deep_speed_multiplier: f32,
    pub swim_stamina_drain_per_sec: f32,
    pub drowning_damage_per_sec: f32,
    pub chest_height: f32,
    pub dirty_water_cooldown_sec: f32,
    pub tired_stamina_fraction: f32,
}

impl Default for SwimmingConfig {
    fn default() -> Self {
        Self {
            // data: swimming.json shallowSpeedMultiplier
            shallow_speed_multiplier: 0.6,
            // data: swimming.json deepSpeedMultiplier
            deep_speed_multiplier: 0.45,
            // data: swimming.json swimStaminaDrainPerSec
            swim_stamina_drain_per_sec: 8.0,
            // data: swimming.json drowningDamagePerSec
            drowning_damage_per_sec: 10.0,
            // data: swimming.json chestHeight
            chest_height: 1.2,
            // data: swimming.json dirtyWaterCooldownSec
            dirty_water_cooldown_sec: 2.0,
            // data: swimming.json tiredStaminaFraction
            tired_stamina_fraction: 0.3,
        }
    }
}

impl SwimmingConfig {
    pub fn from_repo() -> Result<Self, Box<dyn std::error::Error>> {
        let path = paths::repo_path("data/swimming.json");
        Self::load(&path)
    }

    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let text = std::fs::read_to_string(path)?;
        Self::from_json(&text).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

/// Warning state shown when swimming in deep water with low or depleted stamina.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WaterWarning {
    None,
    Tired,
    Drowning,
}

/// Evaluates the swimming stamina warning for the current frame.
///
/// Only warns in deep water: [`WaterWarning::Drowning`] at 0 stamina,
/// [`WaterWarning::Tired`] below `cfg.tired_stamina_fraction`, and
/// [`WaterWarning::None`] otherwise.
pub fn water_warning(
    stamina_current: f32,
    stamina_max: f32,
    deep: bool,
    cfg: &SwimmingConfig,
) -> WaterWarning {
    if !deep {
        return WaterWarning::None;
    }
    if stamina_current <= 0.0 {
        return WaterWarning::Drowning;
    }
    if stamina_max <= 0.0 {
        return WaterWarning::None;
    }
    let fraction = (stamina_current / stamina_max).clamp(0.0, 1.0);
    if fraction < cfg.tired_stamina_fraction {
        WaterWarning::Tired
    } else {
        WaterWarning::None
    }
}

/// Result of evaluating water physics on the player.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaterMovementResult {
    pub zone: LakeZone,
    pub speed_multiplier: f32,
    pub can_sprint: bool,
    pub can_jump: bool,
    pub stamina_drain: f32,
    pub drowning_damage: f32,
    pub float_y: Option<f32>,
    pub disarm_weapons: bool,
}

/// Checks if a 2D position is on any dock in the lake.
pub fn is_on_dock(lake: &LakeLayout, p: Vec2) -> bool {
    // new: DECISIONS 2026-11-23
    let edge_pad = 0.5;
    // new: DECISIONS 2026-11-23
    let end_pad = 1.0;
    let half_w = lake.config.docks.width * 0.5 + edge_pad;
    for dock in &lake.docks {
        let dir = dock.forward();
        let perp = Vec2::new(-dir.y, dir.x);
        let to_p = p - dock.start;
        let along = to_p.dot(dir);
        let across = to_p.dot(perp).abs();
        if along >= -end_pad && along <= dock.length() + end_pad && across <= half_w {
            return true;
        }
    }
    false
}

/// Pure speed multiplier for a lake zone.
pub fn speed_multiplier_for_zone(zone: LakeZone, cfg: &SwimmingConfig) -> f32 {
    match zone {
        LakeZone::Land | LakeZone::Beach => 1.0,
        LakeZone::Shallow => cfg.shallow_speed_multiplier,
        LakeZone::Deep => cfg.deep_speed_multiplier,
    }
}

/// Evaluates water movement rules for the player.
pub fn evaluate_water(
    _pos: Vec3,
    zone: LakeZone,
    water_level: f32,
    stamina_current: f32,
    is_on_dock: bool,
    cfg: &SwimmingConfig,
    dt: f32,
) -> WaterMovementResult {
    if is_on_dock {
        return WaterMovementResult {
            zone,
            speed_multiplier: 1.0,
            can_sprint: true,
            can_jump: true,
            stamina_drain: 0.0,
            drowning_damage: 0.0,
            float_y: None,
            disarm_weapons: false,
        };
    }

    match zone {
        LakeZone::Land | LakeZone::Beach => WaterMovementResult {
            zone,
            speed_multiplier: 1.0,
            can_sprint: true,
            can_jump: true,
            stamina_drain: 0.0,
            drowning_damage: 0.0,
            float_y: None,
            disarm_weapons: false,
        },
        LakeZone::Shallow => WaterMovementResult {
            zone,
            speed_multiplier: cfg.shallow_speed_multiplier,
            can_sprint: true,
            can_jump: true,
            stamina_drain: 0.0,
            drowning_damage: 0.0,
            float_y: None,
            disarm_weapons: false,
        },
        LakeZone::Deep => {
            let float_y = water_level - cfg.chest_height;
            let stamina_drain = cfg.swim_stamina_drain_per_sec * dt;
            let drowning_damage = if stamina_current <= 0.0 {
                cfg.drowning_damage_per_sec * dt
            } else {
                0.0
            };
            WaterMovementResult {
                zone,
                speed_multiplier: cfg.deep_speed_multiplier,
                can_sprint: false,
                can_jump: false,
                stamina_drain,
                drowning_damage,
                float_y: Some(float_y),
                disarm_weapons: true,
            }
        }
    }
}

/// Classifies a river point as `Land`, `Shallow`, or `Deep` from water level, riverbed height, and player Y.
pub fn river_zone_at(
    player_y: f32,
    ground_y: f32,
    river_water_level: Option<f32>,
    cfg: &SwimmingConfig,
) -> LakeZone {
    let Some(water_y) = river_water_level else {
        return LakeZone::Land;
    };
    if player_y >= water_y {
        return LakeZone::Land;
    }
    let depth = water_y - ground_y;
    if depth >= cfg.chest_height {
        LakeZone::Deep
    } else if depth > 0.0 {
        LakeZone::Shallow
    } else {
        LakeZone::Land
    }
}

/// Checks if a player can gather dirty water while standing in shallow water.
pub fn try_gather_dirty_water(
    zone: LakeZone,
    cooldown_timer: &mut f32,
    cooldown_sec: f32,
) -> bool {
    if zone == LakeZone::Shallow && *cooldown_timer <= 0.0 {
        *cooldown_timer = cooldown_sec;
        true
    } else {
        false
    }
}
