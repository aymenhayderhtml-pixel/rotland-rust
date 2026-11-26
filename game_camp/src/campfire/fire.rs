use glam::{Vec2, Vec3};

use game_core::items::Inventory;
use super::visuals::CampfireVisuals;

/// Total in-game burn hours for a fueled campfire.
// Godot: Campfire.cs BurnHours
pub const BURN_HOURS: f32 = 5.0;
/// Item ID of wood needed to relight the fire.
// Godot: Campfire.cs WoodId
pub const WOOD_ID: &str = "wood";
/// The root floats this far above the ground so interaction rays clear the terrain.
// Godot: Campfire.cs RootLift
pub const ROOT_LIFT: f32 = 0.4;
pub use crate::crafting::CAMPFIRE_RANGE;
/// Real seconds in one in-game hour (matching TimeOfDay MinutesPerGameHour = 1.0).
pub const SECONDS_PER_HOUR: f32 = 60.0;

/// Outcome of interacting with a campfire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampfireInteractResult {
    /// Fire is lit: opens crafting panel at campfire station.
    OpenCrafting,
    /// Fire was out: 1 wood consumed from bag and fire relit.
    Relit,
    /// Fire is out and player has no wood.
    NeedsWood,
}

/// Campfire on the ground. Burns for `BURN_HOURS` in-game hours, then goes out.
/// When out, one wood relights it. Lit campfire acts as cooking station.
/// Port of `scripts/World/Campfire.cs`.
#[derive(Clone, Debug, PartialEq)]
pub struct Campfire {
    pub position: Vec3,
    pub remaining_hours: f32,
    pub permanent: bool,
    pub visuals: CampfireVisuals,
}

impl Campfire {
    /// Makes a new campfire at ground level (applies ROOT_LIFT).
    // Godot: Campfire.cs Create
    pub fn create(ground: Vec3) -> Self {
        Self::restore(ground + Vec3::new(0.0, ROOT_LIFT, 0.0), BURN_HOURS, false)
    }

    /// Makes a permanent settlement campfire at ground level (never burns out).
    pub fn create_permanent(ground: Vec3) -> Self {
        Self::restore(ground + Vec3::new(0.0, ROOT_LIFT, 0.0), BURN_HOURS, true)
    }

    /// Makes a campfire at exact root position with given remaining hours.
    // Godot: Campfire.cs Restore
    pub fn restore(root_position: Vec3, remaining_hours: f32, permanent: bool) -> Self {
        let remaining = if permanent {
            BURN_HOURS
        } else {
            remaining_hours.clamp(0.0, BURN_HOURS)
        };
        let is_lit = remaining > 0.0;
        Self {
            position: root_position,
            remaining_hours: remaining,
            permanent,
            visuals: CampfireVisuals::new(is_lit),
        }
    }

    pub fn is_lit(&self) -> bool {
        self.remaining_hours > 0.0
    }

    /// Advances burn time by in-game hours.
    // Godot: Campfire.cs _Process
    pub fn advance_hours(&mut self, hours: f32) {
        if !self.permanent {
            self.remaining_hours = (self.remaining_hours - hours).max(0.0);
        }
        self.visuals.set_lit(self.is_lit());
    }

    /// Advances burn time and flicker visuals by real elapsed seconds.
    pub fn tick(&mut self, dt_seconds: f32) {
        if dt_seconds > 0.0 {
            self.advance_hours(dt_seconds / SECONDS_PER_HOUR);
            self.visuals.update(dt_seconds);
        }
    }

    /// Flat horizontal distance from `pos` to this campfire.
    pub fn distance_flat(&self, pos: Vec3) -> f32 {
        Vec2::new(self.position.x - pos.x, self.position.z - pos.z).length()
    }

    /// Prompt text: "[E] Cook" when lit, "[E] Add wood to relight" when out with wood,
    /// or "Campfire is out" when out with no wood.
    // Godot: Campfire.cs GetPrompt
    pub fn prompt(&self, inventory: Option<&Inventory>) -> &'static str {
        if self.is_lit() {
            "[E] Cook"
        } else if inventory.is_some_and(|inv| inv.has(WOOD_ID, 1)) {
            "[E] Add wood to relight"
        } else {
            "Campfire is out"
        }
    }

    /// Handles interaction: if out and inventory has 1 wood, consumes wood and relights fire.
    // Godot: Campfire.cs Interact
    pub fn interact(&mut self, inventory: &mut Inventory) -> CampfireInteractResult {
        if self.is_lit() {
            CampfireInteractResult::OpenCrafting
        } else if inventory.remove(WOOD_ID, 1) {
            self.remaining_hours = BURN_HOURS;
            self.visuals.set_lit(true);
            CampfireInteractResult::Relit
        } else {
            CampfireInteractResult::NeedsWood
        }
    }
}
