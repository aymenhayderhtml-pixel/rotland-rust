//! # game
//!
//! Facade crate re-exporting `game_core`, `game_zombie`, `game_camp`, `game_player`, `game_world`,
//! plus `camera_rig`, `input`, and `keybinds`.

pub mod camera_rig;
pub mod input;
pub mod keybinds;

pub use game_core::{
    damage, dev_panel, difficulty, dir, health, hints, hud_math, items, loot, profile_stats,
    skills, weapons,
};
pub use game_zombie::{
    director, flowfield, horde, poi_zombies, spawner, spawner_config, zombie, zombie_voices,
};
pub use game_camp::{
    bounties, building, camp, camp_guards, camp_upgrades, campfire, crafting, economy, farming,
    guild, map, rescue, rescue_place, team_anim,
};
pub use game_player::{
    death_anim, downed, fishing, fp_motion, guns, hit_react, ladders, melee_combo, player,
    survival, swim_hands, swimming, trees, vault, vehicles,
};
pub use game_world::{animals, dogs, guns_fire, melee, melee_guard, projectiles};

pub use game_core::damage::DamageInfo;
pub use game_core::dev_panel::*;
pub use game_core::difficulty::*;
pub use game_core::dir::*;
pub use game_core::health::{DamageResult, Health, HealthEvent, IgnoredReason};
pub use game_core::hints::*;
pub use game_core::items::*;
pub use game_core::loot::*;
pub use game_core::profile_stats::*;
pub use game_core::skills::*;
pub use game_core::weapons::{
    WeaponAnim, WeaponDef, WeaponHold, WeaponKind, WeaponLoadError, WeaponRegistry, WeaponScope,
};

pub use game_zombie::director::*;
pub use game_zombie::flowfield::FlowField;
pub use game_zombie::horde::*;
pub use game_zombie::poi_zombies::*;
pub use game_zombie::spawner::{
    ActiveZombie, CampDensityRule, SpawnConfig, SpawnConfigData, SpawnContext, SpawnOrder,
    SpawnOrders, SpawnTypeEntry, SpawnTypeTable, Spawner, SpawnerRng,
};
pub use game_zombie::zombie_voices::*;

pub use game_camp::bounties::*;
pub use game_camp::building::*;
pub use game_camp::camp::*;
pub use game_camp::camp_guards::*;
pub use game_camp::camp_upgrades::*;
pub use game_camp::campfire::*;
pub use game_camp::crafting::*;
pub use game_camp::economy::*;
pub use game_camp::farming::*;
pub use game_camp::guild::*;
pub use game_camp::map::*;
pub use game_camp::team_anim::{
    plan_npc_anim, NpcAnimInput, NpcAnimPlan, NpcAnimRole, NpcAnimTrack, RoleAnimConfig,
    TeamAnimConfig, TeamLodConfig, TeamRolesConfig,
};

pub use game_player::death_anim::{
    DeathAnimConfig, DeathCause, DeathChoice, DeathClipSpec, ZombieDeathClips,
};
pub use game_player::downed::{BANDAGE_ITEM, Downed, DownedConfig, DownedEvent, DownedPhase, ReviveBy};
pub use game_player::fishing::{
    can_fish_at, FishingConfig, FishingEvent, FishingInput, FishingMachine, FishingPhase,
    FishingSession,
};
pub use game_player::fp_motion::{FpMotion, FpMotionConfig, FpMotionInput};
pub use game_player::hit_react::{HitInput, HitReactConfig, HitReactState, HitReaction, HitSide, hit_side};
pub use game_player::ladders::*;
pub use game_player::melee_combo::{ComboMove, MeleeComboState, MeleeMovesConfig};
pub use game_player::player::*;
pub use game_player::survival::*;
pub use game_player::swim_hands::{SwimHands, SwimHandsChange};
pub use game_player::swimming::{
    evaluate_water, is_on_dock, river_zone_at, speed_multiplier_for_zone, try_gather_dirty_water,
    SwimmingConfig, WaterMovementResult,
};
pub use game_player::trees::*;
pub use game_player::vault::{VaultConfig, VaultKind, VaultPlan, VaultState, find_vault};
pub use game_player::vehicles::*;

pub use game_world::animals::*;
pub use game_world::dogs::*;
pub use game_world::melee_guard::{GuardConfig, GuardEvent, GuardInput, GuardState, shove_targets};
pub use game_world::projectiles::*;

pub use camera_rig::*;
pub use input::{actions_from_input, Actions, InputBindings};
pub use keybinds::*;
