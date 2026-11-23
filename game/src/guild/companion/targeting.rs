//! Zombie target lookup and validation for companion combat.

use glam::Vec3;

use super::config::CompanionConfig;
use super::geometry::*;
use super::squad::MemberBody;
use super::tuning::*;
use super::types::{CompanionTarget, CompanionWorld};
use crate::guild::recruits::{RecruitOrder, RecruitRole};

pub(super) fn find_zombie(zombies: &[CompanionTarget], id: u32) -> Option<CompanionTarget> {
    zombies.iter().copied().find(|z| z.id == id)
}

pub(super) fn find_nearest_zombie(
    zombies: &[CompanionTarget],
    origin: Vec3,
    max_range: f32,
) -> Option<CompanionTarget> {
    let max_sq = max_range.max(0.0) * max_range.max(0.0);
    let mut best: Option<(CompanionTarget, f32)> = None;
    for &z in zombies {
        let d_sq = xz(z.position).distance_squared(xz(origin));
        if d_sq <= max_sq {
            let better = match best {
                None => true,
                Some((b, b_sq)) => d_sq < b_sq || (d_sq == b_sq && z.id < b.id),
            };
            if better {
                best = Some((z, d_sq));
            }
        }
    }
    best.map(|(z, _)| z)
}

pub(super) fn select_combat_target(
    member: &MemberBody,
    world: &CompanionWorld<'_>,
    cfg: &CompanionConfig,
) -> Option<CompanionTarget> {
    match member.order {
        RecruitOrder::Stay => find_nearest_zombie(world.zombies, member.position, cfg.attack_range),
        RecruitOrder::Guard => {
            let max_r = cfg.guard_radius.min(cfg.guard_leash).max(0.0);
            let max_sq = max_r * max_r;
            let mut best: Option<(CompanionTarget, f32)> = None;
            for &z in world.zombies {
                if xz(z.position).distance_squared(xz(world.camp)) <= max_sq {
                    let d_sq = xz(z.position).distance_squared(xz(member.position));
                    let better = match best {
                        None => true,
                        Some((b, b_sq)) => d_sq < b_sq || (d_sq == b_sq && z.id < b.id),
                    };
                    if better {
                        best = Some((z, d_sq));
                    }
                }
            }
            best.map(|(z, _)| z)
        }
        RecruitOrder::Follow => match member.role {
            RecruitRole::Fighter => {
                let fight_sq = cfg.fight_range.max(0.0) * cfg.fight_range.max(0.0);
                let mut best: Option<(CompanionTarget, f32)> = None;
                for &z in world.zombies {
                    let d_self = xz(z.position).distance_squared(xz(member.position));
                    let d_player = xz(z.position).distance_squared(xz(world.player));
                    if d_self <= fight_sq || d_player <= fight_sq {
                        let better = match best {
                            None => true,
                            Some((b, b_sq)) => d_self < b_sq || (d_self == b_sq && z.id < b.id),
                        };
                        if better {
                            best = Some((z, d_self));
                        }
                    }
                }
                best.map(|(z, _)| z)
            }
            RecruitRole::Guard => {
                let near_player = (cfg.fight_range * GUARD_FOLLOW_ENGAGE_RATIO).max(0.0);
                let near_player_sq = near_player * near_player;
                let atk_sq = cfg.attack_range.max(0.0) * cfg.attack_range.max(0.0);
                let mut best: Option<(CompanionTarget, f32)> = None;
                for &z in world.zombies {
                    let d_player = xz(z.position).distance_squared(xz(world.player));
                    let d_self = xz(z.position).distance_squared(xz(member.position));
                    if d_player <= near_player_sq || d_self <= atk_sq {
                        let better = match best {
                            None => true,
                            Some((b, b_sq)) => d_player < b_sq || (d_player == b_sq && z.id < b.id),
                        };
                        if better {
                            best = Some((z, d_player));
                        }
                    }
                }
                best.map(|(z, _)| z)
            }
            RecruitRole::Scavenger => None,
        },
    }
}

pub(super) fn is_target_still_valid(
    member: &MemberBody,
    target_pos: Vec3,
    world: &CompanionWorld<'_>,
    cfg: &CompanionConfig,
) -> bool {
    match member.order {
        RecruitOrder::Stay => xz_dist(member.position, target_pos) <= cfg.attack_range,
        RecruitOrder::Guard => xz_dist(world.camp, target_pos) <= cfg.guard_leash,
        RecruitOrder::Follow => {
            let max_r = cfg.fight_range * CHASE_DISENGAGE_MULT;
            xz_dist(world.player, target_pos) <= max_r
                || xz_dist(member.position, target_pos) <= max_r
        }
    }
}
