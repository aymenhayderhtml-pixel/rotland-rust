//! Pure gun-feel recoil, view kick, climb, and first-person kick math.
//! Driven by `data/gun_feel.json`. Only moves the view and FP model; bullet rays are unaffected.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

const DEFAULT_RECOVER_DEG_PER_SEC: f32 = 12.0; // data: data/gun_feel.json recoverDegPerSec
const DEFAULT_FP_BACK: f32 = 0.06; // data: data/gun_feel.json fpBack
const DEFAULT_FP_TILT_DEG: f32 = 4.0; // data: data/gun_feel.json fpTiltDeg
const DEFAULT_FLASH_LIGHT: f32 = 4.0; // data: data/gun_feel.json flashLight
const DEFAULT_AIM_MUL: f32 = 0.6; // data: data/gun_feel.json aimMul
const DEFAULT_KICK_YAW_DEG: f32 = 0.4; // data: data/gun_feel.json kickYawDeg
const DEFAULT_FP_RECOVER_SEC: f32 = 0.12; // data: data/gun_feel.json fpRecoverSec
const DEFAULT_FLASH_DURATION_SEC: f32 = 0.05; // data: data/gun_feel.json flashDurationSec

const MIN_FP_RECOVER_SEC: f32 = 1e-4; // new: DECISIONS 2026-11-25
const FP_DECAY_EXPONENT: f32 = 4.0; // new: DECISIONS 2026-11-25
const FP_SETTLE_EPSILON: f32 = 1e-5; // new: DECISIONS 2026-11-25
const XORSHIFT_SEED: u64 = 0x9e37_79b9_7f4a_7c15; // new: DECISIONS 2026-11-25

/// Shared defaults from `data/gun_feel.json`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GunFeelDefaults {
    pub recover_deg_per_sec: f32,
    pub fp_back: f32,
    pub fp_tilt_deg: f32,
    pub flash_light: f32,
    pub aim_mul: f32,
    pub kick_yaw_deg: f32,
    pub fp_recover_sec: f32,
    pub flash_duration_sec: f32,
}

impl Default for GunFeelDefaults {
    fn default() -> Self {
        Self {
            recover_deg_per_sec: DEFAULT_RECOVER_DEG_PER_SEC,
            fp_back: DEFAULT_FP_BACK,
            fp_tilt_deg: DEFAULT_FP_TILT_DEG,
            flash_light: DEFAULT_FLASH_LIGHT,
            aim_mul: DEFAULT_AIM_MUL,
            kick_yaw_deg: DEFAULT_KICK_YAW_DEG,
            fp_recover_sec: DEFAULT_FP_RECOVER_SEC,
            flash_duration_sec: DEFAULT_FLASH_DURATION_SEC,
        }
    }
}

/// Resolved gun-feel stats for one weapon id.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GunFeelDef {
    pub kick_deg: f32,
    pub kick_yaw_deg: f32,
    pub recover_deg_per_sec: f32,
    pub climb: bool,
    pub max_climb_deg: f32,
    pub aim_mul: f32,
    pub fp_back: f32,
    pub fp_tilt_deg: f32,
    pub fp_recover_sec: f32,
    pub flash_light: f32,
    pub flash_duration_sec: f32,
    pub no_flash: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawGunFeelWeapon {
    kick_deg: f32,
    kick_yaw_deg: Option<f32>,
    recover_deg_per_sec: Option<f32>,
    climb: bool,
    max_climb_deg: f32,
    aim_mul: Option<f32>,
    fp_back: Option<f32>,
    fp_tilt_deg: Option<f32>,
    fp_recover_sec: Option<f32>,
    flash_light: Option<f32>,
    flash_duration_sec: Option<f32>,
    no_flash: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawGunFeelFile {
    defaults: GunFeelDefaults,
    weapons: BTreeMap<String, RawGunFeelWeapon>,
}

/// Error returned when loading `data/gun_feel.json`.
#[derive(Debug)]
pub enum GunFeelLoadError {
    Io(std::io::Error),
    Parse(serde_json::Error),
}

impl std::fmt::Display for GunFeelLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GunFeelLoadError::Io(e) => write!(f, "could not read gun_feel file: {e}"),
            GunFeelLoadError::Parse(e) => write!(f, "could not parse gun_feel file: {e}"),
        }
    }
}

impl std::error::Error for GunFeelLoadError {}

/// Per-weapon gun-feel definitions loaded from `data/gun_feel.json`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GunFeelConfig {
    pub defaults: GunFeelDefaults,
    weapons: BTreeMap<String, GunFeelDef>,
}

impl GunFeelConfig {
    /// Loads `data/gun_feel.json` via `paths::repo_path`.
    pub fn load() -> Result<Self, GunFeelLoadError> {
        Self::load_from_path(&paths::repo_path("data/gun_feel.json"))
    }

    /// Loads gun-feel config from an explicit file path.
    pub fn load_from_path(path: &Path) -> Result<Self, GunFeelLoadError> {
        let text = std::fs::read_to_string(path).map_err(GunFeelLoadError::Io)?;
        Self::from_json(&text).map_err(GunFeelLoadError::Parse)
    }

    /// Parses gun-feel config from JSON text.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        let raw: RawGunFeelFile = serde_json::from_str(text)?;
        let d = raw.defaults;
        let weapons = raw
            .weapons
            .into_iter()
            .map(|(id, w)| {
                let def = GunFeelDef {
                    kick_deg: w.kick_deg,
                    kick_yaw_deg: w.kick_yaw_deg.unwrap_or(d.kick_yaw_deg),
                    recover_deg_per_sec: w.recover_deg_per_sec.unwrap_or(d.recover_deg_per_sec),
                    climb: w.climb,
                    max_climb_deg: w.max_climb_deg,
                    aim_mul: w.aim_mul.unwrap_or(d.aim_mul),
                    fp_back: w.fp_back.unwrap_or(d.fp_back),
                    fp_tilt_deg: w.fp_tilt_deg.unwrap_or(d.fp_tilt_deg),
                    fp_recover_sec: w.fp_recover_sec.unwrap_or(d.fp_recover_sec),
                    flash_light: w.flash_light.unwrap_or(d.flash_light),
                    flash_duration_sec: w.flash_duration_sec.unwrap_or(d.flash_duration_sec),
                    no_flash: w.no_flash,
                };
                (id, def)
            })
            .collect();
        Ok(Self {
            defaults: d,
            weapons,
        })
    }

    /// Returns the gun-feel definition for `weapon_id`, if configured.
    pub fn get(&self, weapon_id: &str) -> Option<&GunFeelDef> {
        self.weapons.get(weapon_id)
    }
}

/// Live recoil/kick state updated each shot and recovered each frame.
#[derive(Clone, Debug, PartialEq)]
pub struct GunFeelState {
    pub pitch_deg: f32,
    pub yaw_deg: f32,
    pub climb_deg: f32,
    pub fp_back: f32,
    pub fp_tilt_deg: f32,
    pub rng_state: u64,
}

impl Default for GunFeelState {
    fn default() -> Self {
        Self {
            pitch_deg: 0.0,
            yaw_deg: 0.0,
            climb_deg: 0.0,
            fp_back: 0.0,
            fp_tilt_deg: 0.0,
            rng_state: XORSHIFT_SEED,
        }
    }
}

impl GunFeelState {
    /// Applies one shot's view kick, optional climb, and first-person weapon kick.
    pub fn on_shot(&mut self, def: &GunFeelDef, aiming: bool) {
        let mul = if aiming { def.aim_mul } else { 1.0 };
        let kick = def.kick_deg * mul;
        if def.climb {
            self.climb_deg = (self.climb_deg + kick).min(def.max_climb_deg);
            let max_pitch = (def.max_climb_deg - self.climb_deg).max(0.0);
            self.pitch_deg = self.pitch_deg.min(max_pitch);
        } else {
            self.pitch_deg += kick;
        }
        let yaw_rand = self.next_signed_unit();
        self.yaw_deg += yaw_rand * def.kick_yaw_deg * mul;
        self.fp_back += def.fp_back * mul;
        self.fp_tilt_deg += def.fp_tilt_deg * mul;
    }

    /// Recovers view kick toward 0 at `recover_deg_per_sec` and springs FP kick back over ~`fp_recover_sec`.
    pub fn update(&mut self, dt: f32, def: Option<&GunFeelDef>) {
        if dt <= 0.0 || !dt.is_finite() {
            return;
        }
        let recover_rate = def
            .map_or(DEFAULT_RECOVER_DEG_PER_SEC, |d| d.recover_deg_per_sec)
            .max(0.0);
        let step = recover_rate * dt;
        self.pitch_deg = move_toward_zero(self.pitch_deg, step);
        self.yaw_deg = move_toward_zero(self.yaw_deg, step);
        self.climb_deg = move_toward_zero(self.climb_deg, step);

        let fp_recover = def
            .map_or(DEFAULT_FP_RECOVER_SEC, |d| d.fp_recover_sec)
            .max(MIN_FP_RECOVER_SEC);
        let decay = (-dt * FP_DECAY_EXPONENT / fp_recover).exp();
        self.fp_back *= decay;
        self.fp_tilt_deg *= decay;
        if self.fp_back.abs() < FP_SETTLE_EPSILON {
            self.fp_back = 0.0;
        }
        if self.fp_tilt_deg.abs() < FP_SETTLE_EPSILON {
            self.fp_tilt_deg = 0.0;
        }
    }

    /// Current camera kick in degrees: `(pitch_deg, yaw_deg)`.
    pub fn view_kick_deg(&self) -> (f32, f32) {
        (self.pitch_deg + self.climb_deg, self.yaw_deg)
    }

    /// Current first-person weapon kick: `(back_metres, tilt_deg)`.
    pub fn fp_kick(&self) -> (f32, f32) {
        (self.fp_back, self.fp_tilt_deg)
    }

    fn next_signed_unit(&mut self) -> f32 {
        let mut x = if self.rng_state == 0 {
            XORSHIFT_SEED
        } else {
            self.rng_state
        };
        x ^= x << 13; // new: DECISIONS 2026-11-25
        x ^= x >> 7; // new: DECISIONS 2026-11-25
        x ^= x << 17; // new: DECISIONS 2026-11-25
        self.rng_state = x;
        let unit = (x >> 40) as f32 / ((1u64 << 24) - 1) as f32; // new: DECISIONS 2026-11-25
        unit * 2.0 - 1.0
    }
}

fn move_toward_zero(val: f32, step: f32) -> f32 {
    if val > step {
        val - step
    } else if val < -step {
        val + step
    } else {
        0.0
    }
}
