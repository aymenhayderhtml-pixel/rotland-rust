//! Per-frame companion update: targeting, follow, guard, stay, combat, and regeneration.

use glam::Vec2;
use glam::Vec3;

use super::geometry::*;
use super::squad::CompanionSquad;
use super::targeting::*;
use super::tuning::*;
use super::types::{CompanionAnim, CompanionEvent, CompanionWorld};
use crate::guild::recruits::{RecruitOrder, RecruitRole};

impl CompanionSquad {
    /// Advances all teammates by `dt` seconds using `move_body(from, wanted_step)` for ground/wall collision.
    pub fn tick(
        &mut self,
        world: &CompanionWorld<'_>,
        move_body: &mut dyn FnMut(Vec3, Vec3) -> Vec3,
        dt: f32,
        events: &mut Vec<CompanionEvent>,
    ) {
        let dt = if dt.is_finite() && dt > 0.0 { dt } else { 0.0 };
        if self.members.is_empty() {
            return;
        }

        let player_fwd_xz = normalize_xz_or(world.player_forward, Vec2::new(0.0, -1.0));
        let player_right_xz = Vec2::new(-player_fwd_xz.y, player_fwd_xz.x);

        let follower_count = self
            .members
            .iter()
            .filter(|m| !m.downed && m.order == RecruitOrder::Follow)
            .count();

        self.positions_scratch.clear();
        for m in &self.members {
            self.positions_scratch.push((m.position, m.downed));
        }

        let cfg = &self.cfg;
        let mut follower_slot = 0usize;

        for (slot, member) in self.members.iter_mut().enumerate() {
            if member.downed {
                member.anim = CompanionAnim::Downed;
                member.target_id = None;
                member.flee_target_id = None;
                continue;
            }

            let my_follow_slot = if member.order == RecruitOrder::Follow {
                let s = follower_slot;
                follower_slot += 1;
                s
            } else {
                0
            };

            member.attack_timer = (member.attack_timer - dt).max(0.0);
            member.in_combat_timer = (member.in_combat_timer - dt).max(0.0);
            member.retarget_timer -= dt;

            // Validate any active target against the live zombie list and order rules.
            let mut active_target = member
                .target_id
                .and_then(|id| find_zombie(world.zombies, id));
            if let Some(t) = active_target
                && !is_target_still_valid(member, t.position, world, cfg)
            {
                active_target = None;
            }
            if active_target.is_none() {
                member.target_id = None;
            }

            let mut flee_target = member
                .flee_target_id
                .and_then(|id| find_zombie(world.zombies, id));
            if let Some(ft) = flee_target
                && xz_dist(member.position, ft.position) > cfg.scavenger_flee_range * FLEE_ESCAPE_MULT
            {
                flee_target = None;
            }
            if flee_target.is_none() {
                member.flee_target_id = None;
                member.cornered_blocked = false;
            }

            // Staggered target search so teammates do not all scan on the same frame.
            if member.retarget_timer <= 0.0 {
                let period = cfg.retarget_seconds.max(0.0);
                member.retarget_timer = if period > 0.0 {
                    (member.retarget_timer + period).clamp(0.0, period)
                } else {
                    0.0
                };

                if member.role == RecruitRole::Scavenger && member.order != RecruitOrder::Stay {
                    flee_target = find_nearest_zombie(
                        world.zombies,
                        member.position,
                        cfg.scavenger_flee_range,
                    );
                    member.flee_target_id = flee_target.map(|z| z.id);
                    if flee_target.is_none() {
                        member.target_id = None;
                        active_target = None;
                    }
                } else if active_target.is_none() {
                    active_target = select_combat_target(member, world, cfg);
                    member.target_id = active_target.map(|z| z.id);
                }
            }

            // Follow teleport: if far beyond teleportDistance and outside player's view, snap near player.
            if member.order == RecruitOrder::Follow {
                let spot_xz = follow_spot(
                    world.player,
                    player_fwd_xz,
                    player_right_xz,
                    my_follow_slot,
                    follower_count,
                    cfg,
                );
                let to_member = xz(member.position) - xz(world.player);
                let dist = to_member.length();
                if dist > cfg.teleport_distance && dist > EPSILON_SQ {
                    let view_dot = (to_member / dist).dot(player_fwd_xz);
                    if view_dot <= TELEPORT_VIEW_DOT {
                        let dest = Vec3::new(spot_xz.x, world.player.y, spot_xz.y);
                        let landed = move_body(dest, Vec3::ZERO);
                        if landed.is_finite() {
                            member.position = landed;
                        }
                        member.target_id = None;
                        member.flee_target_id = None;
                        member.anim = CompanionAnim::Idle;
                        member.yaw = yaw_from_xz(player_fwd_xz, member.yaw);
                        self.positions_scratch[slot].0 = member.position;
                        continue;
                    }
                }
            }

            let sep = compute_separation(
                slot,
                member.position,
                world.player,
                &self.positions_scratch,
            );

            // Scavenger flee vs cornered defense.
            if member.role == RecruitRole::Scavenger
                && member.order != RecruitOrder::Stay
                && let Some(threat) = flee_target
            {
                let anchor = if member.order == RecruitOrder::Guard {
                    world.camp
                } else {
                    world.player
                };
                let dist_to_z = xz_dist(member.position, threat.position);
                let away = normalize_vec2_or(
                    xz(member.position) - xz(threat.position),
                    fallback_dir(slot),
                );
                let to_anchor_vec = xz(anchor) - xz(member.position);
                let dist_to_anchor = to_anchor_vec.length();
                let to_anchor = normalize_vec2_or(to_anchor_vec, away);

                let cornered = dist_to_z <= cfg.attack_range
                    && (dist_to_anchor <= PLAYER_SEPARATION + 0.5 // new: DECISIONS 2026-11-23
                        || to_anchor.dot(away) < -0.5 // new: DECISIONS 2026-11-23
                        || member.cornered_blocked);

                if cornered {
                    active_target = Some(threat);
                    member.target_id = Some(threat.id);
                } else {
                    let flee_dir = if to_anchor.dot(away) > -0.2 { // new: DECISIONS 2026-11-23
                        normalize_vec2_or(to_anchor + away, away)
                    } else {
                        away
                    };
                    let running = dist_to_z < cfg.scavenger_flee_range * FLEE_SPRINT_RATIO;
                    let speed = if running { cfg.run_speed } else { cfg.walk_speed };
                    let raw_step = flee_dir * (speed * dt) + sep * (cfg.walk_speed * dt);
                    let step = clamp_guard_step(member, raw_step, world.camp, cfg);
                    let before_dist = dist_to_z;
                    apply_step(member, step, move_body);
                    let after_dist = xz_dist(member.position, threat.position);
                    member.cornered_blocked =
                        dist_to_z <= cfg.attack_range && after_dist <= before_dist + 0.01; // new: DECISIONS 2026-11-23
                    member.yaw = yaw_from_xz(flee_dir, member.yaw);
                    member.anim = if running {
                        CompanionAnim::Run
                    } else {
                        CompanionAnim::Walk
                    };
                    member.in_combat_timer = COMBAT_HEAL_DELAY;
                    self.positions_scratch[slot].0 = member.position;
                    continue;
                }
            }

            // Active combat: attack if in range, otherwise close distance (except Stay).
            if let Some(target) = active_target {
                let to_target = xz(target.position) - xz(member.position);
                let dist = to_target.length();
                member.in_combat_timer = COMBAT_HEAL_DELAY;
                member.yaw = yaw_from_xz(to_target, member.yaw);

                if dist <= cfg.attack_range {
                    member.anim = CompanionAnim::Attack;
                    if member.attack_timer <= 0.0 {
                        let damage = cfg.attack_damage(member.role, member.level);
                        events.push(CompanionEvent::Hit {
                            companion: member.id.clone(),
                            zombie_id: target.id,
                            damage,
                        });
                        member.attack_timer = cfg.attack_cooldown.max(0.0);
                    }
                    if member.order != RecruitOrder::Stay && sep.length_squared() > EPSILON_SQ {
                        let step = clamp_guard_step(
                            member,
                            sep * (cfg.walk_speed * dt),
                            world.camp,
                            cfg,
                        );
                        apply_step(member, step, move_body);
                    }
                    self.positions_scratch[slot].0 = member.position;
                    continue;
                } else if member.order != RecruitOrder::Stay {
                    let running = dist > cfg.run_distance;
                    let speed = if running { cfg.run_speed } else { cfg.walk_speed };
                    let desired_stop = (cfg.attack_range * 0.85).max(0.0); // new: DECISIONS 2026-11-23
                    let move_dist = (dist - desired_stop).max(0.0).min(speed * dt);
                    let dir = normalize_vec2_or(to_target, fallback_dir(slot));
                    let raw_step = dir * move_dist + sep * (cfg.walk_speed * dt);
                    let step = clamp_guard_step(member, raw_step, world.camp, cfg);
                    apply_step(member, step, move_body);
                    member.anim = if running {
                        CompanionAnim::Run
                    } else {
                        CompanionAnim::Walk
                    };
                    self.positions_scratch[slot].0 = member.position;
                    continue;
                } else {
                    member.target_id = None;
                }
            }

            // Non-combat movement by order.
            match member.order {
                RecruitOrder::Follow => {
                    let spot = follow_spot(
                        world.player,
                        player_fwd_xz,
                        player_right_xz,
                        my_follow_slot,
                        follower_count,
                        cfg,
                    );
                    let to_spot = spot - xz(member.position);
                    let dist = to_spot.length();
                    let dist_to_player = xz_dist(member.position, world.player);
                    if dist > STOP_RADIUS {
                        let running = dist > cfg.run_distance || dist_to_player > cfg.run_distance;
                        let speed = if running { cfg.run_speed } else { cfg.walk_speed };
                        let dir = to_spot / dist;
                        let step = dir * (speed * dt).min(dist) + sep * (cfg.walk_speed * dt);
                        apply_step(member, step, move_body);
                        member.yaw = yaw_from_xz(dir, member.yaw);
                        member.anim = if running {
                            CompanionAnim::Run
                        } else {
                            CompanionAnim::Walk
                        };
                    } else if sep.length_squared() > EPSILON_SQ {
                        apply_step(member, sep * (cfg.walk_speed * dt), move_body);
                        member.anim = CompanionAnim::Walk;
                    } else {
                        member.yaw = yaw_from_xz(player_fwd_xz, member.yaw);
                        member.anim = CompanionAnim::Idle;
                    }
                }
                RecruitOrder::Guard => {
                    let patrol_r = (cfg.guard_radius * GUARD_PATROL_RADIUS_RATIO).max(0.0);
                    let dist_from_camp = xz_dist(member.position, world.camp);
                    if dist_from_camp > cfg.guard_radius {
                        let to_camp = xz(world.camp) - xz(member.position);
                        let dir = normalize_vec2_or(to_camp, fallback_dir(slot));
                        let step = clamp_guard_step(
                            member,
                            dir * (cfg.walk_speed * dt).min(dist_from_camp)
                                + sep * (cfg.walk_speed * dt),
                            world.camp,
                            cfg,
                        );
                        apply_step(member, step, move_body);
                        member.yaw = yaw_from_xz(dir, member.yaw);
                        member.anim = CompanionAnim::Walk;
                    } else if member.patrol_pause > 0.0 {
                        member.patrol_pause = (member.patrol_pause - dt).max(0.0);
                        member.anim = CompanionAnim::Idle;
                    } else {
                        let waypoint = xz(world.camp)
                            + patrol_offset(slot, member.patrol_step, patrol_r);
                        let to_wp = waypoint - xz(member.position);
                        let dist = to_wp.length();
                        if dist > STOP_RADIUS {
                            let speed = cfg.walk_speed * GUARD_PATROL_SPEED_RATIO;
                            let dir = to_wp / dist;
                            let step = clamp_guard_step(
                                member,
                                dir * (speed * dt).min(dist) + sep * (speed * dt),
                                world.camp,
                                cfg,
                            );
                            apply_step(member, step, move_body);
                            member.yaw = yaw_from_xz(dir, member.yaw);
                            member.anim = CompanionAnim::Walk;
                        } else {
                            member.patrol_step = (member.patrol_step + 1) % PATROL_WAYPOINTS;
                            member.patrol_pause = cfg.patrol_pause_seconds.max(0.0);
                            member.anim = CompanionAnim::Idle;
                        }
                    }
                }
                RecruitOrder::Stay => {
                    let home = xz(world.camp) + camp_slot_offset(slot);
                    let to_home = home - xz(member.position);
                    let dist = to_home.length();
                    if dist > STOP_RADIUS {
                        let dir = to_home / dist;
                        let step = dir * (cfg.walk_speed * dt).min(dist);
                        apply_step(member, step, move_body);
                        member.yaw = yaw_from_xz(dir, member.yaw);
                        member.anim = CompanionAnim::Walk;
                    } else {
                        member.anim = CompanionAnim::Idle;
                    }
                }
            }

            // Out-of-combat passive regeneration.
            if member.in_combat_timer <= 0.0
                && member.target_id.is_none()
                && member.flee_target_id.is_none()
                && member.health < member.max_health
            {
                member.health =
                    (member.health + cfg.heal_per_second.max(0.0) * dt).min(member.max_health);
            }

            self.positions_scratch[slot].0 = member.position;
        }
    }

}
