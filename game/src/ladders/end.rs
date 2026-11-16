use glam::Vec3;

use super::rail::LadderRail;

/// The interactable target at one end of a ladder.
/// Foot end climbs up; deck end climbs down.
/// Port of `scripts/World/LadderEnd.cs`.
#[derive(Clone, Debug, PartialEq)]
pub struct LadderEnd {
    pub position: Vec3,
    /// True for the deck end (climbs down), false for the foot end (climbs up).
    pub is_top: bool,
    pub rail: LadderRail,
}

impl LadderEnd {
    pub fn new(rail: LadderRail, is_top: bool) -> Self {
        let position = if is_top {
            rail.deck_interaction_point()
        } else {
            rail.foot_interaction_point()
        };
        Self {
            position,
            is_top,
            rail,
        }
    }

    /// Interaction prompt: "[E] Climb down" or "[E] Climb ladder".
    // Godot: LadderEnd.cs GetPrompt
    pub fn prompt(&self) -> &'static str {
        if self.is_top {
            "[E] Climb down"
        } else {
            "[E] Climb ladder"
        }
    }

    /// Whether player can start a climb here (not already climbing).
    // Godot: LadderEnd.cs CanInteract
    pub fn can_interact(&self, already_climbing: bool) -> bool {
        !already_climbing
    }
}
