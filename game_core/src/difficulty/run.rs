use crate::difficulty::settings::DifficultySettings;

/// Runtime difficulty applied during gameplay (port of scripts/Difficulty/RunDifficulty.cs).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RunDifficulty {
    pub current: DifficultySettings,
}

impl RunDifficulty {
    pub fn new(current: DifficultySettings) -> Self {
        Self { current }
    }

    /// Scales zombie bite damage dealt to the player.
    /// Other damage types (hunger, thirst, fall, bullets, gas) and non-player targets are unchanged.
    // Godot: RunDifficulty.cs ScaleIncoming
    pub fn scale_incoming(&self, damage_type: &str, is_player: bool, amount: f32) -> f32 {
        if damage_type == "bite" && is_player {
            amount * self.current.zombie_damage
        } else {
            amount
        }
    }
}
