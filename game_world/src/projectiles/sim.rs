//! Continuous-collision projectile simulation and weapon launch helpers.

use glam::Vec3;
use physics::PhysicsWorld;

use crate::guns_fire::{GunRng, GunTarget};
use game_core::weapons::WeaponDef;
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

/// Finds the nearest collision segment along a ray from `from` to `to` against gun targets and the physics world.
/// Prioritizes targets over walls when the target hit is closer than the wall.
pub fn raycast_projectile(
    from: Vec3,
    to: Vec3,
    targets: &[GunTarget],
    physics: &PhysicsWorld,
) -> Option<Hit> {
    let delta = to - from;
    let dist = delta.length();
    if dist < 1e-6 {
        return None;
    }
    let dir = delta / dist;
    let wall = physics.raycast(from, dir, dist);

    let mut nearest_target: Option<(f32, u64, bool)> = None;
    for target in targets {
        if let Some((d, head)) = hit_target(from, dir, target)
            && d <= dist
            && nearest_target.is_none_or(|(nd, _, _)| d < nd)
        {
            nearest_target = Some((d, target.id, head));
        }
    }

    match (nearest_target, wall) {
        (Some((d, id, head)), wall) if wall.is_none_or(|w| d < w.distance) => {
            Some(Hit::Zombie {
                point: from + dir * d,
                id,
                headshot: head,
            })
        }
        (_, Some(w)) => Some(Hit::World { point: w.point }),
        _ => None,
    }
}

fn hit_target(origin: Vec3, dir: Vec3, target: &GunTarget) -> Option<(f32, bool)> {
    let head = ray_sphere(origin, dir, target.head_center, target.head_radius).map(|d| (d, true));
    let body = ray_body(origin, dir, target).map(|d| (d, false));
    match (head, body) {
        (Some(h), Some(b)) => Some(if h.0 <= b.0 { h } else { b }),
        (h, b) => h.or(b),
    }
}

fn ray_sphere(origin: Vec3, dir: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let oc = origin - center;
    let b = oc.dot(dir);
    let c = oc.length_squared() - radius * radius;
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    let near = -b - disc.sqrt();
    (near >= 0.0).then_some(near)
}

fn ray_body(origin: Vec3, dir: Vec3, target: &GunTarget) -> Option<f32> {
    let bottom = target.position.y;
    let top = target.head_center.y - target.head_radius;
    if top <= bottom {
        return None;
    }
    let ox = origin.x - target.position.x;
    let oz = origin.z - target.position.z;
    let a = dir.x * dir.x + dir.z * dir.z;
    if a < 1e-6 {
        return None;
    }
    let b = ox * dir.x + oz * dir.z;
    let c = ox * ox + oz * oz - target.body_radius * target.body_radius;
    let disc = b * b - a * c;
    if disc < 0.0 {
        return None;
    }
    let near = (-b - disc.sqrt()) / a;
    if near < 0.0 {
        return None;
    }
    let y = origin.y + dir.y * near;
    (y >= bottom && y <= top).then_some(near)
}
