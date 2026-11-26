//! Type definitions, state and effects for camp upgrades.

use std::fmt;
use serde::{Deserialize, Serialize};

/// The five camp upgrade branches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpgradeLine {
    Walls,
    Guards,
    Workshop,
    Traders,
    Gunsmith,
}

impl UpgradeLine {
    pub const ALL: [Self; 5] = [
        Self::Walls,
        Self::Guards,
        Self::Workshop,
        Self::Traders,
        Self::Gunsmith,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Walls => "walls",
            Self::Guards => "guards",
            Self::Workshop => "workshop",
            Self::Traders => "traders",
            Self::Gunsmith => "gunsmith",
        }
    }
}

impl fmt::Display for UpgradeLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A specific item requirement and quantity for upgrading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialCost {
    #[serde(alias = "id")]
    pub item_id: String,
    pub count: u32,
}

impl MaterialCost {
    pub fn new(item_id: impl Into<String>, count: u32) -> Self {
        Self {
            item_id: item_id.into(),
            count,
        }
    }
}

/// Full cost of an upgrade: currency, materials, and in-game construction hours.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeCost {
    #[serde(default)]
    pub cash: i32,
    #[serde(default)]
    pub materials: Vec<MaterialCost>,
    #[serde(default)]
    pub build_hours: f32,
}

/// Prerequisite requirement on another upgrade line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpgradeRequirement {
    pub line: UpgradeLine,
    pub level: u8,
}

/// An upgrade currently being constructed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpgradeInProgress {
    pub line: UpgradeLine,
    pub target_level: u8,
    pub hours_left: f32,
}

/// Persistent save state for camp upgrades.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CampUpgradeState {
    pub walls_level: u8,
    pub guards_level: u8,
    pub workshop_level: u8,
    pub traders_level: u8,
    pub gunsmith_level: u8,
    pub in_progress: Option<UpgradeInProgress>,
    pub wall_hp: f32,
}

impl CampUpgradeState {
    pub fn level(&self, line: UpgradeLine) -> u8 {
        match line {
            UpgradeLine::Walls => self.walls_level,
            UpgradeLine::Guards => self.guards_level,
            UpgradeLine::Workshop => self.workshop_level,
            UpgradeLine::Traders => self.traders_level,
            UpgradeLine::Gunsmith => self.gunsmith_level,
        }
    }

    pub fn set_level(&mut self, line: UpgradeLine, lvl: u8) {
        match line {
            UpgradeLine::Walls => self.walls_level = lvl,
            UpgradeLine::Guards => self.guards_level = lvl,
            UpgradeLine::Workshop => self.workshop_level = lvl,
            UpgradeLine::Traders => self.traders_level = lvl,
            UpgradeLine::Gunsmith => self.gunsmith_level = lvl,
        }
    }

    pub fn has_walls(&self) -> bool {
        self.walls_level > 0
    }

    pub fn wall_hp(&self) -> f32 {
        self.wall_hp
    }

    pub fn in_progress(&self) -> Option<&UpgradeInProgress> {
        self.in_progress.as_ref()
    }
}

/// Reasons an upgrade cannot be started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CantStart {
    MaxLevel,
    MissingRequirement {
        line: UpgradeLine,
        required_level: u8,
    },
    Busy,
    NotEnoughCash {
        required: i32,
        actual: i32,
    },
    MissingItem {
        item_id: String,
        required: u32,
    },
}

impl fmt::Display for CantStart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MaxLevel => write!(f, "upgrade line already at maximum level"),
            Self::MissingRequirement { line, required_level } => {
                write!(f, "requires {} at level {}", line, required_level)
            }
            Self::Busy => write!(f, "another upgrade is already in progress"),
            Self::NotEnoughCash { required, actual } => {
                write!(f, "not enough cash: need ${}, have ${}", required, actual)
            }
            Self::MissingItem { item_id, required } => {
                write!(f, "missing item: need {}x {}", required, item_id)
            }
        }
    }
}

impl std::error::Error for CantStart {}

/// Cumulative camp stats and bonuses derived from state + config.
#[derive(Debug, Clone, PartialEq)]
pub struct CampEffects {
    pub safe_zone_half_bonus: f32,
    pub wall_hp_max: f32,
    pub gate_count: u32,
    pub guard_count: u32,
    pub guard_damage: f32,
    pub guard_range: f32,
    pub guard_fire_interval: f32,
    pub tower_count: u32,
    pub recipe_tier: u8,
    pub trader_slots_bonus: u32,
    pub has_doctor: bool,
    pub has_mechanic: bool,
    pub trader_stock_tier: u8,
    pub gunsmith_tier: u8,
}
