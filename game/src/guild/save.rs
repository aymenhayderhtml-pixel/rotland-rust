//! Single save state struct GuildState for persisting guild progress.

use serde::{Deserialize, Serialize};

use super::recruits::GuildRecruit;
use super::shop::GuildShopItem;
use super::tasks::GuildTask;

/// Complete guild state preserved across save files.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GuildState {
    #[serde(default)]
    pub reputation: i32,
    #[serde(default)]
    pub last_refresh_day: i32,
    #[serde(default)]
    pub offered_tasks: Vec<GuildTask>,
    #[serde(default)]
    pub active_tasks: Vec<GuildTask>,
    #[serde(default)]
    pub completed_task_ids: Vec<String>,
    #[serde(default)]
    pub hired_recruits: Vec<GuildRecruit>,
    #[serde(default)]
    pub available_recruits: Vec<GuildRecruit>,
    #[serde(default)]
    pub shop_items: Vec<GuildShopItem>,
}
