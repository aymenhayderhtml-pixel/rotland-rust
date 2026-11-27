//! New Halden: the big dying city (v1.1). Pure layout logic, no GPU.

pub mod config;
pub mod curbs;
pub mod damage;
pub mod damage_burnt;
pub mod herds;
mod herds_grid;
mod herds_route;
#[cfg(test)]
mod herds_tests;
pub mod layout;
pub mod life;
mod life_billboards;
mod life_config;
mod life_flags;
mod life_helpers;
mod life_perches;
mod life_wires;
#[cfg(test)]
mod life_tests;
pub mod loot;
mod loot_helpers;
#[cfg(test)]
mod loot_tests;
pub mod overgrowth;
pub mod place;
pub mod places;
pub mod places_floor;
mod places_data;
#[cfg(test)]
mod places_tests;
pub mod army_line;
#[cfg(test)]
mod army_line_tests;
pub mod climb;
mod climb_hulls;
#[cfg(test)]
mod climb_tests;
pub mod fire_escapes;
pub mod rooftops;
#[cfg(test)]
mod rooftops_tests;
pub mod styles;
pub mod tower_parts;
#[cfg(test)]
mod tower_parts_tests;
pub mod towers;
pub mod traces;
pub mod highway;
pub mod jam;
pub mod jam_chunks;
pub mod paint;
pub mod state;

#[cfg(test)]
mod layout_tests;

pub use config::MetroConfig;
pub use curbs::{CurbTri, build_curbs, build_curbs_in_chunk};
pub use damage::{
    DamageConfig, DamageSites, Site, damage_props_in_chunk, damage_sites, default_damage,
    load_damage, validate_damage,
};
pub use overgrowth::{
    DistrictOvergrowth, OvergrowthConfig, default_overgrowth, load_overgrowth,
    overgrowth_props_in_chunk, overgrowth_props_in_chunk_with_config,
};
pub use paint::{MetroLook, metro_ground_color};
pub use layout::{MetroBlock, MetroDistrict, MetroLayout, MetroStreet, Reserved};
pub use highway::{Highway, HighwayHeightField, highway_height_at, plan_highway};
pub use jam::{JamProp, plan_jam};
pub use jam_chunks::jam_props_in_chunk;
pub use place::{MetroPlacementFrame, MetroSite, place_metro, place_metro_in_frame};
pub use state::MetroWorld;
pub use styles::{StyleDef, TowerStyle, default_styles, load_styles, style_for, validate_styles};
pub use tower_parts::{
    DamageMode, PartKind, TowerPart, tower_parts, tower_parts_with_damage,
    tower_parts_with_extras,
};
pub use towers::{TowerKind, TowerSpot, plan_towers, plan_towers_with_styles};
pub use traces::{
    CampSite, CarWallsConfig, LobbyDressingEntry, TraceConfig, camp_sites, car_wall_sites,
    default_traces, load_traces, lobby_dressing, traces_props_in_chunk,
};
pub use herds::{
    HerdRoute, HerdsConfig, HerdsDistrictCount, herd_routes, herd_routes_avoiding, route_pos,
};
pub use life::{
    CrowPerch, FlapKind, Flapper, LifeConfig, PerchKind, crow_perches_in_chunk, flappers_in_chunk,
    life_props_in_chunk,
};
pub use loot::{MetroLootConfig, metro_loot_in_chunk, metro_loot_in_chunk_with_config};
pub use places::{
    LotRole, PLACE_KINDS, PlaceChunk, PlaceSite, lot_role, place_info, place_site, place_sites,
    places_in_chunk, places_in_chunk_with,
};
pub use army_line::{
    ARMY_LINE_POI_ID, ArmyLineSite, army_line_blocked_points, army_line_in_chunk, army_line_site,
    army_line_spawn_offsets,
};
pub use climb::{
    ClimbBox, ClimbChunk, ClimbConfig, ClimbKind, ClimbLadder, ClimbTower, FloorCrateEntry,
    StopKind, climb_hull_boxes, climb_in_chunk, climb_in_chunk_with, climb_towers, roof_deck_y,
};
pub use fire_escapes::{fire_escape_ladders, fire_escape_towers};
pub use rooftops::{RoofLink, roof_links, rooftops_in_chunk};

#[cfg(test)]
mod towers_tests;
