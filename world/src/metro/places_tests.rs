//! Tests for `metro/places.rs` and `metro/places_data.rs` (kept in a sibling file to stay small).

use std::collections::{HashMap, HashSet};
use std::path::Path;

use glam::Vec2;

use crate::metro::config::MetroConfig;
use crate::metro::damage::{damage_sites, default_damage};
use crate::metro::highway::{Highway, HighwayHeightField};
use crate::metro::jam::plan_jam;
use crate::metro::layout::{MetroLayout, Reserved};
use crate::metro::paint::MetroLook;
use crate::metro::place::MetroSite;
use crate::metro::places::{
    PLACE_KINDS, PlaceChunk, PlaceSite, lot_role, place_info, place_sites, places_in_chunk,
    places_in_chunk_with,
};
use crate::metro::places_data::{PlaceData, place_data};
use crate::metro::state::{MetroWorld, index_towers_by_block};
use crate::metro::towers::plan_towers;

/// How far outside the 2x2 group a place entry may still land (the forecourt into the street).
const GROUP_MARGIN_M: f32 = 6.0; // new: DECISIONS 2026-11-27

/// A small fixture place file: entries in the main lot and in every yard.
const FIXTURE: &str = r#"{
  "model": "metro_KIND",
  "footprint": [40, 32],
  "floors": 5,
  "crates": [
    { "x": -6.0, "y": 0.0, "z": 2.0, "yaw": 0.0, "model": "med_cabinet", "table": "TABLE" },
    { "x": 6.0, "y": 1.5, "z": -2.0, "yaw": 1.0, "model": "metro_locker" }
  ],
  "props": [ { "x": 0.0, "y": 0.0, "z": 6.0, "yaw": 0.5, "model": "gurney" } ],
  "yards": {
    "a": [ { "x": 4.0, "y": 0.0, "z": 4.0, "yaw": 0.0, "model": "ambulance_wreck" } ],
    "b": [ { "x": -8.0, "y": 0.0, "z": 8.0, "yaw": 3.0, "model": "triage_tent" } ],
    "c": [ { "x": 12.0, "y": 0.0, "z": -12.0, "yaw": 4.0, "model": "body_bag" } ]
  },
  "yardCrates": {
    "a": [ { "x": -4.0, "y": 0.0, "z": -4.0, "yaw": 0.0, "model": "army_crate" } ],
    "b": [],
    "c": [ { "x": 0.0, "y": 0.0, "z": 0.0, "yaw": 0.0, "model": "army_crate" } ]
  }
}"#;

/// Entries per place in `FIXTURE`: 1 POI, 4 crates (2 main lot + 2 yards), 4 props (1 + 3).
const PER_PLACE: usize = 9; // new: DECISIONS 2026-11-27

fn fixture(kind: &str, table: &str) -> PlaceData {
    let json = FIXTURE.replace("KIND", kind).replace("TABLE", table);
    PlaceData::from_json(&json).expect("fixture place data must load")
}

fn fixtures() -> HashMap<Reserved, PlaceData> {
    PLACE_KINDS
        .iter()
        .map(|&(kind, key, table)| (kind, fixture(key, table)))
        .collect()
}

fn test_metro() -> MetroWorld {
    let cfg = MetroConfig::load_from_file(paths::repo_path("data/metro.json"))
        .expect("data/metro.json must load");
    let site = MetroSite {
        key: cfg.key.clone(),
        name: cfg.name.clone(),
        center: Vec2::new(6000.0, 0.0),
        yaw: 0.3,
        half_size: (cfg.size_blocks as f32 * cfg.block_size) * 0.5,
        ground_y: 0.0,
    };
    let layout = MetroLayout::generate(site.clone(), &cfg);
    let towers = plan_towers(&layout, &cfg);
    let entrance = site.to_world(Vec2::new(0.0, site.half_size));
    let out = (entrance - site.center).normalize();
    let highway = Highway {
        from_town: "blackwater".into(),
        points: vec![entrance + out * 500.0, entrance],
        width: cfg.highway.highway_width,
    };
    let highway_field = HighwayHeightField::build(&highway, 0.0, &|_x, _z| 0.0, &cfg);
    let jam = plan_jam(&highway, &layout, &cfg);
    let towers_by_block = index_towers_by_block(&towers);
    let damage = damage_sites(&layout, &towers, default_damage());
    let look = MetroLook::load_from_file(paths::repo_path("data/metro.json")).expect("look");
    MetroWorld { config: cfg, layout, towers, highway, highway_field, jam, look, towers_by_block, damage }
}

/// Site-local bounds of the place's 2x2 group, grown by `margin` metres.
fn group_bounds(metro: &MetroWorld, site: &PlaceSite, margin: f32) -> (Vec2, Vec2) {
    let mut lo = Vec2::splat(f32::MAX);
    let mut hi = Vec2::splat(f32::MIN);
    for lot in std::iter::once(site.main_lot).chain(site.yard_lots) {
        let block = metro.layout.block_at(lot.0, lot.1).expect("place lot exists");
        lo = lo.min(block.rect.0);
        hi = hi.max(block.rect.1);
    }
    (lo - Vec2::splat(margin), hi + Vec2::splat(margin))
}

/// Every chunk that can hold a point of the place's group, grown by `GROUP_MARGIN_M`.
fn group_chunks(metro: &MetroWorld, site: &PlaceSite) -> Vec<(i32, i32)> {
    let (lo, hi) = group_bounds(metro, site, GROUP_MARGIN_M);
    let corners = [Vec2::new(lo.x, lo.y), Vec2::new(hi.x, lo.y), Vec2::new(hi.x, hi.y), Vec2::new(lo.x, hi.y)];
    let mut min = Vec2::splat(f32::MAX);
    let mut max = Vec2::splat(f32::MIN);
    for c in corners.map(|p| metro.layout.site.to_world(p)) {
        min = min.min(c);
        max = max.max(c);
    }
    let mut out = Vec::new();
    for cx in ((min.x / 64.0).floor() as i32)..=((max.x / 64.0).floor() as i32) {
        for cz in ((min.y / 64.0).floor() as i32)..=((max.y / 64.0).floor() as i32) {
            out.push((cx, cz));
        }
    }
    out
}

/// Every chunk that can hold a point of any place (two groups can share a chunk cell).
fn all_chunks(metro: &MetroWorld) -> Vec<(i32, i32)> {
    let mut chunks: Vec<(i32, i32)> =
        place_sites(&metro.layout).iter().flat_map(|s| group_chunks(metro, s)).collect();
    chunks.sort_unstable();
    chunks.dedup();
    chunks
}

fn collect(metro: &MetroWorld, data: &HashMap<Reserved, PlaceData>) -> Vec<PlaceChunk> {
    let lookup = |kind: Reserved| data.get(&kind);
    all_chunks(metro).into_iter().map(|c| places_in_chunk_with(metro, c, &|_, _| 0.0, &lookup)).collect()
}

fn fold(chunks: Vec<PlaceChunk>) -> PlaceChunk {
    chunks.into_iter().fold(PlaceChunk::default(), |mut acc, out| {
        acc.pois.extend(out.pois);
        acc.props.extend(out.props);
        acc.crates.extend(out.crates);
        acc
    })
}

#[test]
fn three_sites_exist_one_per_kind_and_the_main_lot_faces_an_avenue() {
    let metro = test_metro();
    let sites = place_sites(&metro.layout);
    assert_eq!(sites.len(), PLACE_KINDS.len(), "one site per place kind");
    for (kind, _, _) in PLACE_KINDS {
        assert!(sites.iter().any(|s| s.kind == kind), "missing site {kind:?}");
    }
    for site in &sites {
        let block = metro.layout.block_at(site.main_lot.0, site.main_lot.1).expect("main lot");
        assert_eq!(block.reserved, Some(site.kind), "main lot belongs to the place");
        let centre = metro.layout.site.to_world((block.rect.0 + block.rect.1) * 0.5);
        let front = site.front();
        assert!((front.length() - 1.0).abs() < 1e-4, "front is a unit vector");
        let street = metro.layout.street_at(centre + front * 32.0).expect("front hits a street");
        assert!(street.avenue, "{:?} front is not on an avenue", site.kind);
        assert!(site.pos.distance(centre) < 2.5, "{:?} pos is not by the lot centre", site.kind);
        assert!((centre - site.pos).dot(front) > 0.0, "{:?} pos is not set back", site.kind);
    }
}

#[test]
fn yards_are_the_other_three_lots_of_the_group() {
    let metro = test_metro();
    for site in place_sites(&metro.layout) {
        let mut all = vec![site.main_lot];
        all.extend(site.yard_lots);
        assert_eq!(all.iter().copied().collect::<HashSet<_>>().len(), 4, "lots repeat");
        for lot in &all {
            assert_eq!(
                metro.layout.block_at(lot.0, lot.1).map(|b| b.reserved),
                Some(Some(site.kind)),
                "{:?} lot {lot:?} outside the group",
                site.kind
            );
        }
        let [a, b, _c] = site.yard_lots;
        assert!(a.0 == site.main_lot.0 || a.1 == site.main_lot.1, "a is not beside the main lot");
        assert!(
            b.0 != site.main_lot.0 && b.1 != site.main_lot.1,
            "b is not diagonal to the main lot"
        );
    }
}

#[test]
fn every_entry_lands_inside_its_group_and_on_the_terrain() {
    let metro = test_metro();
    let data = fixtures();
    for site in place_sites(&metro.layout) {
        let lookup = |kind: Reserved| if kind == site.kind { data.get(&kind) } else { None };
        let (lo, hi) = group_bounds(&metro, &site, GROUP_MARGIN_M);
        for chunk in group_chunks(&metro, &site) {
            let out = places_in_chunk_with(&metro, chunk, &|_, _| 12.0, &lookup);
            let points = out
                .pois
                .iter()
                .map(|p| p.position)
                .chain(out.props.iter().map(|p| p.position))
                .chain(out.crates.iter().map(|c| c.position));
            for p in points {
                assert!(p.y >= 12.0 - 1e-4, "entry below the ground: {p:?}");
                let local = metro.layout.site.to_local(Vec2::new(p.x, p.z));
                assert!(
                    (lo.x..=hi.x).contains(&local.x) && (lo.y..=hi.y).contains(&local.y),
                    "{:?} entry at local {local:?} outside the group",
                    site.kind
                );
            }
        }
    }
}

#[test]
fn every_entry_is_emitted_by_exactly_one_chunk() {
    let metro = test_metro();
    let mut seen: HashMap<(String, i32, i32), usize> = HashMap::new();
    for out in collect(&metro, &fixtures()) {
        for item in out
            .pois
            .iter()
            .map(|p| (p.id.clone(), p.position))
            .chain(out.props.iter().map(|p| (p.model.clone(), p.position)))
            .chain(out.crates.iter().map(|c| (c.crate_id.clone(), c.position)))
        {
            *seen.entry((item.0, item.1.x.round() as i32, item.1.z.round() as i32)).or_default() += 1;
        }
    }
    let repeated: Vec<_> = seen.iter().filter(|(_, n)| **n != 1).collect();
    assert!(repeated.is_empty(), "emitted more than once: {repeated:?}");
    assert_eq!(seen.len(), PLACE_KINDS.len() * PER_PLACE, "entries dropped");
}

#[test]
fn every_place_emits_its_poi_props_and_crates() {
    let metro = test_metro();
    let all = fold(collect(&metro, &fixtures()));
    assert_eq!(all.pois.len(), PLACE_KINDS.len(), "one POI per place");
    assert_eq!(all.props.len(), PLACE_KINDS.len() * 4, "one main-lot prop + three yard props");
    for (kind, key, table) in PLACE_KINDS {
        let poi = all.pois.iter().find(|p| p.id == format!("metro_{key}"));
        let poi = poi.unwrap_or_else(|| panic!("missing POI for {kind:?}"));
        assert_eq!(poi.building_id, format!("metro_{key}"));
        assert_eq!(poi.loot_table.as_deref(), Some(table));
        let prefix = format!("metro_{key}_c");
        let mut ids: Vec<String> =
            all.crates.iter().filter(|c| c.crate_id.starts_with(&prefix)).map(|c| c.crate_id.clone()).collect();
        ids.sort();
        let expected: Vec<String> = (0..4).map(|i| format!("{prefix}{i}")).collect();
        assert_eq!(ids, expected, "{key} crate ids");
        for spot in all.crates.iter().filter(|c| c.crate_id.starts_with(&prefix)) {
            assert_eq!(spot.tier, 0, "{key} crate tier");
            assert!(spot.model.is_some(), "{key} crate model");
            // A crate entry without a table falls back to the place's own table.
            assert_eq!(spot.table.as_deref(), Some(table), "{key} crate table");
        }
    }
}

#[test]
fn a_lot_local_origin_lands_on_that_lots_centre() {
    let metro = test_metro();
    let data = fixtures();
    // `yardCrates.c` holds one crate at its lot's local origin, and it is crate index 3.
    for site in place_sites(&metro.layout) {
        let lookup = |kind: Reserved| if kind == site.kind { data.get(&kind) } else { None };
        let (key, _) = place_info(site.kind).expect("a place kind");
        let centre = site.lot_centre(&metro.layout, site.yard_lots[2]).expect("lot c");
        let found = group_chunks(&metro, &site)
            .into_iter()
            .flat_map(|c| places_in_chunk_with(&metro, c, &|_, _| 0.0, &lookup).crates)
            .find(|s| s.crate_id == format!("metro_{key}_c3"));
        let spot = found.unwrap_or_else(|| panic!("missing yard-c crate for {key}"));
        let at = Vec2::new(spot.position.x, spot.position.z);
        assert!(at.distance(centre) < 1e-3, "{key} lot frame off by {:?}", at - centre);
    }
}

#[test]
fn the_same_layout_gives_the_same_result_and_far_chunks_are_empty() {
    let metro = test_metro();
    let data = fixtures();
    assert_eq!(collect(&metro, &data), collect(&metro, &data));
    assert_eq!(place_sites(&metro.layout), place_sites(&metro.layout));
    let lookup = |kind: Reserved| data.get(&kind);
    let far = places_in_chunk_with(&metro, (99_999, -99_999), &|_, _| 0.0, &lookup);
    assert_eq!(far, PlaceChunk::default(), "a far chunk must be empty");
}

#[test]
fn a_missing_data_file_emits_nothing_for_that_kind() {
    let metro = test_metro();
    let empty: HashMap<Reserved, PlaceData> = HashMap::new();
    let lookup = |kind: Reserved| empty.get(&kind);
    for site in place_sites(&metro.layout) {
        for chunk in group_chunks(&metro, &site) {
            let out = places_in_chunk_with(&metro, chunk, &|_, _| 0.0, &lookup);
            assert_eq!(out, PlaceChunk::default());
        }
    }
}

#[test]
fn every_place_lot_has_a_paint_role_and_the_park_does_not() {
    let metro = test_metro();
    let roles = metro.layout.blocks.iter().filter_map(|b| lot_role(&metro.layout, b)).count();
    assert_eq!(roles, PLACE_KINDS.len() * 4, "four lots per place");
    let park = metro.layout.blocks.iter().find(|b| b.reserved == Some(Reserved::Park)).expect("park");
    assert!(lot_role(&metro.layout, park).is_none(), "the park keeps its own paint");
}

#[test]
fn fixture_json_round_trips_and_rejects_bad_entries() {
    let data = fixture("hospital", "metro_hospital");
    let json = serde_json::to_string(&data).expect("fixture serialises");
    assert_eq!(PlaceData::from_json(&json).expect("fixture round-trips"), data);
    assert_eq!(data.model, "metro_hospital");
    assert_eq!(data.floors, 5);
    assert_eq!(data.yards.a.len(), 1);
    assert_eq!(data.yard_crates.c.len(), 1);
    assert_eq!(data.props[0].scale, 1.0, "a missing scale defaults to 1");
    assert!(PlaceData::from_json(&json.replace("\"floors\":5", "\"floors\":0")).is_err());
    assert!(PlaceData::from_json(&json.replace("\"model\":\"gurney\"", "\"model\":\"\"")).is_err());
}

#[test]
fn the_place_kind_keys_and_tables_are_the_shared_names() {
    assert_eq!(place_info(Reserved::Hospital), Some(("hospital", "metro_hospital")));
    assert_eq!(place_info(Reserved::Police), Some(("police", "metro_police")));
    assert_eq!(place_info(Reserved::FireStation), Some(("fire_station", "metro_fire")));
    assert_eq!(place_info(Reserved::Park), None);
    assert_eq!(place_info(Reserved::ArmyLine), None);
}

/// Every `.glb` name under `assets/models`.
fn model_names() -> HashSet<String> {
    fn walk(dir: &Path, out: &mut HashSet<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "glb")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                out.insert(stem.to_string());
            }
        }
    }
    let mut out = HashSet::new();
    walk(&paths::repo_path("assets/models"), &mut out);
    out
}

#[test]
fn the_real_place_files_load_and_name_models_that_exist() {
    let models = model_names();
    assert!(!models.is_empty(), "assets/models must hold some .glb files");
    let metro = test_metro();
    for (kind, key, _table) in PLACE_KINDS {
        let path = paths::repo_path(&format!("data/places/{key}.json"));
        if !path.exists() {
            // R42-03/04/05 own these files; this test turns on when they land.
            continue;
        }
        let data = PlaceData::load(&path).unwrap_or_else(|e| panic!("{key} place data: {e}"));
        assert_eq!(place_data(kind), Some(&data), "{key} must come from the loaded cache");
        assert!(models.contains(&data.model), "{key} building model missing");
        for model in data.all_entries().map(|e| &e.model).filter(|m| !m.is_empty()) {
            assert!(models.contains(model), "{key} names missing model '{model}'");
        }
        let site = place_sites(&metro.layout)
            .into_iter()
            .find(|s| s.kind == kind)
            .expect("the site exists");
        let chunks = group_chunks(&metro, &site);
        let lookup = |k: Reserved| (k == kind).then_some(&data);
        let found = fold(chunks.iter().map(|&c| places_in_chunk_with(&metro, c, &|_, _| 0.0, &lookup)).collect());
        let wanted = 1 + data.all_entries().count();
        assert_eq!(found.pois.len() + found.props.len() + found.crates.len(), wanted, "{key} lost entries");
        // The default entry point reads the real files and still places the POI exactly once.
        let real = fold(chunks.iter().map(|&c| places_in_chunk(&metro, c, &|_, _| 0.0)).collect());
        let poi_id = format!("metro_{key}");
        assert_eq!(real.pois.iter().filter(|p| p.id == poi_id).count(), 1, "{key} POI count");
        assert!(real.props.len() >= found.props.len(), "{key} lost props");
        assert!(real.crates.len() >= found.crates.len(), "{key} lost crates");
    }
}
