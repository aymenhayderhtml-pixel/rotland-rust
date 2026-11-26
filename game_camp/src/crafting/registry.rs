use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use crate::crafting::recipe::{RecipeDef, CAMPFIRE_STATION, HAND_STATION};
use game_core::items::ItemRegistry;

#[derive(Debug)]
pub enum RecipeLoadError {
    Io(std::io::Error),
    Parse(serde_json::Error),
    Validation(String),
}

impl std::fmt::Display for RecipeLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "could not read recipes file: {e}"),
            Self::Parse(e) => write!(f, "could not parse recipes file: {e}"),
            Self::Validation(e) => write!(f, "invalid recipe definition: {e}"),
        }
    }
}

impl std::error::Error for RecipeLoadError {}

/// Registry of all recipes loaded from data/recipes.json.
/// Port of `scripts/Crafting/RecipeRegistry.cs`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RecipeRegistry {
    recipes: Vec<RecipeDef>,
    by_id: BTreeMap<String, usize>,
}

impl RecipeRegistry {
    /// Loads recipes from the default repository path `data/recipes.json`.
    /// // Godot: RecipeRegistry.cs EnsureLoaded
    pub fn from_repo() -> Result<Self, RecipeLoadError> {
        let path = paths::repo_path("data/recipes.json");
        Self::load(&path)
    }

    /// Reads recipes from the given file path.
    pub fn load(path: &Path) -> Result<Self, RecipeLoadError> {
        let text = std::fs::read_to_string(path).map_err(RecipeLoadError::Io)?;
        Self::from_json(&text)
    }

    /// Parses recipe definitions from JSON text and validates them.
    pub fn from_json(text: &str) -> Result<Self, RecipeLoadError> {
        let list: Vec<RecipeDef> =
            serde_json::from_str(text).map_err(RecipeLoadError::Parse)?;
        Self::from_vec(list, None)
    }

    fn from_vec(list: Vec<RecipeDef>, items: Option<&ItemRegistry>) -> Result<Self, RecipeLoadError> {
        if list.is_empty() {
            return Err(RecipeLoadError::Validation("recipes list is empty".to_string()));
        }

        let mut by_id = BTreeMap::new();
        for (i, recipe) in list.iter().enumerate() {
            Self::validate_recipe(recipe, items)?;
            if by_id.insert(recipe.id.clone(), i).is_some() {
                return Err(RecipeLoadError::Validation(format!(
                    "Recipe id '{}' appears twice",
                    recipe.id
                )));
            }
        }

        Ok(Self {
            recipes: list,
            by_id,
        })
    }

    /// Validates a single recipe definition.
    /// // Godot: RecipeRegistry.cs Validate
    pub fn validate_recipe(recipe: &RecipeDef, items: Option<&ItemRegistry>) -> Result<(), RecipeLoadError> {
        if recipe.id.is_empty() {
            return Err(RecipeLoadError::Validation("recipe has no id".to_string()));
        }
        if recipe.station != HAND_STATION && recipe.station != CAMPFIRE_STATION {
            return Err(RecipeLoadError::Validation(format!(
                "Recipe '{}' has unknown station '{}'",
                recipe.id, recipe.station
            )));
        }
        if recipe.time <= 0.0 {
            return Err(RecipeLoadError::Validation(format!(
                "Recipe '{}' has time {}, must be above 0",
                recipe.id, recipe.time
            )));
        }
        if recipe.inputs.is_empty() || recipe.outputs.is_empty() {
            return Err(RecipeLoadError::Validation(format!(
                "Recipe '{}' needs at least one input and one output",
                recipe.id
            )));
        }

        let mut input_ids = HashSet::new();
        for input in &recipe.inputs {
            if !input_ids.insert(&input.id) {
                return Err(RecipeLoadError::Validation(format!(
                    "Recipe '{}' lists input '{}' twice",
                    recipe.id, input.id
                )));
            }
        }

        for item in recipe.inputs.iter().chain(recipe.outputs.iter()) {
            if item.count < 1 {
                return Err(RecipeLoadError::Validation(format!(
                    "Recipe '{}' has count {} for '{}', must be at least 1",
                    recipe.id, item.count, item.id
                )));
            }
            if let Some(registry) = items
                && !registry.contains(&item.id)
            {
                return Err(RecipeLoadError::Validation(format!(
                    "Recipe '{}' uses item '{}', which is not in items.json",
                    recipe.id, item.id
                )));
            }
        }

        if let Some(xp) = &recipe.xp
            && (xp.skill.is_empty() || xp.amount <= 0.0)
        {
            return Err(RecipeLoadError::Validation(format!(
                "Recipe '{}' has invalid XP configuration",
                recipe.id
            )));
        }

        Ok(())
    }

    /// Validates all recipes against an explicit ItemRegistry.
    pub fn validate(&self, items: &ItemRegistry) -> Result<(), RecipeLoadError> {
        for recipe in &self.recipes {
            Self::validate_recipe(recipe, Some(items))?;
        }
        Ok(())
    }

    pub fn all(&self) -> &[RecipeDef] {
        &self.recipes
    }

    pub fn get(&self, id: &str) -> Option<&RecipeDef> {
        self.by_id.get(id).map(|&idx| &self.recipes[idx])
    }

    pub fn try_get(&self, id: &str) -> Option<&RecipeDef> {
        self.get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.by_id.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.recipes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }
}
