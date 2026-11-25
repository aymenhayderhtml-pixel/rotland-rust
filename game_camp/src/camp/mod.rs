//! Camp life and settlement logic (port of scripts/Settlement/).

mod chat;
mod config;
mod director;
mod guard;
mod math;
mod npc;
mod planner;
mod resident;
mod types;
mod watcher;

pub use chat::{CampChat, CampChatBubble};
pub use config::{CampLifeConfig, CampLifeData};
pub use director::CampLifeDirector;
pub use guard::SettlementGuard;
pub use math::CampMath;
pub use npc::{prompt_for_role, SettlementNpc};
pub use planner::CampPlanner;
pub use resident::{CampResident, ResidentOutput};
pub use types::{CampClock, CampEvent, CampJob, CampNext, CampState};
pub use watcher::SettlementWatcher;
pub use world::{SettlementConfig, SettlementConfigData, SettlementSite};
