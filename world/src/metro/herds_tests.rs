//! Tests for `herds.rs` (kept in a sibling file to stay under the size rule).

use glam::Vec2;

use crate::metro::config::MetroConfig;
use crate::metro::herds::{HerdsConfig, herd_routes, herd_routes_avoiding, route_pos};
use crate::metro::layout::{MetroDistrict, MetroLayout};
use crate::metro::place::MetroSite;

fn metro_cfg() -> MetroConfig {
    MetroConfig::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json loads")
}

fn test_layout() -> MetroLayout {
    let cfg = metro_cfg();
    let site = MetroSite {
        key: cfg.key.clone(),
        name: cfg.name.clone(),
        center: Vec2::new(1000.0, -2000.0),
        half_size: cfg.size_blocks as f32 * cfg.block_size * 0.5,
        yaw: 0.4,
        ground_y: 10.0,
    };
    MetroLayout::generate(site, &cfg)
}

#[test]
fn herd_routes_are_deterministic_and_vary_with_seed() {
    let layout = test_layout();
    let cfg = HerdsConfig::default();
    assert!(cfg.validate().is_ok());

    let a1 = herd_routes(&layout, &cfg, 12345);
    let a2 = herd_routes(&layout, &cfg, 12345);
    let b = herd_routes(&layout, &cfg, 67890);

    assert_eq!(a1, a2);
    assert_ne!(a1, b);
}

#[test]
fn herd_counts_per_district_match_config() {
    let layout = test_layout();
    let cfg = HerdsConfig::default();
    let routes = herd_routes(&layout, &cfg, 42);

    let edge = routes.iter().filter(|r| r.zone == MetroDistrict::Edge).count();
    let midtown = routes.iter().filter(|r| r.zone == MetroDistrict::Midtown).count();
    let core = routes.iter().filter(|r| r.zone == MetroDistrict::Core).count();

    assert_eq!(edge, cfg.count.edge);
    assert_eq!(midtown, cfg.count.midtown);
    assert_eq!(core, cfg.count.core);
    for r in &routes {
        assert!(r.size >= cfg.size[0] && r.size <= cfg.size[1]);
    }
}

#[test]
fn route_points_lie_on_streets_avoid_reserved_and_form_closed_loops() {
    let layout = test_layout();
    let cfg = HerdsConfig::default();
    let routes = herd_routes(&layout, &cfg, 99);
    assert!(!routes.is_empty());

    for r in &routes {
        assert!(r.points.len() >= 4, "route {} needs >= 4 corners", r.id);
        for i in 0..r.points.len() {
            let a = r.points[i];
            let b = r.points[(i + 1) % r.points.len()];
            assert!(layout.street_at(a).is_some(), "corner {a:?} not on a street");
            let mid = a.lerp(b, 0.5);
            assert!(layout.street_at(mid).is_some(), "leg midpoint {mid:?} not on a street");
            if let Some(block) = layout.block_at_point(mid) {
                assert!(block.reserved.is_none(), "route entered reserved block {:?}", block.reserved);
            }
        }
    }
}

#[test]
fn herd_routes_avoiding_skips_blocked_street_segments() {
    let layout = test_layout();
    let cfg = HerdsConfig::default();
    let baseline = herd_routes(&layout, &cfg, 7);
    let blocked_mid = baseline[0].points[0].lerp(baseline[0].points[1], 0.5);

    let rerouted = herd_routes_avoiding(&layout, &cfg, &[blocked_mid], 7);
    for r in &rerouted {
        for i in 0..r.points.len() {
            let a = r.points[i];
            let b = r.points[(i + 1) % r.points.len()];
            let seg_len = a.distance(b).max(1e-4);
            let t = ((blocked_mid - a).dot(b - a) / (seg_len * seg_len)).clamp(0.0, 1.0);
            let closest = a.lerp(b, t);
            assert!(
                closest.distance(blocked_mid) > 1.0,
                "route {} crossed blocked segment at {blocked_mid:?}",
                r.id
            );
        }
    }
}

#[test]
fn route_pos_advances_at_speed_and_yaw_matches_godot_forward() {
    let layout = test_layout();
    let cfg = HerdsConfig::default();
    let routes = herd_routes(&layout, &cfg, 55);
    let r = &routes[0];

    let dt = 2.0_f64;
    let (p0, yaw0) = route_pos(r, 0.0);
    let (p1, _yaw1) = route_pos(r, dt);

    let expected_step = r.speed * dt as f32;
    let moved = p0.distance(p1);
    assert!((moved - expected_step).abs() < 1e-3, "moved {moved} != {expected_step}");

    let fwd = Vec2::new(-yaw0.sin(), -yaw0.cos());
    let actual_dir = (p1 - p0).normalize();
    assert!(fwd.dot(actual_dir) > 0.999, "yaw {yaw0} forward {fwd:?} != {actual_dir:?}");
}
