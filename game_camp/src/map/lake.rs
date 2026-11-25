//! Lake map geometry, exploration checks, and visualization data.
//!
//! Provides pure map data extracted from a [`world::LakeLayout`]: simplified shore
//! polygon, dock line segments, village lot rectangles, fishing spots, and label position.

use glam::Vec2;
use world::{FishingSpotKind, LakeLayout};

use super::explore::{CELLS_PER_CHUNK, CELL_SIZE, MapExplore};

// data: map.json max_shore_points
pub const MAX_SHORE_POINTS: usize = 64;

// data: map.json water_color
pub const LAKE_WATER_COLOR: [u8; 4] = [66, 135, 245, 255];

// data: map.json shore_color
pub const LAKE_SHORE_COLOR: [u8; 4] = [27, 79, 138, 255];

// data: map.json shore_width
pub const LAKE_SHORE_WIDTH: f32 = 2.0;

// data: map.json dock_color
pub const LAKE_DOCK_COLOR: [u8; 4] = [122, 82, 48, 255];

// data: map.json dock_width
pub const LAKE_DOCK_WIDTH: f32 = 2.5;

// data: map.json village_lot_color
pub const LAKE_VILLAGE_LOT_COLOR: [u8; 4] = [120, 122, 125, 255];

// data: map.json village_lot_outline
pub const LAKE_VILLAGE_LOT_OUTLINE: [u8; 4] = [74, 76, 78, 255];

// data: map.json fishing_spot_color
pub const LAKE_FISHING_SPOT_COLOR: [u8; 4] = [240, 180, 41, 255];

// data: map.json fishing_spot_radius
pub const LAKE_FISHING_SPOT_RADIUS: f32 = 3.5;

// data: map.json label_color
pub const LAKE_LABEL_COLOR: [u8; 4] = [245, 240, 225, 255];

// data: map.json label_size
pub const LAKE_LABEL_SIZE: f32 = 16.0;

/// A dock line segment on the map.
#[derive(Debug, Clone, PartialEq)]
pub struct LakeMapDock {
    pub start: Vec2,
    pub end: Vec2,
}

/// A fishing village lot rectangle on the map with 4 corner vertices.
#[derive(Debug, Clone, PartialEq)]
pub struct LakeMapLot {
    pub corners: [Vec2; 4],
    pub center: Vec2,
}

/// A fishing spot point on the map.
#[derive(Debug, Clone, PartialEq)]
pub struct LakeMapFishingSpot {
    pub position: Vec2,
    pub kind: FishingSpotKind,
}

/// Pure geometric map data extracted from a [`LakeLayout`].
#[derive(Debug, Clone, PartialEq)]
pub struct LakeMapData {
    pub center: Vec2,
    pub radius: Vec2,
    pub shore: Vec<Vec2>,
    pub docks: Vec<LakeMapDock>,
    pub village_lots: Vec<LakeMapLot>,
    pub fishing_spots: Vec<LakeMapFishingSpot>,
    pub label_pos: Vec2,
    pub water_bounds: (Vec2, Vec2),
}

impl LakeMapData {
    /// Builds map data from a [`LakeLayout`], simplifying the shore polygon to at most [`MAX_SHORE_POINTS`].
    pub fn from_layout(layout: &LakeLayout) -> Self {
        let shore = simplify_shore(&layout.shore, MAX_SHORE_POINTS);
        let docks = layout
            .docks
            .iter()
            .map(|d| LakeMapDock {
                start: d.start,
                end: d.end,
            })
            .collect();
        let village_lots = layout
            .village_lots
            .iter()
            .map(|lot| LakeMapLot {
                corners: lot.corners(),
                center: lot.center,
            })
            .collect();
        let fishing_spots = layout
            .fishing_spots
            .iter()
            .map(|s| LakeMapFishingSpot {
                position: s.position,
                kind: s.kind,
            })
            .collect();

        Self {
            center: layout.center,
            radius: layout.radius,
            shore,
            docks,
            village_lots,
            fishing_spots,
            label_pos: layout.center,
            water_bounds: layout.water_bounds(),
        }
    }

    /// Returns true if the shore polygon forms a closed polygon with at least 3 vertices.
    pub fn is_polygon_closed(&self) -> bool {
        self.shore.len() >= 3
    }
}

/// Simplifies a closed shore polygon to at most `max_points` points by uniform angular sub-sampling.
pub fn simplify_shore(shore: &[Vec2], max_points: usize) -> Vec<Vec2> {
    if shore.len() <= max_points || max_points == 0 {
        return shore.to_vec();
    }
    let n = shore.len();
    let mut out = Vec::with_capacity(max_points);
    for i in 0..max_points {
        let idx = (i * n) / max_points;
        out.push(shore[idx]);
    }
    out
}

/// Returns true if any part of the lake, docks, fishing village, or fishing spots has been discovered.
pub fn is_lake_discovered(lake: &LakeMapData, explore: &MapExplore) -> bool {
    if explore.is_revealed(MapExplore::cell_of(lake.center)) {
        return true;
    }
    if lake.docks.iter().any(|d| {
        explore.is_revealed(MapExplore::cell_of(d.start))
            || explore.is_revealed(MapExplore::cell_of(d.end))
    }) {
        return true;
    }
    if lake
        .village_lots
        .iter()
        .any(|lot| explore.is_revealed(MapExplore::cell_of(lot.center)))
    {
        return true;
    }
    if lake
        .fishing_spots
        .iter()
        .any(|s| explore.is_revealed(MapExplore::cell_of(s.position)))
    {
        return true;
    }
    if lake
        .shore
        .iter()
        .any(|&pt| explore.is_revealed(MapExplore::cell_of(pt)))
    {
        return true;
    }

    // Check revealed chunks overlapping the lake's water bounds
    let (min_b, max_b) = lake.water_bounds;
    let min_c = MapExplore::chunk_of_cell(MapExplore::cell_of(min_b));
    let max_c = MapExplore::chunk_of_cell(MapExplore::cell_of(max_b));
    for cz in min_c.y..=max_c.y {
        for cx in min_c.x..=max_c.x {
            let chunk = glam::IVec2::new(cx, cz);
            if explore.mask_of(chunk) != 0 {
                for lz in 0..CELLS_PER_CHUNK {
                    for lx in 0..CELLS_PER_CHUNK {
                        let cell = glam::IVec2::new(
                            cx * CELLS_PER_CHUNK + lx,
                            cz * CELLS_PER_CHUNK + lz,
                        );
                        if explore.is_revealed(cell) {
                            let pt = (cell.as_vec2() + Vec2::splat(0.5)) * CELL_SIZE;
                            if pt.x >= min_b.x && pt.x <= max_b.x && pt.y >= min_b.y && pt.y <= max_b.y {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}
