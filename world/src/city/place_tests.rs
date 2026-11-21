use std::collections::HashMap;
use glam::{IVec2, Vec2};
use paths::repo_path;
use crate::city::config::District;
use crate::city::place::{is_ground_cover, is_street_prop};
use crate::generator::WorldGenerator;
use crate::poi::PoiConfig;
use crate::scatter::ScatterConfig;
use crate::settlement::SettlementConfig;

fn load_test_generator(seed: i32) -> WorldGenerator {
    WorldGenerator::from_data_dir(seed, repo_path("data")).expect("WorldGenerator::from_data_dir")
}

#[test]
fn city_lots_show_up_in_exactly_one_chunk() {
    let generator = load_test_generator(42);
    let city = generator.city().expect("city should be present");
    assert!(!city.lots.is_empty(), "city should have lots");

    let min_chunk_x = (city.origin.x / 64.0).floor() as i32;
    let min_chunk_y = (city.origin.y / 64.0).floor() as i32;
    let blocks = 10;

    let mut lot_occurrences: HashMap<String, usize> = HashMap::new();

    for cx in (min_chunk_x - 1)..=(min_chunk_x + blocks + 1) {
        for cy in (min_chunk_y - 1)..=(min_chunk_y + blocks + 1) {
            let (pois, _) = generator.chunk_contents(IVec2::new(cx, cy));
            for poi in pois {
                if poi.id.starts_with("city_") {
                    *lot_occurrences.entry(poi.id).or_insert(0) += 1;
                }
            }
        }
    }

    assert_eq!(
        lot_occurrences.len(),
        city.lots.len(),
        "all lots must be placed"
    );
    for idx in 0..city.lots.len() {
        let id = format!("city_{idx}");
        assert_eq!(
            lot_occurrences.get(&id),
            Some(&1),
            "lot {id} must show up in exactly one chunk"
        );
    }
}

#[test]
fn city_no_wild_poi_or_non_park_scatter_inside_city() {
    let generator = load_test_generator(42);
    let city = generator.city().expect("city present");

    let min_chunk_x = (city.origin.x / 64.0).floor() as i32;
    let min_chunk_y = (city.origin.y / 64.0).floor() as i32;
    let blocks = 10;

    for cx in (min_chunk_x - 1)..=(min_chunk_x + blocks + 1) {
        for cy in (min_chunk_y - 1)..=(min_chunk_y + blocks + 1) {
            let (pois, props) = generator.chunk_contents(IVec2::new(cx, cy));

            for poi in pois {
                let p = Vec2::new(poi.position.x, poi.position.z);
                if city.contains(p) {
                    assert!(
                        poi.id.starts_with("city_"),
                        "wild POI {} found inside city at ({}, {})",
                        poi.id, p.x, p.y
                    );
                }
            }

            for prop in props {
                let p = Vec2::new(prop.position.x, prop.position.z);
                if city.contains(p) && !is_street_prop(&prop.model) {
                    let district = city.district_at(p);
                    assert_eq!(
                        district,
                        Some(District::Park),
                        "non-street scatter prop {} inside city must be in Park block, found in {:?}",
                        prop.model, district
                    );
                    assert!(
                        prop.is_tree() || is_ground_cover(&prop.model),
                        "park scatter prop {} must be tree or ground cover",
                        prop.model
                    );
                    assert!(
                        city.street_at(p).is_none(),
                        "scatter prop {} sits on a street inside park",
                        prop.model
                    );
                }
            }
        }
    }
}

#[test]
fn city_blocked_segments_have_blockers() {
    let generator = load_test_generator(42);
    let city = generator.city().expect("city present");

    let blocked_segments: Vec<_> = city.streets.iter().filter(|s| s.blocked).collect();
    assert!(!blocked_segments.is_empty(), "must have some blocked segments");

    let blocker_models = ["barricade", "car_wreck_sedan", "rubble_pile", "jersey_barrier"];

    for seg in blocked_segments {
        let min_cx = (seg.a.x.min(seg.b.x) / 64.0).floor() as i32 - 1;
        let max_cx = (seg.a.x.max(seg.b.x) / 64.0).floor() as i32 + 1;
        let min_cy = (seg.a.y.min(seg.b.y) / 64.0).floor() as i32 - 1;
        let max_cy = (seg.a.y.max(seg.b.y) / 64.0).floor() as i32 + 1;

        let mut found_blocker = false;
        for cx in min_cx..=max_cx {
            for cy in min_cy..=max_cy {
                let (_, props) = generator.chunk_contents(IVec2::new(cx, cy));
                for prop in props {
                    if blocker_models.contains(&prop.model.as_str()) {
                        let p = Vec2::new(prop.position.x, prop.position.z);
                        if seg.contains_point(p) {
                            found_blocker = true;
                            break;
                        }
                    }
                }
                if found_blocker {
                    break;
                }
            }
            if found_blocker {
                break;
            }
        }
        assert!(
            found_blocker,
            "blocked segment from {:?} to {:?} must have at least one blocker within its width",
            seg.a, seg.b
        );
    }
}

#[test]
fn city_no_prop_within_3m() {
    let generator = load_test_generator(42);
    let city = generator.city().expect("city present");

    let min_chunk_x = (city.origin.x / 64.0).floor() as i32;
    let min_chunk_y = (city.origin.y / 64.0).floor() as i32;
    let blocks = 10;

    for cx in (min_chunk_x - 1)..=(min_chunk_x + blocks + 1) {
        for cy in (min_chunk_y - 1)..=(min_chunk_y + blocks + 1) {
            let (_, props) = generator.chunk_contents(IVec2::new(cx, cy));
            let street_props: Vec<_> = props.iter().filter(|p| is_street_prop(&p.model)).collect();
            for sp in &street_props {
                for other in &props {
                    if std::ptr::eq(*sp, other) {
                        continue;
                    }
                    let d = sp.position.distance(other.position);
                    assert!(
                        d >= 3.0 - 1e-4,
                        "chunk ({cx}, {cy}): street prop {} at {:?} and prop {} at {:?} are closer than 3m: d = {d}",
                        sp.model, sp.position, other.model, other.position
                    );
                }
            }
        }
    }
}

#[test]
fn city_camp_chunk_matches_without_city() {
    let generator_city = load_test_generator(42);
    let camp = generator_city.settlements()[0].center;
    let camp_coord = IVec2::new((camp.x / 64.0).floor() as i32, (camp.z / 64.0).floor() as i32);

    let (pois_city, props_city) = generator_city.chunk_contents(camp_coord);

    let settlements = SettlementConfig::load_from_file(repo_path("data/settlements.json")).unwrap();
    let scatter = ScatterConfig::load_from_file(repo_path("data/scatter.json")).unwrap();
    let poi = PoiConfig::load_from_file(repo_path("data/poi.json")).unwrap();
    let generator_no_city = WorldGenerator::with_configs(42, settlements, scatter, poi);

    let (pois_base, props_base) = generator_no_city.chunk_contents(camp_coord);

    assert_eq!(pois_city, pois_base, "camp chunk POIs must match");

    // props match except any road wrecks that might be along the road
    let filtered_props_city: Vec<_> = props_city
        .into_iter()
        .filter(|p| !(p.model == "car_wreck" && !props_base.contains(p)))
        .collect();
    assert_eq!(filtered_props_city, props_base, "camp chunk props must match");
}

#[test]
fn city_deterministic_same_seed() {
    let generator1 = load_test_generator(42);
    let generator2 = load_test_generator(42);

    let city = generator1.city().expect("city present");
    let min_chunk_x = (city.origin.x / 64.0).floor() as i32;
    let min_chunk_y = (city.origin.y / 64.0).floor() as i32;

    let test_coords = [
        IVec2::new(min_chunk_x, min_chunk_y),
        IVec2::new(min_chunk_x + 5, min_chunk_y + 5),
        IVec2::new(min_chunk_x + 9, min_chunk_y + 9),
        IVec2::new(1, -1), // camp area
        IVec2::new(10, 10),
    ];

    for coord in test_coords {
        let (pois1, props1) = generator1.chunk_contents(coord);
        let (pois2, props2) = generator2.chunk_contents(coord);
        assert_eq!(pois1, pois2, "POIs must match for chunk {:?}", coord);
        assert_eq!(props1, props2, "props must match for chunk {:?}", coord);
    }
}
