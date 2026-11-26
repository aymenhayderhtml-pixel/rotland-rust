//! New Halden: the big dying city (v1.1). Pure layout logic, no GPU.

pub mod config;
pub mod layout;
pub mod place;
pub mod towers;
pub mod highway;
pub mod jam;
pub mod jam_chunks;
pub mod paint;
pub mod state;

#[cfg(test)]
mod layout_tests;

pub use config::MetroConfig;
pub use paint::{MetroLook, metro_ground_color};
pub use layout::{MetroBlock, MetroDistrict, MetroLayout, MetroStreet, Reserved};
pub use highway::{Highway, HighwayHeightField, highway_height_at, plan_highway};
pub use jam::{JamProp, plan_jam};
pub use jam_chunks::jam_props_in_chunk;
pub use place::{MetroPlacementFrame, MetroSite, place_metro, place_metro_in_frame};
pub use state::MetroWorld;
pub use towers::{TowerKind, TowerSpot, plan_towers};

#[cfg(test)]
mod towers_tests;
