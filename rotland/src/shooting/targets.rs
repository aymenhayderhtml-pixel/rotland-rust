//! Targets for shooting: the zombie head/body geometry and the `TargetWorld` trait that gun hits go through.

use game::guns_fire::GunTarget;
use game::DamageInfo;
use glam::Vec3;

use crate::zombies::{Zombies, BODY_RADIUS};

/// Height of the zombie's head centre relative to its feet (metres).
pub const HEAD_HEIGHT_OFFSET: f32 = 1.65;
/// Radius of the head sphere target (metres).
pub const HEAD_RADIUS: f32 = 0.14;

/// Abstraction over zombie target querying and damage application.
/// Allows unit testing without needing a GPU device for `Zombies`.
pub trait TargetWorld {
    fn gun_targets(&self) -> Vec<GunTarget>;
    fn hit(&mut self, id: u64, damage: DamageInfo, knockback: Vec3) -> bool;
}

impl TargetWorld for Zombies {
    fn gun_targets(&self) -> Vec<GunTarget> {
        self.list
            .iter()
            .filter(|z| !z.health.is_dead())
            .map(|z| GunTarget {
                id: z.id,
                position: z.position,
                body_radius: BODY_RADIUS,
                head_center: z.position + Vec3::Y * HEAD_HEIGHT_OFFSET,
                head_radius: HEAD_RADIUS,
            })
            .collect()
    }

    fn hit(&mut self, id: u64, damage: DamageInfo, knockback: Vec3) -> bool {
        self.hit(id, damage, knockback)
    }
}
