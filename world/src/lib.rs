pub mod biome;
pub mod camp_layout;
pub mod city;
pub mod color;
pub mod daynight;
pub mod generator;
mod generator_contents;
mod generator_metro;
mod generator_world;
pub mod ground_detail;
pub mod ground_paint;
pub mod lake;
pub mod loot;
pub mod math;
pub mod metro;
pub mod noise;
pub mod poi;
pub mod random;
pub mod rivers;
pub mod roads;
pub mod scatter;
pub mod settlement;
pub mod terrain_paint;
pub mod terrain_shape;
pub mod time;
pub mod towns;
pub mod weather;

pub use biome::ambient_temperature;

pub use biome::{
    BiomeDef, BiomeGround, BiomeHeight, BiomeId, BiomeRegistry, BiomeSample, SectorConfig,
    TierConfig, WorldLayout, WorldLayoutData,
};
pub use camp_layout::{
    camp_gunsmith_prop, camp_gunsmith_prop_with_config, camp_upgrade_props,
    camp_upgrade_props_with_config, wall_section_count, CampLayoutConfigData, CampLevels,
    TowerCorner,
};
pub use city::{
    BuildingWeight, CityBlock, CityConfig, CityConfigData, CityDistrictsData, CityLayout, CityLot,
    District, StreetSegment,
};
pub use color::Color;
pub use daynight::{DayNightLook, SkyColors};
pub use generator::{ChunkGround, GroundTriangle, WorldGenerator};
pub use metro::{
    Highway, HighwayHeightField, JamProp, MetroConfig, MetroDistrict, MetroLayout, MetroLook,
    MetroSite, MetroWorld, PartKind, StyleDef, TowerKind, TowerPart, TowerSpot, TowerStyle, default_styles,
    highway_height_at, jam_props_in_chunk, load_styles, metro_ground_color, place_metro,
    plan_highway, plan_jam, plan_towers, plan_towers_with_styles, style_for, tower_parts,
    tower_parts_with_damage, tower_parts_with_extras, validate_styles,
    // R40: overgrowth, damage, traces
    CampSite, DamageConfig, DamageMode, DamageSites, DistrictOvergrowth, OvergrowthConfig, TraceConfig,
    camp_sites, car_wall_sites, damage_props_in_chunk, damage_sites, default_damage, default_overgrowth,
    default_traces, load_damage, load_overgrowth, load_traces, lobby_dressing,
    overgrowth_props_in_chunk, overgrowth_props_in_chunk_with_config,
    traces_props_in_chunk, validate_damage,
    // R41: herds, life, loot
    CrowPerch, FlapKind, Flapper, HerdRoute, HerdsConfig, HerdsDistrictCount, LifeConfig,
    MetroLootConfig, PerchKind, crow_perches_in_chunk, flappers_in_chunk, herd_routes,
    herd_routes_avoiding, life_props_in_chunk, metro_loot_in_chunk,
    metro_loot_in_chunk_with_config, route_pos,
};
pub use ground_paint::GroundPaint;
pub use lake::{
    place_lake_props, Dock, FishingSpot, FishingSpotKind, LakeConfig, LakeConfigData, LakeLayout,
    LakeZone, VillageLot,
};
pub use loot::{LootPlacer, LootSpot};
pub use math::{lerp, smooth_step};
pub use noise::GodotNoise;
pub use poi::{PoiBuildingDef, PoiConfig, PoiConfigData, PoiGeometry, PoiInstance, PoiPlacer};
pub use random::DotnetRandom;
pub use rivers::{River, RiverNetwork, RiversData};
pub use roads::{Road, RoadKind, RoadNetwork, RoadsData};
pub use scatter::{
    PropInstance, ScatterConfig, ScatterConfigData, ScatterLootData, ScatterPlacer, ScatterRule,
    ScatterVisibility,
};
pub use settlement::{
    SettlementConfig, SettlementConfigData, SettlementGround, SettlementPlacer, SettlementSite,
};
pub use time::{Clock, ClockChanges};
pub use towns::{
    place_towns, ShopkeeperSpot, TownLot, TownShopkeeper, TownSite, TownsData, GUN_SHOP_MODEL,
};
pub use weather::{WeatherData, WeatherKind, WeatherKindDef, WeatherState, WeatherWeight};
