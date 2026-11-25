//! Teammate runtime bodies and the squad controller: sync with guild, damage, morning, saving.

use glam::Vec3;

use super::config::CompanionConfig;
use super::geometry::camp_slot_offset;
use super::tuning::*;
use super::types::{CompanionAnim, CompanionDraw, CompanionEvent};
use crate::guild::recruits::{GuildRecruit, RecruitOrder, RecruitRole, RecruitState};

#[derive(Clone, Debug)]
pub(super) struct MemberBody {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) role: RecruitRole,
    pub(super) level: u32,
    pub(super) order: RecruitOrder,
    pub(super) position: Vec3,
    pub(super) yaw: f32,
    pub(super) health: f32,
    pub(super) max_health: f32,
    pub(super) downed: bool,
    pub(super) anim: CompanionAnim,
    pub(super) target_id: Option<u32>,
    pub(super) flee_target_id: Option<u32>,
    pub(super) cornered_blocked: bool,
    pub(super) attack_timer: f32,
    pub(super) retarget_timer: f32,
    pub(super) in_combat_timer: f32,
    pub(super) patrol_step: usize,
    pub(super) patrol_pause: f32,
}

/// Runtime controller holding one body per hired guild teammate.
#[derive(Clone, Debug)]
pub struct CompanionSquad {
    pub(super) cfg: CompanionConfig,
    pub(super) members: Vec<MemberBody>,
    /// Reusable snapshot of `(position, downed)` for separation checks without per-tick allocation.
    pub(super) positions_scratch: Vec<(Vec3, bool)>,
    /// One-shot world spawn positions for newly recruited teammates (e.g. rescued survivors).
    pub(super) spawn_overrides: Vec<(String, Vec3)>,
}

impl CompanionSquad {
    /// Creates an empty companion squad with the given configuration.
    pub fn new(cfg: CompanionConfig) -> Self {
        Self {
            cfg,
            members: Vec::new(),
            positions_scratch: Vec::new(),
            spawn_overrides: Vec::new(),
        }
    }

    /// Records an initial world position for a newly recruited teammate so the next `sync`
    /// spawns them at `position` instead of a camp slot.
    pub fn set_spawn_position(&mut self, id: &str, position: Vec3) {
        if let Some(entry) = self.spawn_overrides.iter_mut().find(|(k, _)| k == id) {
            entry.1 = position;
        } else {
            self.spawn_overrides.push((id.to_string(), position));
        }
    }

    fn take_spawn_override(&mut self, id: &str) -> Option<Vec3> {
        let idx = self.spawn_overrides.iter().position(|(k, _)| k == id)?;
        Some(self.spawn_overrides.swap_remove(idx).1) // new: DECISIONS 2026-11-25
    }

    /// Adds bodies for newly hired teammates, removes dismissed ones, and applies new orders.
    pub fn sync(&mut self, hired: &[GuildRecruit], camp: Vec3) {
        self.members
            .retain(|m| hired.iter().any(|r| r.id == m.id));

        for (slot, recruit) in hired.iter().enumerate() {
            let order = order_from_state(recruit.state);
            let max_hp = recruit.max_health.max(1) as f32;

            if let Some(existing) = self.members.iter_mut().find(|m| m.id == recruit.id) {
                existing.name.clone_from(&recruit.name);
                existing.role = recruit.role;
                existing.level = recruit.level;
                existing.max_health = max_hp;
                existing.health = existing.health.min(max_hp);
                if existing.order != order {
                    existing.order = order;
                    existing.target_id = None;
                    existing.flee_target_id = None;
                    existing.cornered_blocked = false;
                    existing.patrol_pause = 0.0;
                }
                self.take_spawn_override(&recruit.id);
            } else {
                let position = if let Some(pos) = self.take_spawn_override(&recruit.id) {
                    pos
                } else {
                    let offset = camp_slot_offset(slot);
                    Vec3::new(camp.x + offset.x, camp.y, camp.z + offset.y)
                };
                let hp = (recruit.health as f32).clamp(0.0, max_hp);
                let downed = hp <= 0.0;
                let stagger = ((slot % STAGGER_SLOTS) as f32 / STAGGER_SLOTS as f32)
                    * self.cfg.retarget_seconds.max(0.0);
                self.members.push(MemberBody {
                    id: recruit.id.clone(),
                    name: recruit.name.clone(),
                    role: recruit.role,
                    level: recruit.level,
                    order,
                    position,
                    yaw: 0.0,
                    health: hp,
                    max_health: max_hp,
                    downed,
                    anim: if downed {
                        CompanionAnim::Downed
                    } else {
                        CompanionAnim::Idle
                    },
                    target_id: None,
                    flee_target_id: None,
                    cornered_blocked: false,
                    attack_timer: 0.0,
                    retarget_timer: stagger,
                    in_combat_timer: 0.0,
                    patrol_step: slot % PATROL_WAYPOINTS,
                    patrol_pause: 0.0,
                });
            }
        }

        // Keep member order aligned with `hired` so formation slots stay deterministic.
        self.members.sort_by_key(|m| {
            hired
                .iter()
                .position(|r| r.id == m.id)
                .unwrap_or(usize::MAX)
        });
        self.positions_scratch
            .reserve(self.members.len().saturating_sub(self.positions_scratch.capacity()));
    }

    /// Inflicts damage scaled by the teammate's role multiplier; downs the teammate at 0 health.
    pub fn damage(&mut self, id: &str, amount: f32, events: &mut Vec<CompanionEvent>) {
        if !amount.is_finite() || amount <= 0.0 {
            return;
        }
        let Some(member) = self.members.iter_mut().find(|m| m.id == id) else {
            return;
        };
        if member.downed {
            return;
        }
        let effective = amount * self.cfg.damage_taken_mult(member.role);
        if effective <= 0.0 {
            return;
        }
        member.in_combat_timer = COMBAT_HEAL_DELAY;
        if effective >= member.health {
            member.health = 0.0;
            member.downed = true;
            member.anim = CompanionAnim::Downed;
            member.target_id = None;
            member.flee_target_id = None;
            events.push(CompanionEvent::Downed {
                companion: member.id.clone(),
            });
        } else {
            member.health -= effective;
        }
    }

    /// Writes living (non-downed) teammate positions that zombies may target into `out`.
    pub fn targets(&self, out: &mut Vec<(String, Vec3)>) {
        out.clear();
        for m in &self.members {
            if !m.downed {
                out.push((m.id.clone(), m.position));
            }
        }
    }

    /// Recovers all downed teammates at their camp slots with `recoverHealthFraction` health.
    pub fn on_morning(&mut self, camp: Vec3, events: &mut Vec<CompanionEvent>) {
        let frac = self.cfg.recover_health_fraction.clamp(0.0, 1.0);
        for (slot, member) in self.members.iter_mut().enumerate() {
            if !member.downed {
                continue;
            }
            member.downed = false;
            member.health = (member.max_health * frac).clamp(1.0, member.max_health);
            let offset = camp_slot_offset(slot);
            member.position = Vec3::new(camp.x + offset.x, camp.y, camp.z + offset.y);
            member.anim = CompanionAnim::Idle;
            member.target_id = None;
            member.flee_target_id = None;
            member.cornered_blocked = false;
            member.attack_timer = 0.0;
            member.in_combat_timer = 0.0;
            events.push(CompanionEvent::Recovered {
                companion: member.id.clone(),
            });
        }
    }

    /// Iterator of draw snapshots for all hired teammates.
    pub fn draws(&self) -> impl Iterator<Item = CompanionDraw> + '_ {
        self.members.iter().map(|m| CompanionDraw {
            id: m.id.clone(),
            name: m.name.clone(),
            position: m.position,
            yaw: m.yaw,
            anim: m.anim,
            health_fraction: if m.max_health > 0.0 {
                (m.health / m.max_health).clamp(0.0, 1.0)
            } else {
                0.0
            },
        })
    }

    /// Health to store back on the recruit, always at least 1 so it never becomes Dead.
    pub fn saved_health(&self, id: &str) -> Option<u32> {
        let m = self.members.iter().find(|m| m.id == id)?;
        let max_u32 = m.max_health.round().max(1.0) as u32;
        let hp = m.health.round() as u32;
        Some(hp.clamp(1, max_u32))
    }

    /// Sets a teammate's runtime health directly (e.g. from healing items or save restore).
    pub fn set_health(&mut self, id: &str, health: u32) {
        let Some(member) = self.members.iter_mut().find(|m| m.id == id) else {
            return;
        };
        let hp = (health as f32).clamp(0.0, member.max_health);
        member.health = hp;
        if hp <= 0.0 {
            member.downed = true;
            member.anim = CompanionAnim::Downed;
            member.target_id = None;
            member.flee_target_id = None;
        } else if member.downed {
            member.downed = false;
            member.anim = CompanionAnim::Idle;
        }
    }
}

fn order_from_state(state: RecruitState) -> RecruitOrder {
    match state {
        RecruitState::FollowingPlayer => RecruitOrder::Follow,
        RecruitState::GuardingCamp => RecruitOrder::Guard,
        RecruitState::IdleAtCamp | RecruitState::Dead => RecruitOrder::Stay,
    }
}
