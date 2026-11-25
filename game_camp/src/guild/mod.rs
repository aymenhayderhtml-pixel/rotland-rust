//! Guild core module: task board, recruit teammates, reputation ranks, and guild shop.

mod board;
pub mod companion;
mod config;
mod manager;
mod rank;
mod recruits;
mod roller;
mod save;
mod shop;
pub mod task_hooks;
mod tasks;

pub use board::{GuildAcceptError, GuildBoard, GuildClaimError};
pub use config::{
    GuildConfig, GuildLoadError, GuildRankDef, ItemRewardDef, RecruitDef, ShopItemDef,
    TaskTemplateDef,
};
pub use manager::{GuildManager, MorningReport};
pub use rank::{
    current_rank, next_rank, rank_progress, RANK_ELITE, RANK_LEGEND, RANK_MASTER, RANK_ROOKIE,
    RANK_VETERAN,
};
pub use recruits::{
    GuildRecruit, GuildRecruits, RecruitDismissError, RecruitHireError, RecruitOrder,
    RecruitOrderError, RecruitRole, RecruitState, WageReport,
};
pub use roller::GuildTaskRoller;
pub use save::GuildState;
pub use shop::{GuildShop, GuildShopItem, ShopBuyError};
pub use task_hooks::{
    GuildTaskEvent, GuildTaskHookConfig, GuildTaskHooks, HandIn, NamedPlace, TaskRow,
    tracker_rows,
};
pub use tasks::{GuildReward, GuildTask, GuildTaskKind};
