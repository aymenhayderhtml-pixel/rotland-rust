pub mod config;
pub mod layout;
pub mod lots;
pub mod streets;

pub use config::{
    BuildingWeight, CityConfig, CityConfigData, CityDistrictsData, District,
};
pub use layout::{CityBlock, CityLayout};
pub use lots::CityLot;
pub use streets::StreetSegment;

#[cfg(test)]
mod tests;
