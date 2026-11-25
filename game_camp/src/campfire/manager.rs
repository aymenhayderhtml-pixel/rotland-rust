use glam::{Mat4, Vec3};
use pack::Marker;
use serde_json::Value;

use game_core::items::Inventory;
use super::fire::{Campfire, CAMPFIRE_RANGE};
use super::placer::{place_campfire, PlaceCampfireError};
use super::save::{from_save_campfires, to_save_campfires};

/// Checks if `pos` is within `CAMPFIRE_RANGE` (4.0m flat) of any lit campfire.
pub fn near_campfire(pos: Vec3, fires: &[Campfire]) -> bool {
    fires
        .iter()
        .any(|f| f.is_lit() && f.distance_flat(pos) <= CAMPFIRE_RANGE)
}

/// Finds the nearest lit campfire to `pos` within `range` (flat distance).
// Godot: Campfire.cs FindLitNear
pub fn find_lit_near<'a>(pos: Vec3, fires: &'a [Campfire], range: f32) -> Option<&'a Campfire> {
    let mut best: Option<(&'a Campfire, f32)> = None;
    for fire in fires {
        if !fire.is_lit() {
            continue;
        }
        let d = fire.distance_flat(pos);
        if d <= range {
            let closer = match best {
                None => true,
                Some((_, best_d)) => d < best_d,
            };
            if closer {
                best = Some((fire, d));
            }
        }
    }
    best.map(|(f, _)| f)
}

/// Extracts the permanent settlement campfire from model markers.
/// Port of `scripts/Settlement/SettlementBuilder.cs`.
pub fn settlement_campfire_from_markers(markers: &[Marker], site_matrix: Mat4) -> Option<Campfire> {
    for m in markers {
        if m.name == "campfire" {
            let pos = site_matrix.transform_point3(Vec3::from_array(m.translation));
            return Some(Campfire::create_permanent(pos));
        }
    }
    None
}

/// Container and manager for active and placed campfires in the world.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CampfireManager {
    campfires: Vec<Campfire>,
}

impl CampfireManager {
    pub fn new() -> Self {
        Self {
            campfires: Vec::new(),
        }
    }

    pub fn campfires(&self) -> &[Campfire] {
        &self.campfires
    }

    pub fn campfires_mut(&mut self) -> &mut Vec<Campfire> {
        &mut self.campfires
    }

    pub fn add(&mut self, fire: Campfire) {
        // BUG-003: If a permanent campfire is already within 1m, don't add a duplicate
        if fire.permanent && self.has_permanent_near(fire.position, 1.0) {
            return;
        }
        self.campfires.push(fire);
    }

    /// Checks if a permanent campfire already exists within `range` flat distance of `pos`.
    pub fn has_permanent_near(&self, pos: Vec3, range: f32) -> bool {
        self.campfires
            .iter()
            .any(|f| f.permanent && f.distance_flat(pos) <= range)
    }

    pub fn clear(&mut self) {
        self.campfires.clear();
    }

    pub fn advance_hours(&mut self, hours: f32) {
        for fire in &mut self.campfires {
            fire.advance_hours(hours);
        }
    }

    pub fn tick(&mut self, dt_seconds: f32) {
        for fire in &mut self.campfires {
            fire.tick(dt_seconds);
        }
    }

    pub fn near_campfire(&self, pos: Vec3) -> bool {
        near_campfire(pos, &self.campfires)
    }

    pub fn find_lit_near(&self, pos: Vec3, range: f32) -> Option<&Campfire> {
        find_lit_near(pos, &self.campfires, range)
    }

    pub fn place(
        &mut self,
        player_pos: Vec3,
        facing: Vec3,
        inventory: Option<&mut Inventory>,
        ground: impl Fn(Vec3) -> Option<(Vec3, Vec3)>,
    ) -> Result<Campfire, PlaceCampfireError> {
        let fire = place_campfire(player_pos, facing, &self.campfires, inventory, ground)?;
        self.campfires.push(fire.clone());
        Ok(fire)
    }

    pub fn to_save(&self) -> Value {
        to_save_campfires(&self.campfires)
    }

    pub fn from_save(&mut self, data: &Value) -> Result<(), String> {
        let loaded = from_save_campfires(data)?;
        self.campfires.retain(|f| f.permanent);
        self.campfires.extend(loaded);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permanent_campfire_not_duplicated_within_1m() {
        let mut mgr = CampfireManager::new();
        let fire = Campfire::create_permanent(Vec3::new(10.0, 0.0, 10.0));
        mgr.add(fire.clone());
        assert_eq!(mgr.campfires().len(), 1);

        // Exact duplicate addition skipped
        mgr.add(fire.clone());
        assert_eq!(mgr.campfires().len(), 1);

        // Within 1m flat distance skipped
        let nearby = Campfire::create_permanent(Vec3::new(10.5, 0.0, 10.5));
        mgr.add(nearby);
        assert_eq!(mgr.campfires().len(), 1);

        // Outside 1m is added
        let far = Campfire::create_permanent(Vec3::new(20.0, 0.0, 20.0));
        mgr.add(far);
        assert_eq!(mgr.campfires().len(), 2);
    }
}

