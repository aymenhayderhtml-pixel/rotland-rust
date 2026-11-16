/// Headshot kill XP multiplier (port of scripts/Skills/XpRewards.cs HeadshotMultiplier).
pub const HEADSHOT_MULTIPLIER: f32 = 1.5;

/// Metres of sprinting required for 1 Agility XP (port of scripts/Skills/XpRewards.cs MetresPerAgilityXp).
pub const METRES_PER_AGILITY_XP: f32 = 20.0;

/// Calculates Combat XP awarded for killing a zombie.
/// Rounded away from zero to whole XP matching C# Mathf.Round(walker.Def.Xp * HeadshotMultiplier).
// Godot: XpRewards.cs OnZombieKilled
pub fn combat_xp_for_kill(base_xp: f32, is_headshot: bool) -> f32 {
    if is_headshot {
        ((base_xp * HEADSHOT_MULTIPLIER) as f64).round() as f32
    } else {
        base_xp
    }
}

/// Tracks sprinting distance and awards Agility XP (port of scripts/Skills/XpRewards.cs).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AgilityTracker {
    pub sprint_metres: f32,
}

impl AgilityTracker {
    pub fn new() -> Self {
        Self { sprint_metres: 0.0 }
    }

    /// Adds horizontal sprinting distance in metres and returns integer Agility XP earned.
    // Godot: XpRewards.cs _PhysicsProcess
    pub fn add_sprint_distance(&mut self, horizontal_metres: f32) -> u32 {
        if horizontal_metres <= 0.0 {
            return 0;
        }
        self.sprint_metres += horizontal_metres;
        let xp = (self.sprint_metres / METRES_PER_AGILITY_XP).floor() as u32;
        if xp > 0 {
            self.sprint_metres -= (xp as f32) * METRES_PER_AGILITY_XP;
        }
        xp
    }
}
