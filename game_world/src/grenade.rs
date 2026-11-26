//! Pure grenade physics, arc preview, and blast hit calculation.

use glam::Vec3;

/// Tunable parameters for a thrown grenade.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrenadeConfig {
    pub throw_speed: f32,
    pub gravity: f32,
    pub bounce_restitution: f32,
    pub fuse_sec: f32,
    pub blast_radius: f32,
    pub max_damage: f32,
    pub min_edge_frac: f32,
    pub knockback: f32,
    pub noise_radius: f32,
}

impl Default for GrenadeConfig {
    fn default() -> Self {
        Self {
            // data: weapons.json grenade.throwSpeed
            throw_speed: 14.0,
            // data: weapons.json grenade.gravity
            gravity: 9.8,
            // data: weapons.json grenade.bounceRestitution
            bounce_restitution: 0.3,
            // data: weapons.json grenade.fuse
            fuse_sec: 2.5,
            // data: weapons.json grenade.blastRadius
            blast_radius: 6.0,
            // data: weapons.json grenade.damage
            max_damage: 150.0,
            // data: weapons.json grenade.minEdgeFrac
            min_edge_frac: 0.2,
            // data: weapons.json grenade.knockback
            knockback: 8.0,
            // data: weapons.json grenade.noiseRadius
            noise_radius: 80.0,
        }
    }
}

/// Live state of a thrown grenade in flight or resting on the ground.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActiveGrenade {
    pub position: Vec3,
    pub velocity: Vec3,
    pub fuse_remaining: f32,
    pub bounced: bool,
}

/// One target affected by a grenade detonation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrenadeHit {
    pub target_id: u64,
    pub damage: f32,
    pub knockback: Vec3,
}

/// Advances a grenade by `dt` seconds.
///
/// `raycast(origin, dir, max_dist)` returns `Some((hit_pos, hit_normal))` on surface impact.
/// On first impact, reflects velocity keeping `bounce_restitution` of speed; on subsequent impact,
/// settles at zero velocity. Returns `true` when `fuse_remaining <= 0.0`.
pub fn step_grenade(
    g: &mut ActiveGrenade,
    dt: f32,
    gravity: f32,
    bounce_restitution: f32,
    raycast: impl Fn(Vec3, Vec3, f32) -> Option<(Vec3, Vec3)>,
) -> bool {
    g.fuse_remaining -= dt;

    if g.velocity.length_squared() > 0.0 {
        g.velocity.y -= gravity * dt;
        let disp = g.velocity * dt;
        let dist = disp.length();
        // new: DECISIONS 2026-11-25
        if dist > 1e-6 {
            let dir = disp / dist;
            if let Some((hit_pos, hit_normal)) = raycast(g.position, dir, dist) {
                let n = hit_normal.normalize_or_zero();
                // Offset slightly along the surface normal so the next step does not re-hit.
                // new: DECISIONS 2026-11-25
                g.position = hit_pos + n * 0.01;
                if !g.bounced {
                    g.bounced = true;
                    let reflected = if n.length_squared() > 0.0 {
                        // new: DECISIONS 2026-11-25
                        g.velocity - 2.0 * g.velocity.dot(n) * n
                    } else {
                        -g.velocity
                    };
                    g.velocity = reflected * bounce_restitution;
                } else {
                    g.velocity = Vec3::ZERO;
                }
            } else {
                g.position += disp;
            }
        }
    }

    g.fuse_remaining <= 0.0
}

/// Samples `steps` points along a parabolic throw arc for UI preview.
pub fn compute_arc_points(
    origin: Vec3,
    vel: Vec3,
    gravity: f32,
    steps: usize,
    step_dt: f32,
) -> Vec<Vec3> {
    let mut points = Vec::with_capacity(steps);
    for i in 0..steps {
        let t = i as f32 * step_dt;
        // new: DECISIONS 2026-11-25
        let drop = 0.5 * gravity * t * t;
        points.push(origin + vel * t - Vec3::Y * drop);
    }
    points
}

/// Linear damage falloff from `max_damage` at center (`dist == 0`) down to
/// `max_damage * min_edge_frac` at `dist == radius`, and `0.0` outside `radius`.
pub fn compute_explosion_damage(
    dist: f32,
    radius: f32,
    max_damage: f32,
    min_edge_frac: f32,
) -> f32 {
    if radius <= 0.0 || dist > radius {
        return 0.0;
    }
    let t = (dist / radius).clamp(0.0, 1.0);
    // new: DECISIONS 2026-11-25
    let frac = 1.0 - t * (1.0 - min_edge_frac);
    max_damage * frac
}

/// Computes all targets damaged and knocked back by a detonation at `blast_pos`.
/// Targets whose line of sight to `blast_pos` is blocked (`is_blocked(blast_pos, pos) == true`)
/// receive no damage or knockback.
pub fn compute_explosion_hits(
    blast_pos: Vec3,
    cfg: &GrenadeConfig,
    targets: &[(u64, Vec3)],
    is_blocked: impl Fn(Vec3, Vec3) -> bool,
) -> Vec<GrenadeHit> {
    let mut hits = Vec::new();
    for &(target_id, target_pos) in targets {
        let offset = target_pos - blast_pos;
        let dist = offset.length();
        if dist > cfg.blast_radius {
            continue;
        }
        if is_blocked(blast_pos, target_pos) {
            continue;
        }
        let damage = compute_explosion_damage(
            dist,
            cfg.blast_radius,
            cfg.max_damage,
            cfg.min_edge_frac,
        );
        if damage <= 0.0 {
            continue;
        }
        let kb_scale = 1.0 - (dist / cfg.blast_radius).clamp(0.0, 1.0);
        // new: DECISIONS 2026-11-25
        let dir = if dist > 1e-4 { offset / dist } else { Vec3::Y };
        hits.push(GrenadeHit {
            target_id,
            damage,
            knockback: dir * (cfg.knockback * kb_scale),
        });
    }
    hits
}
