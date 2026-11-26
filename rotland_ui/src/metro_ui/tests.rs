//! Unit tests for New Halden's haze, big-map discovery and minimap streets.

use std::f32::consts::FRAC_PI_4;

use glam::{Vec2, Vec3};
use world::metro::{MetroConfig, MetroLayout, MetroSite};

use super::haze::{MetroHazeConfig, blend_fog, city_signed_distance, haze_weight, skyline_fog};
use super::minimap::minimap_streets;
use super::outline::{discovered, site_corners, streets_on_map};

fn site_at(center: Vec2, yaw: f32) -> MetroSite {
    let cfg = metro_cfg();
    MetroSite {
        key: "new_halden".to_string(),
        name: "New Halden".to_string(),
        center,
        half_size: cfg.size_blocks as f32 * cfg.block_size * 0.5,
        yaw,
        ground_y: 0.0,
    }
}

fn metro_cfg() -> MetroConfig {
    MetroConfig::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json loads")
}

fn layout() -> MetroLayout {
    MetroLayout::generate(site_at(Vec2::new(500.0, -300.0), 0.0), &metro_cfg())
}

fn luma(c: Vec3) -> f32 {
    c.x * 0.2126 + c.y * 0.7152 + c.z * 0.0722
}

#[test]
fn haze_is_full_inside_and_on_the_edge() {
    assert_eq!(haze_weight(-500.0, 250.0), 1.0);
    assert_eq!(haze_weight(0.0, 250.0), 1.0);
}

#[test]
fn haze_is_zero_at_and_beyond_blend_distance() {
    assert_eq!(haze_weight(250.0, 250.0), 0.0);
    assert_eq!(haze_weight(900.0, 250.0), 0.0);
}

#[test]
fn haze_falls_off_smoothly_and_monotonically() {
    let near = haze_weight(50.0, 250.0);
    let mid = haze_weight(125.0, 250.0);
    let far = haze_weight(200.0, 250.0);
    assert!(near > mid && mid > far, "{near} {mid} {far}");
    assert!((mid - 0.5).abs() < 1e-4, "midpoint {mid}");
    assert!(near < 1.0 && far > 0.0);
}

#[test]
fn zero_blend_only_covers_the_inside() {
    assert_eq!(haze_weight(1.0, 0.0), 0.0);
    assert_eq!(haze_weight(-1.0, 0.0), 1.0);
}

#[test]
fn signed_distance_follows_unrotated_and_rotated_squares() {
    let flat = site_at(Vec2::new(100.0, 100.0), 0.0);
    let h = flat.half_size;
    assert!((city_signed_distance(Vec2::new(100.0 + h + 25.0, 100.0), &flat) - 25.0).abs() < 1e-2);
    assert!(city_signed_distance(Vec2::new(100.0, 100.0), &flat) < 0.0);

    let turned = site_at(Vec2::new(100.0, 100.0), FRAC_PI_4);
    let along_axis = turned.to_world(Vec2::new(0.0, turned.half_size + 25.0));
    assert!((city_signed_distance(along_axis, &turned) - 25.0).abs() < 1e-2);
    let corner_inside = turned.to_world(Vec2::new(turned.half_size - 1.0, 0.0));
    assert!(city_signed_distance(corner_inside, &turned) < 0.0);
    assert_eq!(site_corners(&turned).len(), 4);
}

#[test]
fn night_haze_never_brightens_dark_fog() {
    let cfg = MetroHazeConfig::default();
    let night = Vec3::new(0.04, 0.05, 0.08);
    let (color, begin, end) = blend_fog(night, 25.0, 45.0, &cfg, 1.0);
    assert!(
        luma(color) <= luma(night) + 1e-5,
        "night fog brightened to {color:?}"
    );
    assert!(begin > 0.0 && end > begin);
    let (sky_color, _, _) = skyline_fog(&cfg, night);
    assert!(luma(sky_color) <= luma(night) + 1e-5);
}

#[test]
fn blend_at_zero_weight_keeps_the_base_fog_and_at_one_takes_haze() {
    let cfg = MetroHazeConfig::default();
    let base = Vec3::new(0.8, 0.8, 0.8);
    assert_eq!(
        blend_fog(base, 120.0, 190.0, &cfg, 0.0),
        (base, 120.0, 190.0)
    );
    let (color, begin, end) = blend_fog(base, 120.0, 190.0, &cfg, 1.0);
    assert!((begin - cfg.fog_start).abs() < 1e-4);
    assert!((end - cfg.fog_end).abs() < 1e-4);
    assert!(color.distance(cfg.color) < 1e-4, "{color:?}");
}

#[test]
fn skyline_fog_uses_skyline_distances() {
    let cfg = MetroHazeConfig::default();
    let (_, begin, end) = skyline_fog(&cfg, Vec3::ONE);
    assert_eq!((begin, end), (cfg.skyline_fog_start, cfg.skyline_fog_end));
}

#[test]
fn haze_config_parses_from_json_text() {
    let json = r#"{"metro_haze":{"color":[10,20,30],"fog_start":60,"fog_end":180,
        "blend_m":200,"skyline_fog_start":2500,"skyline_fog_end":5500}}"#;
    let cfg = MetroHazeConfig::load(json);
    assert!(cfg.color.distance(Vec3::new(10.0, 20.0, 30.0) / 255.0) < 1e-6);
    assert_eq!(cfg.fog_start, 60.0);
    assert_eq!(cfg.fog_end, 180.0);
    assert_eq!(cfg.blend_m, 200.0);
    assert_eq!(cfg.skyline_fog_start, 2500.0);
    assert_eq!(cfg.skyline_fog_end, 5500.0);
}

#[test]
fn missing_or_broken_haze_keys_keep_defaults() {
    let defaults = MetroHazeConfig::default();
    assert_eq!(MetroHazeConfig::load("{}"), defaults);
    assert_eq!(MetroHazeConfig::load("not json at all"), defaults);
    assert_eq!(
        MetroHazeConfig::load(r#"{"metro_haze":{"color":[1,2]}}"#),
        defaults
    );
}

#[test]
fn repo_sky_json_matches_the_defaults() {
    assert_eq!(MetroHazeConfig::load_default(), MetroHazeConfig::default());
}

#[test]
fn city_is_undiscovered_until_a_sample_point_is_revealed() {
    let site = site_at(Vec2::new(500.0, -300.0), 0.0);
    let mut explore = game::MapExplore::new();
    assert!(!discovered(&site, &explore));
    explore.reveal(site.center, 10.0);
    assert!(discovered(&site, &explore));
}

#[test]
fn only_streets_with_a_revealed_midpoint_are_listed() {
    let layout = layout();
    let mut explore = game::MapExplore::new();
    assert!(streets_on_map(&layout, &explore).is_empty());
    explore.reveal(layout.site.center, 40.0);
    let shown = streets_on_map(&layout, &explore);
    assert!(!shown.is_empty());
    assert!(shown.len() < layout.streets.len());
}

#[test]
fn minimap_drops_a_street_100_m_away_and_keeps_one_under_the_player() {
    let layout = layout();
    let site = &layout.site;
    let far_player = site.to_world(Vec2::new(-site.half_size - 100.0, 0.0));
    assert!(minimap_streets(&layout, far_player, 60.0, 86.0, 2.0).is_empty());

    let near = minimap_streets(&layout, site.center, 60.0, 86.0, 2.0);
    assert!(!near.is_empty());
    assert!(near.iter().all(|s| s.width_px > 0.0));
}

#[test]
fn minimap_avenues_are_wider_than_streets() {
    let layout = layout();
    let streets = minimap_streets(&layout, layout.site.center, 200.0, 86.0, 2.0);
    let avenue = streets
        .iter()
        .find(|s| s.avenue)
        .expect("an avenue in reach");
    let street = streets
        .iter()
        .find(|s| !s.avenue)
        .expect("a street in reach");
    assert!(
        avenue.width_px > street.width_px,
        "{avenue:?} vs {street:?}"
    );
}
