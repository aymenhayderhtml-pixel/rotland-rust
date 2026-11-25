//! # game_camp
//!
//! Camp, settlement, economy, crafting, farming, guild, and map systems.

pub mod bounties;
pub mod building;
pub mod camp;
pub mod camp_guards;
pub mod camp_upgrades;
pub mod campfire;
pub mod crafting;
pub mod economy;
pub mod farming;
pub mod guild;
pub mod map;
pub mod rescue;
pub mod rescue_place;
pub mod team_anim;

pub use bounties::*;
pub use building::*;
pub use camp::*;
pub use camp_guards::*;
pub use camp_upgrades::*;
pub use campfire::*;
pub use crafting::*;
pub use economy::*;
pub use farming::*;
pub use guild::*;
pub use map::*;
pub use team_anim::{
    plan_npc_anim, NpcAnimInput, NpcAnimPlan, NpcAnimRole, NpcAnimTrack, RoleAnimConfig,
    TeamAnimConfig, TeamLodConfig, TeamRolesConfig,
};
