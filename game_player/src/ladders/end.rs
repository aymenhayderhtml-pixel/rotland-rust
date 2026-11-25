use glam::Vec3;

use super::climb::GRAB_RANGE;
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

/// Finds the nearest ladder end within grab range of `player_pos`.
/// Returns the ladder end and its distance to the player, or `None` if none in reach or `already_climbing`.
// Godot: LadderEnd.cs CanInteract
pub fn nearest_ladder_end(
    player_pos: Vec3,
    rails: &[LadderRail],
    already_climbing: bool,
) -> Option<(LadderEnd, f32)> {
    if already_climbing {
        return None;
    }
    let mut best: Option<(LadderEnd, f32)> = None;
    for rail in rails {
        let foot_end = LadderEnd::new(rail.clone(), false);
        if foot_end.can_interact(already_climbing) {
            let d_foot = player_pos
                .distance(foot_end.position)
                .min(player_pos.distance(rail.foot));
            if d_foot <= GRAB_RANGE {
                let closer = match &best {
                    None => true,
                    Some((_, dist)) => d_foot < *dist,
                };
                if closer {
                    best = Some((foot_end, d_foot));
                }
            }
        }

        let deck_end = LadderEnd::new(rail.clone(), true);
        if deck_end.can_interact(already_climbing) {
            let d_deck = player_pos
                .distance(deck_end.position)
                .min(player_pos.distance(rail.deck));
            if d_deck <= GRAB_RANGE {
                let closer = match &best {
                    None => true,
                    Some((_, dist)) => d_deck < *dist,
                };
                if closer {
                    best = Some((deck_end, d_deck));
                }
            }
        }
    }
    best
}

/// Returns the interaction prompt for the nearest ladder end in reach, or `None`.
// Godot: LadderEnd.cs GetPrompt
pub fn ladder_prompt(
    player_pos: Vec3,
    rails: &[LadderRail],
    already_climbing: bool,
) -> Option<&'static str> {
    nearest_ladder_end(player_pos, rails, already_climbing).map(|(end, _)| end.prompt())
}
