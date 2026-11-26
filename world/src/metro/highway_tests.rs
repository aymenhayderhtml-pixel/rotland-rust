//! Tests for the New Halden highway plan and its height field (`highway.rs`, `highway_height.rs`).

use glam::Vec2;

use super::*;
use crate::biome::layout::WorldLayoutData;
use crate::metro::place::place_metro;
use crate::rivers::RiversData;
use crate::towns::TownsData;
use crate::towns::place::rotate_xz_ccw_from_above;

// new: DECISIONS 2026-11-26 (test tolerances, metres)
const TOL_M: f32 = 0.01;
// new: DECISIONS 2026-11-26 (test tolerance for the start point's distance to its street line)
const STREET_LINE_TOL_M: f32 = 0.01;
// new: DECISIONS 2026-11-26 (R38-03 spec: no segment longer than 1.5 point spacings)
const MAX_SEGMENT_FACTOR: f32 = 1.5;
// new: DECISIONS 2026-11-26 (R38-03 spec: no turn sharper than 20 degrees)
const MAX_TURN_DEG: f32 = 20.0;
// new: DECISIONS 2026-11-26 (one traffic lane is 3.5 m wide)
const LANE_WIDTH_M: f32 = 3.5;
// new: DECISIONS 2026-11-26 (test: a probe this far past the highway edge is off the road)
const OFF_ROAD_PROBE_M: f32 = 5.0;
// new: DECISIONS 2026-11-26 (test: synthetic chord half-length across the highway)
const CHORD_HALF_LEN_M: f32 = 500.0;
// new: DECISIONS 2026-11-26 (test: a chord through the middle must hit within this many metres)
const CHORD_HIT_TOL_M: f32 = 1.0;
// new: DECISIONS 2026-11-26 (test: base height used for the flat and dipped terrain)
const BASE_HEIGHT_M: f32 = 50.0;
// new: DECISIONS 2026-11-26 (test: height of the dip under the highway middle)
const DIP_HEIGHT_M: f32 = 30.0;
// new: DECISIONS 2026-11-26 (test: radius of the dip around the highway middle)
const DIP_RADIUS_M: f32 = 100.0;
// new: DECISIONS 2026-11-26 (test: city ground height used for the blend check)
const CITY_GROUND_M: f32 = 20.0;
// new: DECISIONS 2026-11-26 (test: index offset of a point about 300 m from the middle)
const FAR_POINT_OFFSET: usize = 15;
// new: DECISIONS 2026-11-26 (test: fixed seed for the real site)
const SEED: u64 = 1234;

fn load_cfg() -> MetroConfig {
    MetroConfig::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json loads")
}

fn load_roads() -> RoadsData {
    RoadsData::load().expect("data/roads.json loads")
}

/// Every embedded town at its ideal (sector, distance) spot, with its street facing camp.
fn real_towns() -> Vec<TownSite> {
    let towns = TownsData::load_embedded();
    let layout = WorldLayoutData::load().expect("data/world_layout.json loads");
    towns
        .towns
        .iter()
        .map(|def| {
            let biome = def.biome_id().expect("town biome is known");
            let deg = layout
                .sectors
                .iter()
                .find(|s| s.biome == biome)
                .expect("town biome has a sector")
                .center_deg;
            let center = rotate_xz_ccw_from_above(Vec2::X, deg) * def.distance_m;
            TownSite {
                key: def.key.clone(),
                name: def.name.clone(),
                biome,
                center,
                radius: def.radius,
                street_dir: (-center).normalize(),
                lots: Vec::new(),
            }
        })
        .collect()
}

fn synthetic_town(key: &str, center: Vec2, toward: Vec2) -> TownSite {
    TownSite {
        key: key.to_string(),
        name: key.to_string(),
        biome: crate::biome::BiomeId::Swamp,
        center,
        radius: 130.0,
        street_dir: (toward - center).normalize(),
        lots: Vec::new(),
    }
}

fn real_plan() -> (MetroConfig, MetroSite, Highway) {
    let cfg = load_cfg();
    let site = place_metro(SEED, Vec2::ZERO, &cfg);
    let hw = plan_highway(&site, &real_towns(), &load_roads(), &cfg);
    (cfg, site, hw)
}

/// Unit normal to the highway at point index `i`.
fn normal_at(points: &[Vec2], i: usize) -> Vec2 {
    let dir = (points[i + 1] - points[i]).normalize();
    Vec2::new(-dir.y, dir.x)
}

/// Intersection of segments `a`..`b` and `c`..`d`, if they cross.
fn seg_hit(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> Option<Vec2> {
    let r = b - a;
    let s = d - c;
    let denom = r.perp_dot(s);
    if denom.abs() < 1e-9 {
        return None;
    }
    let t = (c - a).perp_dot(s) / denom;
    let u = (c - a).perp_dot(r) / denom;
    ((0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)).then(|| a + r * t)
}

fn polyline_hits(points: &[Vec2], a: Vec2, b: Vec2) -> Vec<Vec2> {
    points
        .windows(2)
        .filter_map(|w| seg_hit(w[0], w[1], a, b))
        .collect()
}

#[test]
fn starts_at_the_nearer_swamp_or_snow_town_on_its_main_street() {
    let (cfg, site, hw) = real_plan();
    let towns = real_towns();
    let nearer = towns
        .iter()
        .filter(|t| t.key == "blackwater" || t.key == "frostmere")
        .min_by(|a, b| {
            a.center
                .distance_squared(site.center)
                .total_cmp(&b.center.distance_squared(site.center))
        })
        .expect("blackwater or frostmere exists");
    assert_eq!(hw.from_town, nearer.key);

    let town = towns.iter().find(|t| t.key == hw.from_town).expect("town listed");
    let start = hw.points[0];
    let off = start - town.center;
    let reach = town.radius + cfg.highway.town_setback_m;
    assert!((off.length() - reach).abs() < 1.0, "start {:.2} m from centre", off.length());
    assert!(off.perp_dot(town.street_dir).abs() < STREET_LINE_TOL_M, "start off the main street");
    assert!(off.dot(site.center - town.center) > 0.0, "start must face the city");
}

#[test]
fn ends_on_the_city_edge_at_an_avenue_line() {
    let (cfg, site, hw) = real_plan();
    let local = site.to_local(*hw.points.last().expect("route has points"));
    let half = site.half_size;
    let edge_coord = local.x.abs().max(local.y.abs());
    assert!((edge_coord - half).abs() < TOL_M, "end not on the square edge: {local:?}");

    // The edge the route ends on is the one with the larger |coordinate|; the other is along it.
    let along = if local.x.abs() > local.y.abs() { local.y } else { local.x };
    let k = ((along + half) / cfg.block_size).round();
    let avenue_line = -half + k * cfg.block_size;
    assert!((along - avenue_line).abs() < 0.5, "end {along} m is not on an avenue line");
    assert_eq!(k as i32 % cfg.avenue_every, 0, "avenue index {k} is not an avenue");
    assert!(k >= 1.0 && k < cfg.size_blocks as f32, "entrance must not be a corner");

    let step = cfg.block_size * cfg.avenue_every as f32;
    let multiple = (along / step).round() * step;
    assert!((along - multiple).abs() < 0.5, "{along} is not a multiple of {step}");
}

#[test]
fn route_is_evenly_spaced_gently_turning_and_inside_the_corridor() {
    let (_cfg, _site, hw) = real_plan();
    let roads = load_roads();
    let max_seg = roads.point_spacing * MAX_SEGMENT_FACTOR;
    for w in hw.points.windows(2) {
        assert!(w[0].distance(w[1]) <= max_seg + TOL_M, "segment {:.2} m", w[0].distance(w[1]));
    }
    for w in hw.points.windows(3) {
        let turn = (w[1] - w[0]).angle_to(w[2] - w[1]).abs().to_degrees();
        assert!(turn <= MAX_TURN_DEG, "turn {turn:.2} deg");
    }
    let first = hw.points[0];
    let last = *hw.points.last().expect("route has points");
    let line = last - first;
    let len_sq = line.length_squared();
    for p in &hw.points {
        let t = ((*p - first).dot(line) / len_sq).clamp(0.0, 1.0);
        let off = p.distance(first + line * t);
        assert!(off <= roads.wiggle_m + TOL_M, "point {off:.2} m off the corridor");
    }
}

#[test]
fn plan_is_deterministic_and_four_lane_width_with_median() {
    let (cfg, site, hw) = real_plan();
    let again = plan_highway(&site, &real_towns(), &load_roads(), &cfg);
    assert_eq!(hw, again);
    assert_eq!(hw.width, cfg.highway.highway_width);
    assert_eq!(cfg.highway.lanes, 4);
    let need = cfg.highway.lanes as f32 * LANE_WIDTH_M + cfg.highway.median_width;
    assert!(hw.width >= need, "width {} below {need}", hw.width);
}

#[test]
fn synthetic_towns_pick_the_nearest_candidate() {
    let cfg = load_cfg();
    let site = place_metro(SEED, Vec2::ZERO, &cfg);
    let roads = load_roads();
    let near_city = site.center + Vec2::X * 300.0;
    let far_blackwater = site.center + Vec2::Y * 3000.0;
    let far_frostmere = site.center - Vec2::Y * 2000.0;

    // Dusthollow is nearer than both candidates, but only Blackwater and Frostmere may be used.
    let towns = vec![
        synthetic_town("dusthollow", near_city, site.center),
        synthetic_town("blackwater", far_blackwater, site.center),
        synthetic_town("frostmere", far_frostmere, site.center),
    ];
    assert_eq!(plan_highway(&site, &towns, &roads, &cfg).from_town, "frostmere");

    let only_blackwater = vec![
        synthetic_town("dusthollow", near_city, site.center),
        synthetic_town("blackwater", far_blackwater, site.center),
    ];
    assert_eq!(plan_highway(&site, &only_blackwater, &roads, &cfg).from_town, "blackwater");

    // Without Blackwater or Frostmere, the nearest town overall is used.
    let no_candidates = vec![
        synthetic_town("ridgeback_station", far_blackwater, site.center),
        synthetic_town("dusthollow", near_city, site.center),
    ];
    assert_eq!(plan_highway(&site, &no_candidates, &roads, &cfg).from_town, "dusthollow");
}

#[test]
fn height_field_follows_the_base_and_ends_at_the_city_ground() {
    let (cfg, _site, hw) = real_plan();
    let flat = |_x: f32, _z: f32| BASE_HEIGHT_M;
    let field = HighwayHeightField::build(&hw, CITY_GROUND_M, &flat, &cfg);
    let mid = hw.points.len() / 2;
    let on_road = hw.points[mid];
    assert!((field.height_at(on_road).expect("on road") - BASE_HEIGHT_M).abs() < TOL_M);
    let last = *hw.points.last().expect("route has points");
    assert!((field.height_at(last).expect("at city") - CITY_GROUND_M).abs() < TOL_M);

    let n = normal_at(&hw.points, mid);
    let off = on_road + n * (hw.width * 0.5 + cfg.highway.shoulder_width + OFF_ROAD_PROBE_M);
    assert_eq!(field.height_at(off), None);
    assert_eq!(highway_height_at(&field, off), None);
    assert_eq!(highway_height_at(&field, on_road), field.height_at(on_road));
}

#[test]
fn height_field_dips_only_where_the_base_dips() {
    let (cfg, _site, hw) = real_plan();
    let mid = hw.points.len() / 2;
    let centre = hw.points[mid];
    let dip = move |x: f32, z: f32| {
        if Vec2::new(x, z).distance(centre) < DIP_RADIUS_M {
            DIP_HEIGHT_M
        } else {
            BASE_HEIGHT_M
        }
    };
    let field = HighwayHeightField::build(&hw, CITY_GROUND_M, &dip, &cfg);
    assert!((field.height_at(centre).expect("on road") - DIP_HEIGHT_M).abs() < TOL_M);
    let far = hw.points[mid + FAR_POINT_OFFSET];
    assert!((field.height_at(far).expect("on road") - BASE_HEIGHT_M).abs() < TOL_M);
}

#[test]
fn flatten_is_road_height_on_the_road_and_eases_back_to_raw() {
    let (cfg, _site, hw) = real_plan();
    let flat = |_x: f32, _z: f32| BASE_HEIGHT_M;
    let field = HighwayHeightField::build(&hw, CITY_GROUND_M, &flat, &cfg);
    let raw = 100.0_f32;
    let mid = hw.points.len() / 2;
    let on_road = hw.points[mid];
    assert!((field.flatten(raw, on_road) - BASE_HEIGHT_M).abs() < TOL_M);

    let n = normal_at(&hw.points, mid);
    let edge = hw.width * 0.5 + cfg.highway.flatten_edge_m * 0.5;
    let blended = field.flatten(raw, on_road + n * edge);
    assert!(blended > BASE_HEIGHT_M + TOL_M && blended < raw - TOL_M, "blend {blended}");

    let far = on_road + n * (hw.width * 0.5 + cfg.highway.flatten_edge_m + OFF_ROAD_PROBE_M);
    assert_eq!(field.flatten(raw, far), raw);
}

#[test]
fn real_route_river_crossings_are_reported_and_synthetic_chords_are_detected() {
    let (_cfg, _site, hw) = real_plan();
    let rivers = RiversData::load().expect("data/rivers.json loads");
    let mut real_hits = Vec::new();
    for r in &rivers.rivers {
        // Same chord math as `place.rs`: camp at the origin, reference direction +X.
        let start = rotate_xz_ccw_from_above(Vec2::X, r.start_angle_deg) * r.start_radius_m;
        let end = rotate_xz_ccw_from_above(Vec2::X, r.target_angle_deg) * r.target_radius_m;
        for p in polyline_hits(&hw.points, start, end) {
            real_hits.push((r.name.clone(), p));
        }
    }
    for (name, p) in &real_hits {
        eprintln!("[highway] river '{name}' crosses the route at ({:.1}, {:.1})", p.x, p.y);
    }
    eprintln!("[highway] real route crossings (chord test): {}", real_hits.len());

    let mid = hw.points.len() / 2;
    let on_road = hw.points[mid];
    let n = normal_at(&hw.points, mid);
    let hits = polyline_hits(&hw.points, on_road - n * CHORD_HALF_LEN_M, on_road + n * CHORD_HALF_LEN_M);
    assert!(!hits.is_empty(), "a chord across the highway must be detected");
    assert!(hits.iter().any(|p| p.distance(on_road) < CHORD_HIT_TOL_M));
}
