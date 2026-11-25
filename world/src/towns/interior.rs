use std::collections::BTreeMap;

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::random::DotnetRandom;
use crate::scatter::PropInstance;
use crate::towns::{TownLot, TownsData};

fn default_door_clear_m() -> f32 {
    1.2 // data: towns.json door_clear_m
}

fn default_footprint_margin_m() -> f32 {
    0.3 // data: towns.json footprint_margin_m
}

/// Interior furniture placement config loaded from `data/towns.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InteriorsConfig {
    #[serde(default = "default_door_clear_m")]
    pub door_clear_m: f32,
    #[serde(default = "default_footprint_margin_m")]
    pub footprint_margin_m: f32,
    #[serde(default)]
    pub buildings: BTreeMap<String, InteriorBuilding>,
}

impl Default for InteriorsConfig {
    fn default() -> Self {
        Self {
            door_clear_m: default_door_clear_m(),
            footprint_margin_m: default_footprint_margin_m(),
            buildings: BTreeMap::new(),
        }
    }
}

/// Per-building interior floor height, door locations, and candidate furniture spots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InteriorBuilding {
    pub floor_y: f32,
    pub count: [usize; 2],
    #[serde(default)]
    pub doors: Vec<[f32; 2]>,
    #[serde(default)]
    pub spots: Vec<InteriorSpot>,
}

/// One candidate furniture spot in building-local space (`x` = right, `y` = forward/+Y in Blender).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InteriorSpot {
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub models: Vec<String>,
}

/// Places 3-6 interior furniture props inside `lot` at the building's floor height,
/// rotated with `lot.yaw`, inside the shrunk footprint, and clear of all doors.
pub fn place(lot: &TownLot, data: &TownsData, seed: i32) -> Vec<PropInstance> {
    let Some(building) = data.interiors.buildings.get(&lot.building) else {
        return Vec::new();
    };

    let margin = data.interiors.footprint_margin_m;
    let max_x = (lot.footprint.x - margin).max(0.0);
    let max_y = (lot.footprint.y - margin).max(0.0);
    let clear_sq = data.interiors.door_clear_m * data.interiors.door_clear_m;

    let mut valid: Vec<usize> = building
        .spots
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            !s.models.is_empty()
                && s.x.abs() <= max_x
                && s.y.abs() <= max_y
                && building.doors.iter().all(|&[dx, dy]| {
                    let rx = s.x - dx;
                    let ry = s.y - dy;
                    rx * rx + ry * ry >= clear_sq
                })
        })
        .map(|(idx, _)| idx)
        .collect();

    if valid.is_empty() {
        return Vec::new();
    }

    // new: DECISIONS 2026-11-25
    const LOT_HASH_ROT: u32 = 13;
    let lot_hash = (lot.pos.x.to_bits() ^ lot.pos.y.to_bits().rotate_left(LOT_HASH_ROT)) as i32;
    let mut rng = DotnetRandom::new(seed.wrapping_add(lot_hash));

    for i in (1..valid.len()).rev() {
        let j = rng.next((i + 1) as i32) as usize;
        valid.swap(i, j);
    }

    let lo = building.count[0].min(valid.len());
    let hi = building.count[1].min(valid.len()).max(lo);
    let count = if hi > lo {
        rng.next_range(lo as i32, (hi + 1) as i32) as usize
    } else {
        lo
    };

    let forward = lot.forward();
    let right = Vec2::new(-forward.y, forward.x);

    valid
        .into_iter()
        .take(count)
        .map(|idx| {
            let spot = &building.spots[idx];
            let model_idx = rng.next(spot.models.len() as i32) as usize;
            let world_xz = lot.pos + right * spot.x + forward * spot.y;
            PropInstance::new(
                &spot.models[model_idx],
                Vec3::new(world_xz.x, building.floor_y, world_xz.y),
                lot.yaw + spot.yaw,
                1.0,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const WALK_IN_BUILDINGS: [&str; 8] = [
        "adobe_house",
        "desert_store",
        "log_cabin_snow",
        "research_station",
        "ranger_station",
        "stilt_shack",
        "bait_shop",
        "motel",
    ];

    fn test_lot(building: &str, data: &TownsData) -> TownLot {
        TownLot {
            building: building.to_string(),
            pos: Vec2::new(120.0, -340.0),
            yaw: 0.75,
            footprint: data.footprint(building),
            loot_table: String::new(),
        }
    }

    #[test]
    fn all_walk_in_buildings_place_3_to_6_pieces_within_bounds_and_clear_of_doors() {
        let data = TownsData::load_embedded();
        let margin = data.interiors.footprint_margin_m;
        let clear_m = data.interiors.door_clear_m;

        for name in WALK_IN_BUILDINGS {
            let building = data
                .interiors
                .buildings
                .get(name)
                .unwrap_or_else(|| panic!("missing interior config for {name}"));
            let lot = test_lot(name, &data);
            let forward = lot.forward();
            let right = Vec2::new(-forward.y, forward.x);

            for seed in 0..10 {
                let props = place(&lot, &data, seed);
                assert!(
                    (3..=6).contains(&props.len()),
                    "{name} seed {seed} placed {} pieces (expected 3..=6)",
                    props.len()
                );

                for prop in &props {
                    assert!(
                        (prop.position.y - building.floor_y).abs() < 1e-5,
                        "{name}: expected floor_y {}, got {}",
                        building.floor_y,
                        prop.position.y
                    );
                    let rel = Vec2::new(prop.position.x, prop.position.z) - lot.pos;
                    let lx = rel.dot(right);
                    let ly = rel.dot(forward);
                    assert!(
                        lx.abs() <= lot.footprint.x - margin + 1e-4
                            && ly.abs() <= lot.footprint.y - margin + 1e-4,
                        "{name}: piece ({lx:.2}, {ly:.2}) outside shrunk footprint ({:.2}, {:.2})",
                        lot.footprint.x - margin,
                        lot.footprint.y - margin
                    );
                    for &[dx, dy] in &building.doors {
                        let dist = Vec2::new(lx - dx, ly - dy).length();
                        assert!(
                            dist >= clear_m - 1e-4,
                            "{name}: piece ({lx:.2}, {ly:.2}) too close ({dist:.2} m) to door ({dx:.2}, {dy:.2})"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn placement_is_deterministic() {
        let data = TownsData::load_embedded();
        let lot = test_lot("adobe_house", &data);
        let a = place(&lot, &data, 42);
        let b = place(&lot, &data, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn non_interior_building_returns_empty() {
        let data = TownsData::load_embedded();
        let lot = test_lot("water_tower", &data);
        assert!(place(&lot, &data, 42).is_empty());
    }
}
