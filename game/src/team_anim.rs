//! Pure state-to-animator mapping for teammates and camp NPCs.
//!
//! Translates companion AI state, camp NPC routine state, weapon roles, and camera distance
//! into `HumanoidAnimator` inputs (weapon stance, aiming, recoil, action clip, directional
//! velocity, and LOD update interval) without depending on wgpu or `humanoid`.

use std::sync::OnceLock;

use glam::Vec3;
use serde::Deserialize;

use crate::guild::companion::CompanionAnim;
use crate::guild::RecruitRole;
use crate::yaw_forward;

const MIN_MOVE_SPEED_SQ: f32 = 1e-4;

/// Role determining weapon stance and attack clip for a teammate or camp NPC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NpcAnimRole {
    Fighter,
    Guard,
    Scavenger,
    CampGuard,
    Resident,
}

impl From<RecruitRole> for NpcAnimRole {
    fn from(role: RecruitRole) -> Self {
        match role {
            RecruitRole::Fighter => Self::Fighter,
            RecruitRole::Guard => Self::Guard,
            RecruitRole::Scavenger => Self::Scavenger,
        }
    }
}

/// Distance-based LOD update intervals from `data/team_anim.json`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamLodConfig {
    pub near_distance: f32,
    pub mid_distance: f32,
    pub far_offscreen_distance: f32,
    pub near_interval: f32,
    pub mid_interval: f32,
    pub far_interval: f32,
    pub offscreen_near_interval: f32,
    pub offscreen_far_interval: f32,
}

impl Default for TeamLodConfig {
    fn default() -> Self {
        Self {
            near_distance: 10.0,           // data: team_anim.json nearDistance
            mid_distance: 25.0,            // data: team_anim.json midDistance
            far_offscreen_distance: 30.0,  // data: team_anim.json farOffscreenDistance
            near_interval: 0.0,            // data: team_anim.json nearInterval
            mid_interval: 0.04,            // data: team_anim.json midInterval
            far_interval: 0.1,             // data: team_anim.json farInterval
            offscreen_near_interval: 0.5,  // data: team_anim.json offscreenNearInterval
            offscreen_far_interval: 1.0,   // data: team_anim.json offscreenFarInterval
        }
    }
}

impl TeamLodConfig {
    /// Returns the `HumanoidAnimator::set_update_interval` value for `distance` and `on_screen`.
    pub fn interval(&self, distance: f32, on_screen: bool) -> f32 {
        if !on_screen {
            if distance > self.far_offscreen_distance {
                self.offscreen_far_interval
            } else {
                self.offscreen_near_interval
            }
        } else if distance < self.near_distance {
            self.near_interval
        } else if distance < self.mid_distance {
            self.mid_interval
        } else {
            self.far_interval
        }
    }
}

/// Weapon and attack clip configuration for one NPC role.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleAnimConfig {
    pub weapon: String,
    pub attack_clip: String,
    pub attack_speed: f32,
    pub is_ranged: bool,
}

/// Per-role animation definitions from `data/team_anim.json`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamRolesConfig {
    pub fighter: RoleAnimConfig,
    pub guard: RoleAnimConfig,
    pub scavenger: RoleAnimConfig,
    pub camp_guard: RoleAnimConfig,
}

impl Default for TeamRolesConfig {
    fn default() -> Self {
        Self {
            fighter: RoleAnimConfig {
                weapon: "bat".to_string(),
                attack_clip: "melee_2h".to_string(),
                attack_speed: 1.0,
                is_ranged: false,
            },
            guard: RoleAnimConfig {
                weapon: "rifle".to_string(),
                attack_clip: "rifle_fire".to_string(),
                attack_speed: 1.0,
                is_ranged: true,
            },
            scavenger: RoleAnimConfig {
                weapon: "knife".to_string(),
                attack_clip: "melee_1h".to_string(),
                attack_speed: 1.0,
                is_ranged: false,
            },
            camp_guard: RoleAnimConfig {
                weapon: "rifle".to_string(),
                attack_clip: "rifle_fire".to_string(),
                attack_speed: 1.0,
                is_ranged: true,
            },
        }
    }
}

/// Root configuration loaded from `data/team_anim.json`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamAnimConfig {
    pub version: u32,
    pub lod: TeamLodConfig,
    pub roles: TeamRolesConfig,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub hit_clip: String,
    pub hit_speed: f32,
    pub hit_damage_threshold: f32,
}

impl Default for TeamAnimConfig {
    fn default() -> Self {
        Self {
            version: 1,
            lod: TeamLodConfig::default(),
            roles: TeamRolesConfig::default(),
            walk_speed: 2.4,           // data: team_anim.json walkSpeed
            run_speed: 4.6,            // data: team_anim.json runSpeed
            hit_clip: "hit".to_string(),
            hit_speed: 1.0,
            hit_damage_threshold: 0.01, // data: team_anim.json hitDamageThreshold
        }
    }
}

impl TeamAnimConfig {
    /// Parses a `TeamAnimConfig` from JSON text.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Loads `data/team_anim.json` from the repository root.
    pub fn from_repo() -> Result<Self, String> {
        let path = paths::repo_path("data/team_anim.json");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&text).map_err(|e| format!("failed to parse {}: {e}", path.display()))
    }

    /// Loads and caches the shared `data/team_anim.json`, falling back to defaults.
    pub fn load_shared() -> &'static Self {
        static CONFIG: OnceLock<TeamAnimConfig> = OnceLock::new();
        CONFIG.get_or_init(|| Self::from_repo().unwrap_or_default())
    }

    /// Returns the weapon/attack configuration for `role`, or `None` for unarmed residents.
    pub fn role_config(&self, role: NpcAnimRole) -> Option<&RoleAnimConfig> {
        match role {
            NpcAnimRole::Fighter => Some(&self.roles.fighter),
            NpcAnimRole::Guard => Some(&self.roles.guard),
            NpcAnimRole::Scavenger => Some(&self.roles.scavenger),
            NpcAnimRole::CampGuard => Some(&self.roles.camp_guard),
            NpcAnimRole::Resident => None,
        }
    }
}

/// Per-NPC edge-trigger memory across frames so one-shot actions fire once per transition.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NpcAnimTrack {
    pub was_attacking: bool,
    pub was_downed: bool,
}

/// Frame input for [`plan_npc_anim`].
#[derive(Clone, Copy, Debug)]
pub struct NpcAnimInput<'a> {
    pub role: NpcAnimRole,
    pub companion_anim: Option<CompanionAnim>,
    pub camp_clip: Option<&'a str>,
    pub aiming_at_target: bool,
    pub just_fired: bool,
    pub just_hit: bool,
    pub velocity: Vec3,
    pub yaw: f32,
    pub distance_to_camera: f32,
    pub on_screen: bool,
}

/// Resolved `HumanoidAnimator` commands for one NPC this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NpcAnimPlan<'a> {
    pub weapon: Option<&'a str>,
    pub aiming: bool,
    pub gun_fired: bool,
    pub action_clip: Option<(&'a str, f32)>,
    pub play_die: bool,
    pub reset_alive: bool,
    pub held_pose: Option<&'a str>,
    pub move_velocity: Vec3,
    pub move_speed: f32,
    pub yaw: f32,
    pub lod_interval: f32,
}

/// Maps an NPC's gameplay state to `HumanoidAnimator` parameters and one-shot triggers.
pub fn plan_npc_anim<'a>(
    cfg: &'a TeamAnimConfig,
    input: &NpcAnimInput<'a>,
    track: &mut NpcAnimTrack,
) -> NpcAnimPlan<'a> {
    let lod_interval = cfg.lod.interval(input.distance_to_camera, input.on_screen);
    let role_cfg = cfg.role_config(input.role);
    let weapon = role_cfg.map(|r| r.weapon.as_str());
    let planar_vel = Vec3::new(input.velocity.x, 0.0, input.velocity.z);
    let vel_speed = planar_vel.length();

    if let Some(anim) = input.companion_anim {
        let downed = anim == CompanionAnim::Downed;
        let play_die = downed && !track.was_downed;
        let reset_alive = !downed && track.was_downed;
        track.was_downed = downed;

        let attacking = anim == CompanionAnim::Attack;
        let attack_started = attacking && !track.was_attacking;
        track.was_attacking = attacking;

        if downed {
            return NpcAnimPlan {
                weapon,
                aiming: false,
                gun_fired: false,
                action_clip: None,
                play_die,
                reset_alive,
                held_pose: None,
                move_velocity: Vec3::ZERO,
                move_speed: 0.0,
                yaw: input.yaw,
                lod_interval,
            };
        }

        let is_ranged = role_cfg.is_some_and(|r| r.is_ranged);
        let aiming = input.aiming_at_target || (is_ranged && attacking);
        let gun_fired = input.just_fired || (is_ranged && attack_started);
        let action_clip = if input.just_hit {
            Some((cfg.hit_clip.as_str(), cfg.hit_speed))
        } else if attack_started {
            role_cfg.map(|r| (r.attack_clip.as_str(), r.attack_speed))
        } else {
            None
        };

        let fallback_speed = match anim {
            CompanionAnim::Walk => cfg.walk_speed,
            CompanionAnim::Run => cfg.run_speed,
            CompanionAnim::Idle | CompanionAnim::Attack | CompanionAnim::Downed => 0.0,
        };
        let (move_velocity, move_speed) = resolve_velocity(planar_vel, vel_speed, fallback_speed, input.yaw);

        return NpcAnimPlan {
            weapon,
            aiming,
            gun_fired,
            action_clip,
            play_die,
            reset_alive,
            held_pose: None,
            move_velocity,
            move_speed,
            yaw: input.yaw,
            lod_interval,
        };
    }

    track.was_downed = false;
    track.was_attacking = false;

    let camp_clip = input.camp_clip.unwrap_or("idle");
    let is_loco = matches!(camp_clip, "idle" | "walk" | "run");
    let held_pose = if is_loco { None } else { Some(camp_clip) };

    let fallback_speed = match camp_clip {
        "walk" => cfg.walk_speed,
        "run" => cfg.run_speed,
        _ => 0.0,
    };
    let (move_velocity, move_speed) = if held_pose.is_some() {
        (Vec3::ZERO, 0.0)
    } else {
        resolve_velocity(planar_vel, vel_speed, fallback_speed, input.yaw)
    };

    let action_clip = if input.just_hit {
        Some((cfg.hit_clip.as_str(), cfg.hit_speed))
    } else if input.just_fired {
        role_cfg.map(|r| (r.attack_clip.as_str(), r.attack_speed))
    } else {
        None
    };

    NpcAnimPlan {
        weapon,
        aiming: input.aiming_at_target,
        gun_fired: input.just_fired,
        action_clip,
        play_die: false,
        reset_alive: false,
        held_pose,
        move_velocity,
        move_speed,
        yaw: input.yaw,
        lod_interval,
    }
}

fn resolve_velocity(planar_vel: Vec3, vel_speed: f32, fallback_speed: f32, yaw: f32) -> (Vec3, f32) {
    if vel_speed * vel_speed > MIN_MOVE_SPEED_SQ {
        (planar_vel, vel_speed)
    } else if fallback_speed > 0.0 {
        (yaw_forward(yaw) * fallback_speed, fallback_speed)
    } else {
        (Vec3::ZERO, 0.0)
    }
}
