//! One hint definition from data/hints.json.
//!
//! Port of Godot C# scripts/UI/HintDef.cs.

use serde::{Deserialize, Serialize};

// Godot: HintDef.cs
/// One first-time tip definition from data/hints.json.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HintDef {
    pub id: String,
    pub text: String,
    pub trigger: String,
    #[serde(default, rename = "doneWhen", alias = "done_when")]
    pub done_when: Option<String>,
}

impl HintDef {
    pub fn new(
        id: impl Into<String>,
        text: impl Into<String>,
        trigger: impl Into<String>,
        done_when: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            trigger: trigger.into(),
            done_when,
        }
    }
}
