use glam::Vec3;

use crate::bounties::def::BountyDef;
use crate::bounties::manager::BountyManager;

/// The notice board in camp (port of scripts/Bounties/BountyBoard.cs).
/// Displays offered bounties and allows the player to inspect and accept them.
// Godot: BountyBoard.cs
#[derive(Clone, Debug, PartialEq)]
pub struct BountyBoard {
    pub position: Vec3,
}

impl Default for BountyBoard {
    fn default() -> Self {
        Self::new(Vec3::ZERO)
    }
}

impl BountyBoard {
    // Godot: BountyBoard.cs Create
    pub fn new(position: Vec3) -> Self {
        Self { position }
    }

    // Godot: BountyBoard.cs GetPrompt
    pub fn prompt() -> &'static str {
        "Read bounty board"
    }

    // Godot: BountyBoard.cs CanInteract
    pub fn can_interact(&self) -> bool {
        true
    }

    /// Returns the bounties currently offered on the board.
    pub fn offered<'a>(&self, manager: &'a BountyManager) -> &'a [BountyDef] {
        manager.offered()
    }

    /// Refreshes the board's offered bounties for the given day and hour.
    pub fn refresh(&self, manager: &mut BountyManager, day: i32, hour: f32) {
        manager.refresh(day, hour);
    }
}
