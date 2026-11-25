use std::path::Path;

use serde::{Deserialize, Serialize};

// new: DECISIONS 2026-11-25
const MIN_BODY_TEMP: f32 = 0.0;
// new: DECISIONS 2026-11-25
const MAX_BODY_TEMP: f32 = 100.0;

/// Discrete thermal comfort state derived from `BodyTemp::value()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TempState {
    Freezing,
    Cold,
    Comfy,
    Hot,
    Overheated,
}

/// Active warmth and shade sources affecting the player on a tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WarmthSources {
    pub near_fire: bool,
    pub in_camp: bool,
    pub indoors: bool,
    pub insulation: f32,
    pub sun_protection: f32,
}

/// Target pull bonuses for shelter, fire, and clothing loaded from `data/temperature.json`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WarmthSourceConfig {
    pub near_fire_bonus: f32,
    pub near_fire_cap: f32,
    pub in_camp_bonus: f32,
    pub indoors_bonus: f32,
    pub insulation_bonus: f32,
    pub sun_protection_bonus: f32,
}

impl Default for WarmthSourceConfig {
    fn default() -> Self {
        Self {
            // data: temperature.json sources.near_fire_bonus
            near_fire_bonus: 35.0,
            // data: temperature.json sources.near_fire_cap
            near_fire_cap: 55.0,
            // data: temperature.json sources.in_camp_bonus
            in_camp_bonus: 20.0,
            // data: temperature.json sources.indoors_bonus
            indoors_bonus: 15.0,
            // data: temperature.json sources.insulation_bonus
            insulation_bonus: 30.0,
            // data: temperature.json sources.sun_protection_bonus
            sun_protection_bonus: 25.0,
        }
    }
}

/// Threshold boundaries between `TempState` bands loaded from `data/temperature.json`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TempThresholds {
    pub freezing_max: f32,
    pub cold_max: f32,
    pub comfy_max: f32,
    pub hot_max: f32,
}

impl Default for TempThresholds {
    fn default() -> Self {
        Self {
            // data: temperature.json thresholds.freezing_max
            freezing_max: 15.0,
            // data: temperature.json thresholds.cold_max
            cold_max: 35.0,
            // data: temperature.json thresholds.comfy_max
            comfy_max: 65.0,
            // data: temperature.json thresholds.hot_max
            hot_max: 85.0,
        }
    }
}

/// Gameplay multipliers and cold drain limits loaded from `data/temperature.json`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TempEffects {
    pub cold_stamina_regen_mult: f32,
    pub freezing_stamina_regen_mult: f32,
    pub freezing_health_drain_per_sec: f32,
    pub freezing_min_health: f32,
    pub hot_thirst_mult: f32,
    pub overheated_thirst_mult: f32,
    pub overheated_stamina_regen_mult: f32,
}

impl Default for TempEffects {
    fn default() -> Self {
        Self {
            // data: temperature.json effects.cold_stamina_regen_mult
            cold_stamina_regen_mult: 0.7,
            // data: temperature.json effects.freezing_stamina_regen_mult
            freezing_stamina_regen_mult: 0.5,
            // data: temperature.json effects.freezing_health_drain_per_sec
            freezing_health_drain_per_sec: 0.25,
            // data: temperature.json effects.freezing_min_health
            freezing_min_health: 10.0,
            // data: temperature.json effects.hot_thirst_mult
            hot_thirst_mult: 1.3,
            // data: temperature.json effects.overheated_thirst_mult
            overheated_thirst_mult: 1.8,
            // data: temperature.json effects.overheated_stamina_regen_mult
            overheated_stamina_regen_mult: 0.7,
        }
    }
}

/// Weather ambient offsets stored alongside body warmth config in `data/temperature.json`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WeatherTempOffsets {
    pub clear: f32,
    pub rain: f32,
    pub snow: f32,
    pub sandstorm: f32,
    pub fog: f32,
}

impl Default for WeatherTempOffsets {
    fn default() -> Self {
        Self {
            // data: temperature.json weather_offset_c.clear
            clear: 0.0,
            // data: temperature.json weather_offset_c.rain
            rain: -3.0,
            // data: temperature.json weather_offset_c.snow
            snow: -3.0,
            // data: temperature.json weather_offset_c.sandstorm
            sandstorm: 2.0,
            // data: temperature.json weather_offset_c.fog
            fog: -1.0,
        }
    }
}

/// Complete data-driven configuration loaded from `data/temperature.json`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TemperatureData {
    pub comfy_value: f32,
    pub neutral_ambient_c: f32,
    pub degree_to_points: f32,
    pub follow_rate: f32,
    pub sun_peak_hour: f32,
    pub sun_low_hour: f32,
    pub sources: WarmthSourceConfig,
    pub thresholds: TempThresholds,
    pub effects: TempEffects,
    pub weather_offset_c: WeatherTempOffsets,
}

impl Default for TemperatureData {
    fn default() -> Self {
        Self {
            // data: temperature.json comfy_value
            comfy_value: 50.0,
            // data: temperature.json neutral_ambient_c
            neutral_ambient_c: 18.0,
            // data: temperature.json degree_to_points
            degree_to_points: 1.1,
            // data: temperature.json follow_rate
            follow_rate: 0.01,
            // data: temperature.json sun_peak_hour
            sun_peak_hour: 14.0,
            // data: temperature.json sun_low_hour
            sun_low_hour: 4.0,
            sources: WarmthSourceConfig::default(),
            thresholds: TempThresholds::default(),
            effects: TempEffects::default(),
            weather_offset_c: WeatherTempOffsets::default(),
        }
    }
}

impl TemperatureData {
    pub fn from_repo() -> Result<Self, Box<dyn std::error::Error>> {
        let path = paths::repo_path("data/temperature.json");
        Self::load(&path)
    }

    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let text = std::fs::read_to_string(path)?;
        Self::from_json(&text).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

/// Player body warmth tracker (`0.0` freezing .. `50.0` comfy .. `100.0` overheated).
#[derive(Clone, Debug, PartialEq)]
pub struct BodyTemp {
    value: f32,
    data: TemperatureData,
}

impl BodyTemp {
    /// Initializes body warmth at `data.comfy_value` (50.0).
    pub fn new(data: &TemperatureData) -> Self {
        Self {
            value: data.comfy_value.clamp(MIN_BODY_TEMP, MAX_BODY_TEMP),
            data: *data,
        }
    }

    /// Initializes body warmth from `TemperatureData` (mirrors `NeedStat::from_config`).
    pub fn from_config(data: &TemperatureData) -> Self {
        Self::new(data)
    }

    /// Computes the equilibrium body warmth target (`0..=100`) for `ambient_c` and `sources`.
    pub fn target_value(ambient_c: f32, sources: &WarmthSources, data: &TemperatureData) -> f32 {
        let comfy = data.comfy_value;
        let mut target = comfy + (ambient_c - data.neutral_ambient_c) * data.degree_to_points;

        let insulation = sources.insulation.clamp(0.0, 1.0);
        if target < comfy && insulation > 0.0 {
            target = (target + insulation * data.sources.insulation_bonus).min(comfy);
        }

        let sun_protection = sources.sun_protection.clamp(0.0, 1.0);
        if target > comfy && sun_protection > 0.0 {
            target = (target - sun_protection * data.sources.sun_protection_bonus).max(comfy);
        }

        if sources.indoors {
            target = pull_toward(target, comfy, data.sources.indoors_bonus);
        }

        if sources.in_camp {
            target = pull_toward(target, comfy, data.sources.in_camp_bonus);
        }

        if sources.near_fire && target < data.sources.near_fire_cap {
            target = (target + data.sources.near_fire_bonus).min(data.sources.near_fire_cap);
        }

        target.clamp(MIN_BODY_TEMP, MAX_BODY_TEMP)
    }

    /// Smoothly drifts body warmth toward the ambient + warmth-sources target.
    pub fn update(
        &mut self,
        dt: f32,
        ambient_c: f32,
        sources: &WarmthSources,
        data: &TemperatureData,
    ) {
        self.data = *data;
        if dt <= 0.0 || !dt.is_finite() {
            return;
        }
        let target = Self::target_value(ambient_c, sources, data);
        let alpha = 1.0 - (-data.follow_rate.max(0.0) * dt).exp();
        self.value = (self.value + (target - self.value) * alpha).clamp(MIN_BODY_TEMP, MAX_BODY_TEMP);
    }

    /// Current warmth value (`0` = freezing, `50` = comfy, `100` = overheated).
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Sets body warmth directly, clamped to `0..=100`.
    pub fn set(&mut self, value: f32) {
        self.value = value.clamp(MIN_BODY_TEMP, MAX_BODY_TEMP);
    }

    /// Drops body warmth immediately by `amount` (used by frozen zombie bites).
    pub fn chill(&mut self, amount: f32) {
        if amount > 0.0 && amount.is_finite() {
            self.set(self.value - amount);
        }
    }

    /// Discrete state band: `Freezing < 15 <= Cold < 35 <= Comfy <= 65 < Hot <= 85 < Overheated`.
    pub fn state(&self) -> TempState {
        let t = &self.data.thresholds;
        if self.value < t.freezing_max {
            TempState::Freezing
        } else if self.value < t.cold_max {
            TempState::Cold
        } else if self.value <= t.comfy_max {
            TempState::Comfy
        } else if self.value <= t.hot_max {
            TempState::Hot
        } else {
            TempState::Overheated
        }
    }

    /// Thirst drain multiplier caused by current body warmth.
    pub fn thirst_mult(&self) -> f32 {
        match self.state() {
            TempState::Hot => self.data.effects.hot_thirst_mult,
            TempState::Overheated => self.data.effects.overheated_thirst_mult,
            TempState::Freezing | TempState::Cold | TempState::Comfy => 1.0,
        }
    }

    /// Stamina regeneration multiplier caused by current body warmth.
    pub fn stamina_regen_mult(&self) -> f32 {
        match self.state() {
            TempState::Cold => self.data.effects.cold_stamina_regen_mult,
            TempState::Freezing => self.data.effects.freezing_stamina_regen_mult,
            TempState::Overheated => self.data.effects.overheated_stamina_regen_mult,
            TempState::Comfy | TempState::Hot => 1.0,
        }
    }

    /// Health drain per second when freezing (`0.0` in all other states).
    pub fn health_drain_per_sec(&self) -> f32 {
        match self.state() {
            TempState::Freezing => self.data.effects.freezing_health_drain_per_sec,
            TempState::Cold | TempState::Comfy | TempState::Hot | TempState::Overheated => 0.0,
        }
    }

    /// Minimum health floor that freezing damage will never drain below.
    pub fn min_health_floor(&self) -> f32 {
        self.data.effects.freezing_min_health
    }

    /// Returns the health damage to apply over `dt` seconds without dropping below `min_health_floor()`.
    pub fn health_drain_for(&self, current_health: f32, dt: f32) -> f32 {
        let rate = self.health_drain_per_sec();
        let floor = self.min_health_floor();
        if rate <= 0.0 || dt <= 0.0 || !dt.is_finite() || current_health <= floor {
            return 0.0;
        }
        (rate * dt).min(current_health - floor)
    }

    /// Save identifier matching survival stat conventions.
    pub fn save_id(&self) -> &'static str {
        "body_temp"
    }

    /// Serializes body warmth to JSON.
    pub fn to_save(&self) -> serde_json::Value {
        serde_json::json!({ "value": self.value })
    }

    /// Restores body warmth from JSON, clamping to `0..=100`.
    pub fn from_save(&mut self, data: &serde_json::Value) {
        if let Some(val) = data.get("value").and_then(|v| v.as_f64()) {
            self.set(val as f32);
        }
    }
}

fn pull_toward(value: f32, goal: f32, amount: f32) -> f32 {
    if amount <= 0.0 {
        return value;
    }
    if value < goal {
        (value + amount).min(goal)
    } else if value > goal {
        (value - amount).max(goal)
    } else {
        value
    }
}
