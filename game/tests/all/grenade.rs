use glam::Vec3;

use game::grenade::{
    compute_arc_points, compute_explosion_damage, compute_explosion_hits, step_grenade,
    ActiveGrenade, GrenadeConfig,
};

#[test]
fn grenade_flight_and_fuse_expiry() {
    let cfg = GrenadeConfig::default();
    let mut g = ActiveGrenade {
        position: Vec3::new(0.0, 2.0, 0.0),
        velocity: Vec3::new(cfg.throw_speed, 0.0, 0.0),
        fuse_remaining: cfg.fuse_sec,
        bounced: false,
    };

    // Step in open air (no hit)
    let exploded = step_grenade(&mut g, 0.5, cfg.gravity, cfg.bounce_restitution, |_, _, _| None);
    assert!(!exploded);
    assert!((g.fuse_remaining - 2.0).abs() < 1e-5);
    assert!(g.position.x > 0.0);
    assert!(g.velocity.y < 0.0);

    // Step past remaining fuse
    let exploded = step_grenade(&mut g, 2.1, cfg.gravity, cfg.bounce_restitution, |_, _, _| None);
    assert!(exploded);
}

#[test]
fn grenade_bounces_once_at_30_percent_then_settles() {
    let cfg = GrenadeConfig::default();
    let mut g = ActiveGrenade {
        position: Vec3::new(0.0, 0.2, 0.0),
        velocity: Vec3::new(0.0, -10.0, 0.0),
        fuse_remaining: cfg.fuse_sec,
        bounced: false,
    };

    // First impact with flat ground (normal +Y)
    let exploded = step_grenade(&mut g, 0.1, cfg.gravity, cfg.bounce_restitution, |origin, dir, dist| {
        if dir.y < 0.0 && origin.y + dir.y * dist <= 0.0 {
            Some((Vec3::new(origin.x, 0.0, origin.z), Vec3::Y))
        } else {
            None
        }
    });
    assert!(!exploded);
    assert!(g.bounced);
    // Speed before impact was ~10.98 m/s downward; reflected upward at 30%
    let expected_up = (10.0 + cfg.gravity * 0.1) * 0.3;
    assert!(
        (g.velocity.y - expected_up).abs() < 1e-4,
        "expected upward speed ~{}, got {}",
        expected_up,
        g.velocity.y
    );

    // Second impact: should settle to zero velocity
    g.position = Vec3::new(0.0, 0.1, 0.0);
    g.velocity = Vec3::new(1.0, -5.0, 0.0);
    let _ = step_grenade(&mut g, 0.1, cfg.gravity, cfg.bounce_restitution, |origin, dir, dist| {
        if dir.y < 0.0 && origin.y + dir.y * dist <= 0.0 {
            Some((Vec3::new(origin.x, 0.0, origin.z), Vec3::Y))
        } else {
            None
        }
    });
    assert_eq!(g.velocity, Vec3::ZERO);
}

#[test]
fn grenade_explosion_damage_falloff() {
    let cfg = GrenadeConfig::default();
    let center = compute_explosion_damage(0.0, cfg.blast_radius, cfg.max_damage, cfg.min_edge_frac);
    let mid = compute_explosion_damage(3.0, cfg.blast_radius, cfg.max_damage, cfg.min_edge_frac);
    let edge = compute_explosion_damage(6.0, cfg.blast_radius, cfg.max_damage, cfg.min_edge_frac);
    let outside = compute_explosion_damage(6.01, cfg.blast_radius, cfg.max_damage, cfg.min_edge_frac);

    assert!((center - 150.0).abs() < 1e-4);
    assert!((mid - 90.0).abs() < 1e-4);
    assert!((edge - 30.0).abs() < 1e-4);
    assert_eq!(outside, 0.0);
}

#[test]
fn grenade_explosion_hits_and_wall_los_blocking() {
    let cfg = GrenadeConfig::default();
    let blast_pos = Vec3::ZERO;
    let targets = [
        (1_u64, Vec3::new(0.0, 0.0, 0.0)), // center
        (2_u64, Vec3::new(3.0, 0.0, 0.0)), // unblocked mid-range
        (3_u64, Vec3::new(-2.0, 0.0, 0.0)), // blocked by wall
        (4_u64, Vec3::new(8.0, 0.0, 0.0)), // out of range
    ];

    let hits = compute_explosion_hits(blast_pos, &cfg, &targets, |_from, to| to.x < 0.0);
    assert_eq!(hits.len(), 2);

    assert_eq!(hits[0].target_id, 1);
    assert!((hits[0].damage - 150.0).abs() < 1e-4);
    assert!((hits[0].knockback.length() - 8.0).abs() < 1e-4);

    assert_eq!(hits[1].target_id, 2);
    assert!((hits[1].damage - 90.0).abs() < 1e-4);
    assert!((hits[1].knockback.x - 4.0).abs() < 1e-4);
}

#[test]
fn grenade_arc_points_parabolic() {
    let origin = Vec3::new(0.0, 1.5, 0.0);
    let vel = Vec3::new(10.0, 5.0, 0.0);
    let points = compute_arc_points(origin, vel, 9.8, 10, 0.1);

    assert_eq!(points.len(), 10);
    assert_eq!(points[0], origin);
    // At t = 0.5 s (index 5): x = 5.0, y = 1.5 + 2.5 - 0.5 * 9.8 * 0.25 = 2.775
    assert!((points[5].x - 5.0).abs() < 1e-4);
    assert!((points[5].y - 2.775).abs() < 1e-4);
}
