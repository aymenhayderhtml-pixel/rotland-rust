use std::collections::BTreeMap;
use std::f32::consts::FRAC_PI_2;

use glam::Vec2;
use serde::Deserialize;

use world::biome::BiomeId;
use world::towns::lots::yaw_from_forward;
use world::towns::place::{LayoutFrame, place_towns_in_frame, sector_center_dir};
use world::towns::{BOARDWALK_MODEL, TownSite, TownsData};

#[derive(Deserialize)]
struct RawWorldLayout {
    core_radius: f32,
    blend_band: f32,
    sectors: Vec<RawSector>,
}

#[derive(Deserialize)]
struct RawSector {
    biome: String,
    center_deg: f32,
}

fn load_layout_frame(reference_dir: Vec2) -> LayoutFrame {
    let raw = include_str!("../../data/world_layout.json");
    let parsed: RawWorldLayout =
        serde_json::from_str(raw).expect("data/world_layout.json must parse");
    let sector_angles_deg = parsed
        .sectors
        .iter()
        .map(|s| {
            let biome = BiomeId::from_key(&s.biome).expect("valid biome key in world_layout.json");
            (biome, s.center_deg)
        })
        .collect();
    LayoutFrame {
        reference_dir,
        min_outer_radius: parsed.core_radius + parsed.blend_band,
        sector_angles_deg,
    }
}

fn count_buildings(site: &TownSite) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for lot in &site.lots {
        *counts.entry(lot.building.as_str()).or_insert(0) += 1;
    }
    counts
}

#[test]
fn yaw_convention_matches_godot() {
    let east_yaw = yaw_from_forward(Vec2::new(1.0, 0.0));
    assert!((east_yaw + FRAC_PI_2).abs() < 1e-5);
    let north_yaw = yaw_from_forward(Vec2::new(0.0, -1.0));
    assert!(north_yaw.abs() < 1e-5);
}

#[test]
fn places_four_towns_in_biome_sectors_outside_core_and_blend() {
    let data = TownsData::load_embedded();
    let frame = load_layout_frame(Vec2::new(0.8, 0.6).normalize());
    let origin = Vec2::new(120.0, -45.0);

    let sites = place_towns_in_frame(&data, &frame, 42, origin, &|_, _| true);
    assert_eq!(sites.len(), 4);

    let expected = [
        ("dusthollow", BiomeId::from_key("desert").unwrap()),
        ("frostmere", BiomeId::from_key("snow").unwrap()),
        ("blackwater", BiomeId::from_key("swamp").unwrap()),
        ("ridgeback_station", BiomeId::from_key("mountains").unwrap()),
    ];

    for (key, biome) in expected {
        let site = sites
            .iter()
            .find(|t| t.key == key)
            .unwrap_or_else(|| panic!("missing town '{key}'"));
        assert_eq!(site.biome, biome);

        let dist = site.center.distance(origin);
        assert!(
            dist - site.radius > frame.min_outer_radius,
            "town '{}' edge ({:.1} m) must be outside core_radius + blend_band ({:.1} m)",
            site.key,
            dist - site.radius,
            frame.min_outer_radius
        );

        let dir_from_origin = (site.center - origin).normalize();
        let expected_dir = sector_center_dir(&frame, biome);
        assert!(
            dir_from_origin.dot(expected_dir) > 0.999,
            "town '{}' should lie along its biome sector centre",
            site.key
        );

        let to_origin = (origin - site.center).normalize();
        assert!(
            site.street_dir.dot(to_origin) > 0.999,
            "town '{}' street_dir should face back toward origin",
            site.key
        );
    }
}

#[test]
fn no_lot_overlaps_and_lots_face_street() {
    let data = TownsData::load_embedded();
    let frame = load_layout_frame(Vec2::X);
    let sites = place_towns_in_frame(&data, &frame, 1337, Vec2::ZERO, &|_, _| true);

    for site in &sites {
        assert!(!site.lots.is_empty(), "town '{}' has no lots", site.key);

        for i in 0..site.lots.len() {
            for j in (i + 1)..site.lots.len() {
                assert!(
                    !site.lots[i].overlaps(&site.lots[j]),
                    "overlap in town '{}' between '{}' ({:?}) and '{}' ({:?})",
                    site.key,
                    site.lots[i].building,
                    site.lots[i].pos,
                    site.lots[j].building,
                    site.lots[j].pos
                );
            }
        }

        for lot in &site.lots {
            for corner in lot.corners() {
                assert!(
                    site.contains_point(corner),
                    "lot '{}' corner {:?} exceeds radius {:.1} in town '{}'",
                    lot.building,
                    corner,
                    site.radius,
                    site.key
                );
            }

            let rel = lot.pos - site.center;
            let proj = site.street_dir * rel.dot(site.street_dir);
            let to_street = (proj - rel).normalize();
            assert!(
                lot.forward().dot(to_street) > 0.999,
                "lot '{}' in '{}' does not face the street",
                lot.building,
                site.key
            );
        }
    }
}

#[test]
fn unique_buildings_appear_once_nearest_center_with_loot_tables() {
    let data = TownsData::load_embedded();
    let frame = load_layout_frame(Vec2::X);
    let sites = place_towns_in_frame(&data, &frame, 99, Vec2::ZERO, &|_, _| true);

    for def in &data.towns {
        let site = sites.iter().find(|t| t.key == def.key).unwrap();
        let counts = count_buildings(site);

        let mut max_unique_along = 0.0_f32;
        let mut min_regular_along = f32::MAX;

        for entry in &def.buildings {
            let count = counts.get(entry.model.as_str()).copied().unwrap_or(0);
            if entry.unique {
                assert_eq!(
                    count, 1,
                    "unique building '{}' in '{}' must appear exactly once",
                    entry.model, def.key
                );
            }
        }

        for lot in &site.lots {
            let expected_loot = def
                .loot_tables
                .get(&lot.building)
                .expect("every placed building must have a loot_tables entry");
            assert_eq!(&lot.loot_table, expected_loot);

            if lot.building == BOARDWALK_MODEL {
                continue;
            }
            let along = (lot.pos - site.center).dot(site.street_dir).abs();
            let is_unique = def
                .buildings
                .iter()
                .any(|b| b.unique && b.model == lot.building);
            if is_unique {
                max_unique_along = max_unique_along.max(along);
            } else {
                min_regular_along = min_regular_along.min(along);
            }
        }

        if min_regular_along < f32::MAX {
            assert!(
                max_unique_along <= min_regular_along + 1e-3,
                "unique buildings in '{}' should sit nearest the center (max_unique={:.1}, min_regular={:.1})",
                def.key,
                max_unique_along,
                min_regular_along
            );
        }
    }

    let blackwater = sites.iter().find(|t| t.key == "blackwater").unwrap();
    let bw_counts = count_buildings(blackwater);
    assert!(bw_counts.get(BOARDWALK_MODEL).copied().unwrap_or(0) > 0);
    assert!(bw_counts.get("stilt_shack").copied().unwrap_or(0) > 0);

    let ridgeback = sites.iter().find(|t| t.key == "ridgeback_station").unwrap();
    let rb_counts = count_buildings(ridgeback);
    let cabins = rb_counts.get("cabin").copied().unwrap_or(0);
    assert!((3..=4).contains(&cabins), "expected 3..=4 cabins, got {cabins}");
}

#[test]
fn placement_and_lots_are_deterministic_for_a_seed() {
    let data = TownsData::load_embedded();
    let frame = load_layout_frame(Vec2::X);

    let first = place_towns_in_frame(&data, &frame, 2026, Vec2::ZERO, &|_, _| true);
    let second = place_towns_in_frame(&data, &frame, 2026, Vec2::ZERO, &|_, _| true);
    assert_eq!(first, second);
}

#[test]
fn failing_flat_check_falls_back_without_panic_and_spiral_finds_nearby_flat_spot() {
    let data = TownsData::load_embedded();
    let frame = load_layout_frame(Vec2::X);
    let origin = Vec2::ZERO;

    // Total failure: every town still places at its fallback center without panicking.
    let fallback_sites = place_towns_in_frame(&data, &frame, 7, origin, &|_, _| false);
    assert_eq!(fallback_sites.len(), 4);
    for (site, def) in fallback_sites.iter().zip(&data.towns) {
        let expected_center = origin + sector_center_dir(&frame, site.biome) * def.distance_m;
        assert!(site.center.distance(expected_center) < 1e-3);
        assert!(!site.lots.is_empty());
    }

    // Partial failure: reject within 150 m of each ideal start, accept beyond 150 m.
    let ideal_centers: Vec<Vec2> = data
        .towns
        .iter()
        .map(|def| {
            let biome = def.biome_id().unwrap();
            origin + sector_center_dir(&frame, biome) * def.distance_m
        })
        .collect();
    let spiral_sites = place_towns_in_frame(&data, &frame, 7, origin, &|pos, _| {
        ideal_centers.iter().all(|&ideal| pos.distance(ideal) >= 150.0)
    });

    assert_eq!(spiral_sites.len(), 4);
    for (site, &ideal) in spiral_sites.iter().zip(&ideal_centers) {
        let offset = site.center.distance(ideal);
        assert!(
            offset >= 150.0 && offset <= data.placement.spiral_max_m + 1e-3,
            "spiral offset {:.1} m out of expected range for '{}'",
            offset,
            site.key
        );
        assert!(site.center.distance(origin) - site.radius > frame.min_outer_radius);
    }
}
