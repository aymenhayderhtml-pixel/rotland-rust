use crate::economy::trader::TraderStock;
use crate::economy::wallet::Wallet;
use crate::items::{Inventory, ItemDef, ItemRegistry};

/// Why one unit cannot be bought right now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuyBlocker {
    SoldOut,
    NotEnoughDollars,
    TooHeavy,
    BagIsFull,
}

impl BuyBlocker {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SoldOut => "Sold out",
            Self::NotEnoughDollars => "Not enough dollars",
            Self::TooHeavy => "Too heavy",
            Self::BagIsFull => "Bag is full",
        }
    }
}

impl std::fmt::Display for BuyBlocker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Hook for returning rounds when selling weapons.
pub trait WeaponUnloader {
    fn return_rounds(&mut self, bag: &mut Inventory, def: &ItemDef);
}

/// What one unit costs the player. At least $1.
/// // Godot: TradeRules.cs BuyPrice
pub fn buy_price(def: &ItemDef, buy_rate: f32) -> i32 {
    ((def.value as f32 * buy_rate).floor() as i32).max(1)
}

/// What the trader pays for one unit. At least $1.
/// // Godot: TradeRules.cs SellPrice
pub fn sell_price(def: &ItemDef, sell_rate: f32) -> i32 {
    ((def.value as f32 * sell_rate).floor() as i32).max(1)
}

/// Why one unit cannot be bought right now, or None when it can.
/// // Godot: TradeRules.cs BuyBlocker
pub fn buy_blocker(
    wallet: &Wallet,
    bag: &Inventory,
    stock: &TraderStock,
    id: &str,
    buy_rate: f32,
    registry: &ItemRegistry,
) -> Option<BuyBlocker> {
    if stock.qty(id) == 0 {
        return Some(BuyBlocker::SoldOut);
    }
    let Some(def) = registry.get(id) else {
        return Some(BuyBlocker::SoldOut);
    };
    if wallet.dollars() < buy_price(def, buy_rate) {
        return Some(BuyBlocker::NotEnoughDollars);
    }
    if bag.can_add_with(id, 1, registry) {
        return None;
    }
    if bag.total_weight_with(registry) + def.weight > bag.max_weight() {
        Some(BuyBlocker::TooHeavy)
    } else {
        Some(BuyBlocker::BagIsFull)
    }
}

/// Buys up to amount units, as many as the stock, the wallet and the bag allow.
/// Returns how many were bought.
/// // Godot: TradeRules.cs Buy
pub fn buy(
    wallet: &mut Wallet,
    bag: &mut Inventory,
    stock: &mut TraderStock,
    id: &str,
    amount: u32,
    buy_rate: f32,
    registry: &ItemRegistry,
) -> u32 {
    let Some(def) = registry.get(id) else {
        return 0;
    };
    let price = buy_price(def, buy_rate);
    if price <= 0 {
        return 0;
    }

    let mut n = amount.min(stock.qty(id));
    let affordable = (wallet.dollars().max(0) as u32) / (price as u32);
    n = n.min(affordable);

    while n > 0 && !bag.can_add_with(id, n, registry) {
        n -= 1;
    }

    if n == 0 {
        return 0;
    }

    let total_cost = (price as u32 * n) as i32;
    wallet.try_spend(total_cost);
    bag.add_with_registry(id, n, registry);
    stock.take(id, n);
    n
}

/// Sells up to amount units from the bag. Sold items do not join trader stock.
/// Returns how many were sold.
/// // Godot: TradeRules.cs Sell
pub fn sell(
    wallet: &mut Wallet,
    bag: &mut Inventory,
    id: &str,
    amount: u32,
    sell_rate: f32,
    registry: &ItemRegistry,
) -> u32 {
    let Some(def) = registry.get(id) else {
        return 0;
    };
    let n = amount.min(bag.count(id));
    if n == 0 {
        return 0;
    }

    let price = sell_price(def, sell_rate);
    for _ in 0..n {
        bag.remove(id, 1);
        wallet.add(price);
    }
    n
}

/// Sells with weapon unloader hook to return loaded rounds when selling a gun.
/// // Godot: TradeRules.cs Sell
pub fn sell_with_unloader(
    wallet: &mut Wallet,
    bag: &mut Inventory,
    id: &str,
    amount: u32,
    sell_rate: f32,
    registry: &ItemRegistry,
    unloader: &mut impl WeaponUnloader,
) -> u32 {
    let Some(def) = registry.get(id) else {
        return 0;
    };
    let n = amount.min(bag.count(id));
    if n == 0 {
        return 0;
    }

    let price = sell_price(def, sell_rate);
    for _ in 0..n {
        bag.remove(id, 1);
        if def.is_weapon() {
            unloader.return_rounds(bag, def);
        }
        wallet.add(price);
    }
    n
}
