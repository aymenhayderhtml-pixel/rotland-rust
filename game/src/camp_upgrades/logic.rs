//! Core business logic for camp upgrades: starting, ticking, cancelling, and wall health.

use super::config::CampUpgradesConfig;
use super::types::{
    CampEffects, CampUpgradeState, CantStart, UpgradeCost, UpgradeInProgress, UpgradeLine,
};

/// Checks if an upgrade can be started.
pub fn can_start(
    state: &CampUpgradeState,
    cfg: &CampUpgradesConfig,
    line: UpgradeLine,
    wallet_cash: i32,
    has_items: impl Fn(&str, u32) -> bool,
) -> Result<UpgradeCost, CantStart> {
    if state.in_progress.is_some() {
        return Err(CantStart::Busy);
    }

    let current = state.level(line);
    let target = current + 1;
    let Some(info) = cfg.level_info(line, target) else {
        return Err(CantStart::MaxLevel);
    };

    if let Some(req) = info.requires
        && state.level(req.line) < req.level
    {
        return Err(CantStart::MissingRequirement {
            line: req.line,
            required_level: req.level,
        });
    }

    if wallet_cash < info.cost.cash {
        return Err(CantStart::NotEnoughCash {
            required: info.cost.cash,
            actual: wallet_cash,
        });
    }

    for mat in &info.cost.materials {
        if !has_items(&mat.item_id, mat.count) {
            return Err(CantStart::MissingItem {
                item_id: mat.item_id.clone(),
                required: mat.count,
            });
        }
    }

    Ok(info.cost.clone())
}

/// Starts an upgrade, setting it as in-progress. Caller handles deducting cash and materials.
pub fn start(
    state: &mut CampUpgradeState,
    cfg: &CampUpgradesConfig,
    line: UpgradeLine,
) -> UpgradeCost {
    assert!(state.in_progress.is_none(), "cannot start upgrade while busy");
    let current = state.level(line);
    let target = current + 1;
    let info = cfg.level_info(line, target).expect("cannot start: invalid or max level");
    state.in_progress = Some(UpgradeInProgress {
        line,
        target_level: target,
        hours_left: info.cost.build_hours,
    });
    info.cost.clone()
}

/// Ticks in-game construction hours. Returns finished (line, new_level) if completed.
pub fn tick(
    state: &mut CampUpgradeState,
    cfg: &CampUpgradesConfig,
    game_hours: f32,
) -> Option<(UpgradeLine, u8)> {
    let mut prog = state.in_progress.take()?;
    prog.hours_left -= game_hours;
    if prog.hours_left <= 0.0 {
        let line = prog.line;
        let new_level = prog.target_level;
        state.set_level(line, new_level);
        if line == UpgradeLine::Walls {
            let max_hp = cfg.effects(state).wall_hp_max;
            state.wall_hp = max_hp;
        }
        Some((line, new_level))
    } else {
        state.in_progress = Some(prog);
        None
    }
}

/// Cancels an in-progress upgrade and returns a 50% refund.
pub fn cancel(
    state: &mut CampUpgradeState,
    cfg: &CampUpgradesConfig,
) -> Option<UpgradeCost> {
    let prog = state.in_progress.take()?;
    let info = cfg.level_info(prog.line, prog.target_level)?;
    let refund = UpgradeCost {
        cash: (info.cost.cash as f32 * cfg.refund_ratio) as i32,
        materials: info.cost.materials.iter().map(|m| super::types::MaterialCost {
            item_id: m.item_id.clone(),
            count: (m.count as f32 * cfg.refund_ratio) as u32,
        }).collect(),
        build_hours: 0.0,
    };
    Some(refund)
}

/// Applies damage to the camp walls.
pub fn damage_walls(state: &mut CampUpgradeState, amount: f32) {
    if state.walls_level > 0 {
        state.wall_hp = (state.wall_hp - amount).max(0.0);
    }
}

/// Repairs the camp walls using supplied materials. Returns the amount of HP restored.
pub fn repair_walls(
    state: &mut CampUpgradeState,
    cfg: &CampUpgradesConfig,
    materials: &[(&str, u32)],
) -> f32 {
    if state.walls_level == 0 {
        return 0.0;
    }
    let max_hp = cfg.effects(state).wall_hp_max;
    let missing = (max_hp - state.wall_hp).max(0.0);
    if missing <= 0.0 {
        return 0.0;
    }

    let mut potential = 0.0;
    for &(item_id, count) in materials {
        if let Some(&hp_per_item) = cfg.wall_repair_hp.get(item_id) {
            potential += count as f32 * hp_per_item;
        }
    }

    let restored = potential.min(missing);
    state.wall_hp = (state.wall_hp + restored).min(max_hp);
    restored
}

/// Returns true if camp walls exist and their HP is depleted to 0.
pub fn walls_breached(state: &CampUpgradeState) -> bool {
    state.walls_level > 0 && state.wall_hp <= 0.0
}

impl CampUpgradeState {
    pub fn can_start(
        &self,
        cfg: &CampUpgradesConfig,
        line: UpgradeLine,
        wallet_cash: i32,
        has_items: impl Fn(&str, u32) -> bool,
    ) -> Result<UpgradeCost, CantStart> {
        can_start(self, cfg, line, wallet_cash, has_items)
    }

    pub fn start(&mut self, cfg: &CampUpgradesConfig, line: UpgradeLine) -> UpgradeCost {
        start(self, cfg, line)
    }

    pub fn tick(&mut self, cfg: &CampUpgradesConfig, game_hours: f32) -> Option<(UpgradeLine, u8)> {
        tick(self, cfg, game_hours)
    }

    pub fn cancel(&mut self, cfg: &CampUpgradesConfig) -> Option<UpgradeCost> {
        cancel(self, cfg)
    }

    pub fn damage_walls(&mut self, amount: f32) {
        damage_walls(self, amount);
    }

    pub fn repair_walls(&mut self, cfg: &CampUpgradesConfig, materials: &[(&str, u32)]) -> f32 {
        repair_walls(self, cfg, materials)
    }

    pub fn walls_breached(&self) -> bool {
        walls_breached(self)
    }

    pub fn effects(&self, cfg: &CampUpgradesConfig) -> CampEffects {
        cfg.effects(self)
    }
}
