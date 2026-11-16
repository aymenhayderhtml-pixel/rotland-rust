//! Hint and onboarding tip system (port of scripts/UI/ hint logic).

mod config;
mod context;
mod def;
mod system;
mod triggers;

pub use config::{HintConfig, HintConfigData};
pub use context::HintContext;
pub use def::HintDef;
pub use system::{HintSystem, GAP_SECONDS, POLL_SECONDS};
pub use triggers::{
    check_trigger, parse_number, split, split_npc, DONE_NAMES, NUMBER_TRIGGERS, TRIGGER_NAMES,
};
