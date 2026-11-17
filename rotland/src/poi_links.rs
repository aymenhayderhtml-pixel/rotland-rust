//! Glue linking world POIs to gameplay systems (DogPoiSite list and map building notes).
//!
//! Port of Godot scripts/Dogs/ stray lot queries and scripts/World/PoiManager.cs / WorldMap.cs.

use std::collections::HashMap;
use glam::{IVec2, Vec2, Vec3};
use game::{DogPoiSite, MapBuilding, MapState};
use world::{LootSpot, PoiConfigData, PoiGeometry, PoiInstance};

/// Converts a world PoiInstance into a game DogPoiSite for stray dog spawning checks.
// Godot: StrayDogSpawner.cs
pub fn build_dog_poi_site(poi: &PoiInstance, poi_config: &PoiConfigData) -> DogPoiSite {
    let half_depth = poi_config.get(&poi.building_id).map_or(0.0, |d| d.half_depth());
    DogPoiSite::new(&poi.id, &poi.building_id, poi.position, poi.yaw, half_depth)
}

/// Converts a slice of PoiInstances into a list of DogPoiSites.
pub fn build_dog_poi_sites(pois: &[PoiInstance], poi_config: &PoiConfigData) -> Vec<DogPoiSite> {
    pois.iter().map(|p| build_dog_poi_site(p, poi_config)).collect()
}

/// One active POI site loaded in the world, with its geometry and associated crate IDs.
// Godot: PoiSite.cs
#[derive(Clone, Debug, PartialEq)]
pub struct ActivePoiSite {
    pub chunk: IVec2,
    pub poi: PoiInstance,
    pub half_depth: f32,
    pub crate_ids: Vec<String>,
}

/// Tracks loaded POI buildings across active streaming chunks.
// Godot: PoiManager.cs
#[derive(Clone, Debug, Default)]
pub struct PoiLinks {
    sites: HashMap<String, ActivePoiSite>,
    chunk_sites: HashMap<IVec2, Vec<String>>,
}

impl PoiLinks {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers the POIs and crates of a newly loaded chunk.
    pub fn on_chunk_loaded(
        &mut self,
        chunk: IVec2,
        pois: &[PoiInstance],
        poi_config: &PoiConfigData,
        crates: &[LootSpot],
    ) {
        let mut ids = Vec::new();
        for poi in pois {
            let half_depth = poi_config.get(&poi.building_id).map_or(0.0, |d| d.half_depth());
            let poi_crates: Vec<String> = crates
                .iter()
                .filter(|c| {
                    if let Some(def) = poi_config.get(&poi.building_id) {
                        PoiGeometry::contains(poi, def, c.position.x, c.position.z, poi_config.lot_margin)
                    } else {
                        false
                    }
                })
                .map(|c| c.crate_id.clone())
                .collect();

            self.sites.insert(
                poi.id.clone(),
                ActivePoiSite {
                    chunk,
                    poi: poi.clone(),
                    half_depth,
                    crate_ids: poi_crates,
                },
            );
            ids.push(poi.id.clone());
        }
        self.chunk_sites.insert(chunk, ids);
    }

    /// Unregisters all POIs belonging to an unloaded chunk.
    pub fn on_chunk_unloaded(&mut self, chunk: IVec2) {
        if let Some(ids) = self.chunk_sites.remove(&chunk) {
            for id in ids {
                self.sites.remove(&id);
            }
        }
    }

    /// Builds the `DogPoiSite` list for all active loaded POIs.
    pub fn dog_poi_sites(&self) -> Vec<DogPoiSite> {
        self.sites
            .values()
            .map(|s| {
                DogPoiSite::new(
                    &s.poi.id,
                    &s.poi.building_id,
                    s.poi.position,
                    s.poi.yaw,
                    s.half_depth,
                )
            })
            .collect()
    }

    /// Scans active POI buildings, adding any within reveal range (60 m) to `MapState`
    /// and updating the looted flag of already seen buildings.
    // Godot: WorldMap.cs NoteNearbyBuildings
    pub fn update_map_buildings(
        &self,
        map_state: &mut MapState,
        player_pos: Vec3,
        is_crate_looted: impl Fn(&str) -> bool,
    ) {
        let player_xz = Vec2::new(player_pos.x, player_pos.z);
        let found_rad_sq = MapState::FOUND_RADIUS * MapState::FOUND_RADIUS;

        for site in self.sites.values() {
            let at = Vec2::new(site.poi.position.x, site.poi.position.z);
            let seen = map_state.has_building(&site.poi.id);
            if !seen && (at - player_xz).length_squared() > found_rad_sq {
                continue;
            }
            let looted = !site.crate_ids.is_empty() && site.crate_ids.iter().all(|c| is_crate_looted(c));
            map_state.note_building(MapBuilding::new(&site.poi.id, &site.poi.building_id, at, looted));
        }
    }

    /// Total number of currently active POI sites.
    pub fn len(&self) -> usize {
        self.sites.len()
    }

    /// True if there are no loaded POIs.
    pub fn is_empty(&self) -> bool {
        self.sites.is_empty()
    }

    /// Gets an active POI site by its ID.
    pub fn get(&self, poi_id: &str) -> Option<&ActivePoiSite> {
        self.sites.get(poi_id)
    }
}

/// Notes nearby buildings directly from a slice of POI instances and loot spots.
pub fn note_nearby_pois(
    map_state: &mut MapState,
    player_pos: Vec3,
    pois: &[PoiInstance],
    crates: &[LootSpot],
    poi_config: &PoiConfigData,
    is_crate_looted: impl Fn(&str) -> bool,
) {
    let player_xz = Vec2::new(player_pos.x, player_pos.z);
    for poi in pois {
        let at = Vec2::new(poi.position.x, poi.position.z);
        let poi_crates: Vec<&str> = crates
            .iter()
            .filter(|c| {
                if let Some(def) = poi_config.get(&poi.building_id) {
                    PoiGeometry::contains(poi, def, c.position.x, c.position.z, poi_config.lot_margin)
                } else {
                    false
                }
            })
            .map(|c| c.crate_id.as_str())
            .collect();
        let looted = !poi_crates.is_empty() && poi_crates.iter().all(|c| is_crate_looted(c));
        map_state.note_nearby_building(player_xz, &poi.id, &poi.building_id, at, looted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use world::PoiBuildingDef;

    fn sample_poi_config() -> PoiConfigData {
        let cabin = PoiBuildingDef {
            id: "cabin".to_string(),
            lot: [10.0, 12.0],
            biome: HashMap::new(),
            chance_per_chunk: 0.5,
            min_flatness: 0.8,
            min_distance_from_spawn: 50.0,
            loot_table: "cabin".to_string(),
            zombies: [1, 3],
        };
        PoiConfigData {
            buildings: vec![cabin],
            ..Default::default()
        }
    }

    #[test]
    fn test_build_dog_poi_site() {
        let cfg = sample_poi_config();
        let poi = PoiInstance::new("poi_0_0", "cabin", Vec3::new(100.0, 5.0, 200.0), 1.57);
        let dog_site = build_dog_poi_site(&poi, &cfg);

        assert_eq!(dog_site.poi_id, "poi_0_0");
        assert_eq!(dog_site.building_id, "cabin");
        assert_eq!(dog_site.position, Vec3::new(100.0, 5.0, 200.0));
        assert_eq!(dog_site.yaw, 1.57);
        assert_eq!(dog_site.half_depth, 6.0); // lot[1] = 12.0 -> half_depth = 6.0
    }

    #[test]
    fn test_poi_links_streaming_lifecycle_and_map_notes() {
        let cfg = sample_poi_config();
        let mut links = PoiLinks::new();
        let chunk = IVec2::new(1, 2);

        let poi = PoiInstance::new("cabin_12", "cabin", Vec3::new(64.0, 0.0, 128.0), 0.0);
        let crate_spot = LootSpot::new("crate_1", Vec3::new(65.0, 0.0, 129.0), 0.0, world::BiomeId::Forest);

        links.on_chunk_loaded(chunk, &[poi], &cfg, &[crate_spot]);
        assert_eq!(links.len(), 1);

        let dog_sites = links.dog_poi_sites();
        assert_eq!(dog_sites.len(), 1);
        assert_eq!(dog_sites[0].poi_id, "cabin_12");
        assert_eq!(dog_sites[0].half_depth, 6.0);

        // Player far away (at origin) -> not noted
        let mut map_state = MapState::new();
        links.update_map_buildings(&mut map_state, Vec3::ZERO, |_| false);
        assert!(!map_state.has_building("cabin_12"));

        // Player approaches (within 60m: distance to 64,128 from 64,100 is 28m)
        links.update_map_buildings(&mut map_state, Vec3::new(64.0, 0.0, 100.0), |_| false);
        assert!(map_state.has_building("cabin_12"));
        assert!(!map_state.found[0].looted);

        // Crate looted -> status updates
        links.update_map_buildings(&mut map_state, Vec3::new(64.0, 0.0, 100.0), |id| id == "crate_1");
        assert!(map_state.found[0].looted);

        // Chunk unloaded -> forgotten
        links.on_chunk_unloaded(chunk);
        assert!(links.is_empty());
        assert!(links.dog_poi_sites().is_empty());
    }
}
