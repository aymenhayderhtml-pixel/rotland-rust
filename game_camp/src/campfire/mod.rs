//! Campfire system: campfire instances, placement, burning logic, visuals, and persistence.
//! Port of `scripts/World/Campfire.cs`, `CampfirePlacer.cs`, `StoredCampfire.cs`, and `CampfireVisuals.cs`.

mod fire;
mod manager;
mod placer;
mod save;
mod visuals;

pub use fire::{
    Campfire, CampfireInteractResult, BURN_HOURS, CAMPFIRE_RANGE, ROOT_LIFT, SECONDS_PER_HOUR,
    WOOD_ID,
};
pub use manager::{
    find_lit_near, near_campfire, settlement_campfire_from_markers, CampfireManager,
};
pub use placer::{
    is_flat, place_campfire, PlaceCampfireError, CAMPFIRE_KIT_ID, CAMPFIRE_SPACING, PLACE_DISTANCE,
};
pub use save::{
    from_save_campfires, to_save_campfires, world_to_chunk, StoredCampfire, CAMPFIRES_KEY,
    WORLDSTATE_SAVE_ID,
};
pub use visuals::{
    CampfireVisuals, BASE_LIGHT_ENERGY, OMNI_RANGE, RING_RADIUS, WARM_LIGHT_COLOR,
};
