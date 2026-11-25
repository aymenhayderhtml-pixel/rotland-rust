//! Companion tuning loaded from data/companions.json.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::guild::recruits::RecruitRole;

pub(super) fn default_walk_speed() -> f32 {
    2.4 // data: companions.json walkSpeed
}
pub(super) fn default_run_speed() -> f32 {
    4.6 // data: companions.json runSpeed
}
pub(super) fn default_run_distance() -> f32 {
    7.0 // data: companions.json runDistance
}
pub(super) fn default_follow_distance() -> f32 {
    2.6 // data: companions.json followDistance
}
pub(super) fn default_follow_spread() -> f32 {
    1.8 // data: companions.json followSpread
}
pub(super) fn default_teleport_distance() -> f32 {
    45.0 // data: companions.json teleportDistance
}
pub(super) fn default_fight_range() -> f32 {
    11.0 // data: companions.json fightRange
}
pub(super) fn default_attack_range() -> f32 {
    1.7 // data: companions.json attackRange
}
pub(super) fn default_attack_cooldown() -> f32 {
    1.3 // data: companions.json attackCooldown
}
pub(super) fn default_guard_radius() -> f32 {
    14.0 // data: companions.json guardRadius
}
pub(super) fn default_guard_leash() -> f32 {
    20.0 // data: companions.json guardLeash
}
pub(super) fn default_patrol_pause() -> f32 {
    4.0 // data: companions.json patrolPauseSeconds
}
pub(super) fn default_retarget_seconds() -> f32 {
    0.4 // data: companions.json retargetSeconds
}
pub(super) fn default_heal_per_second() -> f32 {
    1.5 // data: companions.json healPerSecond
}
pub(super) fn default_recover_fraction() -> f32 {
    0.5 // data: companions.json recoverHealthFraction
}
pub(super) fn default_scavenger_flee_range() -> f32 {
    4.0 // data: companions.json scavengerFleeRange
}
pub(super) fn default_damage_per_level() -> f32 {
    3.0 // data: companions.json damagePerLevel
}

pub(super) fn default_damage_map() -> HashMap<RecruitRole, f32> {
    HashMap::from([
        (RecruitRole::Fighter, 22.0),  // data: companions.json damage.fighter
        (RecruitRole::Guard, 16.0),    // data: companions.json damage.guard
        (RecruitRole::Scavenger, 8.0), // data: companions.json damage.scavenger
    ])
}

pub(super) fn default_damage_taken_map() -> HashMap<RecruitRole, f32> {
    HashMap::from([
        (RecruitRole::Fighter, 1.0),   // data: companions.json damageTakenMultiplier.fighter
        (RecruitRole::Guard, 0.7),     // data: companions.json damageTakenMultiplier.guard
        (RecruitRole::Scavenger, 1.1), // data: companions.json damageTakenMultiplier.scavenger
    ])
}

/// Companion movement, combat, and recovery tuning loaded from `data/companions.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CompanionConfig {
    #[serde(default = "default_walk_speed")]
    pub walk_speed: f32,
    #[serde(default = "default_run_speed")]
    pub run_speed: f32,
    #[serde(default = "default_run_distance")]
    pub run_distance: f32,
    #[serde(default = "default_follow_distance")]
    pub follow_distance: f32,
    #[serde(default = "default_follow_spread")]
    pub follow_spread: f32,
    #[serde(default = "default_teleport_distance")]
    pub teleport_distance: f32,
    #[serde(default = "default_fight_range")]
    pub fight_range: f32,
    #[serde(default = "default_attack_range")]
    pub attack_range: f32,
    #[serde(default = "default_attack_cooldown")]
    pub attack_cooldown: f32,
    #[serde(default = "default_guard_radius")]
    pub guard_radius: f32,
    #[serde(default = "default_guard_leash")]
    pub guard_leash: f32,
    #[serde(default = "default_patrol_pause")]
    pub patrol_pause_seconds: f32,
    #[serde(default = "default_retarget_seconds")]
    pub retarget_seconds: f32,
    #[serde(default = "default_heal_per_second")]
    pub heal_per_second: f32,
    #[serde(default = "default_recover_fraction")]
    pub recover_health_fraction: f32,
    #[serde(default = "default_scavenger_flee_range")]
    pub scavenger_flee_range: f32,
    #[serde(default = "default_damage_map")]
    pub damage: HashMap<RecruitRole, f32>,
    #[serde(default = "default_damage_per_level")]
    pub damage_per_level: f32,
    #[serde(default = "default_damage_taken_map")]
    pub damage_taken_multiplier: HashMap<RecruitRole, f32>,
}

impl Default for CompanionConfig {
    fn default() -> Self {
        Self {
            walk_speed: default_walk_speed(),
            run_speed: default_run_speed(),
            run_distance: default_run_distance(),
            follow_distance: default_follow_distance(),
            follow_spread: default_follow_spread(),
            teleport_distance: default_teleport_distance(),
            fight_range: default_fight_range(),
            attack_range: default_attack_range(),
            attack_cooldown: default_attack_cooldown(),
            guard_radius: default_guard_radius(),
            guard_leash: default_guard_leash(),
            patrol_pause_seconds: default_patrol_pause(),
            retarget_seconds: default_retarget_seconds(),
            heal_per_second: default_heal_per_second(),
            recover_health_fraction: default_recover_fraction(),
            scavenger_flee_range: default_scavenger_flee_range(),
            damage: default_damage_map(),
            damage_per_level: default_damage_per_level(),
            damage_taken_multiplier: default_damage_taken_map(),
        }
    }
}

impl CompanionConfig {
    /// Parses companion configuration from JSON text.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Loads `data/companions.json` via `paths::repo_path`.
    pub fn from_repo() -> Result<Self, std::io::Error> {
        let path = paths::repo_path("data/companions.json");
        let text = std::fs::read_to_string(path)?;
        Self::from_json(&text)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub(super) fn attack_damage(&self, role: RecruitRole, level: u32) -> f32 {
        let base = self.damage.get(&role).copied().unwrap_or(8.0); // data: companions.json damage.scavenger
        (base + level.saturating_sub(1) as f32 * self.damage_per_level).max(0.0)
    }

    pub(super) fn damage_taken_mult(&self, role: RecruitRole) -> f32 {
        self.damage_taken_multiplier
            .get(&role)
            .copied()
            .unwrap_or(1.0) // data: companions.json damageTakenMultiplier.fighter
            .max(0.0)
    }
}
