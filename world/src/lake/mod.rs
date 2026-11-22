pub mod config;
pub mod generate;
pub mod height;
pub mod layout;
pub mod mesh;
pub mod place;
pub mod types;

pub use config::{
    DocksConfig, LakeConfig, LakeConfigData, ShoreNoiseConfig, VillageConfig,
};
pub use layout::LakeLayout;
pub use mesh::{build_lake_water_mesh, LakeWaterMesh};
pub use place::place_lake_props;
pub use types::{
    Dock, FishingSpot, FishingSpotKind, LakeZone, VillageLot,
};

#[cfg(test)]
mod tests;
