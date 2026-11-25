//! Sight, hearing and memory for one zombie (port of ZombieSenses.cs and the hearing rule in Zombie.cs).
//! The Godot version has no day/night difference, so none is modelled here.

use glam::{Vec2, Vec3};
use physics::PhysicsWorld;

use super::defs::ZombieDef;

/// Sight is checked this often, not every frame, to save ray casts.
pub const CHECK_INTERVAL: f32 = 0.2;
/// Within this flat distance the field-of-view cone is ignored: a zombie senses behind it up close.
pub const ALWAYS_SEE_RANGE: f32 = 2.5;
/// The sight ray starts at the zombie's eyes and ends at the target's chest.
pub const EYE_HEIGHT: f32 = 1.6;
pub const CHEST_HEIGHT: f32 = 1.2;

/// A sound. Zombies within `radius * hearing_multiplier` of `position` hear it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Noise {
    pub position: Vec3,
    pub radius: f32,
}

/// Environment multipliers applied to zombie sight range and hearing radius (e.g. weather).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SenseMultipliers {
    /// Multiplier on `sight_range` (weather `visibility`, 0..1). Defaults to `1.0`.
    pub sight_mult: f32,
    /// Multiplier on hearing radius (`1.0 - noise_mask`, 0..1). Defaults to `1.0`.
    pub hearing_mult: f32,
}

impl Default for SenseMultipliers {
    fn default() -> Self {
        Self {
            sight_mult: 1.0,
            hearing_mult: 1.0,
        }
    }
}

impl SenseMultipliers {
    /// Builds multipliers from weather `visibility` and `noise_mask` (both `0..1`).
    pub fn from_weather(visibility: f32, noise_mask: f32) -> Self {
        Self {
            sight_mult: visibility.clamp(0.0, 1.0),
            hearing_mult: (1.0 - noise_mask).clamp(0.0, 1.0),
        }
    }
}

/// True when a zombie with feet at `feet` hears `noise` (3D distance, same as the Godot rule).
pub fn hears(feet: Vec3, noise: &Noise, hearing_multiplier: f32) -> bool {
    feet.distance(noise.position) <= noise.radius * hearing_multiplier
}

/// Sight and memory for one zombie. Call `update` every frame.
#[derive(Clone, Debug)]
pub struct Senses {
    sight_range: f32,
    sight_fov_deg: f32,
    lose_target_time: f32,
    sight_mult: f32,
    hearing_mult: f32,
    /// Counts down to the next sight check.
    timer: f32,
    can_see: bool,
    last_seen: Vec3,
    /// Seconds since the target was last seen. Zero while seen.
    time_since_seen: f32,
}

impl Senses {
    pub fn new(def: &ZombieDef) -> Self {
        Self {
            sight_range: def.sight_range,
            sight_fov_deg: def.sight_fov_deg,
            lose_target_time: def.lose_target_time,
            sight_mult: 1.0,
            hearing_mult: 1.0,
            timer: 0.0,
            can_see: false,
            last_seen: Vec3::ZERO,
            time_since_seen: 0.0,
        }
    }

    pub fn can_see(&self) -> bool {
        self.can_see
    }

    pub fn last_seen(&self) -> Vec3 {
        self.last_seen
    }

    pub fn time_since_seen(&self) -> f32 {
        self.time_since_seen
    }

    /// Sets weather/environment multipliers for sight range and hearing radius.
    pub fn set_multipliers(&mut self, mults: SenseMultipliers) {
        self.sight_mult = mults.sight_mult.max(0.0);
        self.hearing_mult = mults.hearing_mult.max(0.0);
    }

    /// True when this zombie hears `noise`, factoring in both its base `hearing_multiplier`
    /// and the current environment `hearing_mult` (`1.0 - noise_mask`).
    pub fn hears(&self, feet: Vec3, noise: &Noise, hearing_multiplier: f32) -> bool {
        hears(feet, noise, hearing_multiplier * self.hearing_mult)
    }

    /// The zombie has lost sight for longer than `lose_target_time`. It then walks to `last_seen`
    /// instead of chasing. The memory itself is kept until a new sighting.
    pub fn memory_expired(&self) -> bool {
        self.time_since_seen > self.lose_target_time
    }

    /// A hit tells the zombie where its attacker is, even when sight fails.
    pub fn notice_at(&mut self, position: Vec3) {
        self.last_seen = position;
        self.time_since_seen = 0.0;
    }

    /// Forget everything. Used when a pooled zombie is handed out again.
    pub fn reset(&mut self) {
        self.sight_mult = 1.0;
        self.hearing_mult = 1.0;
        self.timer = 0.0;
        self.can_see = false;
        self.last_seen = Vec3::ZERO;
        self.time_since_seen = 0.0;
    }

    /// Test hook: the zombie has lost sight of the player long ago, last seen at `last_seen`.
    pub fn forget_for_test(&mut self, last_seen: Vec3) {
        self.can_see = false;
        self.last_seen = last_seen;
        self.time_since_seen = f32::MAX * 0.5;
    }

    /// `feet` is the zombie's position on the ground, `facing` the way it faces (height is ignored),
    /// `target` the target's feet position, or None when there is no living target.
    pub fn update(
        &mut self,
        dt: f32,
        feet: Vec3,
        facing: Vec3,
        target: Option<Vec3>,
        world: &PhysicsWorld,
    ) {
        let Some(target) = target else {
            self.can_see = false;
            self.time_since_seen += dt;
            return;
        };

        // Only re-checks sight every CHECK_INTERVAL; between checks the last answer holds
        self.timer -= dt;
        if self.timer <= 0.0 {
            self.timer = CHECK_INTERVAL;
            self.can_see = self.sees(feet, facing, target, world);
        }

        if self.can_see {
            self.last_seen = target;
            self.time_since_seen = 0.0;
        } else {
            self.time_since_seen += dt;
        }
    }

    fn in_sight_cone(&self, feet: Vec3, facing: Vec3, target: Vec3) -> bool {
        let to_target = Vec2::new(target.x - feet.x, target.z - feet.z);
        let distance = to_target.length();
        if distance > self.sight_range * self.sight_mult {
            return false;
        }

        // Up close the zombie senses all around it, but walls still block (checked below)
        if distance > ALWAYS_SEE_RANGE {
            let forward = Vec2::new(facing.x, facing.z);
            let half_fov = (self.sight_fov_deg * 0.5).to_radians();
            if angle_between(forward, to_target) > half_fov {
                return false;
            }
        }

        true
    }

    fn sees(&self, feet: Vec3, facing: Vec3, target: Vec3, world: &PhysicsWorld) -> bool {
        if !self.in_sight_cone(feet, facing, target) {
            return false;
        }

        let eye = feet + Vec3::Y * EYE_HEIGHT;
        let chest = target + Vec3::Y * CHEST_HEIGHT;
        let offset = chest - eye;
        let length = offset.length();
        // Any world hit on the way means a wall or prop blocks the view. Characters are not
        // in the world group, so they never block it.
        world.raycast(eye, offset / length, length).is_none()
    }
}

/// Unsigned angle between two horizontal directions, in radians. A zero vector gives 0 (the cone check
/// then passes), the same as Godot's AngleTo.
fn angle_between(a: Vec2, b: Vec2) -> f32 {
    let cross = a.x * b.y - a.y * b.x;
    cross.abs().atan2(a.dot(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_senses(sight_range: f32, sight_fov_deg: f32) -> Senses {
        Senses {
            sight_range,
            sight_fov_deg,
            lose_target_time: 5.0,
            sight_mult: 1.0,
            hearing_mult: 1.0,
            timer: 0.0,
            can_see: false,
            last_seen: Vec3::ZERO,
            time_since_seen: 0.0,
        }
    }

    #[test]
    fn sight_multiplier_scales_sight_range() {
        let mut senses = test_senses(20.0, 120.0);
        let feet = Vec3::ZERO;
        let facing = Vec3::NEG_Z;
        let target_15m = Vec3::new(0.0, 0.0, -15.0);
        let target_9m = Vec3::new(0.0, 0.0, -9.0);

        // Default sight_mult = 1.0 sees 15 m target within 20 m range.
        assert!(senses.in_sight_cone(feet, facing, target_15m));

        // 0.5 visibility shrinks effective sight range to 10 m.
        senses.set_multipliers(SenseMultipliers::from_weather(0.5, 0.0));
        assert!(!senses.in_sight_cone(feet, facing, target_15m));
        assert!(senses.in_sight_cone(feet, facing, target_9m));
    }

    #[test]
    fn hearing_multiplier_masks_noise_radius() {
        let mut senses = test_senses(20.0, 120.0);
        let feet = Vec3::ZERO;
        let far_noise = Noise {
            position: Vec3::new(15.0, 0.0, 0.0),
            radius: 20.0,
        };
        let near_noise = Noise {
            position: Vec3::new(9.0, 0.0, 0.0),
            radius: 20.0,
        };

        // Default hearing_mult = 1.0 hears a 20 m sound from 15 m away.
        assert!(senses.hears(feet, &far_noise, 1.0));

        // 0.5 noise_mask reduces hearing_mult to 0.5 (effective radius 10 m).
        senses.set_multipliers(SenseMultipliers::from_weather(1.0, 0.5));
        assert!(!senses.hears(feet, &far_noise, 1.0));
        assert!(senses.hears(feet, &near_noise, 1.0));
    }
}
