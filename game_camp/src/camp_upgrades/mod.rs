//! Camp upgrades module (walls, guards, workshop, traders).

mod config;
mod logic;
mod types;

pub use config::{
    CampUpgradesConfig, CommonLevelDef, GuardLevelDef, TraderLevelDef, WallLevelDef,
    WorkshopLevelDef,
};
pub use logic::{cancel, can_start, damage_walls, repair_walls, start, tick, walls_breached};
pub use types::{
    CampEffects, CampUpgradeState, CantStart, MaterialCost, UpgradeCost, UpgradeInProgress,
    UpgradeLine, UpgradeRequirement,
};
