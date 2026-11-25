//! When a wild animal notices the player.
// Godot: AnimalSenses.cs

use glam::Vec3;
use super::config::{AnimalBehaviourData, AnimalSpeciesData};

pub struct AnimalSenses;

impl AnimalSenses {
    /// The noticing range for this animal and player state (metres).
    // Godot: AnimalSenses.cs EffectiveRange
    pub fn effective_range(
        species: &AnimalSpeciesData,
        behaviour: &AnimalBehaviourData,
        crouching: bool,
        sprinting: bool,
        stealth_bonus: f32,
    ) -> f32 {
        let mut range = species.detect_range;
        if crouching {
            range *= behaviour.crouch_factor;
        }
        if sprinting {
            range *= behaviour.sprint_factor;
        }
        range * behaviour.min_stealth_factor.max(1.0 - stealth_bonus)
    }

    /// True when the animal, facing its forward (-Z), would notice a player at this point.
    // Godot: AnimalSenses.cs Notices
    #[allow(clippy::too_many_arguments)]
    pub fn notices(
        species: &AnimalSpeciesData,
        behaviour: &AnimalBehaviourData,
        animal_pos: Vec3,
        animal_yaw: f32,
        player_pos: Vec3,
        crouching: bool,
        sprinting: bool,
        stealth_bonus: f32,
    ) -> bool {
        let offset = Vec3::new(player_pos.x - animal_pos.x, 0.0, player_pos.z - animal_pos.z);
        let distance = offset.length();
        if distance > Self::effective_range(species, behaviour, crouching, sprinting, stealth_bonus) {
            return false;
        }
        if distance <= behaviour.close_sense_dist {
            return true;
        }
        if offset.length_squared() < 0.0001 {
            return true;
        }
        let forward = Vec3::new(-animal_yaw.sin(), 0.0, -animal_yaw.cos());
        let angle_rad = forward.angle_between(offset);
        let angle_deg = angle_rad.to_degrees();
        angle_deg <= behaviour.sight_front_deg
    }
}
