//! Map marker collection and building letter helpers.
//!
//! Port of Godot C# scripts/UI/MapMarkers.cs.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::bounties::{BountyDef, CLEAR_POI};
use super::state::MapState;

// Godot: MapMarkers.cs MapMarkerKind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapMarkerKind {
    Camp,
    Building,
    Bounty,
    Death,
    Waypoint,
}

// Godot: MapMarkers.cs MapMarker
/// One thing to draw on the map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapMarker {
    pub kind: MapMarkerKind,
    pub position: Vec2,
    pub building_type: String,
    pub dim: bool,
}

impl MapMarker {
    pub fn new(kind: MapMarkerKind, position: Vec2, building_type: impl Into<String>, dim: bool) -> Self {
        Self {
            kind,
            position,
            building_type: building_type.into(),
            dim,
        }
    }
}

/// The letter drawn inside a building's circle.
// Godot: MapMarkers.cs LetterFor
pub fn letter_for(building_type: &str) -> &'static str {
    match building_type {
        "cabin" => "C",
        "house_small" => "H",
        "gas_station" => "G",
        "barn" => "B",
        "shed" => "S",
        "watchtower" => "T",
        _ => "?",
    }
}

/// Pure: turns the map's memory, camps, and active bounties into markers to draw.
// Godot: MapMarkers.cs Collect
pub fn collect_markers(
    state: &MapState,
    camps: impl IntoIterator<Item = Vec2>,
    bounties: &[BountyDef],
) -> Vec<MapMarker> {
    let mut markers = Vec::new();

    for camp in camps {
        markers.push(MapMarker::new(MapMarkerKind::Camp, camp, "", false));
    }

    for building in &state.found {
        markers.push(MapMarker::new(
            MapMarkerKind::Building,
            building.position,
            building.building_type.clone(),
            building.looted,
        ));
    }

    for bounty in bounties {
        if bounty.bounty_type == CLEAR_POI && !bounty.is_done() {
            markers.push(MapMarker::new(
                MapMarkerKind::Bounty,
                bounty.target_xz(),
                "",
                false,
            ));
        }
    }

    if let Some(death) = state.death_spot {
        markers.push(MapMarker::new(MapMarkerKind::Death, death, "", false));
    }

    if let Some(waypoint) = state.waypoint {
        markers.push(MapMarker::new(MapMarkerKind::Waypoint, waypoint, "", false));
    }

    markers
}
