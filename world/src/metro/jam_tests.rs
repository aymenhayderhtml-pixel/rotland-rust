//! Tests for `jam.rs` (declared from there). Real `data/metro.json` values, a synthetic straight
//! highway into the first avenue at the city edge.

use std::f32::consts::{FRAC_PI_2, TAU};

use glam::Vec2;

use super::*;
use crate::metro::config::MetroConfig;
use crate::metro::highway::Highway;
use crate::metro::layout::MetroLayout;
use crate::metro::place::MetroSite;

/// Test tolerance on the median rule: a prop must sit at least this far from the centre line.
const MEDIAN_CLEARANCE_EXTRA_M: f32 = 0.9; // new: DECISIONS 2026-11-26 (test threshold, from the task)
const KNOWN_FIXED: [&str; 9] = [
    "bus_city",
    "bus_city_side",
    "army_truck_jam",
    "luggage_pile",
    "suitcase_open",
    "clothes_spill",
    "body_seated",
    "body_sheet",
    "highway_sign_gantry",
];

fn cfg() -> MetroConfig {
    MetroConfig::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json loads")
}

fn site(cfg: &MetroConfig) -> MetroSite {
    MetroSite {
        key: cfg.key.clone(),
        name: cfg.name.clone(),
        center: Vec2::new(1000.0, -2000.0),
        half_size: cfg.size_blocks as f32 * cfg.block_size * 0.5,
        yaw: 0.6,
        ground_y: 10.0,
    }
}

/// A straight 3 km highway from outside into the city's first avenue end (town order).
fn highway_for(layout: &MetroLayout, cfg: &MetroConfig) -> Highway {
    let avenue = layout
        .streets
        .iter()
        .find(|s| s.avenue && layout.contains((s.a + s.b) * 0.5))
        .expect("the layout has an avenue");
    let entrance = avenue.a;
    let out = (entrance - layout.site.center).normalize();
    let steps = 30;
    let points = (0..=steps)
        .map(|i| entrance + out * 3000.0 * (1.0 - i as f32 / steps as f32))
        .collect();
    Highway { from_town: "blackwater".into(), points, width: cfg.highway.highway_width }
}

fn setup() -> (MetroConfig, MetroLayout, Highway) {
    let cfg = cfg();
    let layout = MetroLayout::generate(site(&cfg), &cfg);
    let highway = highway_for(&layout, &cfg);
    (cfg, layout, highway)
}

/// Same seats for every car model, so bodies can be tested.
fn with_seats(mut cfg: MetroConfig) -> MetroConfig {
    let seats = vec![[0.3, 0.5, 0.0], [-0.3, 0.5, 0.4]];
    for m in cfg.jam.car_models.clone() {
        cfg.jam.seat_offsets.insert(m.id, seats.clone());
    }
    cfg
}

fn is_car(p: &JamProp) -> bool {
    p.model.starts_with("car_jam")
}

fn dist_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let seg = b - a;
    let t = ((p - a).dot(seg) / seg.length_squared()).clamp(0.0, 1.0);
    p.distance(a + seg * t)
}

/// Distance from `p` to the polyline, and its signed lateral offset (positive = right of
/// the polyline's direction, which runs city to town for the outbound track).
fn highway_offset(points: &[Vec2], p: Vec2) -> (f32, f32) {
    let mut best = (f32::MAX, 0.0);
    for w in points.windows(2) {
        let (a, b) = (w[0], w[1]);
        let seg = b - a;
        let t = ((p - a).dot(seg) / seg.length_squared()).clamp(0.0, 1.0);
        let c = a + seg * t;
        let d = p.distance(c);
        if d < best.0 {
            let dir = seg.normalize();
            best = (d, (p - c).dot(Vec2::new(-dir.y, dir.x)));
        }
    }
    best
}

fn outbound_points(h: &Highway) -> Vec<Vec2> {
    h.points.iter().rev().copied().collect()
}

fn xz(p: &JamProp) -> Vec2 {
    Vec2::new(p.pos.x, p.pos.z)
}

#[test]
fn same_site_gives_the_same_plan_and_another_site_a_different_one() {
    let (cfg, layout, highway) = setup();
    let a = plan_jam(&highway, &layout, &cfg);
    assert!(!a.is_empty());
    assert_eq!(a, plan_jam(&highway, &layout, &cfg));
    let mut moved = site(&cfg);
    moved.center.x += 64.0;
    let other = MetroLayout::generate(moved, &cfg);
    assert_ne!(a, plan_jam(&highway, &other, &cfg));
}

#[test]
fn degenerate_inputs_give_no_jam() {
    let (cfg, layout, mut highway) = setup();
    highway.points.truncate(1);
    assert!(plan_jam(&highway, &layout, &cfg).is_empty());
}

#[test]
fn no_highway_prop_sits_on_the_median() {
    let (cfg, layout, highway) = setup();
    let plan = plan_jam(&highway, &layout, &cfg);
    let limit = cfg.highway.median_width * 0.5 + MEDIAN_CLEARANCE_EXTRA_M;
    let pts = outbound_points(&highway);
    for p in plan.iter().filter(|p| p.model != super::vehicles::GANTRY_MODEL) {
        if avenue_dist(&layout, &highway, xz(p)) <= cfg.avenue_width * 0.5 {
            continue;
        }
        let (_, lateral) = highway_offset(&pts, xz(p));
        assert!(lateral.abs() >= limit, "{} at lateral {lateral} is on the median", p.model);
    }
}

fn avenue_dist(layout: &MetroLayout, highway: &Highway, p: Vec2) -> f32 {
    let entrance = highway.points[highway.points.len() - 1];
    pick_avenues(layout, entrance)
        .iter()
        .map(|&(a, b)| dist_to_segment(p, a, b))
        .fold(f32::MAX, f32::min)
}

#[test]
fn outbound_lanes_are_bumper_to_bumper_and_inbound_nearly_empty() {
    let (cfg, layout, highway) = setup();
    let plan = plan_jam(&highway, &layout, &cfg);
    let pts = outbound_points(&highway);
    let on_hwy: Vec<&JamProp> = plan
        .iter()
        .filter(|p| is_car(p) && avenue_dist(&layout, &highway, xz(p)) > cfg.avenue_width * 0.5)
        .collect();
    let outbound = on_hwy.iter().filter(|p| highway_offset(&pts, xz(p)).1 > 0.0).count() as f32;
    let inbound = on_hwy.len() as f32 - outbound;
    let mean_spacing = (cfg.jam.car_spacing_m[0] + cfg.jam.car_spacing_m[1]) * 0.5;
    let expected = 2.0 * cfg.jam.jam_length / mean_spacing;
    assert!(outbound >= 0.8 * expected && outbound <= 1.2 * expected, "{outbound} vs {expected}");
    assert!(inbound < outbound / 4.0, "inbound {inbound} should be sparse");
}

#[test]
fn buses_and_trucks_follow_the_counts_and_exactly_one_lies_on_its_side() {
    let (cfg, layout, highway) = setup();
    let plan = plan_jam(&highway, &layout, &cfg);
    let buses = plan.iter().filter(|p| p.model.starts_with("bus_city")).count();
    let sides = plan.iter().filter(|p| p.model == "bus_city_side").count();
    let trucks = plan.iter().filter(|p| p.model == "army_truck_jam").count();
    let [bus_lo, bus_hi] = cfg.jam.bus_count;
    let [truck_lo, truck_hi] = cfg.jam.army_truck_count;
    assert!((bus_lo..=bus_hi).contains(&buses), "buses {buses}");
    assert_eq!(sides, 1);
    assert!((truck_lo..=truck_hi).contains(&trucks), "trucks {trucks}");
    let side = plan.iter().find(|p| p.model == "bus_city_side").expect("one side bus");
    let [town_end, city_end] = [highway.points[0], highway.points[highway.points.len() - 1]];
    let outbound_dir = (town_end - city_end).normalize();
    let across = (dir_yaw(outbound_dir) + FRAC_PI_2).rem_euclid(TAU);
    let turn = (side.yaw.rem_euclid(TAU) - across).abs();
    assert!(turn < 1e-4 || (TAU - turn) < 1e-4, "side bus lies across the lanes");
}

#[test]
fn every_prop_is_on_the_highway_or_the_first_two_avenues() {
    let (cfg, layout, highway) = setup();
    let plan = plan_jam(&highway, &layout, &cfg);
    let pts = outbound_points(&highway);
    let (half, avenue_half) = (highway.width * 0.5, cfg.avenue_width * 0.5);
    for p in &plan {
        let (hwy, _) = highway_offset(&pts, xz(p));
        let ave = avenue_dist(&layout, &highway, xz(p));
        assert!(hwy <= half || ave <= avenue_half, "{} at {:?} is off the jam", p.model, p.pos);
    }
    let entrance = highway.points[highway.points.len() - 1];
    assert_eq!(pick_avenues(&layout, entrance).len(), AVENUE_COUNT);
}

#[test]
fn gantry_is_single_centred_and_flat() {
    let (cfg, layout, highway) = setup();
    let plan = plan_jam(&highway, &layout, &cfg);
    let gantries: Vec<&JamProp> = plan.iter().filter(|p| p.model == super::vehicles::GANTRY_MODEL).collect();
    assert_eq!(gantries.len(), 1);
    let (_, lateral) = highway_offset(&outbound_points(&highway), xz(gantries[0]));
    assert!(lateral.abs() < 0.01, "gantry is over the centre line");
    assert!(plan.iter().all(|p| p.pitch == 0.0 && p.roll == 0.0));
}

#[test]
fn luggage_clothes_and_wrecks_appear_in_a_dense_jam() {
    let (cfg, layout, highway) = setup();
    let plan = plan_jam(&highway, &layout, &cfg);
    let count = |m: &str| plan.iter().filter(|p| p.model == m).count();
    assert!(super::vehicles::LUGGAGE_MODELS.iter().map(|m| count(m)).sum::<usize>() > 0, "luggage");
    assert!(count("clothes_spill") > 0, "clothes");
    let cars = plan.iter().filter(|p| is_car(p)).count();
    let burnt = plan.iter().filter(|p| p.model.ends_with(super::vehicles::BURNT_SUFFIX)).count();
    assert!(burnt > 0 && burnt * 4 < cars, "burnt {burnt} of {cars} cars");
}


#[test]
fn no_seats_means_no_bodies() {
    let (mut cfg, layout, highway) = setup();
    cfg.jam.seat_offsets.clear();
    let plan = plan_jam(&highway, &layout, &cfg);
    assert_eq!(plan.iter().filter(|p| p.model == "body_seated").count(), 0);
}

#[test]
fn about_one_car_in_four_is_seated_with_a_body_inside_it() {
    let (_, layout, highway) = setup();
    let cfg = with_seats(cfg());
    let plan = plan_jam(&highway, &layout, &cfg);
    let cars: Vec<&JamProp> = plan.iter().filter(|p| is_car(p)).collect();
    let bodies: Vec<&JamProp> = plan.iter().filter(|p| p.model == "body_seated").collect();
    let ratio = bodies.len() as f32 / cars.len() as f32;
    assert!((0.17..=0.33).contains(&ratio), "body ratio {ratio}");
    for b in &bodies {
        let nearest = cars.iter().map(|c| xz(c).distance(xz(b))).fold(f32::MAX, f32::min);
        assert!(nearest < 2.5, "body {nearest} m from its car");
        assert!(b.pos.y > 0.0, "seat height kept");
    }
}

#[test]
fn model_ids_are_known_and_only_burnt_variants_carry_the_suffix() {
    let (cfg, layout, highway) = setup();
    let plan = plan_jam(&highway, &layout, &cfg);
    let base: Vec<&str> = cfg.jam.car_models.iter().map(|m| m.id.as_str()).collect();
    for p in &plan {
        let known = KNOWN_FIXED.contains(&p.model.as_str())
            || base.contains(&p.model.as_str())
            || p.model.strip_suffix(super::vehicles::BURNT_SUFFIX).is_some_and(|b| base.contains(&b));
        assert!(known, "unknown model {}", p.model);
    }
}

#[test]
fn pick_avenues_are_inside_the_square_and_nearest_first() {
    let (_, layout, highway) = setup();
    let entrance = highway.points[highway.points.len() - 1];
    let picked = pick_avenues(&layout, entrance);
    assert_eq!(picked.len(), AVENUE_COUNT);
    let d0 = picked[0].0.distance(entrance);
    let d1 = picked[1].0.distance(entrance);
    assert!(d0 <= d1);
    for (a, b) in picked {
        assert!(layout.contains((a + b) * 0.5));
    }
}
