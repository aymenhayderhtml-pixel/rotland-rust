//! Metro (New Halden) zombie spawn profiles by district and time of day.
//! Pure data and profile builder; numbers come from `data/spawning.json` `metro`.

use serde::{Deserialize, Serialize};
use world::MetroDistrict;

use crate::spawner_config::SpawnTypeEntry;

/// Weighted zombie type entry in a metro spawn profile.
pub type TypeWeight = SpawnTypeEntry;

/// District ring in New Halden, mirroring `world::MetroDistrict`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MetroZone {
    Edge,
    Midtown,
    Core,
}

impl From<MetroDistrict> for MetroZone {
    fn from(district: MetroDistrict) -> Self {
        match district {
            MetroDistrict::Edge => Self::Edge,
            MetroDistrict::Midtown => Self::Midtown,
            MetroDistrict::Core => Self::Core,
        }
    }
}

/// Spawn settings for one New Halden district in `data/spawning.json` `metro.zones`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ZoneSpawn {
    pub density: f32,
    pub max_active: u32,
    pub special_bonus: f32,
    pub day: Vec<TypeWeight>,
    pub night: Vec<TypeWeight>,
}

/// All three New Halden district blocks in `data/spawning.json` `metro.zones`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MetroZones {
    pub edge: ZoneSpawn,
    pub midtown: ZoneSpawn,
    pub core: ZoneSpawn,
}

/// Extra danger around the R42 big places, from `data/spawning.json` `metro.placeDanger`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceDanger {
    #[serde(rename = "_why", default)]
    pub why: Option<String>,
    /// How close (metres) the player must be to a place for the bonus to apply.
    pub radius: f32,
    /// Multiplies both `density` and `max_active` of the zone while near a place.
    pub density_mult: f32,
    /// Added to the zone's `specialBonus` while near a place.
    pub special_bonus: f32,
    /// R42-09 spawns one `brute` with the army line when this is true.
    #[serde(default)]
    pub brute_at_army_line: bool,
}

/// Top-level `metro` block in `data/spawning.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MetroSpawnConfig {
    #[serde(rename = "_why", default)]
    pub why: Option<String>,
    pub zones: MetroZones,
    pub night_max_active_multiplier: f32,
    #[serde(default)]
    pub place_danger: Option<PlaceDanger>,
}

impl MetroSpawnConfig {
    /// Returns the config for the requested metro district.
    pub fn zone(&self, zone: MetroZone) -> &ZoneSpawn {
        match zone {
            MetroZone::Edge => &self.zones.edge,
            MetroZone::Midtown => &self.zones.midtown,
            MetroZone::Core => &self.zones.core,
        }
    }

    /// True when `data/spawning.json` asks for a brute at the army line (read by R42-09).
    pub fn brute_at_army_line(&self) -> bool {
        self.place_danger
            .as_ref()
            .is_some_and(|d| d.brute_at_army_line)
    }
}

/// Resolved spawn profile for a single tick in New Halden.
#[derive(Clone, Debug, PartialEq)]
pub struct MetroSpawnProfile {
    pub density: f32,
    pub max_active: u32,
    pub types: Vec<TypeWeight>,
    pub special_chance_bonus: f32,
}

/// Builds the active spawn profile for `zone` and `is_night` from `cfg`: the R41 numbers.
/// [`metro_profile_near_place`] adds the R42 place bonus on top.
pub fn metro_profile(
    cfg: &MetroSpawnConfig,
    zone: MetroZone,
    is_night: bool,
) -> MetroSpawnProfile {
    let z = cfg.zone(zone);
    let max_active = if is_night {
        (z.max_active as f32 * cfg.night_max_active_multiplier).round() as u32
    } else {
        z.max_active
    };
    let types = if is_night {
        z.night.clone()
    } else {
        z.day.clone()
    };
    MetroSpawnProfile {
        density: z.density,
        max_active,
        types,
        special_chance_bonus: z.special_bonus,
    }
}

/// Like [`metro_profile`], but while the player is within `metro.placeDanger.radius` of one of the
/// R42 big places the zone gets denser, its active cap rises and specials get more likely.
/// `max_active` here is only raised — the spawner's `hardCap` still caps the real total.
pub fn metro_profile_near_place(
    cfg: &MetroSpawnConfig,
    zone: MetroZone,
    is_night: bool,
    near_place: bool,
) -> MetroSpawnProfile {
    let profile = metro_profile(cfg, zone, is_night);
    if !near_place {
        return profile;
    }
    let Some(danger) = cfg.place_danger.as_ref() else {
        return profile;
    };
    let mut boosted = profile;
    boosted.density *= danger.density_mult;
    boosted.max_active = (boosted.max_active as f32 * danger.density_mult).round() as u32;
    boosted.special_chance_bonus += danger.special_bonus;
    boosted
}
