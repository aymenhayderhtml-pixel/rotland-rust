//! # game_camp
//!
//! Camp, settlement, economy, crafting, farming, guild, and map systems.

pub mod bounties;
pub mod building;
pub mod camp;
pub mod camp_guards;
pub mod camp_upgrades;
pub mod campfire;
pub mod crafting;
pub mod economy;
pub mod farming;
pub mod guild;
pub mod gunsmith;
pub mod map;
pub mod rescue;
pub mod rescue_place;
pub mod rescue_floor;
pub mod team_anim;

pub use bounties::*;
pub use building::*;
pub use camp::*;
pub use camp_guards::*;
pub use camp_upgrades::*;
pub use campfire::*;
pub use crafting::*;
pub use economy::*;
pub use farming::*;
pub use guild::*;
pub use gunsmith::{
    buy_check, fit, fit_at_shop, fit_cost, fit_slot, fit_slot_at_shop, remove_mod, remove_mod_slot,
    repair, repair_all, repair_all_cost, repair_cost, repair_slot, sell_slot, sell_value,
    BuyResult, CampStockEntry, GunsmithConfig, GunsmithConfigData, GunsmithError,
    GunsmithLoadError, GunsmithPriceEntry, GunsmithStock, SellResult, ShopKind, ShopStock,
    TownShopConfig, GUNSMITH_SAVE_ID,
};
pub use map::*;
pub use team_anim::{
    plan_npc_anim, NpcAnimInput, NpcAnimPlan, NpcAnimRole, NpcAnimTrack, RoleAnimConfig,
    TeamAnimConfig, TeamLodConfig, TeamRolesConfig,
};
