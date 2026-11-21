//! Camp layout and upgrade prop positioning.

pub mod types;
pub(crate) mod walls;
pub(crate) mod buildings;

#[cfg(test)]
mod tests;

pub use types::{CampLayoutConfigData, CampLevels, TowerCorner};
pub use walls::wall_section_count;

use crate::scatter::PropInstance;
use crate::settlement::SettlementSite;

/// Generates prop instances for all camp upgrades at the given site and levels.
///
/// Returns an empty vector if all upgrade levels are 0.
pub fn camp_upgrade_props(site: &SettlementSite, levels: CampLevels) -> Vec<PropInstance> {
    camp_upgrade_props_with_config(site, levels, &CampLayoutConfigData::load_default())
}

/// Generates camp upgrade prop instances using a specific layout configuration.
pub fn camp_upgrade_props_with_config(
    site: &SettlementSite,
    levels: CampLevels,
    cfg: &CampLayoutConfigData,
) -> Vec<PropInstance> {
    let mut props = Vec::new();
    walls::place_wall_ring(site, levels.walls, cfg, &mut props);
    buildings::place_buildings(site, levels, cfg, &mut props);
    props
}
