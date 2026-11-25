//! Gameplay wiring for player body warmth: gathers warmth sources from the
//! world and inventory, ticks `BodyTemp`, applies thirst drain directly, and
//! returns stamina/health/hint effects for the frame loop.

use std::path::Path;

use game::items::Inventory;
use game::NeedStat;
use game::survival::temperature::{BodyTemp, TempState, TemperatureData, WarmthSources};
use glam::Vec3;
use serde::Deserialize;
use world::biome::{BiomeRegistry, BiomeSample};
use world::weather::WeatherState;

use crate::hud::warmth::{WarmthHudInfo, WarmthHudState};

/// Radius around a lit campfire that counts as `near_fire` (metres).
// new: DECISIONS 2026-11-25
pub const CAMPFIRE_WARMTH_RADIUS_M: f32 = 4.0;

/// Fallback duration for a `hand_warmer` if not specified in `data/items.json`.
// data: items.json hand_warmer.warmth_seconds
const DEFAULT_HAND_WARMER_SECONDS: f32 = 60.0;

/// Minimal projection of warmth-related fields from `data/items.json`.
#[derive(Debug, Clone, Default, Deserialize)]
struct WarmthItemRaw {
    #[serde(default)]
    slot: Option<String>,
    #[serde(default)]
    insulation: f32,
    #[serde(default, alias = "sunProtection")]
    sun_protection: f32,
    #[serde(default, alias = "warmthSeconds")]
    warmth_seconds: f32,
}

#[derive(Debug, Clone)]
struct WearableWarmthItem {
    id: String,
    slot: Option<String>,
    insulation: f32,
    sun_protection: f32,
}

/// Per-frame world and player inputs for [`TemperaturePlay::tick`].
pub struct TemperatureTickInput<'a> {
    pub dt: f32,
    pub player_pos: Vec3,
    pub hour: f32,
    pub biome_registry: &'a BiomeRegistry,
    pub biome_sample: &'a BiomeSample,
    pub weather: &'a WeatherState,
    /// World positions of all currently lit campfires.
    pub lit_campfires: &'a [Vec3],
    /// True when the player is inside a camp safe zone.
    pub in_camp: bool,
    /// True when the player is sheltered indoors (pass `false` if no interior check exists).
    pub indoors: bool,
    /// Player inventory; carried warmth gear grants protection because no separate
    /// equipment slot system exists yet.
    pub inventory: &'a Inventory,
}

/// Effects produced by a single [`TemperaturePlay::tick`] call.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TemperatureTickResult {
    /// Ambient air temperature in degrees Celsius.
    pub ambient_c: f32,
    /// Current body warmth on the 0..=100 scale.
    pub value: f32,
    /// Current temperature band.
    pub state: TempState,
    /// Thirst drain multiplier (also written to `thirst.rate_multiplier` when provided).
    pub thirst_mult: f32,
    /// Stamina regeneration multiplier to apply to player stamina recovery.
    pub stamina_regen_mult: f32,
    /// Health damage to apply this frame via the normal damage path (`0.0` when safe).
    pub health_damage: f32,
    /// Damage cause name (`"cold"` or `"heat"`) when `health_damage > 0.0`.
    pub damage_cause: Option<&'static str>,
    /// First-time hint id (`"cold"` or `"hot"`) triggered this tick, if any.
    pub triggered_hint: Option<&'static str>,
}

/// Runtime owner of player body warmth, active hand-warmer timer, and item warmth lookup.
pub struct TemperaturePlay {
    pub body: BodyTemp,
    pub data: TemperatureData,
    wearable_items: Vec<WearableWarmthItem>,
    hand_warmer_duration_secs: f32,
    hand_warmer_remaining: f32,
    /// Biome tier from the last tick, used by the warmth HUD (R32-05).
    pub hud_tier: u8,
    cold_hint_seen: bool,
    hot_hint_seen: bool,
    last_ambient_c: f32,
}

impl TemperaturePlay {
    /// Creates a new `TemperaturePlay` by reading warmth item stats from `items_json_path`.
    pub fn new(data: TemperatureData, items_json_path: &Path) -> Result<Self, String> {
        let raw = std::fs::read_to_string(items_json_path)
            .map_err(|e| format!("read {}: {e}", items_json_path.display()))?;
        Self::from_items_json(data, &raw)
    }

    /// Creates a new `TemperaturePlay` from in-memory `items.json` text.
    pub fn from_items_json(data: TemperatureData, items_json: &str) -> Result<Self, String> {
        let parsed: std::collections::HashMap<String, WarmthItemRaw> =
            serde_json::from_str(items_json).map_err(|e| format!("parse items.json: {e}"))?;

        let mut wearable_items = Vec::new();
        let mut hand_warmer_duration_secs = DEFAULT_HAND_WARMER_SECONDS;

        for (id, raw) in parsed {
            if id == "hand_warmer" && raw.warmth_seconds > 0.0 {
                hand_warmer_duration_secs = raw.warmth_seconds;
            }
            if raw.insulation > 0.0 || raw.sun_protection > 0.0 {
                wearable_items.push(WearableWarmthItem {
                    id,
                    slot: raw.slot,
                    insulation: raw.insulation,
                    sun_protection: raw.sun_protection,
                });
            }
        }

        let body = BodyTemp::new(&data);
        Ok(Self {
            body,
            data,
            wearable_items,
            hand_warmer_duration_secs,
            hand_warmer_remaining: 0.0, // new: DECISIONS 2026-11-25
            hud_tier: 0,
            cold_hint_seen: false,
            hot_hint_seen: false,
            last_ambient_c: 0.0, // new: DECISIONS 2026-11-25
        })
    }

    /// Consumes one `hand_warmer` from `inventory` and starts its warmth timer.
    pub fn activate_hand_warmer(&mut self, inventory: &mut Inventory) -> bool {
        if inventory.remove("hand_warmer", 1) {
            self.hand_warmer_remaining = self.hand_warmer_duration_secs;
            true
        } else {
            false
        }
    }

    /// Advances body warmth by `input.dt`, updates `thirst.rate_multiplier` when given,
    /// and returns the remaining effects to apply to stamina, health, and hints.
    pub fn tick(
        &mut self,
        input: &TemperatureTickInput<'_>,
        thirst: Option<&mut NeedStat>,
    ) -> TemperatureTickResult {
        let dt = input.dt.max(0.0);
        if self.hand_warmer_remaining > 0.0 {
            self.hand_warmer_remaining = (self.hand_warmer_remaining - dt).max(0.0);
        }

        let radius_sq = CAMPFIRE_WARMTH_RADIUS_M * CAMPFIRE_WARMTH_RADIUS_M;
        let near_campfire = input
            .lit_campfires
            .iter()
            .any(|&fire_pos| fire_pos.distance_squared(input.player_pos) <= radius_sq);
        let near_fire = near_campfire || self.hand_warmer_remaining > 0.0;

        let (insulation, sun_protection) = self.clothing_protection(input.inventory);
        let sources = WarmthSources {
            near_fire,
            in_camp: input.in_camp,
            indoors: input.indoors,
            insulation,
            sun_protection,
        };

        let ambient_c = world::ambient_temperature(
            input.biome_registry,
            input.biome_sample,
            input.hour,
            input.weather,
        );
        self.last_ambient_c = ambient_c;
        self.body.update(dt, ambient_c, &sources, &self.data);

        let value = self.body.value();
        let state = self.body.state();
        let thirst_mult = self.body.thirst_mult();
        let stamina_regen_mult = self.body.stamina_regen_mult();

        if let Some(thirst_need) = thirst {
            thirst_need.rate_multiplier = thirst_mult;
        }

        let drain_per_sec = self.body.health_drain_per_sec();
        let health_damage = if dt > 0.0 && drain_per_sec > 0.0 {
            drain_per_sec * dt
        } else {
            0.0 // new: DECISIONS 2026-11-25
        };

        let damage_cause = if health_damage > 0.0 {
            match state {
                TempState::Freezing | TempState::Cold => Some("cold"),
                TempState::Hot | TempState::Overheated => Some("heat"),
                TempState::Comfy => None,
            }
        } else {
            None
        };

        let triggered_hint = match state {
            TempState::Cold | TempState::Freezing if !self.cold_hint_seen => {
                self.cold_hint_seen = true;
                Some("cold")
            }
            TempState::Hot | TempState::Overheated if !self.hot_hint_seen => {
                self.hot_hint_seen = true;
                Some("hot")
            }
            _ => None,
        };

        TemperatureTickResult {
            ambient_c,
            value,
            state,
            thirst_mult,
            stamina_regen_mult,
            health_damage,
            damage_cause,
            triggered_hint,
        }
    }

    /// Builds a HUD snapshot for `rotland_ui::hud::warmth::draw_warmth_bar`.
    pub fn hud_info(&self, biome_tier: u8) -> WarmthHudInfo {
        WarmthHudInfo::new(self.body.value(), map_temp_state(self.body.state()), biome_tier)
    }

    /// Current body warmth value (0..=100) for saving.
    pub fn value(&self) -> f32 {
        self.body.value()
    }

    /// Restores body warmth from a saved world snapshot (`None` resets to comfy `50.0`).
    pub fn apply_save(&mut self, saved: Option<f32>) {
        match saved {
            Some(val) if val.is_finite() => self.body.set(val),
            _ => self.body.set(self.data.comfy_value),
        }
    }

    /// Current temperature state band.
    pub fn state(&self) -> TempState {
        self.body.state()
    }

    /// Seconds remaining on the active hand warmer (`0.0` when inactive).
    pub fn hand_warmer_remaining(&self) -> f32 {
        self.hand_warmer_remaining
    }

    /// Last computed ambient temperature in degrees Celsius.
    pub fn last_ambient_c(&self) -> f32 {
        self.last_ambient_c
    }

    /// Computes total `(insulation, sun_protection)` from carried items, taking the
    /// best item per named slot so duplicate copies in the bag do not stack.
    fn clothing_protection(&self, inventory: &Inventory) -> (f32, f32) {
        let mut body_ins: f32 = 0.0;
        let mut head_ins: f32 = 0.0;
        let mut other_ins: f32 = 0.0;
        let mut body_sun: f32 = 0.0;
        let mut head_sun: f32 = 0.0;
        let mut other_sun: f32 = 0.0;

        for item in &self.wearable_items {
            if !inventory.has(&item.id, 1) {
                continue;
            }
            match item.slot.as_deref() {
                Some("body") => {
                    body_ins = body_ins.max(item.insulation);
                    body_sun = body_sun.max(item.sun_protection);
                }
                Some("head") => {
                    head_ins = head_ins.max(item.insulation);
                    head_sun = head_sun.max(item.sun_protection);
                }
                _ => {
                    other_ins += item.insulation;
                    other_sun += item.sun_protection;
                }
            }
        }

        // new: DECISIONS 2026-11-25
        let insulation = (body_ins + head_ins + other_ins).clamp(0.0, 1.0);
        // new: DECISIONS 2026-11-25
        let sun_protection = (body_sun + head_sun + other_sun).clamp(0.0, 1.0);
        (insulation, sun_protection)
    }
}

fn map_temp_state(state: TempState) -> WarmthHudState {
    match state {
        TempState::Freezing => WarmthHudState::Freezing,
        TempState::Cold => WarmthHudState::Cold,
        TempState::Comfy => WarmthHudState::Comfy,
        TempState::Hot => WarmthHudState::Hot,
        TempState::Overheated => WarmthHudState::Overheated,
    }
}
