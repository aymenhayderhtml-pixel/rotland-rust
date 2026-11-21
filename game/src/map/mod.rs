//! Map, compass markers, and exploration system (port of scripts/UI/ and scripts/World/ map logic).

mod buildings;
mod city;
mod compass;
mod explore;
mod icons;
mod markers;
mod math;
mod minimap;
mod minimap_tiles;
mod state;
mod tiles;

pub use buildings::*;
pub use city::{
    CITY_BLOCKED_COLOR, CITY_ROAD_COLOR, CITY_STREET_COLOR, CITY_STREET_MIN_PX, city_center,
    district_center, explored_segment_spans, is_city_center_revealed, is_district_revealed,
    is_road_discovered, is_segment_any_explored, is_segment_explored, map_visible_world_bounds,
    segment_in_bounds,
};
pub use compass::{
    HALF_VIEW_DEG, bearing_to_view_deg, direction_name, marker_x, relative_angle_deg,
};
pub use explore::{CELL_SIZE, CELLS_PER_CHUNK, MapExplore};
pub use icons::{
    BOUNTY_COLOR, BUILDING_COLOR, CAMP_COLOR, DIM_COLOR, MapIconKind, OUTLINE_COLOR, PLAYER_COLOR,
    SKULL_COLOR, WAYPOINT_COLOR,
};
pub use markers::{MapMarker, MapMarkerKind, collect_markers, letter_for};
pub use math::{MAP_DEFAULT_ZOOM, SCALES, screen_to_world, step_zoom, world_to_screen};
pub use minimap::{
    MinimapCampOutline, MinimapCampSite, MinimapConfig, MinimapCorner, MinimapMarker,
    clip_segment_to_circle, collect_minimap_camp_outlines, collect_minimap_markers,
    collect_minimap_zombies, minimap_chunk_window, minimap_circle, minimap_player_axes,
    world_to_minimap_offset,
};
pub use minimap_tiles::{
    MINIMAP_PATCH_SIZE, MINIMAP_UNEXPLORED_RGB, minimap_patch_bounds, render_minimap_patch,
    sample_chunk_terrain_bilinear, smooth_explored_coverage,
};
pub use state::{MapBuilding, MapState};
pub use tiles::{
    CLIFF_COLOR, CLIFF_SLOPE, TEXEL_METRES, TEXELS_PER_CHUNK, UNEXPLORED_COLOR,
    render_chunk_terrain_rgb, render_chunk_tiles,
};
