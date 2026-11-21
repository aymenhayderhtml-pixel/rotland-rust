//! City map exploration helpers and visibility culling.
//!
//! Provides pure geometric and exploration logic for displaying city streets,
//! camp roads, and district labels on the map screen.

use glam::Vec2;
use world::{CityLayout, District};

use super::explore::{CELL_SIZE, MapExplore};

// data: minimap.json street_color
pub const CITY_STREET_COLOR: [u8; 4] = [69, 69, 74, 255];
// data: minimap.json road_color
pub const CITY_ROAD_COLOR: [u8; 4] = [90, 83, 72, 255];
// data: minimap.json blocked_color
pub const CITY_BLOCKED_COLOR: [u8; 4] = [217, 53, 43, 255];
// data: minimap.json street_min_px
pub const CITY_STREET_MIN_PX: f32 = 2.0;

/// Returns true if all sample cells along the segment from `a` to `b` are revealed in `explore`.
pub fn is_segment_explored(a: Vec2, b: Vec2, explore: &MapExplore) -> bool {
    let d = b - a;
    let len = d.length();
    if len < 1e-4 {
        return explore.is_revealed(MapExplore::cell_of(a));
    }
    let steps = (len / CELL_SIZE).ceil().max(1.0) as usize;
    for i in 0..steps {
        let t = (i as f32 + 0.5) / (steps as f32);
        let pt = a + d * t;
        if !explore.is_revealed(MapExplore::cell_of(pt)) {
            return false;
        }
    }
    true
}

/// Returns true if any sample cell along the segment from `a` to `b` is revealed in `explore`.
pub fn is_segment_any_explored(a: Vec2, b: Vec2, explore: &MapExplore) -> bool {
    let d = b - a;
    let len = d.length();
    if len < 1e-4 {
        return explore.is_revealed(MapExplore::cell_of(a));
    }
    let steps = (len / CELL_SIZE).ceil().max(1.0) as usize;
    for i in 0..steps {
        let t = (i as f32 + 0.5) / (steps as f32);
        let pt = a + d * t;
        if explore.is_revealed(MapExplore::cell_of(pt)) {
            return true;
        }
    }
    false
}

/// Splits a segment into contiguous sub-segments where the underlying cells are explored.
pub fn explored_segment_spans(a: Vec2, b: Vec2, explore: &MapExplore) -> Vec<(Vec2, Vec2)> {
    let d = b - a;
    let len = d.length();
    if len < 1e-4 {
        if explore.is_revealed(MapExplore::cell_of(a)) {
            return vec![(a, b)];
        }
        return Vec::new();
    }
    let steps = (len / CELL_SIZE).ceil().max(1.0) as usize;
    let mut spans = Vec::new();
    let mut start_idx: Option<usize> = None;
    for i in 0..steps {
        let t = (i as f32 + 0.5) / (steps as f32);
        let pt = a + d * t;
        let revealed = explore.is_revealed(MapExplore::cell_of(pt));
        if revealed {
            if start_idx.is_none() {
                start_idx = Some(i);
            }
        } else if let Some(start) = start_idx.take() {
            let t0 = start as f32 / steps as f32;
            let t1 = i as f32 / steps as f32;
            spans.push((a + d * t0, a + d * t1));
        }
    }
    if let Some(start) = start_idx {
        let t0 = start as f32 / steps as f32;
        spans.push((a + d * t0, b));
    }
    spans
}

/// Returns true if the camp road has been discovered (the player has explored any cell along it).
pub fn is_road_discovered(road: &[Vec2], explore: &MapExplore) -> bool {
    if road.is_empty() {
        return false;
    }
    if road.len() == 1 {
        return explore.is_revealed(MapExplore::cell_of(road[0]));
    }
    road.windows(2).any(|w| is_segment_any_explored(w[0], w[1], explore))
}

/// The center point of the city in world XZ coordinates.
pub fn city_center(city: &CityLayout) -> Vec2 {
    city.origin + Vec2::splat(city.size * 0.5)
}

/// Returns true if the city center or any block in the Core district has been explored.
pub fn is_city_center_revealed(city: &CityLayout, explore: &MapExplore) -> bool {
    let center = city_center(city);
    if explore.is_revealed(MapExplore::cell_of(center)) {
        return true;
    }
    is_district_revealed(city, District::Core, explore)
}

/// Returns true if any block in the given district is present and has its center explored.
pub fn is_district_revealed(city: &CityLayout, district: District, explore: &MapExplore) -> bool {
    let block_size = world::WorldGenerator::CHUNK_SIZE;
    city.blocks.iter().any(|b| {
        b.present
            && b.district == district
            && explore.is_revealed(MapExplore::cell_of(
                (b.coord.as_vec2() + Vec2::splat(0.5)) * block_size,
            ))
    })
}

/// Computes the average center point of all present blocks belonging to the specified district.
pub fn district_center(city: &CityLayout, district: District) -> Option<Vec2> {
    let block_size = world::WorldGenerator::CHUNK_SIZE;
    let mut sum = Vec2::ZERO;
    let mut count = 0usize;
    for b in &city.blocks {
        if b.present && b.district == district {
            sum += (b.coord.as_vec2() + Vec2::splat(0.5)) * block_size;
            count += 1;
        }
    }
    if count == 0 {
        None
    } else {
        Some(sum / count as f32)
    }
}

/// Computes the visible world XZ bounding box `(min, max)` for the current map view.
pub fn map_visible_world_bounds(centre: Vec2, scale: f32, view_size: Vec2, margin: f32) -> (Vec2, Vec2) {
    let half_ext = (view_size * 0.5) / scale.max(1e-4) + Vec2::splat(margin);
    (centre - half_ext, centre + half_ext)
}

/// Checks whether a line segment `a..b` intersects or lies within the bounding box `[min, max]`.
pub fn segment_in_bounds(a: Vec2, b: Vec2, min: Vec2, max: Vec2, margin: f32) -> bool {
    let seg_min = a.min(b) - Vec2::splat(margin);
    let seg_max = a.max(b) + Vec2::splat(margin);
    !(seg_max.x < min.x || seg_min.x > max.x || seg_max.y < min.y || seg_min.y > max.y)
}
