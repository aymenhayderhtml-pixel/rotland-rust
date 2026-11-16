//! Map, compass markers, and exploration system (port of scripts/UI/ and scripts/World/ map logic).

mod compass;
mod explore;
mod icons;
mod markers;
mod math;
mod state;
mod tiles;

pub use compass::{
    bearing_to_view_deg, direction_name, marker_x, relative_angle_deg, HALF_VIEW_DEG,
};
pub use explore::{MapExplore, CELLS_PER_CHUNK, CELL_SIZE};
pub use icons::{
    MapIconKind, BOUNTY_COLOR, BUILDING_COLOR, CAMP_COLOR, DIM_COLOR, OUTLINE_COLOR, PLAYER_COLOR,
    SKULL_COLOR, WAYPOINT_COLOR,
};
pub use markers::{collect_markers, letter_for, MapMarker, MapMarkerKind};
pub use math::{screen_to_world, step_zoom, world_to_screen, MAP_DEFAULT_ZOOM, SCALES};
pub use state::{MapBuilding, MapState};
pub use tiles::{
    render_chunk_tiles, CLIFF_COLOR, CLIFF_SLOPE, TEXELS_PER_CHUNK, TEXEL_METRES, UNEXPLORED_COLOR,
};
