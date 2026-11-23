pub mod config;
pub mod session;
pub mod state;

pub use config::FishingConfig;
pub use session::{FishingEvent, FishingInput, FishingSession};
pub use state::{FishingMachine, FishingPhase};

use glam::{Vec2, Vec3};
use world::{LakeLayout, LakeZone};

/// Checks whether the player can fish at the given location (not swimming in deep water,
/// within `max_dist` of water, and facing it).
pub fn can_fish_at(
    player_pos: Vec3,
    player_yaw: f32,
    lake: &LakeLayout,
    max_dist: f32,
) -> bool {
    let p = Vec2::new(player_pos.x, player_pos.z);
    if lake.zone_at(p) == LakeZone::Deep && !crate::swimming::is_on_dock(lake, p) {
        return false;
    }

    let forward_2d = Vec2::new(-player_yaw.sin(), -player_yaw.cos());
    // new: DECISIONS 2026-11-23
    let steps = 4;
    for i in 1..=steps {
        let d = (max_dist / steps as f32) * (i as f32);
        let check_pos = p + forward_2d * d;
        let zone = lake.zone_at(check_pos);
        if zone == LakeZone::Shallow || zone == LakeZone::Deep {
            return true;
        }
    }
    false
}
