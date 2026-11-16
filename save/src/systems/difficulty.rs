use serde::{Deserialize, Serialize};

/// What happens on death.
/// Matches `scripts/Difficulty/DeathRule.cs`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeathRule {
    LoseAll,
    KeepHalf,
    KeepAll,
}

impl Default for DeathRule {
    fn default() -> Self {
        Self::LoseAll
    }
}

fn default_multiplier() -> f32 {
    1.0
}

/// Multipliers and death rules for world difficulty.
/// Port of `scripts/Difficulty/WorldDifficulty.cs` and `DifficultySettings.cs` (SaveId: "difficulty").
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DifficultySaveData {
    #[serde(default = "default_multiplier")]
    pub zombies: f32,
    #[serde(default = "default_multiplier")]
    pub loot: f32,
    #[serde(default = "default_multiplier")]
    pub damage: f32,
    #[serde(default)]
    pub death: DeathRule,
}

impl Default for DifficultySaveData {
    fn default() -> Self {
        Self {
            zombies: 1.0,
            loot: 1.0,
            damage: 1.0,
            death: DeathRule::LoseAll,
        }
    }
}

impl DifficultySaveData {
    pub fn new(zombies: f32, loot: f32, damage: f32, death: DeathRule) -> Self {
        Self {
            zombies,
            loot,
            damage,
            death,
        }
    }

    /// Returns "Relaxed", "Normal", "Hard", or "Custom" matching `DifficultySettings.Label()`.
    pub fn label(&self) -> &'static str {
        const EPS: f32 = 0.001;
        if (self.zombies - 0.5).abs() < EPS
            && (self.loot - 1.5).abs() < EPS
            && (self.damage - 0.5).abs() < EPS
            && self.death == DeathRule::KeepAll
        {
            "Relaxed"
        } else if (self.zombies - 1.0).abs() < EPS
            && (self.loot - 1.0).abs() < EPS
            && (self.damage - 1.0).abs() < EPS
            && self.death == DeathRule::LoseAll
        {
            "Normal"
        } else if (self.zombies - 1.5).abs() < EPS
            && (self.loot - 0.75).abs() < EPS
            && (self.damage - 1.5).abs() < EPS
            && self.death == DeathRule::LoseAll
        {
            "Hard"
        } else {
            "Custom"
        }
    }
}
