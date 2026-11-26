pub mod biome;
pub mod camp_layout;
pub mod city;
pub mod color;
pub mod daynight;
pub mod generator;
mod generator_contents;
mod generator_world;
pub mod ground_detail;
pub mod ground_paint;
pub mod lake;
pub mod loot;
pub mod math;
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
