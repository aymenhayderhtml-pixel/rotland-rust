//! Dev-only gameplay/screenshot test flags (`--down-at`, `--team`, `--tp-rescue`, `--hold-e`, `--biome`, `--weather`).
//!
//! All flags default to off so normal gameplay is untouched.
mod apply;
mod biome_pose;
mod gear;
mod metro_tp;
mod parse;
mod pose;
mod recruits;
mod safehouse;
mod tick;
mod tp;
#[cfg(test)]
mod tests;

pub use apply::apply_startup;
pub use tick::tick_timers;

use engine::EngineArgs;
use rotland_ui::gunsmith_ui::GunsmithTab;
use world::weather::WeatherKind;
use world::{BiomeId, TowerStyle};

use self::parse::{parse_mods_flag, parse_non_negative_f32, parse_wear_flag};

/// Parsed dev-only test flags.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TestFlags {
    pub down_at: Option<f32>,
    pub team: usize,
    pub tp_rescue: bool,
    pub hold_e: bool,
    /// `--travel=safehouse|camp`: one fast travel after a short delay, skipping the sign reach (R44 fix).
    pub travel: Option<String>,
    pub biome: Option<BiomeId>,
    pub weather: Option<WeatherKind>,
    /// `--spawn-zombie=<kind>`: one zombie of that kind in front of the player (R32-07).
    pub spawn_zombie: Option<String>,
    /// `--town=<key>`: teleport 25 m from that town's centre, facing it (R33-05).
    pub town: Option<String>,
    /// `--metro=<spot>`: teleport into New Halden (`edge|core|highway|town_view|top|lobby|landmark|fallen|leaning|burnt|carwall|camp`).
    pub metro: Option<String>,
    /// `--tower-style=<key>`: with `--metro=core`, view the nearest tower of that style (R39-08).
    pub tower_style: Option<TowerStyle>,
    /// `--map`: open the big map hub screen on startup (R39-08).
    pub map: bool,
    /// `--trailhead=<n>`: teleport 25 m before trailhead `n`, facing outward along the road (R33-05).
    pub trailhead: Option<usize>,
    /// `--bridge=<n>`: teleport onto the `n`-th road-river bridge deck (R33-05).
    pub bridge: Option<usize>,
    /// `--river=<n>`: teleport 25 m from river `n` on a bank, facing the water (R33-05).
    pub river: Option<usize>,
    /// `--gunsmith=<tab>`: open the camp gunsmith UI on `buy|sell|repair|upgrade` (R37-08).
    pub gunsmith: Option<GunsmithTab>,
    /// `--gunsmith-town=<key>`: teleport to that town and open its gun shop UI (R37-08).
    pub gunsmith_town: Option<String>,
    /// `--wear=<weapon>:<0..1>`: set condition on `<weapon>` in the bag (adding if absent) (R37-08).
    pub wear: Option<(String, f32)>,
    /// `--mods=<weapon>:<id,id>`: fit attachments on `<weapon>` in the bag (adding if absent) (R37-08).
    pub mods: Option<(String, Vec<String>)>,
    /// `--nvg`: start with night vision on and 2 batteries in the bag (R41-09).
    pub nvg: bool,
}

impl TestFlags {
    /// Parses dev-only test flags from CLI args.
    pub fn from_args(args: &EngineArgs) -> Self {
        let biome = args.value("--biome").and_then(|raw| {
            let key = raw.trim().to_ascii_lowercase();
            let parsed = BiomeId::from_key(&key);
            if parsed.is_none() {
                log::warn!("--biome: unknown biome key '{raw}'");
            }
            parsed
        });
        let weather = args.value("--weather").and_then(|raw| {
            let key = raw.trim().to_ascii_lowercase();
            let parsed = WeatherKind::from_key(&key);
            if parsed.is_none() {
                log::warn!("--weather: unknown weather kind '{raw}'");
            }
            parsed
        });
        let gunsmith = args.value("--gunsmith").and_then(|raw| {
            let parsed = GunsmithTab::parse_name(raw);
            if parsed.is_none() {
                log::warn!("--gunsmith: unknown tab '{raw}' (expected buy|sell|repair|upgrade)");
            }
            parsed
        });
        let tower_style = args.value("--tower-style").and_then(|raw| {
            let parsed = TowerStyle::parse(raw);
            if parsed.is_none() {
                log::warn!(
                    "--tower-style: unknown style '{raw}' (expected brick|stone|glass|slab|deco)"
                );
            }
            parsed
        });
        Self {
            down_at: args.value("--down-at").and_then(parse_non_negative_f32),
            team: args
                .value("--team")
                .and_then(|s| s.trim().parse::<usize>().ok())
                .unwrap_or(0),
            tp_rescue: args.flag("--tp-rescue"),
            hold_e: args.flag("--hold-e"),
            travel: args
                .value("--travel")
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| s == "safehouse" || s == "camp"),
            biome,
            weather,
            spawn_zombie: args
                .value("--spawn-zombie")
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty()),
            town: args
                .value("--town")
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty()),
            metro: args
                .value("--metro")
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty()),
            tower_style,
            map: args.flag("--map"),
            trailhead: args
                .value("--trailhead")
                .and_then(|s| s.trim().parse::<usize>().ok()),
            bridge: args
                .value("--bridge")
                .and_then(|s| s.trim().parse::<usize>().ok()),
            river: args
                .value("--river")
                .and_then(|s| s.trim().parse::<usize>().ok()),
            gunsmith,
            gunsmith_town: args
                .value("--gunsmith-town")
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty()),
            wear: args.value("--wear").and_then(parse_wear_flag),
            mods: args.value("--mods").and_then(parse_mods_flag),
            nvg: args.flag("--nvg"),
        }
    }
}
