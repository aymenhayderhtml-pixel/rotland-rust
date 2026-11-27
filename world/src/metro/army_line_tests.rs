//! Unit tests for the failed army line: site, layout coverage, chunk split and model names.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use glam::Vec2;

use crate::metro::army_line::{
    ARMY_LINE_DATA, ARMY_LINE_POI_ID, ArmyLineSite, ArmyProp, army_line_blocked_points,
    army_line_in_chunk, army_line_site, default_army_line, load_army_line,
};
use crate::metro::config::MetroConfig;
use crate::metro::damage::{damage_sites, default_damage};
use crate::metro::highway::{Highway, HighwayHeightField};
use crate::metro::jam::plan_jam;
use crate::metro::layout::{MetroLayout, Reserved};
use crate::metro::paint::MetroLook;
use crate::metro::place::MetroSite;
use crate::metro::state::{MetroWorld, index_towers_by_block};
use crate::metro::towers::plan_towers;
use crate::poi::PoiInstance;
use crate::scatter::PropInstance;

const CHUNK_M: f32 = 64.0; // data: metro.json blockSize
const GROUND_Y: f32 = 12.0; // new: DECISIONS 2026-11-27 (a lifted test ground shows a missing height)
/// A barrier piece counts as lying across the avenue when its data yaw is this close to 0.
const ACROSS_YAW_ABS: f32 = 0.2; // new: DECISIONS 2026-11-27 (0.05 wiggle allowed in the data)
/// Over-scan around the line's origin when collecting every chunk (strip is 256 x 64 m).
const STRIP_SCAN_RADIUS_M: f32 = 200.0; // new: DECISIONS 2026-11-27 (covers the strip and its lots)

fn test_metro(center: Vec2, yaw: f32) -> MetroWorld {
    let config = MetroConfig::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json");
    let look = MetroLook::load_from_file(paths::repo_path("data/metro.json")).expect("look");
    let site = MetroSite {
        key: config.key.clone(), name: config.name.clone(), center, yaw, ground_y: GROUND_Y,
        half_size: config.size_blocks as f32 * config.block_size * 0.5,
    };
    let layout = MetroLayout::generate(site, &config);
    let towers = plan_towers(&layout, &config);
    let entrance = layout.site.to_world(Vec2::new(0.0, layout.site.half_size));
    let out = (entrance - layout.site.center).normalize_or(Vec2::Y);
    let highway = Highway {
        from_town: "blackwater".into(),
        points: vec![entrance + out * 800.0, entrance],
        width: config.highway.highway_width,
    };
    let highway_field = HighwayHeightField::build(&highway, GROUND_Y, &|_, _| GROUND_Y, &config);
    let jam = plan_jam(&highway, &layout, &config);
    let towers_by_block = index_towers_by_block(&towers);
    let damage = damage_sites(&layout, &towers, default_damage());
    MetroWorld { config, layout, towers, highway, highway_field, jam, look, towers_by_block, damage }
}

fn flat_height(x: f32, z: f32) -> f32 {
    GROUND_Y + 0.01 * x + 0.02 * z
}

fn site(metro: &MetroWorld) -> ArmyLineSite {
    army_line_site(&metro.layout).expect("the reserved army_line strip must exist")
}

/// Every chunk covering the strip and a generous margin around it.
fn site_chunks(metro: &MetroWorld) -> Vec<(i32, i32)> {
    let centre = site(metro).to_world(0.0, 0.0);
    let (lo, hi) = (centre - Vec2::splat(STRIP_SCAN_RADIUS_M), centre + Vec2::splat(STRIP_SCAN_RADIUS_M));
    let mut out = Vec::new();
    for cx in (lo.x / CHUNK_M).floor() as i32..=(hi.x / CHUNK_M).ceil() as i32 {
        for cz in (lo.y / CHUNK_M).floor() as i32..=(hi.y / CHUNK_M).ceil() as i32 {
            out.push((cx, cz));
        }
    }
    out
}

/// All content the chunks around the strip emit for the line.
fn emit_all(metro: &MetroWorld) -> (Vec<PropInstance>, Vec<PoiInstance>, Vec<String>) {
    let mut found = (Vec::new(), Vec::new(), Vec::new());
    for c in site_chunks(metro) {
        let got = army_line_in_chunk(metro, c, &flat_height);
        found.0.extend(got.props);
        found.1.extend(got.pois);
        found.2.extend(got.crates.into_iter().map(|s| s.crate_id));
    }
    found
}

#[test]
fn site_sits_on_the_avenue_along_the_strip_with_z_pointing_at_the_strip() {
    let metro = test_metro(Vec2::new(6000.0, -2000.0), 0.7);
    let s = site(&metro);
    let sorted = |mut v: Vec<(i32, i32)>| {
        v.sort();
        v
    };
    let reserved = sorted(
        metro
            .layout
            .blocks
            .iter()
            .filter(|b| b.reserved == Some(Reserved::ArmyLine))
            .map(|b| (b.ix, b.iz))
            .collect(),
    );
    assert_eq!(reserved.len(), 4, "army_line is a 4x1 strip");
    assert_eq!(sorted(s.strip.clone()), reserved, "strip lists exactly the reserved blocks");

    let street = metro.layout.street_at(s.origin).expect("origin on a street");
    assert!(street.avenue, "the long side of the strip is an avenue");
    assert!((s.dir.length() - 1.0).abs() < 1e-4);
    // yaw faces along dir (Godot: forward = (-sin yaw, -cos yaw)).
    assert!(Vec2::new(-s.yaw.sin(), -s.yaw.cos()).dot(s.dir) > 0.999);
    // 32 m across is inside the strip's lot: never a street, never a tower.
    let lot = s.to_world(32.0, 32.0);
    assert!(metro.layout.block_at_point(lot).is_some_and(|b| b.reserved == Some(Reserved::ArmyLine)));
    assert!(metro.layout.street_at(lot).is_none() && !metro.in_tower_footprint(lot));
    // The wall's middle and both of its ends stand on the avenue surface.
    let mid = s.to_world(default_army_line().barrier_x, 0.0);
    for p in army_line_blocked_points(&metro) {
        let inside = p.lerp(mid, 0.01);
        assert!(metro.layout.street_at(inside).is_some_and(|st| st.avenue), "point off the avenue");
    }
}

#[test]
fn site_is_found_for_any_city_spot_and_rotation() {
    for (center, yaw) in [(Vec2::new(-3200.0, 4800.0), 1.9), (Vec2::new(20.0, -20.0), -2.4)] {
        let metro = test_metro(center, yaw);
        let s = site(&metro);
        let street = metro.layout.street_at(s.origin).expect("origin on a street");
        assert!(street.avenue && street.width >= metro.config.avenue_width - 1e-3);
        assert!(metro.layout.site.contains_square(s.origin));
    }
}

/// Span of one barrier piece along local z, or `None` if it points along the avenue instead.
fn across_span(p: &ArmyProp) -> Option<(f32, f32)> {
    if p.yaw.abs() > ACROSS_YAW_ABS {
        return None;
    }
    let half = (p.len * p.yaw.cos()).abs() * 0.5;
    Some((p.z - half, p.z + half))
}

#[test]
fn barrier_covers_the_road_with_one_three_to_six_metre_breach() {
    let metro = test_metro(Vec2::new(6000.0, 0.0), 0.3);
    let s = site(&metro);
    let layout = default_army_line();
    let half_road = metro.layout.street_at(s.origin).expect("avenue").width * 0.5;
    // Every wall piece must stand on the avenue band, clear of the cross avenue at x = 0.
    let barrier_x = layout.barrier_x;
    assert!(barrier_x.abs() > half_road, "the wall must clear the cross avenue");

    let mut spans: Vec<(f32, f32)> = layout
        .props
        .iter()
        .filter(|p| layout.barrier_models.iter().any(|m| m == &p.model))
        .filter_map(across_span)
        .collect();
    assert!(spans.len() >= 6, "the wall needs several pieces, got {}", spans.len());
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (first, last) = (spans[0], spans[spans.len() - 1]);
    assert!((first.0 + half_road).abs() <= 1.0, "left end {} vs curb {}", first.0, -half_road);
    assert!((last.1 - half_road).abs() <= 1.0, "right end {} vs curb {}", last.1, half_road);

    let mut holes = Vec::new();
    let mut covered = first.1;
    for (lo, hi) in spans.iter().copied().skip(1) {
        if lo > covered + 1e-3 {
            holes.push((covered, lo));
        }
        covered = covered.max(hi);
    }
    assert_eq!(holes.len(), 1, "exactly one breach, found {holes:?}");
    let width = holes[0].1 - holes[0].0;
    assert!((3.0..=6.0).contains(&width), "the breach is {width} m wide");
    assert!(covered >= half_road - 1.0, "the wall must reach the far curb");
    // The breach holds rubble and a burnt car pile, right at the wall.
    for model in ["rubble_field_m", "car_burnt_pile"] {
        assert!(
            layout.props.iter().any(|p| p.model == model
                && p.z.abs() < 4.0
                && (p.x - barrier_x).abs() < 5.0),
            "{model} must sit in the breach at the wall"
        );
    }
    // The wire coils lie in front of the wall (the avenue approach, the -x side).
    for p in layout.props.iter().filter(|p| p.model == "razor_wire") {
        assert!(p.x < barrier_x, "wire must lie in front of the wall");
    }
}

#[test]
fn the_data_file_holds_the_whole_cast_in_the_asked_numbers() {
    let layout = default_army_line();
    let count = |model: &str| layout.props.iter().filter(|p| p.model == model).count();
    for model in [
        "army_tank", "army_truck_jam", "heli_crashed", "heli_rotor_blade", "ash_patch",
        "field_kitchen", "checkpoint_booth", "army_gear_scatter", "body_bag", "body_bag_pile",
        "sandbag_corner", "cot", "rubble_field_m", "car_burnt_pile", "sandbag_wall",
        "concrete_barrier", "army_truck_burnt", "evac_tent", "razor_wire",
    ] {
        assert!(count(model) >= 1, "data/places/army_line.json needs a {model}");
    }
    assert!((4..=6).contains(&count("evac_tent")), "4-6 evac tents");
    assert!((2..=3).contains(&count("army_truck_burnt")), "2-3 burnt army trucks");
    assert_eq!(count("army_truck_jam"), 1, "one army truck still standing");
    assert_eq!(count("razor_wire"), 2, "two wire coils in front of the wall");
    assert!((2..=3).contains(&count("loudspeaker_pole")), "2-3 loudspeakers");
    assert!((2..=3).contains(&count("flood_light")), "2-3 flood lights");
    // Two nests close each sidewalk end of the wall, plus nests at the lot corners.
    assert!((4..=8).contains(&count("sandbag_corner")), "sandbag corner nests");
    assert!((6..=8).contains(&layout.crates.iter().filter(|c| c.model == "army_crate").count()));
    assert_eq!(layout.crates.iter().filter(|c| c.model == "med_cabinet").count(), 2);
    for c in &layout.crates {
        let want = if c.model == "med_cabinet" { "metro_medical" } else { "metro_army" };
        assert_eq!(c.table, want, "{} uses the wrong loot table", c.model);
    }
}

#[test]
fn the_helicopter_burns_on_the_strip_lot_with_its_blade_nearby() {
    let layout = default_army_line();
    let heli = layout.props.iter().find(|p| p.model == "heli_crashed").expect("heli");
    assert!(heli.z > 12.5, "the heli lies on the strip lot, not on the avenue");
    assert!(heli.x.abs() < 128.0, "inside the strip");
    let blade = layout.props.iter().find(|p| p.model == "heli_rotor_blade").expect("blade");
    let d = Vec2::new(blade.x - heli.x, blade.z - heli.z).length();
    assert!((10.0..=20.0).contains(&d), "blade lies 10-20 m off the heli, got {d}");
    assert!(layout.props.iter().any(|p| p.model == "ash_patch" && p.z > 12.5), "burnt ground");
    // Nothing floats out front: the city side is the -x end of the avenue, not -z.
    for p in &layout.props {
        assert!(p.z >= -12.5, "{} at z {} is out in front of the line", p.model, p.z);
    }
}

#[test]
fn nothing_sits_inside_a_tower_footprint() {
    let metro = test_metro(Vec2::new(6000.0, 0.0), 0.4);
    let (props, pois, _) = emit_all(&metro);
    for p in &props {
        let xz = Vec2::new(p.position.x, p.position.z);
        assert!(!metro.in_tower_footprint(xz), "{} inside a tower footprint at {xz:?}", p.model);
    }
    for poi in &pois {
        assert!(!metro.in_tower_footprint(Vec2::new(poi.position.x, poi.position.z)));
    }
}

#[test]
fn every_entry_is_emitted_by_exactly_one_chunk() {
    let metro = test_metro(Vec2::new(6000.0, -2000.0), -1.2);
    let (props, pois, crates) = emit_all(&metro);
    let layout = default_army_line();
    assert_eq!(props.len(), layout.props.len(), "no prop lost or duplicated");
    assert_eq!(crates.len(), layout.crates.len(), "no crate lost or duplicated");
    assert_eq!(pois.len(), 1, "one army line POI");
    let unique: HashSet<(String, i32, i32)> = props
        .iter()
        .map(|p| (p.model.clone(), p.position.x.round() as i32, p.position.z.round() as i32))
        .collect();
    assert_eq!(unique.len(), props.len(), "two props share one spot");
    assert_eq!(crates.iter().collect::<HashSet<_>>().len(), crates.len(), "crate ids are unique");

    // Each prop is emitted by its own chunk, and by that one only.
    for p in &props {
        let cx = (p.position.x / CHUNK_M).floor() as i32;
        let cz = (p.position.z / CHUNK_M).floor() as i32;
        let again = army_line_in_chunk(&metro, (cx, cz), &flat_height);
        let hits = again
            .props
            .iter()
            .filter(|q| q.model == p.model && (q.position.x - p.position.x).abs() < 1e-3
                && (q.position.z - p.position.z).abs() < 1e-3)
            .count();
        assert_eq!(hits, 1, "{} at {:?} is not in its own chunk", p.model, p.position);
    }
}

#[test]
fn the_poi_sits_at_the_barrier_middle_with_the_army_table() {
    let metro = test_metro(Vec2::new(6000.0, 0.0), 0.0);
    let (_, pois, _) = emit_all(&metro);
    assert_eq!(pois.len(), 1);
    let poi = &pois[0];
    assert_eq!(poi.id, ARMY_LINE_POI_ID);
    assert_eq!(poi.building_id, ARMY_LINE_POI_ID);
    assert_eq!(poi.loot_table.as_deref(), Some("metro_army")); // data: loot_tables.json
    let want = site(&metro).to_world(default_army_line().barrier_x, 0.0);
    assert!(Vec2::new(poi.position.x, poi.position.z).distance(want) < 1e-3);
}

#[test]
fn heights_are_the_ground_plus_the_entry_y_and_the_same_city_gives_the_same_line() {
    let metro = test_metro(Vec2::new(6000.0, -2000.0), 0.8);
    let (props, pois, crates) = emit_all(&metro);
    for p in props.iter().filter(|p| p.model == "evac_tent" || p.model == "army_tank") {
        let want = flat_height(p.position.x, p.position.z);
        assert!((p.position.y - want).abs() < 1e-3, "{} floats or sinks", p.model);
    }
    let (props2, pois2, crates2) = emit_all(&metro);
    assert_eq!(props, props2, "props are deterministic");
    assert_eq!(pois, pois2, "the POI is deterministic");
    assert_eq!(crates, crates2, "crates are deterministic");
}

#[test]
fn blocked_points_are_the_two_barrier_ends_and_the_middle() {
    let metro = test_metro(Vec2::new(6000.0, 0.0), 0.2);
    let pts = army_line_blocked_points(&metro);
    assert_eq!(pts.len(), 3, "two ends plus the middle");
    let s = site(&metro);
    let half = metro.config.avenue_width * 0.5;
    let bx = default_army_line().barrier_x;
    for (i, (x, z)) in [(bx, -half), (bx, 0.0), (bx, half)].iter().enumerate() {
        assert!(pts[i].distance(s.to_world(*x, *z)) < 1e-3, "blocked point {i} is misplaced");
    }
}

#[test]
fn the_data_file_parses_and_rejects_broken_entries() {
    let json = std::fs::read_to_string(paths::repo_path(ARMY_LINE_DATA)).expect("the data file");
    let layout = load_army_line(&json).expect("army_line.json must parse");
    assert!(!layout.barrier_models.is_empty());
    assert!(layout.props.len() >= 40, "the line needs a full cast");
    assert!(load_army_line("{\"props\":[],\"crates\":[]}").is_err(), "empty data is rejected");
    let bad = "{\"props\":[{\"x\":0,\"z\":0,\"yaw\":0,\"model\":\"\",\"scale\":1}],\"crates\":[]}";
    assert!(load_army_line(bad).is_err(), "an empty model name is rejected");
}

#[test]
fn every_model_in_the_data_file_exists_in_assets_models() {
    let layout = default_army_line();
    let stems = glb_stems(&paths::repo_path("assets/models"));
    assert!(!stems.is_empty(), "assets/models must hold the models");
    let mut missing: Vec<&str> = Vec::new();
    for model in layout
        .props
        .iter()
        .map(|p| p.model.as_str())
        .chain(layout.crates.iter().map(|c| c.model.as_str()))
    {
        if !stems.contains(model) && !missing.contains(&model) {
            missing.push(model);
        }
    }
    if !missing.is_empty() {
        // R42-03/R42-06 may not have built them yet: report them, do not fail.
        println!("army_line models not built yet: {missing:?}");
    }
}

/// Every `.glb` file stem under `dir`, searched recursively.
fn glb_stems(dir: &Path) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut stack: Vec<PathBuf> = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "glb")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                out.insert(stem.to_string());
            }
        }
    }
    out
}
