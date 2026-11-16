//! Read-only view struct providing data to camp screens.

use game::{
    Bank, BountyManager, CraftingSystem, EconomyConfigData, Inventory, ItemDef, ItemRegistry,
    RecipeRegistry, TraderStock, Wallet,
};
use glam::Vec3;

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
        self.wallet.map(|w| w.dollars()).unwrap_or(0)
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
