//! Configuration data structures and loader for camp upgrades.

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

use super::types::{
    CampEffects, CampUpgradeState, UpgradeCost, UpgradeLine, UpgradeRequirement,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WallLevelDef {
    pub level: u8,
    pub name: String,
    pub cost: UpgradeCost,
    #[serde(default)]
    pub requires: Option<UpgradeRequirement>,
    pub wall_hp: f32,
    pub gate_count: u32,
    pub safe_zone_half_bonus: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardLevelDef {
    pub level: u8,
    pub name: String,
    pub cost: UpgradeCost,
    #[serde(default)]
    pub requires: Option<UpgradeRequirement>,
    pub guard_count: u32,
    pub guard_damage: f32,
    pub guard_range: f32,
    pub guard_fire_interval: f32,
    pub tower_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkshopLevelDef {
    pub level: u8,
    pub name: String,
    pub cost: UpgradeCost,
    #[serde(default)]
    pub requires: Option<UpgradeRequirement>,
    pub recipe_tier: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraderLevelDef {
    pub level: u8,
    pub name: String,
    pub cost: UpgradeCost,
    #[serde(default)]
    pub requires: Option<UpgradeRequirement>,
    pub trader_slots_bonus: u32,
    pub has_doctor: bool,
    pub has_mechanic: bool,
    pub trader_stock_tier: u8,
}

/// Root configuration struct matching data/camp_upgrades.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampUpgradesConfig {
    #[serde(default = "default_refund_ratio")]
    pub refund_ratio: f32,
    #[serde(default)]
    pub wall_repair_hp: HashMap<String, f32>,
    pub walls: Vec<WallLevelDef>,
    pub guards: Vec<GuardLevelDef>,
    pub workshop: Vec<WorkshopLevelDef>,
    pub traders: Vec<TraderLevelDef>,
}

fn default_refund_ratio() -> f32 {
    0.5
}

pub struct CommonLevelDef<'a> {
    pub cost: &'a UpgradeCost,
    pub requires: Option<&'a UpgradeRequirement>,
}

impl CampUpgradesConfig {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let cfg = Self::from_json(&content)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        cfg.validate()
            .map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(cfg)
    }

    pub fn load_default() -> Result<Self, String> {
        let path = paths::repo_path("data/camp_upgrades.json");
        Self::load_from_file(&path)
    }

    pub fn max_level(&self, line: UpgradeLine) -> u8 {
        match line {
            UpgradeLine::Walls => self.walls.len().saturating_sub(1) as u8,
            UpgradeLine::Guards => self.guards.len().saturating_sub(1) as u8,
            UpgradeLine::Workshop => self.workshop.len().saturating_sub(1) as u8,
            UpgradeLine::Traders => self.traders.len().saturating_sub(1) as u8,
        }
    }

    pub fn level_info(&self, line: UpgradeLine, level: u8) -> Option<CommonLevelDef<'_>> {
        match line {
            UpgradeLine::Walls => self.walls.get(level as usize).map(|d| CommonLevelDef {
                cost: &d.cost,
                requires: d.requires.as_ref(),
            }),
            UpgradeLine::Guards => self.guards.get(level as usize).map(|d| CommonLevelDef {
                cost: &d.cost,
                requires: d.requires.as_ref(),
            }),
            UpgradeLine::Workshop => self.workshop.get(level as usize).map(|d| CommonLevelDef {
                cost: &d.cost,
                requires: d.requires.as_ref(),
            }),
            UpgradeLine::Traders => self.traders.get(level as usize).map(|d| CommonLevelDef {
                cost: &d.cost,
                requires: d.requires.as_ref(),
            }),
        }
    }

    pub fn level_cost(&self, line: UpgradeLine, level: u8) -> Option<&UpgradeCost> {
        self.level_info(line, level).map(|info| info.cost)
    }

    pub fn level_requirement(&self, line: UpgradeLine, level: u8) -> Option<&UpgradeRequirement> {
        self.level_info(line, level).and_then(|info| info.requires)
    }

    pub fn effects(&self, state: &CampUpgradeState) -> CampEffects {
        let wall_idx = (state.walls_level as usize).min(self.walls.len().saturating_sub(1));
        let guard_idx = (state.guards_level as usize).min(self.guards.len().saturating_sub(1));
        let ws_idx = (state.workshop_level as usize).min(self.workshop.len().saturating_sub(1));
        let tr_idx = (state.traders_level as usize).min(self.traders.len().saturating_sub(1));

        let walls = &self.walls[wall_idx];
        let guards = &self.guards[guard_idx];
        let ws = &self.workshop[ws_idx];
        let traders = &self.traders[tr_idx];

        CampEffects {
            safe_zone_half_bonus: walls.safe_zone_half_bonus,
            wall_hp_max: walls.wall_hp,
            gate_count: walls.gate_count,
            guard_count: guards.guard_count,
            guard_damage: guards.guard_damage,
            guard_range: guards.guard_range,
            guard_fire_interval: guards.guard_fire_interval,
            tower_count: guards.tower_count,
            recipe_tier: ws.recipe_tier,
            trader_slots_bonus: traders.trader_slots_bonus,
            has_doctor: traders.has_doctor,
            has_mechanic: traders.has_mechanic,
            trader_stock_tier: traders.trader_stock_tier,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.walls.len() < 4 {
            return Err("walls must have at least 4 levels (0..=3)".to_string());
        }
        if self.guards.len() < 4 {
            return Err("guards must have at least 4 levels (0..=3)".to_string());
        }
        if self.workshop.len() < 4 {
            return Err("workshop must have at least 4 levels (0..=3)".to_string());
        }
        // data: camp_upgrades.json traders
        if self.traders.len() < 5 {
            return Err("traders must have at least 5 levels (0..=4)".to_string());
        }

        for (i, def) in self.walls.iter().enumerate() {
            if def.level as usize != i {
                return Err(format!("walls level index mismatch: expected {i}, got {}", def.level));
            }
        }
        for (i, def) in self.guards.iter().enumerate() {
            if def.level as usize != i {
                return Err(format!("guards level index mismatch: expected {i}, got {}", def.level));
            }
        }
        for (i, def) in self.workshop.iter().enumerate() {
            if def.level as usize != i {
                return Err(format!("workshop level index mismatch: expected {i}, got {}", def.level));
            }
        }
        for (i, def) in self.traders.iter().enumerate() {
            if def.level as usize != i {
                return Err(format!("traders level index mismatch: expected {i}, got {}", def.level));
            }
        }

        for line in UpgradeLine::ALL {
            let max_lvl = self.max_level(line);
            for lvl in 0..=max_lvl {
                if let Some(req) = self.level_requirement(line, lvl)
                    && req.level > self.max_level(req.line)
                {
                    let req_max = self.max_level(req.line);
                    return Err(format!(
                        "{line:?} level {lvl} requires {:?} level {} which exceeds max {req_max}",
                        req.line, req.level
                    ));
                }
            }
        }

        if self.refund_ratio < 0.0 || self.refund_ratio > 1.0 {
            return Err("refundRatio must be between 0.0 and 1.0".to_string());
        }

        Ok(())
    }
}
