//! Where a shot goes: spread rays from the aim direction, and what each ray hits first.
//! Port of GunHandler.AimDirection and the hit half of GunHandler.TryFire / ResolveHit.

use glam::Vec3;
use physics::PhysicsWorld;

use crate::damage::DamageInfo;
use crate::weapons::WeaponDef;
use crate::zombie::Noise;

/// Small deterministic random numbers (xorshift64). The same seed gives the same spread, so a shot
/// can be replayed in tests. It is not meant for anything that needs good statistics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GunRng {
    state: u64,
}

impl GunRng {
    pub fn new(seed: u64) -> Self {
        // xorshift never leaves the all-zero state, so a zero seed gets a fixed non-zero one.
        Self {
            state: if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// A number in [0, 1). Uses the top 24 bits, which an f32 holds exactly.
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / 16_777_216.0
    }
}

/// One straight shot: where it starts and its unit direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

/// A zombie (or other target) as the shot sees it. `position` is the feet. The body is a vertical
/// cylinder from the feet up to the bottom of the head sphere, so the head sits above the body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GunTarget {
    pub id: u64,
    pub position: Vec3,
    pub body_radius: f32,
    pub head_center: Vec3,
    pub head_radius: f32,
}

/// What one ray hit first. `target` is None when a wall or the ground stopped it; walls take no
/// damage, so `damage` is None for them.
#[derive(Clone, Debug, PartialEq)]
pub struct GunHit {
    pub target: Option<u64>,
    pub point: Vec3,
    pub distance: f32,
    pub damage: Option<DamageInfo>,
}

/// One ray per pellet, each tilted inside the spread cone. Aiming uses the tighter aim spread
/// (the same two values the C# uses; there is no extra tightening).
pub fn fire_rays(
    def: &WeaponDef,
    origin: Vec3,
    aim_dir: Vec3,
    aiming: bool,
    rng: &mut GunRng,
) -> Vec<Ray> {
    let spread = if aiming { def.spread_aim_deg } else { def.spread_hip_deg };
    // A shotgun has several pellets; every gun fires at least one ray.
    (0..def.pellets.max(1))
        .map(|_| Ray {
            origin,
            dir: spread_direction(aim_dir, spread, rng),
        })
        .collect()
}

/// A direction tilted by a random amount inside a cone of the given half-angle (degrees).
/// Picking a uniform point on the disc (sqrt of the random radius) keeps the spread round, not
/// bunched in the middle.
fn spread_direction(aim_dir: Vec3, spread_deg: f32, rng: &mut GunRng) -> Vec3 {
    let forward = aim_dir.normalize_or_zero();
    if spread_deg <= 0.0 || forward == Vec3::ZERO {
        return forward;
    }
    let (right, up) = perpendicular_basis(forward);
    let radius = rng.next_f32().sqrt() * spread_deg.to_radians();
    let angle = rng.next_f32() * std::f32::consts::TAU;
    let sideways = (right * angle.cos() + up * angle.sin()) * radius.sin();
    (forward * radius.cos() + sideways).normalize_or_zero()
}

/// Two unit vectors at right angles to `forward` and to each other.
fn perpendicular_basis(forward: Vec3) -> (Vec3, Vec3) {
    // Looking straight up or down, Y is parallel to the aim, so use X as the helper instead.
    let helper = if forward.y.abs() > 0.99 { Vec3::X } else { Vec3::Y };
    let right = forward.cross(helper).normalize();
    let up = right.cross(forward).normalize();
    (right, up)
}

/// The noise a shot makes. It is centred on the player's feet, like the C# (not on the camera).
pub fn shot_noise(def: &WeaponDef, player_position: Vec3) -> Noise {
    Noise {
        position: player_position,
        radius: def.noise_radius,
    }
}

/// For each ray, the first thing it touches within `max_range`: a target or the world. Targets
/// are checked by shape (no physics), since the physics query only sees the world.
/// Headshots deal `headshot_multiplier` times the damage.
pub fn resolve_hits(
    rays: &[Ray],
    def: &WeaponDef,
    targets: &[GunTarget],
    physics: &PhysicsWorld,
    max_range: f32,
) -> Vec<GunHit> {
    rays.iter()
        .filter_map(|ray| resolve_one(ray, def, targets, physics, max_range))
        .collect()
}

fn resolve_one(
    ray: &Ray,
    def: &WeaponDef,
    targets: &[GunTarget],
    physics: &PhysicsWorld,
    max_range: f32,
) -> Option<GunHit> {
    let dir = ray.dir.normalize_or_zero();
    if dir == Vec3::ZERO {
        return None;
    }

    let wall = physics.raycast(ray.origin, dir, max_range);

    // Nearest target within range, with whether the head was the part hit.
    let mut nearest: Option<(f32, u64, bool)> = None;
    for target in targets {
        if let Some((distance, head)) = hit_target(ray.origin, dir, target)
            && distance <= max_range
            && nearest.is_none_or(|(d, _, _)| distance < d)
        {
            nearest = Some((distance, target.id, head));
        }
    }

    match (nearest, wall) {
        // A wall at the same distance counts as the wall, so a target behind it never takes a hit.
        (Some((distance, id, head)), wall) if wall.is_none_or(|w| distance < w.distance) => {
            let point = ray.origin + dir * distance;
            let amount = if head {
                def.damage * def.headshot_multiplier
            } else {
                def.damage
            };
            Some(GunHit {
                target: Some(id),
                point,
                distance,
                damage: Some(DamageInfo {
                    amount,
                    source: None,
                    hit_point: point,
                    direction: dir,
                    damage_type: "bullet".to_string(),
                    headshot: head,
                }),
            })
        }
        (_, Some(w)) => Some(GunHit {
            target: None,
            point: w.point,
            distance: w.distance,
            damage: None,
        }),
        _ => None,
    }
}

/// Distance to the first part of `target` the ray touches, and whether that part is the head.
fn hit_target(origin: Vec3, dir: Vec3, target: &GunTarget) -> Option<(f32, bool)> {
    let head = ray_sphere(origin, dir, target.head_center, target.head_radius).map(|d| (d, true));
    let body = ray_body(origin, dir, target).map(|d| (d, false));
    match (head, body) {
        (Some(h), Some(b)) => Some(if h.0 <= b.0 { h } else { b }),
        (h, b) => h.or(b),
    }
}

/// Entry distance of a ray into a sphere. None if it misses, or starts inside (that does not count).
/// `dir` must be unit length.
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

/// Entry distance of a ray into the body: a vertical cylinder from the feet to the bottom of the
/// head sphere. Only the side is tested; the top is covered by the head sphere.
fn ray_body(origin: Vec3, dir: Vec3, target: &GunTarget) -> Option<f32> {
    let bottom = target.position.y;
    let top = target.head_center.y - target.head_radius;
    if top <= bottom {
        return None;
    }
    let ox = origin.x - target.position.x;
    let oz = origin.z - target.position.z;
    // In the XZ plane the cylinder is a circle; a ray flying straight up or down misses its side.
    let a = dir.x * dir.x + dir.z * dir.z;
    if a < 1e-8 {
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
