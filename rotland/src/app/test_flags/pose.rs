//! Teleport poses for the dev flags: spot near a rescue survivor, town, trailhead and river.

use glam::{Vec2, Vec3};
use game::rescue_place::RescueSpot;

/// Distance in metres to place the player from the nearest unrescued survivor for `--tp-rescue`.
const RESCUE_TP_DISTANCE_M: f32 = 6.0; // new: DECISIONS 2026-11-25
/// Distance in metres to place the player from town/trailhead/river targets.
pub(super) const WORLD_TP_DISTANCE_M: f32 = 25.0; // new: DECISIONS 2026-11-25

/// Computes the player position (`6 m` away) and Godot yaw facing the nearest unrescued survivor.
pub fn tp_rescue_pose(player_pos: Vec3, spots: &[RescueSpot]) -> Option<(Vec3, f32)> {
    let nearest = spots.iter().min_by(|a, b| {
        let da = (a.position.x - player_pos.x).powi(2) + (a.position.z - player_pos.z).powi(2);
        let db = (b.position.x - player_pos.x).powi(2) + (b.position.z - player_pos.z).powi(2);
        da.total_cmp(&db)
    })?;

    let mut from_spot = Vec3::new(
        player_pos.x - nearest.position.x,
        0.0,
        player_pos.z - nearest.position.z,
    );
    if from_spot.length_squared() <= 1e-4 {
        from_spot = Vec3::Z;
    } else {
        from_spot = from_spot.normalize();
    }

    let pos = nearest.position + from_spot * RESCUE_TP_DISTANCE_M;
    let to_spot = nearest.position - pos;
    let yaw = (-to_spot.x).atan2(-to_spot.z);
    Some((pos, yaw))
}

/// Computes `(pos_xz, godot_yaw)` 25 m from `town.center` along `-street_dir`, facing the town centre.
pub fn town_tp_pose(town: &world::towns::TownSite) -> (Vec2, f32) {
    let dir = town.street_dir.normalize_or(Vec2::X);
    let pos = town.center - dir * WORLD_TP_DISTANCE_M;
    let yaw = (-dir.x).atan2(-dir.y);
    (pos, yaw)
}

/// Computes `(pos_xz, godot_yaw)` 25 m before `trailhead`, facing outward along the nearest road.
pub fn trailhead_tp_pose(trailhead: Vec2, roads: &[world::roads::Road], camp: Vec2) -> (Vec2, f32) {
    let outward = (trailhead - camp).normalize_or(Vec2::X);
    let mut best_dist = f32::INFINITY;
    let mut road_dir = outward;
    for road in roads {
        if road.points.len() < 2 {
            continue;
        }
        let d_start = road.points[0].distance_squared(trailhead);
        if d_start < best_dist {
            best_dist = d_start;
            road_dir = (road.points[1] - road.points[0]).normalize_or(outward);
        }
        let last = road.points.len() - 1;
        let d_end = road.points[last].distance_squared(trailhead);
        if d_end < best_dist {
            best_dist = d_end;
            road_dir = (road.points[last - 1] - road.points[last]).normalize_or(outward);
        }
    }
    if road_dir.dot(outward) < 0.0 {
        road_dir = -road_dir;
    }
    let pos = trailhead - road_dir * WORLD_TP_DISTANCE_M;
    let yaw = (-road_dir.x).atan2(-road_dir.y);
    (pos, yaw)
}

/// Computes `(pos_xz, godot_yaw)` on the bank 25 m from river `river`'s midpoint, facing the water.
pub fn river_tp_pose(river: &world::rivers::River) -> Option<(Vec2, f32)> {
    if river.points.len() < 2 {
        return None;
    }
    let mid = river.points.len() / 2;
    let p_prev = river.points[mid.saturating_sub(1)];
    let p_next = river.points[(mid + 1).min(river.points.len() - 1)];
    let tangent = (p_next - p_prev).normalize_or(Vec2::X);
    let bank_normal = Vec2::new(-tangent.y, tangent.x);
    let center = river.points[mid];
    let pos = center + bank_normal * WORLD_TP_DISTANCE_M;
    let to_water = -bank_normal;
    let yaw = (-to_water.x).atan2(-to_water.y);
    Some((pos, yaw))
}
