//! Shared helpers for the shooting unit tests: mock zombies, a flat physics floor and the weapon registry.

use game::{Health, WeaponRegistry};
use glam::{IVec2, Vec3};
use physics::{PhysicsWorld, CHUNK_SAMPLES};
use game::guns_fire::GunTarget;
use game::DamageInfo;

use crate::zombies::BODY_RADIUS;

use super::targets::{TargetWorld, HEAD_HEIGHT_OFFSET, HEAD_RADIUS};

pub(super) struct MockZombie {
    pub(super) id: u64,
    pub(super) position: Vec3,
    pub(super) health: Health,
}

pub(super) struct MockWorld {
    pub(super) zombies: Vec<MockZombie>,
}

impl TargetWorld for MockWorld {
    fn gun_targets(&self) -> Vec<GunTarget> {
        self.zombies
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

    fn hit(&mut self, id: u64, damage: DamageInfo, _knockback: Vec3) -> bool {
        let Some(z) = self.zombies.iter_mut().find(|z| z.id == id) else { return false };
        let res = z.health.take_damage(damage);
        res.died_now
    }
}

pub(super) fn flat_world() -> PhysicsWorld {
    let mut world = PhysicsWorld::new();
    let heights = vec![0.0f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
    world.add_ground_chunk(IVec2::new(0, 0), &heights);
    world
}

pub(super) fn load_weapons() -> WeaponRegistry {
    WeaponRegistry::load(&paths::repo_path("data/weapons.json"))
        .expect("data/weapons.json must load")
}
