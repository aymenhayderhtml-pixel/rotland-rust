//! Building discovery and map notes (port of scripts/World/WorldMap.cs NoteNearbyBuildings).

use glam::Vec2;

use super::state::{MapBuilding, MapState};

/// A POI building lot that can be discovered on the map.
// Godot: PoiSite.cs
#[derive(Clone, Debug, PartialEq)]
pub struct PoiSiteInfo {
    pub poi_id: String,
    pub building_type: String,
    pub position: Vec2,
    pub crate_ids: Vec<String>,
}

impl PoiSiteInfo {
    pub fn new(
        poi_id: impl Into<String>,
        building_type: impl Into<String>,
        position: Vec2,
        crate_ids: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            poi_id: poi_id.into(),
            building_type: building_type.into(),
            position,
            crate_ids: crate_ids.into_iter().map(Into::into).collect(),
        }
    }

    /// True if all crates for this site are marked as looted.
    /// If there are no crates, returns false (matching Godot `site.CrateIds.Count > 0`).
    pub fn is_looted(&self, is_crate_looted: impl Fn(&str) -> bool) -> bool {
        if self.crate_ids.is_empty() {
            return false;
        }
        self.crate_ids.iter().all(|id| is_crate_looted(id))
    }
}

impl MapState {
    /// Notes a building if within reveal/found range (60 m) or already known (updating its looted status).
    ///
    /// Returns true if the building was noted or updated in `found`.
    // Godot: WorldMap.cs NoteNearbyBuildings
    pub fn note_nearby_building(
        &mut self,
        player_xz: Vec2,
        poi_id: &str,
        building_type: &str,
        building_xz: Vec2,
        looted: bool,
    ) -> bool {
        let seen = self.has_building(poi_id);
        let dist_sq = (building_xz - player_xz).length_squared();
        let found_rad_sq = Self::FOUND_RADIUS * Self::FOUND_RADIUS;
        if !seen && dist_sq > found_rad_sq {
            return false;
        }
        self.note_building(MapBuilding::new(poi_id, building_type, building_xz, looted));
        true
    }

    /// Scans a list of POI building sites, noting any within found radius (60 m)
    /// and updating the looted status of any already recorded.
    /// Returns the number of newly discovered buildings.
    // Godot: WorldMap.cs NoteNearbyBuildings
    pub fn note_nearby_poi_sites(
        &mut self,
        player_xz: Vec2,
        sites: &[PoiSiteInfo],
        is_crate_looted: impl Fn(&str) -> bool,
    ) -> usize {
        let mut newly_noted = 0;
        for site in sites {
            let was_seen = self.has_building(&site.poi_id);
            let looted = site.is_looted(&is_crate_looted);
            if self.note_nearby_building(
                player_xz,
                &site.poi_id,
                &site.building_type,
                site.position,
                looted,
            ) && !was_seen
            {
                newly_noted += 1;
            }
        }
        newly_noted
    }
}
