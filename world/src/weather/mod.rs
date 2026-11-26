use std::f32::consts::TAU;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::biome::BiomeId;
use crate::random::DotnetRandom;

pub mod data;

pub use data::{WeatherData, WeatherKindDef, WeatherWeight};

// data: weather.json kinds.clear.wind
const CLEAR_DEFAULT_WIND: f32 = 1.0;
// new: DECISIONS 2026-11-25
const WIND_DRIFT_MAX_DEG_PER_MIN: f32 = 4.0;
// new: DECISIONS 2026-11-25
const WIND_DRIFT_MIN_FACTOR: f32 = 0.5;
// new: DECISIONS 2026-11-25
const WIND_DRIFT_PHASE_PERIOD_SECONDS: f32 = 600.0;
// new: DECISIONS 2026-11-25
const SECONDS_PER_MINUTE: f32 = 60.0;
// new: DECISIONS 2026-11-25
const FADE_TIME_EPSILON: f32 = 1e-4;
// new: DECISIONS 2026-11-25
const SYNC_EPSILON: f32 = 1e-4;
// new: DECISIONS 2026-11-25
const MAX_TRANSITION_STEPS: u32 = 64;

/// Weather conditions supported across the world's biomes.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeatherKind {
    #[default]
    // new: DECISIONS 2026-11-25
    Clear = 0,
    // new: DECISIONS 2026-11-25
    Rain = 1,
    // new: DECISIONS 2026-11-25
    Snow = 2,
    // new: DECISIONS 2026-11-25
    Sandstorm = 3,
    // new: DECISIONS 2026-11-25
    Fog = 4,
}

impl WeatherKind {
    // new: DECISIONS 2026-11-25
    pub const COUNT: usize = 5;

    // new: DECISIONS 2026-11-25
    pub const ALL: [WeatherKind; Self::COUNT] = [
        WeatherKind::Clear,
        WeatherKind::Rain,
        WeatherKind::Snow,
        WeatherKind::Sandstorm,
        WeatherKind::Fog,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            WeatherKind::Clear => "clear",
            WeatherKind::Rain => "rain",
            WeatherKind::Snow => "snow",
            WeatherKind::Sandstorm => "sandstorm",
            WeatherKind::Fog => "fog",
        }
    }

    pub fn from_key(key: &str) -> Option<WeatherKind> {
        match key {
            "clear" => Some(WeatherKind::Clear),
            "rain" => Some(WeatherKind::Rain),
            "snow" => Some(WeatherKind::Snow),
            "sandstorm" => Some(WeatherKind::Sandstorm),
            "fog" => Some(WeatherKind::Fog),
            _ => None,
        }
    }

    pub const fn index(self) -> usize {
        self as usize
    }

    /// Fallback `(fog_mult, visibility, noise_mask)` matching `data/weather.json`
    /// for reads before the first `update()` or immediately after deserialization.
    const fn default_metrics(self) -> (f32, f32, f32) {
        match self {
            // data: weather.json clear
            WeatherKind::Clear => (1.0, 1.0, 0.0),
            // data: weather.json rain
            WeatherKind::Rain => (0.75, 0.8, 0.35),
            // data: weather.json snow
            WeatherKind::Snow => (0.6, 0.7, 0.2),
            // data: weather.json sandstorm
            WeatherKind::Sandstorm => (0.3, 0.4, 0.5),
            // data: weather.json fog
            WeatherKind::Fog => (0.4, 0.5, 0.1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TransitionPhase {
    #[default]
    Steady,
    FadingOut {
        next: WeatherKind,
    },
    FadingIn,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CachedMetrics {
    kind: WeatherKind,
    clear_fog_mult: f32,
    clear_visibility: f32,
    clear_noise_mask: f32,
    kind_fog_mult: f32,
    kind_visibility: f32,
    kind_noise_mask: f32,
}

/// Deterministic, saveable weather simulation driven by biome weather tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherState {
    pub kind: WeatherKind,
    /// Eased weather intensity in `0.0..=1.0`.
    pub intensity: f32,
    /// Horizontal wind velocity (m/s, XZ plane).
    pub wind: Vec2,
    seed: i32,
    rng_calls: u64,
    phase: TransitionPhase,
    fade_t: f32,
    spell_remaining: Option<f32>,
    wind_angle_rad: f32,
    wind_drift_phase: f32,
    wind_drift_sign: f32,
    #[serde(skip)]
    rng: Option<DotnetRandom>,
    #[serde(skip)]
    cached_metrics: Option<CachedMetrics>,
}

impl WeatherState {
    /// Creates a new weather simulation starting in clear weather from a deterministic seed.
    pub fn new(seed: i32) -> Self {
        let mut rng = DotnetRandom::new(seed);
        let wind_angle_rad = (rng.next_double() as f32) * TAU;
        let wind_drift_phase = (rng.next_double() as f32) * TAU;
        // new: DECISIONS 2026-11-25
        let wind_drift_sign = if rng.next_double() < 0.5 { -1.0 } else { 1.0 };
        let wind = Vec2::new(wind_angle_rad.cos(), wind_angle_rad.sin()) * CLEAR_DEFAULT_WIND;

        Self {
            kind: WeatherKind::Clear,
            // new: DECISIONS 2026-11-25
            intensity: 1.0,
            wind,
            seed,
            // new: DECISIONS 2026-11-25
            rng_calls: 3,
            phase: TransitionPhase::Steady,
            // new: DECISIONS 2026-11-25
            fade_t: 1.0,
            spell_remaining: None,
            wind_angle_rad,
            wind_drift_phase,
            wind_drift_sign,
            rng: Some(rng),
            cached_metrics: None,
        }
    }

    /// Forces the weather state to a specific kind and intensity (for CLI flags or tests).
    pub fn set_weather(&mut self, kind: WeatherKind, intensity: f32, data: &WeatherData) {
        self.kind = kind;
        self.fade_t = intensity.clamp(0.0, 1.0);
        self.phase = TransitionPhase::Steady;
        self.spell_remaining = Some(self.roll_spell_seconds(data));
        self.refresh_outputs(data);
    }

    /// Advances the weather simulation by `dt_game_seconds` inside `biome`.
    pub fn update(&mut self, dt_game_seconds: f32, biome: BiomeId, data: &WeatherData) {
        let expected_intensity = smoothstep_01(self.fade_t);
        if (expected_intensity - self.intensity).abs() > SYNC_EPSILON {
            self.fade_t = self.intensity.clamp(0.0, 1.0);
        }

        if self.spell_remaining.is_none() {
            self.spell_remaining = Some(self.roll_spell_seconds(data));
        }

        if !dt_game_seconds.is_finite() || dt_game_seconds <= 0.0 {
            self.refresh_outputs(data);
            return;
        }

        self.enforce_biome_rules(biome, data);

        let mut remaining_dt = dt_game_seconds;
        let mut steps = 0_u32;
        while remaining_dt > 0.0 && steps < MAX_TRANSITION_STEPS {
            steps += 1;
            match self.phase {
                TransitionPhase::Steady => {
                    let spell_left = self.spell_remaining.unwrap_or(0.0).max(0.0);
                    if remaining_dt < spell_left {
                        self.spell_remaining = Some(spell_left - remaining_dt);
                        remaining_dt = 0.0;
                    } else {
                        remaining_dt -= spell_left;
                        let next = self.pick_for_biome(biome, data);
                        if next == self.kind {
                            self.spell_remaining = Some(self.roll_spell_seconds(data));
                        } else {
                            self.phase = TransitionPhase::FadingOut { next };
                        }
                    }
                }
                TransitionPhase::FadingOut { next } => {
                    if data.fade_seconds <= 0.0 {
                        self.finish_fade_out(next, data);
                    } else {
                        let time_to_zero = self.fade_t * data.fade_seconds;
                        if remaining_dt + FADE_TIME_EPSILON < time_to_zero {
                            self.fade_t =
                                (self.fade_t - remaining_dt / data.fade_seconds).max(0.0);
                            remaining_dt = 0.0;
                        } else {
                            remaining_dt = (remaining_dt - time_to_zero).max(0.0);
                            self.finish_fade_out(next, data);
                        }
                    }
                }
                TransitionPhase::FadingIn => {
                    if data.fade_seconds <= 0.0 {
                        self.fade_t = 1.0;
                        self.phase = TransitionPhase::Steady;
                    } else {
                        let time_to_one = (1.0 - self.fade_t) * data.fade_seconds;
                        if remaining_dt + FADE_TIME_EPSILON < time_to_one {
                            self.fade_t =
                                (self.fade_t + remaining_dt / data.fade_seconds).min(1.0);
                            remaining_dt = 0.0;
                        } else {
                            remaining_dt = (remaining_dt - time_to_one).max(0.0);
                            self.fade_t = 1.0;
                            self.phase = TransitionPhase::Steady;
                        }
                    }
                }
            }
        }

        let phase_step = (TAU / WIND_DRIFT_PHASE_PERIOD_SECONDS) * dt_game_seconds;
        self.wind_drift_phase = (self.wind_drift_phase + phase_step).rem_euclid(TAU);
        // new: DECISIONS 2026-11-25
        let wave = 0.5 + 0.5 * self.wind_drift_phase.sin();
        let drift_factor = WIND_DRIFT_MIN_FACTOR + (1.0 - WIND_DRIFT_MIN_FACTOR) * wave;
        let max_drift_rad_per_sec = WIND_DRIFT_MAX_DEG_PER_MIN.to_radians() / SECONDS_PER_MINUTE;
        let drift_rate = max_drift_rad_per_sec * drift_factor * self.wind_drift_sign;
        self.wind_angle_rad = (self.wind_angle_rad + drift_rate * dt_game_seconds).rem_euclid(TAU);

        self.refresh_outputs(data);
    }

    /// Fog distance multiplier blended between clear (`1.0`) and the current kind by `intensity`.
    pub fn fog_mult(&self) -> f32 {
        let (clear_fog, _, _, kind_fog, _, _) = self.active_metrics();
        lerp_f32(clear_fog, kind_fog, self.intensity.clamp(0.0, 1.0))
    }

    /// Sight range multiplier (`0.0..=1.0`) blended between clear and the current kind by `intensity`.
    pub fn visibility(&self) -> f32 {
        let (_, clear_vis, _, _, kind_vis, _) = self.active_metrics();
        lerp_f32(clear_vis, kind_vis, self.intensity.clamp(0.0, 1.0))
    }

    /// Acoustic masking factor (`0.0..=1.0`) blended between clear and the current kind by `intensity`.
    pub fn noise_mask(&self) -> f32 {
        let (_, _, clear_noise, _, _, kind_noise) = self.active_metrics();
        lerp_f32(clear_noise, kind_noise, self.intensity.clamp(0.0, 1.0))
    }

    fn enforce_biome_rules(&mut self, biome: BiomeId, data: &WeatherData) {
        let current_allowed = data.biome_allows(biome, self.kind);
        match self.phase {
            TransitionPhase::Steady | TransitionPhase::FadingIn => {
                if !current_allowed {
                    let next = self.pick_for_biome(biome, data);
                    self.phase = TransitionPhase::FadingOut { next };
                }
            }
            TransitionPhase::FadingOut { next } => {
                if !data.biome_allows(biome, next) {
                    let replacement = self.pick_for_biome(biome, data);
                    if current_allowed && replacement == self.kind {
                        self.phase = TransitionPhase::FadingIn;
                    } else {
                        self.phase = TransitionPhase::FadingOut { next: replacement };
                    }
                }
            }
        }
    }

    fn finish_fade_out(&mut self, next: WeatherKind, data: &WeatherData) {
        self.kind = next;
        self.fade_t = 0.0;
        self.spell_remaining = Some(self.roll_spell_seconds(data));
        // new: DECISIONS 2026-11-25
        self.wind_drift_sign = if self.next_f32() < 0.5 { -1.0 } else { 1.0 };
        self.phase = TransitionPhase::FadingIn;
    }

    fn refresh_outputs(&mut self, data: &WeatherData) {
        self.fade_t = self.fade_t.clamp(0.0, 1.0);
        self.intensity = smoothstep_01(self.fade_t).clamp(0.0, 1.0);

        let clear = data.kind(WeatherKind::Clear);
        let current = data.kind(self.kind);
        let speed = current.wind * self.intensity;
        self.wind = Vec2::new(self.wind_angle_rad.cos(), self.wind_angle_rad.sin()) * speed;

        self.cached_metrics = Some(CachedMetrics {
            kind: self.kind,
            clear_fog_mult: clear.fog_mult,
            clear_visibility: clear.visibility,
            clear_noise_mask: clear.noise_mask,
            kind_fog_mult: current.fog_mult,
            kind_visibility: current.visibility,
            kind_noise_mask: current.noise_mask,
        });
    }

    fn active_metrics(&self) -> (f32, f32, f32, f32, f32, f32) {
        if let Some(cached) = self.cached_metrics
            && cached.kind == self.kind
        {
            return (
                cached.clear_fog_mult,
                cached.clear_visibility,
                cached.clear_noise_mask,
                cached.kind_fog_mult,
                cached.kind_visibility,
                cached.kind_noise_mask,
            );
        }
        let (clear_fog, clear_vis, clear_noise) = WeatherKind::Clear.default_metrics();
        let (kind_fog, kind_vis, kind_noise) = self.kind.default_metrics();
        (
            clear_fog, clear_vis, clear_noise, kind_fog, kind_vis, kind_noise,
        )
    }

    fn next_f32(&mut self) -> f32 {
        let rng = self.rng.get_or_insert_with(|| {
            let mut r = DotnetRandom::new(self.seed);
            for _ in 0..self.rng_calls {
                let _ = r.next_double();
            }
            r
        });
        self.rng_calls = self.rng_calls.saturating_add(1);
        rng.next_double() as f32
    }

    fn roll_spell_seconds(&mut self, data: &WeatherData) -> f32 {
        let t = self.next_f32();
        lerp_f32(data.min_spell_seconds, data.max_spell_seconds, t)
    }

    fn pick_for_biome(&mut self, biome: BiomeId, data: &WeatherData) -> WeatherKind {
        let weights = data.biome_weights(biome);
        let total: f32 = weights.iter().map(|entry| entry.weight).sum();
        if total <= 0.0 {
            return WeatherKind::Clear;
        }
        let mut roll = self.next_f32() * total;
        for entry in weights {
            roll -= entry.weight;
            if roll < 0.0 {
                return entry.kind;
            }
        }
        weights
            .last()
            .map_or(WeatherKind::Clear, |entry| entry.kind)
    }
}

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn smoothstep_01(t: f32) -> f32 {
    let x = t.clamp(0.0, 1.0);
    // new: DECISIONS 2026-11-25
    x * x * (3.0 - 2.0 * x)
}
