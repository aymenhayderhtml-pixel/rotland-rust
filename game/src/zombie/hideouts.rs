//! Checks if a target is out of reach in a hideout (roof or ladder) to trigger banging (port of ZombieHideouts.cs).

use glam::Vec3;
use physics::PhysicsWorld;

const ROOF_CHECK_HEIGHT: f32 = 4.0;
const ROOF_NORMAL_MAX_Y: f32 = -0.5;

/// Tells whether a player/target is under a roof (inside a building) in a way zombies cannot reach directly.
pub fn is_under_roof(target: Vec3, world: &PhysicsWorld) -> bool {
    let head = target + Vec3::Y * 1.0;
    if let Some(hit) = world.raycast(head, Vec3::Y, ROOF_CHECK_HEIGHT)
        && hit.normal.y <= ROOF_NORMAL_MAX_Y
    {
        return true;
    }
    false
}
