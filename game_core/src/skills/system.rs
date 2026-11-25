use std::collections::HashMap;
use std::sync::Arc;

use crate::skills::def::SkillDef;
use crate::skills::registry::SkillRegistry;

/// Current level and partial XP inside the level for one skill.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkillProgress {
    pub level: u32,
    pub xp: f32,
}

impl Default for SkillProgress {
    fn default() -> Self {
        Self {
            level: 1,
            xp: 0.0,
        }
    }
}

/// The player's skill progression and stat bonuses (port of scripts/Skills/SkillSystem.cs).
/// Saved under the save id "skills".
#[derive(Clone, Debug)]
pub struct SkillSystem {
    registry: Arc<SkillRegistry>,
    skills: HashMap<String, SkillProgress>,
}

impl SkillSystem {
    pub const SAVE_ID: &'static str = "skills";

    /// Creates a fresh skill system with all skills at level 1 and 0 XP.
    pub fn new(registry: Arc<SkillRegistry>) -> Self {
        let mut skills = HashMap::new();
        for def in registry.all() {
            skills.insert(def.id.clone(), SkillProgress::default());
        }
        Self { registry, skills }
    }

    /// Loads SkillRegistry from repo and initializes all skills.
    pub fn from_repo() -> Result<Self, Box<dyn std::error::Error>> {
        let registry = SkillRegistry::from_repo()?;
        Ok(Self::new(Arc::new(registry)))
    }

    pub fn registry(&self) -> &SkillRegistry {
        &self.registry
    }

    /// Current level of a skill (starts at 1). Returns 1 if unknown.
    // Godot: SkillSystem.cs GetLevel
    pub fn get_level(&self, id: &str) -> u32 {
        self.skills.get(id).map_or(1, |p| p.level)
    }

    /// XP earned inside the current level. Returns 0.0 if unknown.
    // Godot: SkillSystem.cs GetXp
    pub fn get_xp(&self, id: &str) -> f32 {
        self.skills.get(id).map_or(0.0, |p| p.xp)
    }

    /// XP still needed to reach the next level. Returns 0 at or above max level.
    // Godot: SkillSystem.cs GetXpToNext
    pub fn get_xp_to_next(&self, id: &str) -> u32 {
        let Some(def) = self.registry.get(id) else {
            return 0;
        };
        let level = self.get_level(id);
        if level >= def.max_level {
            0
        } else {
            def.xp_to_next(level)
        }
    }

    /// Sum over all skills of (bonus per level) * (level - 1).
    /// Zero when every skill is at level 1.
    // Godot: SkillSystem.cs GetBonus
    pub fn get_bonus(&self, stat: &str) -> f32 {
        let mut total = 0.0;
        for def in self.registry.all() {
            if let Some(&per_level) = def.bonus_per_level.get(stat) {
                let level = self.get_level(&def.id);
                if level > 1 {
                    total += per_level * ((level - 1) as f32);
                }
            }
        }
        total
    }

    /// Adds XP and handles level-ups. Excess XP beyond max level is discarded.
    /// Returns (granted_xp, list_of_new_levels).
    // Godot: SkillSystem.cs AddXp
    pub fn add_xp(&mut self, id: &str, amount: f32) -> (f32, Vec<u32>) {
        if amount <= 0.0 {
            return (0.0, Vec::new());
        }
        let Some(def) = self.registry.get(id).cloned() else {
            return (0.0, Vec::new());
        };

        let progress = self.skills.entry(id.to_string()).or_default();
        let left_to_max = Self::xp_left_to_max(&def, progress);
        let granted = amount.min(left_to_max);
        if granted <= 0.0 {
            return (0.0, Vec::new());
        }

        progress.xp += granted;
        let mut new_levels = Vec::new();

        while progress.level < def.max_level && progress.xp >= def.xp_to_next(progress.level) as f32 {
            progress.xp -= def.xp_to_next(progress.level) as f32;
            progress.level += 1;
            new_levels.push(progress.level);
        }

        if progress.level >= def.max_level {
            progress.xp = 0.0;
        }

        (granted, new_levels)
    }

    /// XP from current progress to max level.
    // Godot: SkillSystem.cs XpLeftToMax
    fn xp_left_to_max(def: &SkillDef, progress: &SkillProgress) -> f32 {
        let mut left = -progress.xp;
        for level in progress.level..def.max_level {
            left += def.xp_to_next(level) as f32;
        }
        left
    }

    /// Saves each skill as `{ "level": int, "xp": float }`.
    // Godot: SkillSystem.cs Save
    pub fn to_save(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        for (id, progress) in &self.skills {
            map.insert(
                id.clone(),
                serde_json::json!({
                    "level": progress.level,
                    "xp": progress.xp,
                }),
            );
        }
        serde_json::Value::Object(map)
    }

    /// Loads saved skills. Unknown ids are ignored; level and XP are clamped to valid ranges.
    // Godot: SkillSystem.cs Load
    pub fn from_save(&mut self, data: &serde_json::Value) {
        let Some(obj) = data.as_object() else {
            return;
        };

        for (key, val) in obj {
            let Some(def) = self.registry.get(key) else {
                continue;
            };
            let Some(fields) = val.as_object() else {
                continue;
            };

            let level_val = fields
                .get("level")
                .and_then(|v| v.as_u64())
                .map_or(1, |v| v as u32);
            let xp_val = fields
                .get("xp")
                .and_then(|v| v.as_f64())
                .map_or(0.0, |v| v as f32);

            let clamped_level = level_val.clamp(1, def.max_level);
            let clamped_xp = if clamped_level >= def.max_level {
                0.0
            } else {
                xp_val.clamp(0.0, def.xp_to_next(clamped_level) as f32)
            };

            let progress = self.skills.entry(key.clone()).or_default();
            progress.level = clamped_level;
            progress.xp = clamped_xp;
        }
    }
}
