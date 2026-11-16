use glam::Vec3;

/// One ladder: its foot and deck positions (from `ladder_bottom*` / `ladder_top*` model markers).
/// Port of `scripts/World/LadderRail.cs`.
#[derive(Clone, Debug, PartialEq)]
pub struct LadderRail {
    /// Position where the climber stands at the bottom on the ground.
    pub foot: Vec3,
    /// The step-off point on the deck at the top.
    pub deck: Vec3,
    /// Suffix identifier pairing the markers (e.g. "" for watchtower, "_1" for settlement ladder 1).
    pub suffix: String,
}

impl LadderRail {
    /// The climber rises this far above the deck before stepping onto it, clearing the slab edge.
    // Godot: LadderRail.cs ClearHeight
    pub const CLEAR_HEIGHT: f32 = 0.4;
    /// Interaction marker height above foot and deck.
    // Godot: LadderEnd.cs Height
    pub const END_HEIGHT: f32 = 0.9;

    pub fn new(foot: Vec3, deck: Vec3, suffix: impl Into<String>) -> Self {
        Self {
            foot,
            deck,
            suffix: suffix.into(),
        }
    }

    /// Flat direction from foot toward deck: the way the climber faces the ladder.
    // Godot: LadderRail.cs Facing
    pub fn facing(&self) -> Vec3 {
        let mut flat = self.deck - self.foot;
        flat.y = 0.0;
        if flat.length_squared() > 1e-6 {
            flat.normalize()
        } else {
            Vec3::NEG_Z
        }
    }

    /// Interaction target position at the foot.
    pub fn foot_interaction_point(&self) -> Vec3 {
        self.foot + Vec3::new(0.0, Self::END_HEIGHT, 0.0)
    }

    /// Interaction target position at the deck.
    pub fn deck_interaction_point(&self) -> Vec3 {
        self.deck + Vec3::new(0.0, Self::END_HEIGHT, 0.0)
    }
}
