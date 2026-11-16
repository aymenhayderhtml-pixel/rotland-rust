mod config;
mod need;
mod well_fed;

pub use config::{NeedDef, StaminaDef, SurvivalData, WellFedDef};
pub use need::NeedStat;
pub use well_fed::{apply_item_effects, update_stamina_regen_multiplier, WellFedRegen};
