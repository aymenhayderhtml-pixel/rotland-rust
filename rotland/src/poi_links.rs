//! Glue linking world POIs to gameplay systems (DogPoiSite list and map building notes).
//!
//! Port of Godot scripts/Dogs/ stray lot queries and scripts/World/PoiManager.cs / WorldMap.cs.

use glam::{Vec2, Vec3};
use game::{DogPoiSite, MapState};
use world::{LootSpot, PoiConfigData, PoiGeometry, PoiInstance};

/// Converts a world PoiInstance into a game DogPoiSite for stray dog spawning checks.
// Godot: StrayDogSpawner.cs
pub fn build_dog_poi_site(poi: &PoiInstance, poi_config: &PoiConfigData) -> DogPoiSite {
    let half_depth = poi_config.get(&poi.building_id).map_or(0.0, |d| d.half_depth());
    DogPoiSite::new(&poi.id, &poi.building_id, poi.position, poi.yaw, half_depth)
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
    use std::collections::HashMap;

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
            zombie_models: Vec::new(),
            wake_height_m: 0.0,
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
}
