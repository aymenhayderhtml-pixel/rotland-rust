use glam::Vec3;

use crate::farming::def::CropStage;

/// Pure farming rules: stages, watering, yield, spacing, and ground checks.
// Godot: FarmRules.cs
pub struct FarmRules;

impl FarmRules {
    /// Sprout until a third of the growing time, young until ripe, then ripe.
    // Godot: FarmRules.cs StageOf
    pub fn stage_of(growth: f64, total_hours: f64) -> CropStage {
        if growth >= total_hours {
            CropStage::Ripe
        } else if growth >= total_hours / 3.0 {
            CropStage::Young
        } else {
            CropStage::Sprout
        }
    }

    /// A bed watered at watered_at keeps growing until watered_at + window (in hours).
    /// Never watered (-1): false.
    // Godot: FarmRules.cs IsWatered
    pub fn is_watered(now: f64, watered_at: f64, window_hours: f64) -> bool {
        watered_at >= 0.0 && (now - watered_at) <= window_hours
    }

    /// Base yield plus one for each farming level in bonus_levels reached by the player.
    // Godot: FarmRules.cs YieldFor
    pub fn yield_for(base_yield: u32, farming_level: u32, bonus_levels: &[u32]) -> u32 {
        let mut total = base_yield;
        for &level in bonus_levels {
            if farming_level >= level {
                total += 1;
            }
        }
        total
    }

    /// True when spot is at least spacing metres from every other bed centre (flat XZ distance).
    // Godot: FarmRules.cs SpotIsFree
    pub fn spot_is_free<'a>(
        spot: Vec3,
        others: impl IntoIterator<Item = &'a Vec3>,
        spacing: f32,
    ) -> bool {
        let spacing_sq = spacing * spacing;
        for other in others {
            let dx = spot.x - other.x;
            let dz = spot.z - other.z;
            if dx * dx + dz * dz < spacing_sq {
                return false;
            }
        }
        true
    }

    /// Ground normal Y this close to straight up is flat enough to till (~23 degrees max).
    // Godot: FarmRules.cs IsFlat
    pub fn is_flat(normal: Vec3) -> bool {
        normal.y >= 0.92
    }

    /// roll is in [0, 1). A roll under chance passes.
    // Godot: FarmRules.cs Passes
    pub fn passes(roll: f64, chance: f64) -> bool {
        roll < chance
    }
}
