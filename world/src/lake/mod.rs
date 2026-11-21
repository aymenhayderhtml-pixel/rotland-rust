pub mod config;
pub mod layout;

pub use config::{
    DocksConfig, LakeConfig, LakeConfigData, ShoreNoiseConfig, VillageConfig,
};
pub use layout::{
    Dock, FishingSpot, FishingSpotKind, LakeLayout, LakeZone, VillageLot,
};

#[cfg(test)]
mod tests;
