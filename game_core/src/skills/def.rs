use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Stats and scaling for one skill (port of scripts/Skills/SkillDef.cs).
/// Loaded from data/skills.json by SkillRegistry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDef {
    /// The key in skills.json. SkillRegistry fills it in.
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default = "default_max_level")]
    pub max_level: u32,
    #[serde(default = "default_xp_base")]
    pub xp_base: f32,
    #[serde(default = "default_xp_growth")]
    pub xp_growth: f32,
    /// Bonus added per level above 1, keyed by stat name such as "meleeDamage".
    #[serde(default)]
    pub bonus_per_level: HashMap<String, f32>,
}

fn default_max_level() -> u32 {
    20
}

fn default_xp_base() -> f32 {
    100.0
}

fn default_xp_growth() -> f32 {
    1.35
}

impl Default for SkillDef {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            max_level: 20,
            xp_base: 100.0,
            xp_growth: 1.35,
            bonus_per_level: HashMap::new(),
        }
    }
}

impl SkillDef {
    /// XP needed to advance from this level to the next.
    /// Rounded away from zero to whole XP matching C# Math.Round(..., MidpointRounding.AwayFromZero).
    // Godot: SkillDef.cs XpToNext
    pub fn xp_to_next(&self, level: u32) -> u32 {
        if level == 0 {
            return 0;
        }
        let exp = (level as i32) - 1;
        let needed = (self.xp_base as f64) * (self.xp_growth as f64).powi(exp);
        needed.round() as u32
    }
}
