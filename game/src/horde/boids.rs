//! Flocking and steering calculations for horde leaders and followers.

use glam::Vec3;

/// Flat (XZ) unit vector of `v`, or `Vec3::ZERO` if too short.
pub fn flat_unit(v: Vec3) -> Vec3 {
    let flat = Vec3::new(v.x, 0.0, v.z);
    let len = (flat.x * flat.x + flat.z * flat.z).sqrt();
    if len < 1e-4 {
        Vec3::ZERO
    } else {
        flat / len
    }
}

/// Computes desired velocity for a horde leader.
pub fn steer_leader(
    leader_pos: Vec3,
    target_pos: Option<Vec3>,
    speed: f32,
    flow_dir_at: impl Fn(Vec3) -> Option<Vec3>,
) -> Vec3 {
    if let Some(dir) = flow_dir_at(leader_pos) {
        let u = flat_unit(dir);
        if u != Vec3::ZERO {
            return u * speed;
        }
    }
    if let Some(target) = target_pos {
        let to_target = target - leader_pos;
        let u = flat_unit(to_target);
        if u != Vec3::ZERO {
            return u * speed;
        }
    }
    Vec3::ZERO
}

/// Computes desired velocity for a follower relative to the leader.
/// Uses O(1) leader-offset calculation for separation, cohesion, and alignment.
#[allow(clippy::too_many_arguments)]
pub fn steer_follower(
    member_pos: Vec3,
    leader_pos: Vec3,
    leader_dir: Vec3,
    spacing: f32,
    max_distance: f32,
    speed: f32,
    cohesion_weight: f32,
    separation_weight: f32,
    alignment_weight: f32,
) -> Vec3 {
    let delta = member_pos - leader_pos;
    let dist = (delta.x * delta.x + delta.z * delta.z).sqrt();

    // Alignment: align with leader's heading
    let alignment = if leader_dir != Vec3::ZERO {
        flat_unit(leader_dir)
    } else {
        Vec3::ZERO
    };

    // Separation: push away if closer than desired spacing
    let separation = if dist < spacing && dist > 1e-4 {
        flat_unit(delta) * (1.0 - dist / spacing)
    } else if dist <= 1e-4 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        Vec3::ZERO
    };

    // Cohesion: pull towards leader if further than spacing
    let cohesion = if dist > spacing {
        let pull = ((dist - spacing) / spacing).min(2.0);
        let base = flat_unit(-delta) * pull;
        if dist > max_distance {
            base * 2.0
        } else {
            base
        }
    } else {
        Vec3::ZERO
    };

    let force = alignment * alignment_weight + separation * separation_weight + cohesion * cohesion_weight;
    let dir = flat_unit(force);
    if dir == Vec3::ZERO {
        return Vec3::ZERO;
    }

    // Speed adjustment: catch up if falling behind, ease up if ahead
    let speed_mult = if dist > spacing * 2.0 {
        1.5 // data: director.json maxActiveMultiplier
    } else if dist < spacing * 0.5 {
        0.8 // data: director.json maxActiveMultiplier
    } else {
        1.0
    };

    dir * (speed * speed_mult)
}
