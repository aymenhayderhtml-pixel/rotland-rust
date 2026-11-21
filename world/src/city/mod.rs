pub mod config;
pub mod layout;
pub mod lots;
pub mod place;
pub mod streets;

pub use config::{
    BuildingWeight, CityConfig, CityConfigData, CityDistrictsData, District,
};
pub use layout::{CityBlock, CityLayout};
pub use lots::CityLot;
pub use place::place_street_props;
pub use streets::StreetSegment;

#[cfg(test)]
mod place_tests;
#[cfg(test)]
mod tests;
