//! New Halden UI: grey-green haze math, big-map outline and streets, and minimap streets.

pub mod draw_map;
pub mod haze;
pub mod minimap;
pub mod outline;

#[cfg(test)]
mod tests;

pub use draw_map::{MetroMapConfig, draw_metro_map};
pub use haze::{MetroHazeConfig, blend_fog, city_signed_distance, haze_weight, skyline_fog};
pub use minimap::{MinimapStreet, minimap_streets};
pub use outline::{discovered, site_corners, streets_on_map};
