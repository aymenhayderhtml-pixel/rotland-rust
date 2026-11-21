//! Continuous-collision projectile simulation and weapon launch helpers.

use glam::Vec3;

use crate::guns_fire::GunRng;
use crate::weapons::WeaponDef;
use super::types::{Hit, Projectile, ProjectileEvent, ProjectileKind};

/// Pure simulation of active projectiles across time steps.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectileSim {
    projectiles: Vec<Projectile>,
    rng: GunRng,
}

impl Default for ProjectileSim {
    fn default() -> Self {
        Self::new(0)
    }
}

impl ProjectileSim {
    pub fn new(seed: u64) -> Self {
        let mut rng = GunRng::new(seed);
        rng.next_u64();
        rng.next_u64();
        rng.next_u64();
        rng.next_u64();
        Self {
            projectiles: Vec::new(),
            rng,
        }
    }

    pub fn spawn(&mut self, projectile: Projectile) {
        self.projectiles.push(projectile);
    }

    pub fn len(&self) -> usize {
        self.projectiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.projectiles.is_empty()
    }

    pub fn projectiles(&self) -> &[Projectile] {
        &self.projectiles
    }

    /// Advances the simulation by dt. Calls raycast(from, to) for each projectile to check
    /// for collisions along the flight path (preventing tunnelling at high speeds).
    pub fn step(
        &mut self,
        dt: f32,
        mut raycast: impl FnMut(Vec3, Vec3) -> Option<Hit>,
    ) -> Vec<ProjectileEvent> {
        let mut events = Vec::new();
        let mut remaining = Vec::with_capacity(self.projectiles.len());

        for mut proj in self.projectiles.drain(..) {
            proj.vel.y -= proj.gravity * dt;
            let step_offset = proj.vel * dt;
            let from = proj.pos;
            let to = from + step_offset;

            if let Some(hit) = raycast(from, to) {
                match hit {
                    Hit::Zombie { id, headshot, .. } => {
                        let damage = if headshot {
                            proj.damage * proj.headshot_multiplier
                        } else {
                            proj.damage
                        };
                        events.push(ProjectileEvent::HitZombie {
                            id,
                            damage,
                            headshot,
                        });
                    }
                    Hit::World { point } => {
                        let roll = self.rng.next_f32();
                        let pickup = if roll < proj.pickup_chance {
                            match proj.kind {
                                ProjectileKind::Arrow => Some("arrow".to_string()),
                                ProjectileKind::ThrownSpear => Some("spear".to_string()),
                            }
                        } else {
                            None
                        };
                        events.push(ProjectileEvent::HitWorld { point, pickup });
                    }
                }
            } else {
                proj.pos = to;
                proj.flight_time += dt;
                if proj.flight_time >= proj.max_flight_seconds {
                    events.push(ProjectileEvent::Expired);
                } else {
                    remaining.push(proj);
                }
            }
        }

        self.projectiles = remaining;
        events
    }
}

/// Creates an arrow projectile fired from a bow based on draw amount (0..1).
pub fn bow_shot(draw: f32, aim_origin: Vec3, aim_dir: Vec3, def: &WeaponDef) -> Projectile {
    let draw_clamped = draw.clamp(0.0, 1.0);
    let speed = def.arrow_speed * draw_clamped;
    let damage = def.damage * draw_clamped;
    let dir = aim_dir.normalize_or_zero();
    let vel = dir * speed;
    let gravity = if def.gravity > 0.0 {
        def.gravity
    } else {
        // data: weapons.json gravity
        9.8
    };
    let headshot_multiplier = if def.headshot_multiplier > 0.0 {
        def.headshot_multiplier
    } else {
        // data: weapons.json headshotMultiplier
        2.5
    };
    let max_flight_seconds = if def.max_flight_seconds > 0.0 {
        def.max_flight_seconds
    } else {
        // data: weapons.json maxFlightSeconds
        10.0
    };
    let pickup_chance = if def.pickup_chance > 0.0 {
        def.pickup_chance
    } else {
        // data: weapons.json pickupChance
        0.6
    };
    Projectile {
        pos: aim_origin,
        vel,
        kind: ProjectileKind::Arrow,
        damage,
        owner: None,
        headshot_multiplier,
        gravity,
        max_flight_seconds,
        flight_time: 0.0,
        pickup_chance,
    }
}

/// Creates a thrown spear projectile from a spear weapon definition.
pub fn spear_throw(aim_origin: Vec3, aim_dir: Vec3, def: &WeaponDef) -> Projectile {
    let speed = if def.throw_speed > 0.0 {
        def.throw_speed
    } else {
        // data: weapons.json throwSpeed
        20.0
    };
    let damage = if def.throw_damage > 0.0 {
        def.throw_damage
    } else {
        // data: weapons.json throwDamage
        45.0
    };
    let dir = aim_dir.normalize_or_zero();
    let vel = dir * speed;
    let gravity = if def.gravity > 0.0 {
        def.gravity
    } else {
        // data: weapons.json gravity
        9.8
    };
    let headshot_multiplier = if def.headshot_multiplier > 0.0 {
        def.headshot_multiplier
    } else {
        // data: weapons.json headshotMultiplier
        2.5
    };
    let max_flight_seconds = if def.max_flight_seconds > 0.0 {
        def.max_flight_seconds
    } else {
        // data: weapons.json maxFlightSeconds
        10.0
    };
    let pickup_chance = if def.pickup_chance > 0.0 {
        def.pickup_chance
    } else {
        1.0
    };
    Projectile {
        pos: aim_origin,
        vel,
        kind: ProjectileKind::ThrownSpear,
        damage,
        owner: None,
        headshot_multiplier,
        gravity,
        max_flight_seconds,
        flight_time: 0.0,
        pickup_chance,
    }
}
