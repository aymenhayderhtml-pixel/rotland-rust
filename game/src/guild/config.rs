//! Guild configuration and data definitions loaded from data/guild.json.

use std::path::Path;
use serde::{Deserialize, Serialize};

fn default_reset_hour() -> f32 {
    // data: guild.json reset_hour
    6.0
}

fn default_max_offered() -> usize {
    // data: guild.json max_offered
    5
}

fn default_max_active() -> usize {
    // data: guild.json max_active
    3
}

/// Definition of a guild reputation rank.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuildRankDef {
    pub id: String,
    pub name: String,
    pub min_rep: i32,
    pub max_recruits: usize,
    pub shop_tier: u32,
}

/// Optional bonus item reward definition.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemRewardDef {
    pub item_id: String,
    pub count: u32,
}

/// Template for generating tasks on the board.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskTemplateDef {
    pub id: String,
    pub kind: String,
    pub title: String,
    #[serde(default)]
    pub zombie_type: Option<String>,
    #[serde(default)]
    pub in_city: Option<bool>,
    #[serde(default)]
    pub item_id: Option<String>,
    #[serde(default)]
    pub place_name: Option<String>,
    #[serde(default)]
    pub poi_id: Option<String>,
    #[serde(default)]
    pub target_range: [u32; 2],
    #[serde(default)]
    pub dollars_range: [i32; 2],
    #[serde(default)]
    pub rep_range: [i32; 2],
    #[serde(default)]
    pub item_reward: Option<ItemRewardDef>,
}

/// Recruit candidate definition in the guild roster.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecruitDef {
    pub id: String,
    pub name: String,
    pub role: String,
    pub level: u32,
    pub daily_wage: u32,
    pub health: u32,
    pub hire_cost: u32,
}

/// Guild shop item definition with tier, price, and daily stock.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShopItemDef {
    pub item_id: String,
    pub tier: u32,
    pub price: u32,
    pub stock: u32,
}

/// Errors when loading guild configuration.
#[derive(Debug)]
pub enum GuildLoadError {
    Io(std::io::Error),
    Json(serde_json::Error),
    NotFound(String),
}

impl std::fmt::Display for GuildLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "guild IO error: {err}"),
            Self::Json(err) => write!(f, "guild JSON parse error: {err}"),
            Self::NotFound(path) => write!(f, "guild config file not found: {path}"),
        }
    }
}

impl std::error::Error for GuildLoadError {}

impl From<std::io::Error> for GuildLoadError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for GuildLoadError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

/// Root guild configuration loaded from data/guild.json.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GuildConfig {
    #[serde(default = "default_reset_hour")]
    pub reset_hour: f32,
    #[serde(default = "default_max_offered")]
    pub max_offered: usize,
    #[serde(default = "default_max_active")]
    pub max_active: usize,
    #[serde(default)]
    pub ranks: Vec<GuildRankDef>,
    #[serde(default)]
    pub tasks: Vec<TaskTemplateDef>,
    #[serde(default)]
    pub recruits: Vec<RecruitDef>,
    #[serde(default)]
    pub shop: Vec<ShopItemDef>,
}

impl Default for GuildConfig {
    fn default() -> Self {
        Self {
            reset_hour: default_reset_hour(),
            max_offered: default_max_offered(),
            max_active: default_max_active(),
            ranks: default_ranks(),
            tasks: Vec::new(),
            recruits: Vec::new(),
            shop: Vec::new(),
        }
    }
}

fn default_ranks() -> Vec<GuildRankDef> {
    vec![
        GuildRankDef {
            id: "rookie".to_string(),
            name: "Rookie".to_string(),
            min_rep: 0,
            max_recruits: 1,
            shop_tier: 1,
        },
        GuildRankDef {
            id: "veteran".to_string(),
            name: "Veteran".to_string(),
            min_rep: 100,
            max_recruits: 2,
            shop_tier: 2,
        },
        GuildRankDef {
            id: "elite".to_string(),
            name: "Elite".to_string(),
            // data: guild.json min_rep
            min_rep: 250,
            max_recruits: 3,
            shop_tier: 3,
        },
        GuildRankDef {
            id: "master".to_string(),
            name: "Master".to_string(),
            // data: guild.json min_rep
            min_rep: 500,
            max_recruits: 4,
            shop_tier: 4,
        },
        GuildRankDef {
            id: "legend".to_string(),
            name: "Legend".to_string(),
            min_rep: 1000,
            // data: guild.json max_recruits
            max_recruits: 5,
            // data: guild.json shop_tier
            shop_tier: 5,
        },
    ]
}

impl GuildConfig {
    /// Loads guild configuration from data/guild.json using repository path discovery.
    pub fn from_repo() -> Result<Self, GuildLoadError> {
        let path = paths::repo_path("data/guild.json");
        Self::from_file(&path)
    }

    /// Loads guild configuration from an explicit file path.
    pub fn from_file(path: &Path) -> Result<Self, GuildLoadError> {
        if !path.exists() {
            return Err(GuildLoadError::NotFound(path.display().to_string()));
        }
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    /// Parses guild configuration from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, GuildLoadError> {
        let cfg: Self = serde_json::from_str(json)?;
        Ok(cfg)
    }
}
