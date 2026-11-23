//! Pure 2D/3D helpers: planar distance, yaw, follow spots, separation and ground step clamping.

use std::f32::consts::FRAC_PI_2;

use glam::{Vec2, Vec3};

use super::squad::MemberBody;
use super::tuning::*;
use crate::guild::recruits::RecruitOrder;
use super::config::CompanionConfig;

pub(super) fn xz(v: Vec3) -> Vec2 {
    Vec2::new(v.x, v.z)
}

pub(super) fn xz_dist(a: Vec3, b: Vec3) -> f32 {
    xz(a).distance(xz(b))
}

pub(super) fn normalize_xz_or(v: Vec3, fallback: Vec2) -> Vec2 {
    normalize_vec2_or(xz(v), fallback)
}

pub(super) fn normalize_vec2_or(v: Vec2, fallback: Vec2) -> Vec2 {
    let len_sq = v.length_squared();
    if len_sq.is_finite() && len_sq > EPSILON_SQ {
        v / len_sq.sqrt()
    } else {
        fallback
    }
}

pub(super) fn yaw_from_xz(dir_xz: Vec2, fallback: f32) -> f32 {
    let len_sq = dir_xz.length_squared();
    if len_sq.is_finite() && len_sq > EPSILON_SQ {
        (-dir_xz.x).atan2(-dir_xz.y)
    } else {
        fallback
    }
}

pub(super) fn fallback_dir(slot: usize) -> Vec2 {
    let angle = slot as f32 * FRAC_PI_2;
    Vec2::new(angle.cos(), angle.sin())
}

pub(super) fn camp_slot_offset(slot: usize) -> Vec2 {
    let angle = slot as f32 * FRAC_PI_2;
    Vec2::new(angle.cos(), angle.sin()) * CAMP_SLOT_RADIUS
}

pub(super) fn patrol_offset(slot: usize, step: usize, radius: f32) -> Vec2 {
    let angle = (slot + step) as f32 * FRAC_PI_2;
    Vec2::new(angle.cos(), angle.sin()) * radius
}

pub(super) fn follow_spot(
    player: Vec3,
    fwd: Vec2,
    right: Vec2,
    slot: usize,
    _total: usize,
    cfg: &CompanionConfig,
) -> Vec2 {
    // Beside the player, not behind: behind puts them between the camera and the player.
    let side = if slot.is_multiple_of(2) { 1.0 } else { -1.0 }; // new: DECISIONS 2026-11-23
    let rank = (slot / 2) as f32;
    let lateral = side * cfg.follow_distance;
    let back = cfg.follow_distance * FOLLOW_BACK_FRACTION + rank * cfg.follow_spread;
    xz(player) - fwd * back + right * lateral
}
pub(super) fn compute_separation(
    self_slot: usize,
    self_pos: Vec3,
    player_pos: Vec3,
    others: &[(Vec3, bool)],
) -> Vec2 {
    let mut push = Vec2::ZERO;
    let my_xz = xz(self_pos);

    let from_player = my_xz - xz(player_pos);
    let p_dist = from_player.length();
    if p_dist < PLAYER_SEPARATION {
        let dir = if p_dist > EPSILON_SQ {
            from_player / p_dist
        } else {
            fallback_dir(self_slot)
        };
        let strength = (PLAYER_SEPARATION - p_dist) / PLAYER_SEPARATION;
        push += dir * strength;
    }

    for (j, &(other_pos, downed)) in others.iter().enumerate() {
        if j == self_slot || downed {
            continue;
        }
        let diff = my_xz - xz(other_pos);
        let d = diff.length();
        if d < TEAMMATE_SEPARATION {
            let dir = if d > EPSILON_SQ {
                diff / d
            } else if self_slot < j {
                Vec2::X
            } else {
                -Vec2::X
            };
            let strength = (TEAMMATE_SEPARATION - d) / TEAMMATE_SEPARATION;
            push += dir * strength;
        }
    }

    let len_sq = push.length_squared();
    if len_sq > 1.0 {
        push / len_sq.sqrt()
    } else {
        push
    }
}

pub(super) fn clamp_guard_step(
    member: &MemberBody,
    step_xz: Vec2,
    camp: Vec3,
    cfg: &CompanionConfig,
) -> Vec2 {
    if member.order != RecruitOrder::Guard {
        return step_xz;
    }
    let next_xz = xz(member.position) + step_xz;
    let from_camp = next_xz - xz(camp);
    let dist = from_camp.length();
    let leash = cfg.guard_leash.max(0.0);
    if dist > leash && dist > EPSILON_SQ {
        let clamped = xz(camp) + (from_camp / dist) * leash;
        clamped - xz(member.position)
    } else {
        step_xz
    }
}

pub(super) fn apply_step(
    member: &mut MemberBody,
    step_xz: Vec2,
    move_body: &mut dyn FnMut(Vec3, Vec3) -> Vec3,
) {
    if !step_xz.is_finite() {
        return;
    }
    let wanted = Vec3::new(step_xz.x, 0.0, step_xz.y);
    let next = move_body(member.position, wanted);
    if next.is_finite() {
        member.position = next;
    }
}
