//! Hint configuration data loader and validator.
//!
//! Port of Godot C# scripts/UI/HintConfigData.cs and scripts/UI/HintConfig.cs.

use std::collections::HashSet;
use std::path::Path;
use serde::{Deserialize, Serialize};

use super::def::HintDef;
use super::triggers::{check_trigger, DONE_NAMES};

fn default_enabled() -> bool {
    true
}

fn default_show_seconds() -> f32 {
    8.0
}

// Godot: HintConfigData.cs
/// Shape of data/hints.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HintConfigData {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_show_seconds", rename = "showSeconds", alias = "show_seconds")]
    pub show_seconds: f32,
    #[serde(default)]
    pub hints: Vec<HintDef>,
}

impl Default for HintConfigData {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            show_seconds: default_show_seconds(),
            hints: Vec::new(),
        }
    }
}

// Godot: HintConfig.cs
pub struct HintConfig;

impl HintConfig {
    /// Loads hint configuration from a JSON string and validates every entry.
    pub fn from_json(json: &str) -> Result<HintConfigData, String> {
        let data: HintConfigData = serde_json::from_str(json)
            .map_err(|e| format!("failed to parse hint json: {e}"))?;
        Self::check(&data)?;
        Ok(data)
    }

    /// Loads hint configuration from a file path.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<HintConfigData, String> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("could not open {:?}: {e}", path.as_ref()))?;
        Self::from_json(&content)
    }

    /// Loads the real data/hints.json file from the repository data folder.
    pub fn load_default() -> Result<HintConfigData, String> {
        let path = paths::repo_path("data/hints.json");
        Self::load_from_file(path)
    }

    /// Validates all hint entries, ensuring non-empty and unique IDs, valid triggers,
    /// and recognized doneWhen event names.
    // Godot: HintConfig.cs Check
    pub fn check(data: &HintConfigData) -> Result<(), String> {
        let mut ids = HashSet::new();
        for hint in &data.hints {
            if hint.id.is_empty() {
                return Err("hint id is empty".to_string());
            }
            if !ids.insert(&hint.id) {
                return Err(format!("hint id '{}' is used twice", hint.id));
            }
            if let Err(problem) = check_trigger(&hint.trigger) {
                return Err(format!("hint '{}': {problem}", hint.id));
            }
            if let Some(done) = &hint.done_when
                && !DONE_NAMES.contains(&done.as_str())
            {
                return Err(format!("hint '{}' has unknown doneWhen '{done}'", hint.id));
            }
        }
        Ok(())
    }
}
