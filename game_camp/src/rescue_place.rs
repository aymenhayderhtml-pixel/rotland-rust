//! Seeded placement of rescue survivors at house/shop POIs away from camp.

use std::collections::HashSet;

use glam::{IVec2, Quat, Vec2, Vec3};
use serde::{Deserialize, Serialize};
use world::{DotnetRandom, PoiConfigData, PoiInstance, PoiPlacer, TownLot, TownSite, WorldGenerator};

use crate::guild::{GuildRecruit, RecruitDef, RecruitOrder, RecruitRole};
use game_core::loot::crate_seed;
use crate::rescue::RescueConfig;
use crate::rescue_floor::is_climb_floor_poi;

/// POI building IDs eligible to hold a trapped survivor.
pub const RESCUE_BUILDINGS: &[&str] = &["house_small", "cabin", "shop", "apartment", "fish_shack"];
/// Town building models eligible to hold a trapped survivor.
pub const TOWN_RESCUE_BUILDINGS: &[&str] = &[
    "adobe_house",
    "motel",
    "desert_store",
    "log_cabin_snow",
    "research_station",
    "bait_shop",
    "stilt_shack",
    "ranger_station",
    "cabin",
];

/// Extra metres in front of the building's front lot edge so the survivor stands outside walls.
const FRONT_OFFSET_EXTRA: f32 = 1.5; // new: DECISIONS 2026-11-25
/// Chunk radius around camp scanned for wild house/cabin POIs at world start.
const WORLD_SEARCH_CHUNKS: i32 = 20; // new: DECISIONS 2026-11-25

/// A deterministic world spot where a survivor is waiting to be rescued.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RescueSpot {
    pub id: String,
    pub poi_id: String,
    pub building_id: String,
    pub name: String,
    pub role: RecruitRole,
    pub level: u32,
    pub daily_wage: u32,
    pub max_health: u32,
    pub position: Vec3,
    pub yaw: f32,
}

impl RescueSpot {
    /// Converts this rescue spot into a free hired teammate with standing order Follow.
    pub fn to_recruit(&self) -> GuildRecruit {
        let mut recruit = GuildRecruit::new(
            &self.id,
            &self.name,
            self.role,
            self.level,
            self.daily_wage,
            self.max_health,
            0,
        );
        recruit.set_order(RecruitOrder::Follow);
        recruit
    }
}

/// True if `building_id` is a house or shop POI type that can hold a survivor.
pub fn is_rescue_building(building_id: &str) -> bool {
    RESCUE_BUILDINGS.contains(&building_id)
}

/// True if `building` is an enclosed town building model that can hold a survivor.
pub fn is_town_rescue_building(building: &str) -> bool {
    TOWN_RESCUE_BUILDINGS.contains(&building)
}

/// Parses a recruit role string from `data/guild.json` into `RecruitRole`.
pub fn parse_recruit_role(role: &str) -> RecruitRole {
    match role.to_lowercase().as_str() {
        "fighter" => RecruitRole::Fighter,
        "scavenger" => RecruitRole::Scavenger,
        "guard" => RecruitRole::Guard,
        _ => RecruitRole::Fighter,
    }
}

fn flat_dist(a: Vec3, b: Vec3) -> f32 {
    let dx = a.x - b.x;
    let dz = a.z - b.z;
    (dx * dx + dz * dz).sqrt()
}

fn near_any_camp(pos: Vec3, camp_centers: &[Vec3], min_dist: f32) -> bool {
    camp_centers.iter().any(|&c| flat_dist(pos, c) < min_dist)
}

fn spot_position(poi: &PoiInstance, poi_config: &PoiConfigData) -> Vec3 {
    let half_depth = poi_config.get(&poi.building_id).map_or(0.0, |d| d.half_depth());
    let offset = Quat::from_rotation_y(poi.yaw) * Vec3::new(0.0, 0.0, half_depth + FRONT_OFFSET_EXTRA);
    poi.position + offset
}

/// Deterministically selects up to `cfg.max_survivors` rescue spots from the given POIs.
pub fn place_survivors(
    world_seed: i32,
    pois: &[PoiInstance],
    poi_config: &PoiConfigData,
    camp_centers: &[Vec3],
    cfg: &RescueConfig,
    recruits: &[RecruitDef],
) -> Vec<RescueSpot> {
    if cfg.max_survivors == 0 || cfg.chance_per_poi <= 0.0 {
        return Vec::new();
    }

    let mut candidates: Vec<&PoiInstance> = pois
        .iter()
        .filter(|p| {
            is_rescue_building(&p.building_id)
                && !near_any_camp(p.position, camp_centers, cfg.min_dist_from_camp)
        })
        .collect();
    candidates.sort_by(|a, b| {
        a.id.cmp(&b.id)
            .then_with(|| a.position.x.to_bits().cmp(&b.position.x.to_bits()))
            .then_with(|| a.position.z.to_bits().cmp(&b.position.z.to_bits()))
    });

    let fallback = [RecruitDef {
        id: "rescue_default".to_string(),
        name: "Alex Mercer".to_string(),
        role: "fighter".to_string(),
        level: 1,
        daily_wage: 15, // data: guild.json daily_wage
        health: 100,
        hire_cost: 0,
    }];
    let templates: &[RecruitDef] = if recruits.is_empty() { &fallback } else { recruits };

    let mut spots = Vec::new();
    let mut used_names: HashSet<&str> = HashSet::new();

    for poi in candidates {
        let seed = crate_seed(world_seed, &format!("{}/rescue", poi.id));
        let mut rng = DotnetRandom::new(seed);
        if rng.next_double() >= cfg.chance_per_poi as f64 {
            continue;
        }

        let unused: Vec<&RecruitDef> = templates
            .iter()
            .filter(|r| !used_names.contains(r.name.as_str()))
            .collect();
        let tmpl = if unused.is_empty() {
            &templates[rng.next(templates.len() as i32) as usize]
        } else {
            unused[rng.next(unused.len() as i32) as usize]
        };
        used_names.insert(tmpl.name.as_str());

        let id = format!("rescue_{}", spots.len() + 1);
        spots.push(RescueSpot {
            id,
            poi_id: poi.id.clone(),
            building_id: poi.building_id.clone(),
            name: tmpl.name.clone(),
            role: parse_recruit_role(&tmpl.role),
            level: tmpl.level,
            daily_wage: tmpl.daily_wage,
            max_health: tmpl.health,
            position: spot_position(poi, poi_config),
            yaw: poi.yaw,
        });

        if spots.len() >= cfg.max_survivors {
            break;
        }
    }

    spots
}

/// Collects candidate POIs across wild chunks around camp, the city, and the lake village.
pub fn collect_candidate_pois(world_gen: &WorldGenerator) -> Vec<PoiInstance> {
    let camp = world_gen
        .settlements()
        .first()
        .map_or(Vec3::ZERO, |s| s.center);
    let camp_chunk = IVec2::new(
        (camp.x / WorldGenerator::CHUNK_SIZE).floor() as i32,
        (camp.z / WorldGenerator::CHUNK_SIZE).floor() as i32,
    );

    let mut pois = Vec::new();
    for dz in -WORLD_SEARCH_CHUNKS..=WORLD_SEARCH_CHUNKS {
        for dx in -WORLD_SEARCH_CHUNKS..=WORLD_SEARCH_CHUNKS {
            let coord = camp_chunk + IVec2::new(dx, dz);
            let mut placed = PoiPlacer::place(world_gen, world_gen.poi_config(), coord);
            if let Some(city) = world_gen.city() {
                placed.retain(|p| !city.contains(Vec2::new(p.position.x, p.position.z)));
            }
            if let Some(lake) = world_gen.lake() {
                placed.retain(|p| {
                    let xz = Vec2::new(p.position.x, p.position.z);
                    !lake.contains_region(xz)
                        || lake.shore_distance(xz) > lake.config.beach_width + 10.0
                });
            }
            pois.extend(placed);
        }
    }

    if let Some(city) = world_gen.city() {
        for (idx, lot) in city.lots.iter().enumerate() {
            if is_rescue_building(&lot.building_id) {
                let y = world_gen.height(lot.center.x, lot.center.y);
                pois.push(PoiInstance::new(
                    format!("city_{idx}"),
                    &lot.building_id,
                    Vec3::new(lot.center.x, y, lot.center.y),
                    lot.yaw,
                ));
            }
        }
    }

    if let Some(lake) = world_gen.lake() {
        for (idx, lot) in lake.village_lots.iter().enumerate() {
            if is_rescue_building(&lot.building_id) {
                let y = world_gen.height(lot.center.x, lot.center.y);
                pois.push(PoiInstance::new(
                    format!("lake_village_{idx}"),
                    &lot.building_id,
                    Vec3::new(lot.center.x, y, lot.center.y),
                    lot.yaw,
                ));
            }
        }
    }

    pois
}

fn town_lot_spot_position(lot: &TownLot) -> Vec3 {
    let offset = Quat::from_rotation_y(lot.yaw) * Vec3::new(0.0, 0.0, lot.footprint.y + FRONT_OFFSET_EXTRA);
    Vec3::new(lot.pos.x, 0.0, lot.pos.y) + offset
}

/// Deterministically places `cfg.per_town` (e.g. 1-2) rescue spots per town from eligible town lots.
pub fn place_town_survivors(
    world_seed: i32,
    towns: &[TownSite],
    cfg: &RescueConfig,
    recruits: &[RecruitDef],
    id_offset: usize,
    used_names: &mut HashSet<String>,
) -> Vec<RescueSpot> {
    let lo = cfg.per_town[0].min(cfg.per_town[1]);
    let hi = cfg.per_town[0].max(cfg.per_town[1]);
    if hi == 0 || towns.is_empty() {
        return Vec::new();
    }

    let fallback = [RecruitDef {
        id: "rescue_default".to_string(),
        name: "Alex Mercer".to_string(),
        role: "fighter".to_string(),
        level: 1,
        daily_wage: 15, // data: guild.json daily_wage
        health: 100,
        hire_cost: 0,
    }];
    let templates: &[RecruitDef] = if recruits.is_empty() { &fallback } else { recruits };
    let mut spots = Vec::new();

    for town in towns {
        let mut eligible: Vec<(usize, &TownLot)> = town
            .lots
            .iter()
            .enumerate()
            .filter(|(_, lot)| is_town_rescue_building(&lot.building))
            .collect();
        if eligible.is_empty() {
            continue;
        }
        let seed = crate_seed(world_seed, &format!("town_{}/rescue", town.key));
        let mut rng = DotnetRandom::new(seed);
        let want = if hi > lo {
            rng.next_range(lo as i32, hi as i32 + 1) as usize
        } else {
            lo
        }
        .min(eligible.len());

        for _ in 0..want {
            let pick_idx = rng.next(eligible.len() as i32) as usize;
            let (lot_idx, lot) = eligible.swap_remove(pick_idx);
            let unused: Vec<&RecruitDef> = templates
                .iter()
                .filter(|r| !used_names.contains(r.name.as_str()))
                .collect();
            let tmpl = if unused.is_empty() {
                &templates[rng.next(templates.len() as i32) as usize]
            } else {
                unused[rng.next(unused.len() as i32) as usize]
            };
            used_names.insert(tmpl.name.clone());

            let id = format!("rescue_{}", id_offset + spots.len() + 1);
            spots.push(RescueSpot {
                id,
                poi_id: format!("town_{}_{lot_idx}", town.key),
                building_id: lot.building.clone(),
                name: tmpl.name.clone(),
                role: parse_recruit_role(&tmpl.role),
                level: tmpl.level,
                daily_wage: tmpl.daily_wage,
                max_health: tmpl.health,
                position: town_lot_spot_position(lot),
                yaw: lot.yaw,
            });
        }
    }

    spots
}

/// Places rescue survivors for a `WorldGenerator` and snaps their Y coordinate to terrain height.
pub fn place_survivors_in_world(
    world_gen: &WorldGenerator,
    cfg: &RescueConfig,
    recruits: &[RecruitDef],
) -> Vec<RescueSpot> {
    // Climb-floor POIs get their own survivors (rescue_floor.rs), so the random pass skips them.
    let pois: Vec<PoiInstance> = collect_candidate_pois(world_gen)
        .into_iter()
        .filter(|p| !is_climb_floor_poi(&p.id))
        .collect();
    let camp_centers: Vec<Vec3> = world_gen.settlements().iter().map(|s| s.center).collect();
    let mut spots = place_survivors(
        world_gen.seed(),
        &pois,
        world_gen.poi_config(),
        &camp_centers,
        cfg,
        recruits,
    );
    let mut used_names: HashSet<String> = spots.iter().map(|s| s.name.clone()).collect();
    let town_spots = place_town_survivors(
        world_gen.seed(),
        world_gen.towns(),
        cfg,
        recruits,
        spots.len(),
        &mut used_names,
    );
    let keep_core = cfg.max_survivors.saturating_sub(town_spots.len());
    spots.truncate(keep_core);
    spots.extend(town_spots);
    spots.truncate(cfg.max_survivors);
    for (idx, spot) in spots.iter_mut().enumerate() {
        spot.id = format!("rescue_{}", idx + 1);
        spot.position.y = world_gen.height(spot.position.x, spot.position.z);
    }
    spots
}
