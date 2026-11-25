use serde::{Deserialize, Serialize};

pub const HAND_STATION: &str = "hand";
pub const CAMPFIRE_STATION: &str = "campfire";
/// Metres from a lit campfire that a campfire craft allows. Walking further cancels it.
pub const CAMPFIRE_RANGE: f32 = 4.0;

/// One ingredient or result line of a recipe: an item id and how many.
/// Port of `scripts/Crafting/RecipeItem.cs`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeItem {
    pub id: String,
    #[serde(default = "default_count")]
    pub count: u32,
}

fn default_count() -> u32 {
    1
}

impl RecipeItem {
    pub fn new(id: impl Into<String>, count: u32) -> Self {
        Self {
            id: id.into(),
            count,
        }
    }
}

/// XP a recipe gives on finish: skill id (e.g. "cooking") and amount.
/// Port of `scripts/Crafting/RecipeXp.cs`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeXp {
    pub skill: String,
    pub amount: f32,
}

impl RecipeXp {
    pub fn new(skill: impl Into<String>, amount: f32) -> Self {
        Self {
            skill: skill.into(),
            amount,
        }
    }
}

fn default_station() -> String {
    HAND_STATION.to_string()
}

/// One recipe definition loaded from data/recipes.json.
/// Port of `scripts/Crafting/RecipeDef.cs`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RecipeDef {
    pub id: String,
    pub name: String,
    #[serde(default = "default_station")]
    pub station: String,
    pub inputs: Vec<RecipeItem>,
    pub outputs: Vec<RecipeItem>,
    pub time: f32,
    pub xp: Option<RecipeXp>,
}

impl Default for RecipeDef {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            station: default_station(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            time: 0.0,
            xp: None,
        }
    }
}
