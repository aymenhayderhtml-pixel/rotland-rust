use std::path::Path;
use serde::{Deserialize, Serialize};

pub const KILL_ZOMBIES: &str = "kill_zombies";
pub const KILL_AT_NIGHT: &str = "kill_at_night";
pub const CLEAR_POI: &str = "clear_poi";
pub const LOOT_CRATES: &str = "loot_crates";
pub const DELIVER_ITEMS: &str = "deliver_items";
pub const HUNT_ANIMALS: &str = "hunt_animals";

fn default_reset_hour() -> i32 {
    6
}

fn default_daily_count() -> i32 {
    3
}

fn default_weekly_days() -> i32 {
    7
}

fn default_max_active() -> i32 {
    2
}

fn default_count_range() -> [i32; 2] {
    [1, 1]
}

/// One bounty template loaded from data/bounties.json.
/// Ranges [min, max] are rolled per bounty.
// Godot: BountyTemplateData.cs
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BountyTemplateData {
    pub id: String,
    /// kill_zombies, kill_at_night, clear_poi, loot_crates, deliver_items, or hunt_animals.
    #[serde(rename = "type")]
    pub bounty_type: String,
    /// Shown on the board. {n} is the count, {what} the building or item name.
    #[serde(default)]
    pub text: String,
    #[serde(default = "default_count_range")]
    pub count: [i32; 2],
    #[serde(default)]
    pub dollars: [i32; 2],
    #[serde(default)]
    pub xp: [i32; 2],
    /// Skill that gets the XP (combat, hunting, agility, farming, cooking).
    #[serde(default)]
    pub skill: String,
    /// kill_zombies only: count just this zombie id (e.g. runner). None means any zombie.
    #[serde(default)]
    pub zombie: Option<String>,
    /// deliver_items only: item id to hand in.
    #[serde(default)]
    pub item: Option<String>,
    /// hunt_animals only: wild animal species id. None means any animal.
    #[serde(default)]
    pub animal: Option<String>,
    /// clear_poi only: building ids; one is picked per roll.
    #[serde(default)]
    pub buildings: Vec<String>,
}

impl Default for BountyTemplateData {
    fn default() -> Self {
        Self {
            id: String::new(),
            bounty_type: String::new(),
            text: String::new(),
            count: default_count_range(),
            dollars: [0, 0],
            xp: [0, 0],
            skill: String::new(),
            zombie: None,
            item: None,
            animal: None,
            buildings: Vec::new(),
        }
    }
}

/// Shape of data/bounties.json.
// Godot: BountyConfigData.cs
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BountyConfigData {
    #[serde(default = "default_reset_hour")]
    pub reset_hour: i32,
    #[serde(default = "default_daily_count")]
    pub daily_count: i32,
    #[serde(default = "default_weekly_days")]
    pub weekly_days: i32,
    #[serde(default = "default_max_active")]
    pub max_active: i32,
    #[serde(default)]
    pub daily: Vec<BountyTemplateData>,
    #[serde(default)]
    pub weekly: Vec<BountyTemplateData>,
}

impl Default for BountyConfigData {
    fn default() -> Self {
        Self {
            reset_hour: default_reset_hour(),
            daily_count: default_daily_count(),
            weekly_days: default_weekly_days(),
            max_active: default_max_active(),
            daily: Vec::new(),
            weekly: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum BountyLoadError {
    Io(std::io::Error),
    Parse(serde_json::Error),
    Validation(String),
}

impl std::fmt::Display for BountyLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "could not read bounties file: {e}"),
            Self::Parse(e) => write!(f, "could not parse bounties file: {e}"),
            Self::Validation(e) => write!(f, "invalid bounty configuration: {e}"),
        }
    }
}

impl std::error::Error for BountyLoadError {}

/// Loader and validator for data/bounties.json.
// Godot: BountyConfig.cs
pub struct BountyConfig;

impl BountyConfig {
    /// Loads and validates configuration from the repo's data/bounties.json.
    // Godot: BountyConfig.cs Get
    pub fn from_repo() -> Result<BountyConfigData, BountyLoadError> {
        let path = paths::repo_path("data/bounties.json");
        Self::load(&path)
    }

    /// Reads and validates a bounties.json file at `path`.
    pub fn load(path: &Path) -> Result<BountyConfigData, BountyLoadError> {
        let text = std::fs::read_to_string(path).map_err(BountyLoadError::Io)?;
        Self::from_json(&text)
    }

    /// Parses bounties.json text and validates the configuration.
    pub fn from_json(text: &str) -> Result<BountyConfigData, BountyLoadError> {
        let data: BountyConfigData =
            serde_json::from_str(text).map_err(BountyLoadError::Parse)?;
        Self::validate(&data)?;
        Ok(data)
    }

    /// Validates bounty ranges, counts, and template structures.
    // Godot: BountyConfig.cs Validate
    pub fn validate(d: &BountyConfigData) -> Result<(), BountyLoadError> {
        if d.reset_hour < 0 || d.reset_hour > 23 || d.daily_count < 1 || d.weekly_days < 1 || d.max_active < 1 {
            return Err(BountyLoadError::Validation(
                "data/bounties.json has a value out of range".to_string(),
            ));
        }
        if d.daily.is_empty() || d.weekly.is_empty() {
            return Err(BountyLoadError::Validation(
                "data/bounties.json needs at least one daily and one weekly bounty".to_string(),
            ));
        }
        for template in d.daily.iter().chain(d.weekly.iter()) {
            Self::check_template(template)?;
        }
        Ok(())
    }

    // Godot: BountyConfig.cs Check
    fn check_template(t: &BountyTemplateData) -> Result<(), BountyLoadError> {
        let where_msg = format!("data/bounties.json bounty '{}'", t.id);
        let valid_type = matches!(
            t.bounty_type.as_str(),
            KILL_ZOMBIES | KILL_AT_NIGHT | CLEAR_POI | LOOT_CRATES | DELIVER_ITEMS | HUNT_ANIMALS
        );
        if !valid_type {
            return Err(BountyLoadError::Validation(format!(
                "{where_msg} has unknown type '{}'",
                t.bounty_type
            )));
        }
        if t.count[0] < 1
            || t.count[0] > t.count[1]
            || t.dollars[0] < 0
            || t.dollars[0] > t.dollars[1]
            || t.xp[0] < 0
            || t.xp[0] > t.xp[1]
        {
            return Err(BountyLoadError::Validation(format!(
                "{where_msg} has a bad [min, max] range"
            )));
        }
        if t.bounty_type == CLEAR_POI && t.buildings.is_empty() {
            return Err(BountyLoadError::Validation(format!(
                "{where_msg} needs at least one building"
            )));
        }
        if t.bounty_type == DELIVER_ITEMS && t.item.is_none() {
            return Err(BountyLoadError::Validation(format!(
                "{where_msg} needs an item"
            )));
        }
        Ok(())
    }
}
