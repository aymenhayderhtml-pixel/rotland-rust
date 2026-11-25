//! Guild shop inventory and purchasing logic tiered by reputation rank.

use serde::{Deserialize, Serialize};

/// An item offered for sale in the guild shop.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuildShopItem {
    pub item_id: String,
    pub tier: u32,
    pub price: u32,
    pub stock: u32,
    pub max_stock: u32,
}

impl GuildShopItem {
    /// Creates a new guild shop item with full stock.
    pub fn new(item_id: impl Into<String>, tier: u32, price: u32, max_stock: u32) -> Self {
        Self {
            item_id: item_id.into(),
            tier,
            price,
            stock: max_stock,
            max_stock,
        }
    }
}

/// Errors that can occur when purchasing from the guild shop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShopBuyError {
    ItemNotFound,
    TierLocked { required_tier: u32, player_tier: u32 },
    OutOfStock { requested: u32, available: u32 },
    InsufficientFunds { needed: u32, available: i32 },
}

impl std::fmt::Display for ShopBuyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ItemNotFound => write!(f, "Item not found in guild shop."),
            Self::TierLocked {
                required_tier,
                player_tier,
            } => write!(
                f,
                "Item requires guild shop tier {required_tier} (current: {player_tier})."
            ),
            Self::OutOfStock {
                requested,
                available,
            } => write!(
                f,
                "Insufficient stock (requested: {requested}, available: {available})."
            ),
            Self::InsufficientFunds { needed, available } => {
                write!(f, "Cannot afford purchase ({needed}$ needed, {available}$ available).")
            }
        }
    }
}

impl std::error::Error for ShopBuyError {}

/// The guild shop maintaining stock and handling purchase transactions.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct GuildShop {
    pub items: Vec<GuildShopItem>,
}

impl GuildShop {
    /// Creates a new guild shop with the given inventory.
    pub fn new(items: Vec<GuildShopItem>) -> Self {
        Self { items }
    }

    /// All items in the shop regardless of player rank.
    pub fn items(&self) -> &[GuildShopItem] {
        &self.items
    }

    /// Items unlocked and visible to a player with the given shop tier.
    pub fn available_items(&self, player_tier: u32) -> Vec<&GuildShopItem> {
        self.items.iter().filter(|i| i.tier <= player_tier).collect()
    }

    /// Purchases an item from the shop, deducting funds and stock.
    pub fn buy(
        &mut self,
        item_id: &str,
        count: u32,
        player_tier: u32,
        funds: &mut i32,
    ) -> Result<u32, ShopBuyError> {
        if count == 0 {
            return Ok(0);
        }
        let item = self
            .items
            .iter_mut()
            .find(|i| i.item_id == item_id)
            .ok_or(ShopBuyError::ItemNotFound)?;

        if item.tier > player_tier {
            return Err(ShopBuyError::TierLocked {
                required_tier: item.tier,
                player_tier,
            });
        }
        if item.stock < count {
            return Err(ShopBuyError::OutOfStock {
                requested: count,
                available: item.stock,
            });
        }
        let total_cost = item.price.saturating_mul(count);
        if *funds < total_cost as i32 {
            return Err(ShopBuyError::InsufficientFunds {
                needed: total_cost,
                available: *funds,
            });
        }

        *funds -= total_cost as i32;
        item.stock -= count;
        Ok(total_cost)
    }

    /// Restocks all items to their maximum daily capacity.
    pub fn restock(&mut self) {
        for item in &mut self.items {
            item.stock = item.max_stock;
        }
    }
}
