use std::collections::HashMap;
use std::path::Path;

use crate::skills::def::SkillDef;

/// Registry of all skills loaded from data/skills.json (port of scripts/Skills/SkillRegistry.cs).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkillRegistry {
    defs: HashMap<String, SkillDef>,
    all: Vec<SkillDef>,
}

impl SkillRegistry {
    pub fn from_repo() -> Result<Self, Box<dyn std::error::Error>> {
        let path = paths::repo_path("data/skills.json");
        Self::load(&path)
    }

    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let text = std::fs::read_to_string(path)?;
        Self::from_json(&text).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        let value: serde_json::Value = serde_json::from_str(text)?;
        let mut defs = HashMap::new();
        let mut all = Vec::new();

        if let Some(obj) = value.as_object() {
            for (key, val) in obj {
                let mut def: SkillDef = serde_json::from_value(val.clone())?;
                def.id = key.clone();
                defs.insert(key.clone(), def.clone());
                all.push(def);
            }
        }

        Ok(Self { defs, all })
    }

    /// All skills in the order defined in skills.json.
    // Godot: SkillRegistry.cs All
    pub fn all(&self) -> &[SkillDef] {
        &self.all
    }

    /// Finds a skill definition by its id.
    // Godot: SkillRegistry.cs Get / TryGet
    pub fn get(&self, id: &str) -> Option<&SkillDef> {
        self.defs.get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.defs.contains_key(id)
    }
}
