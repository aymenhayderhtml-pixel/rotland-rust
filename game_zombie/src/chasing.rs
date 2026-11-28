//! Counts zombies that are chasing or attacking near a point (R44 safe house travel check).

use glam::{Vec2, Vec3};

/// Counts entries whose flag is true (chasing or attacking) and whose flat (XZ) distance to
/// `pos` is within `radius`. Height is ignored. Uses squared distances (no sqrt).
pub fn count_chasing_near(
    zombies: impl IntoIterator<Item = (Vec3, bool)>,
    pos: Vec3,
    radius: f32,
) -> usize {
    let radius_sq = radius * radius;
    zombies
        .into_iter()
        .filter(|(zombie_pos, chasing)| {
            let flat = Vec2::new(zombie_pos.x - pos.x, zombie_pos.z - pos.z);
            *chasing && flat.length_squared() <= radius_sq
        })
        .count()
}
