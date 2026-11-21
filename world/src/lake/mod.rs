pub mod config;
pub mod layout;
pub mod place;

pub use config::{
    DocksConfig, LakeConfig, LakeConfigData, ShoreNoiseConfig, VillageConfig,
};
pub use layout::{
    Dock, FishingSpot, FishingSpotKind, LakeLayout, LakeZone, VillageLot,
};
pub use place::place_lake_props;

#[cfg(test)]
mod tests;
