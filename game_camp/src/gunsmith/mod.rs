//! Gunsmith weapon shop: config, per-shop stock, and scrap-based buy/sell/repair/upgrade rules.

mod actions;
mod config;
mod stock;

pub use actions::{
    buy, buy_check, fit, fit_at_shop, fit_cost, fit_slot, fit_slot_at_shop, remove_mod,
    remove_mod_slot, repair, repair_all, repair_all_cost, repair_cost, repair_slot, sell,
    sell_slot, sell_value, BuyResult, GunsmithError, SellResult,
};
pub use config::{
    CampStockEntry, GunsmithConfig, GunsmithConfigData, GunsmithLoadError, GunsmithPriceEntry,
    TownShopConfig,
};
pub use stock::{GunsmithStock, ShopKind, ShopStock, GUNSMITH_SAVE_ID};
