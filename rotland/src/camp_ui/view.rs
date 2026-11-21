//! Read-only view structs providing data to camp screens.

use game::camp_upgrades::UpgradeLine;
use game::{
    Bank, BountyManager, CraftingSystem, EconomyConfigData, Inventory, ItemDef, ItemRegistry,
    RecipeRegistry, TraderStock, Wallet,
};
use glam::Vec3;

/// Default refund fraction when cancelling an in-progress camp upgrade.
pub const DEFAULT_UPGRADE_REFUND_RATIO: f32 = 0.5; // data: camp_upgrades.json refundRatio

/// A single material requirement row for a camp upgrade level.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UpgradeCostItem {
    pub name: String,
    pub have: u32,
    pub need: u32,
}

/// Details for the next buildable level of a camp upgrade line.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NextUpgradeView {
    pub name: String,
    pub description: String,
    pub cash_cost: i32,
    pub items: Vec<UpgradeCostItem>,
    pub build_hours: f32,
    /// `None` when the upgrade can start; `Some(reason)` explains why it cannot.
    pub cant_start_reason: Option<String>,
}

impl NextUpgradeView {
    pub fn can_start(&self) -> bool {
        self.cant_start_reason.is_none()
    }
}

/// Read-only snapshot for one column (upgrade line) on the Camp Board.
#[derive(Clone, Debug, PartialEq)]
pub struct UpgradeLineView {
    pub line: UpgradeLine,
    pub current_level: u8,
    pub max_level: u8,
    /// Short summary of what the player currently has on this line.
    pub current_summary: String,
    /// `None` when the line is already at `max_level`.
    pub next: Option<NextUpgradeView>,
}

impl UpgradeLineView {
    pub fn is_max_level(&self) -> bool {
        self.current_level >= self.max_level || self.next.is_none()
    }
}

/// Read-only snapshot for a camp upgrade currently under construction.
#[derive(Clone, Debug, PartialEq)]
pub struct UpgradeInProgressView {
    pub line: UpgradeLine,
    pub target_level: u8,
    /// Display name of the level being built; if empty, falls back to the line's next-level name.
    pub target_name: String,
    pub hours_left: f32,
    pub total_hours: f32,
}

/// Plain snapshot filled by the caller for `CampScreen::Upgrades` (Camp Board).
#[derive(Clone, Debug, PartialEq)]
pub struct CampUpgradesView {
    pub lines: [UpgradeLineView; 4],
    pub in_progress: Option<UpgradeInProgressView>,
    pub wall_hp: f32,
    pub wall_hp_max: f32,
    pub player_cash: i32,
    pub refund_ratio: f32,
    pub has_doctor: bool,
    pub has_mechanic: bool,
}

impl Default for CampUpgradesView {
    fn default() -> Self {
        Self {
            lines: [
                UpgradeLineView {
                    line: UpgradeLine::Walls,
                    current_level: 0,
                    max_level: 3, // data: camp_upgrades.json walls
                    current_summary: String::new(),
                    next: None,
                },
                UpgradeLineView {
                    line: UpgradeLine::Guards,
                    current_level: 0,
                    max_level: 3, // data: camp_upgrades.json guards
                    current_summary: String::new(),
                    next: None,
                },
                UpgradeLineView {
                    line: UpgradeLine::Workshop,
                    current_level: 0,
                    max_level: 3, // data: camp_upgrades.json workshop
                    current_summary: String::new(),
                    next: None,
                },
                UpgradeLineView {
                    line: UpgradeLine::Traders,
                    current_level: 0,
                    max_level: 4, // data: camp_upgrades.json traders
                    current_summary: String::new(),
                    next: None,
                },
            ],
            in_progress: None,
            wall_hp: 0.0,
            wall_hp_max: 0.0,
            player_cash: 0,
            refund_ratio: DEFAULT_UPGRADE_REFUND_RATIO,
            has_doctor: false,
            has_mechanic: false,
        }
    }
}

impl CampUpgradesView {
    pub fn line(&self, line: UpgradeLine) -> Option<&UpgradeLineView> {
        self.lines.iter().find(|entry| entry.line == line)
    }
}

/// Read-only snapshot of game state provided to the camp screens for rendering.
pub struct CampView<'a> {
    pub wallet: Option<&'a Wallet>,
    pub inventory: Option<&'a Inventory>,
    pub bank: Option<&'a Bank>,
    pub trader_stock: Option<&'a TraderStock>,
    pub crafting: Option<&'a CraftingSystem>,
    pub recipe_registry: Option<&'a RecipeRegistry>,
    pub bounty_manager: Option<&'a BountyManager>,
    pub item_registry: Option<&'a ItemRegistry>,
    pub economy_config: Option<&'a EconomyConfigData>,
    pub upgrades: Option<CampUpgradesView>,
    pub recipe_tier: u8,
    pub near_lit_campfire: bool,
    pub player_pos: Vec3,
    pub is_dead: bool,
}

impl<'a> Default for CampView<'a> {
    fn default() -> Self {
        Self {
            wallet: None,
            inventory: None,
            bank: None,
            trader_stock: None,
            crafting: None,
            recipe_registry: None,
            bounty_manager: None,
            item_registry: None,
            economy_config: None,
            upgrades: None,
            recipe_tier: 0,
            near_lit_campfire: false,
            player_pos: Vec3::ZERO,
            is_dead: false,
        }
    }
}

impl<'a> CampView<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_dollars(&self) -> i32 {
        if let Some(wallet) = self.wallet {
            wallet.dollars()
        } else if let Some(upgrades) = &self.upgrades {
            upgrades.player_cash
        } else {
            0
        }
    }

    pub fn bank_dollars(&self) -> i32 {
        self.bank.map(|b| b.dollars()).unwrap_or(0)
    }

    pub fn buy_rate(&self) -> f32 {
        self.economy_config.map(|c| c.buy_rate).unwrap_or(1.0)
    }

    pub fn sell_rate(&self) -> f32 {
        self.economy_config.map(|c| c.sell_rate).unwrap_or(0.5)
    }

    pub fn item_def<'s>(&'s self, id: &str) -> Option<&'s ItemDef> {
        self.item_registry.and_then(|r| r.get(id))
    }

    pub fn item_name<'s>(&'s self, id: &'s str) -> &'s str {
        if let Some(def) = self.item_def(id) {
            &def.name
        } else {
            id
        }
    }
}
