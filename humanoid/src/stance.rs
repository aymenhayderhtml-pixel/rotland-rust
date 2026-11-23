//! Data-driven weapon holding stances, aiming transitions, and left-hand IK blending.

use std::collections::HashMap;
use std::sync::OnceLock;

use glam::Vec3;
use serde::Deserialize;
use serde::de::Error as _;

const DEFAULT_VERSION: u32 = 1; // data: stances.json version
const DEFAULT_READY_BLEND_RATE: f32 = 10.0; // data: stances.json readyBlendRate
const DEFAULT_AIM_BLEND_RATE: f32 = 14.0; // data: stances.json aimBlendRate
const DEFAULT_IK_BLEND_RATE: f32 = 14.0; // data: stances.json ikBlendRate
const WEIGHT_MIN: f32 = 0.0; // new: DECISIONS 2026-11-23
const WEIGHT_MAX: f32 = 1.0; // new: DECISIONS 2026-11-23

/// Broad weapon holding category mapped from `data/stances.json`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StanceClass {
    Pistol,
    Rifle,
    Shotgun,
    Melee1H,
    Melee2H,
    Spear,
    Bow,
    Rod,
}

impl StanceClass {
    fn from_key(key: &str) -> Option<Self> {
        match key {
            "pistol" => Some(Self::Pistol),
            "rifle" => Some(Self::Rifle),
            "shotgun" => Some(Self::Shotgun),
            "melee_1h" => Some(Self::Melee1H),
            "melee_2h" => Some(Self::Melee2H),
            "spear" => Some(Self::Spear),
            "bow" => Some(Self::Bow),
            "rod" => Some(Self::Rod),
            _ => None,
        }
    }
}

/// Weapon grip position and Euler rotation (in degrees) relative to `WeaponMount`.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct HoldTransform {
    pub pos: Vec3,
    pub rot_deg: Vec3,
}

/// Resolved stance definition for a single weapon id.
#[derive(Clone, Debug, PartialEq)]
pub struct StanceDef {
    pub class: StanceClass,
    pub ready_clip: String,
    pub aim_clip: Option<String>,
    pub left_hand_grip: bool,
    pub grip_when_aiming: bool,
    pub ready_hold: HoldTransform,
    pub aim_hold: HoldTransform,
    pub off_hand_grip: Option<Vec3>,
}

/// Parsed weapon stance definitions and blend rates from `data/stances.json`.
#[derive(Clone, Debug)]
pub struct StanceTable {
    ready_rate: f32,
    aim_rate: f32,
    ik_rate: f32,
    weapons: HashMap<String, StanceDef>,
}

impl Default for StanceTable {
    fn default() -> Self {
        Self {
            ready_rate: DEFAULT_READY_BLEND_RATE,
            aim_rate: DEFAULT_AIM_BLEND_RATE,
            ik_rate: DEFAULT_IK_BLEND_RATE,
            weapons: HashMap::new(),
        }
    }
}

impl StanceTable {
    /// Parses a `stances.json` document into per-weapon `StanceDef` entries.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        let raw: RawStanceFile = serde_json::from_str(text)?;
        let _ = raw.version;
        let mut weapons = HashMap::with_capacity(raw.weapons.len());

        for (weapon_id, class_key) in &raw.weapons {
            let class = StanceClass::from_key(class_key).ok_or_else(|| {
                serde_json::Error::custom(format!(
                    "unknown stance class '{class_key}' for weapon '{weapon_id}'"
                ))
            })?;
            let class_def = raw.classes.get(class_key).ok_or_else(|| {
                serde_json::Error::custom(format!(
                    "missing class definition '{class_key}' for weapon '{weapon_id}'"
                ))
            })?;
            let ov = raw.overrides.get(weapon_id);
            let ready_hold = ov
                .and_then(|o| o.ready_hold.as_ref())
                .map(RawHold::to_hold)
                .unwrap_or_else(|| class_def.ready_hold.to_hold());
            let aim_hold = ov
                .and_then(|o| o.aim_hold.as_ref())
                .map(RawHold::to_hold)
                .unwrap_or_else(|| {
                    class_def
                        .aim_hold
                        .as_ref()
                        .map_or(ready_hold, RawHold::to_hold)
                });
            let off_hand_grip = ov
                .and_then(|o| o.off_hand_grip)
                .or(class_def.off_hand_grip)
                .map(Vec3::from_array)
                .or_else(|| crate::weapon::off_hand_grip_for(weapon_id));

            weapons.insert(
                weapon_id.clone(),
                StanceDef {
                    class,
                    ready_clip: class_def.ready_clip.clone(),
                    aim_clip: class_def.aim_clip.clone(),
                    left_hand_grip: class_def.left_hand_grip,
                    grip_when_aiming: class_def.grip_when_aiming,
                    ready_hold,
                    aim_hold,
                    off_hand_grip,
                },
            );
        }

        Ok(Self {
            ready_rate: raw.ready_blend_rate,
            aim_rate: raw.aim_blend_rate,
            ik_rate: raw.ik_blend_rate,
            weapons,
        })
    }

    /// Cached with `OnceLock` from `paths::repo_path("data/stances.json")`.
    /// If missing or broken, logs a warning once and returns an empty table.
    pub fn load_shared() -> &'static StanceTable {
        static SHARED: OnceLock<StanceTable> = OnceLock::new();
        SHARED.get_or_init(|| {
            let path = paths::repo_path("data/stances.json");
            match std::fs::read_to_string(&path) {
                Ok(text) => match Self::from_json(&text) {
                    Ok(table) => table,
                    Err(err) => {
                        log::warn!(
                            "failed to parse {}: {err}; using empty stance table",
                            path.display()
                        );
                        Self::default()
                    }
                },
                Err(err) => {
                    log::warn!(
                        "failed to read {}: {err}; using empty stance table",
                        path.display()
                    );
                    Self::default()
                }
            }
        })
    }

    pub fn for_weapon(&self, weapon: &str) -> Option<&StanceDef> {
        self.weapons.get(weapon)
    }

    pub fn ready_rate(&self) -> f32 {
        self.ready_rate
    }

    pub fn aim_rate(&self) -> f32 {
        self.aim_rate
    }

    pub fn ik_rate(&self) -> f32 {
        self.ik_rate
    }
}

/// Per-character weapon stance and aim blend state machine.
#[derive(Clone, Debug, Default)]
pub struct StanceController {
    current: Option<StanceDef>,
    aiming: bool,
    ready_weight: f32,
    aim_weight: f32,
    left_ik_weight: f32,
}

impl StanceController {
    pub fn new() -> Self {
        Self::default()
    }

    /// Switches the active weapon stance. When the definition changes, the new stance
    /// fades in from zero and aiming resets cleanly without leftover clips.
    pub fn set_weapon(&mut self, def: Option<&StanceDef>) {
        if self.current.as_ref() == def {
            return;
        }
        self.current = def.cloned();
        self.aiming = false;
        self.ready_weight = WEIGHT_MIN;
        self.aim_weight = WEIGHT_MIN;
        self.left_ik_weight = WEIGHT_MIN;
    }

    /// Requests aiming. Ignored when no stance is active or the stance has no `aim_clip`.
    pub fn set_aiming(&mut self, aiming: bool) {
        let can_aim = self.current.as_ref().is_some_and(|d| d.aim_clip.is_some());
        self.aiming = aiming && can_aim;
    }

    pub fn is_aiming(&self) -> bool {
        self.aiming
    }

    /// Advances ready, aim, and left-hand IK blend weights with frame-rate independent smoothing.
    pub fn tick(&mut self, dt: f32, rates: (f32, f32, f32), action_active: bool) {
        let (ready_rate, aim_rate, ik_rate) = rates;
        let Some(def) = &self.current else {
            self.ready_weight = WEIGHT_MIN;
            self.aim_weight = WEIGHT_MIN;
            self.left_ik_weight = WEIGHT_MIN;
            return;
        };

        if !dt.is_finite() || dt <= WEIGHT_MIN {
            self.ready_weight = clamp_unit(self.ready_weight);
            self.aim_weight = clamp_unit(self.aim_weight);
            self.left_ik_weight = clamp_unit(self.left_ik_weight);
            return;
        }

        let has_aim = def.aim_clip.is_some();
        let aim_active = self.aiming && has_aim;
        let ready_target = WEIGHT_MAX;
        let aim_target = if aim_active { WEIGHT_MAX } else { WEIGHT_MIN };
        let wants_grip = if aim_active {
            def.grip_when_aiming
        } else {
            def.left_hand_grip
        };
        let ik_target = if !action_active && def.off_hand_grip.is_some() && wants_grip {
            WEIGHT_MAX
        } else {
            WEIGHT_MIN
        };

        self.ready_weight = exp_approach(self.ready_weight, ready_target, ready_rate, dt);
        self.aim_weight = exp_approach(self.aim_weight, aim_target, aim_rate, dt);
        self.left_ik_weight = exp_approach(self.left_ik_weight, ik_target, ik_rate, dt);
    }

    pub fn ready_clip(&self) -> Option<&str> {
        self.current.as_ref().map(|d| d.ready_clip.as_str())
    }

    pub fn aim_clip(&self) -> Option<&str> {
        self.current.as_ref().and_then(|d| d.aim_clip.as_deref())
    }

    pub fn ready_weight(&self) -> f32 {
        self.ready_weight
    }

    pub fn aim_weight(&self) -> f32 {
        self.aim_weight
    }

    pub fn left_ik_weight(&self) -> f32 {
        self.left_ik_weight
    }

    /// Returns the weapon hold transform blended from `ready_hold` to `aim_hold` by `aim_weight`.
    pub fn hold(&self) -> HoldTransform {
        let Some(def) = &self.current else {
            return HoldTransform::default();
        };
        let t = clamp_unit(self.aim_weight);
        HoldTransform {
            pos: def.ready_hold.pos.lerp(def.aim_hold.pos, t),
            rot_deg: def.ready_hold.rot_deg.lerp(def.aim_hold.rot_deg, t),
        }
    }

    pub fn off_hand_grip(&self) -> Option<Vec3> {
        self.current.as_ref().and_then(|d| d.off_hand_grip)
    }
}

fn clamp_unit(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(WEIGHT_MIN, WEIGHT_MAX)
    } else {
        WEIGHT_MIN
    }
}

fn exp_approach(current: f32, target: f32, rate: f32, dt: f32) -> f32 {
    let cur = clamp_unit(current);
    if !rate.is_finite() || rate <= WEIGHT_MIN {
        return cur;
    }
    let k = WEIGHT_MAX - (-rate * dt).exp();
    clamp_unit(cur + (target - cur) * k)
}

const fn default_version() -> u32 {
    DEFAULT_VERSION
}

const fn default_ready_rate() -> f32 {
    DEFAULT_READY_BLEND_RATE
}

const fn default_aim_rate() -> f32 {
    DEFAULT_AIM_BLEND_RATE
}

const fn default_ik_rate() -> f32 {
    DEFAULT_IK_BLEND_RATE
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawHold {
    pos: [f32; 3],
    rot: [f32; 3],
}

impl RawHold {
    fn to_hold(&self) -> HoldTransform {
        HoldTransform {
            pos: Vec3::from_array(self.pos),
            rot_deg: Vec3::from_array(self.rot),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawClassDef {
    ready_clip: String,
    #[serde(default)]
    aim_clip: Option<String>,
    #[serde(default)]
    left_hand_grip: bool,
    #[serde(default)]
    grip_when_aiming: bool,
    ready_hold: RawHold,
    #[serde(default)]
    aim_hold: Option<RawHold>,
    #[serde(default)]
    off_hand_grip: Option<[f32; 3]>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawOverride {
    #[serde(default)]
    ready_hold: Option<RawHold>,
    #[serde(default)]
    aim_hold: Option<RawHold>,
    #[serde(default)]
    off_hand_grip: Option<[f32; 3]>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawStanceFile {
    #[serde(default = "default_version")]
    version: u32,
    #[serde(default = "default_ready_rate")]
    ready_blend_rate: f32,
    #[serde(default = "default_aim_rate")]
    aim_blend_rate: f32,
    #[serde(default = "default_ik_rate")]
    ik_blend_rate: f32,
    classes: HashMap<String, RawClassDef>,
    weapons: HashMap<String, String>,
    #[serde(default)]
    overrides: HashMap<String, RawOverride>,
}
