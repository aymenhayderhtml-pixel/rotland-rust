//! Procedural finger grip layer for held weapons.
//!
//! Reads per-weapon-class thumb and finger curl amounts from `data/grips.json`,
//! fades smoothly on weapon changes and left-hand grip transitions, and rotates
//! the 8 finger/thumb joints in place without per-frame allocations.

use std::collections::HashMap;
use std::sync::OnceLock;

use glam::{Quat, Vec3};
use serde::Deserialize;

use crate::stance::StanceClass;

const DEFAULT_FADE_RATE: f32 = 12.0; // data: grips.json fadeRate
const DEFAULT_MAX_FINGERS_DEG: f32 = 75.0; // data: grips.json maxFingersCurlDeg
const DEFAULT_MAX_THUMB_DEG: f32 = 50.0; // data: grips.json maxThumbCurlDeg
const THUMB_OPPOSE_RATIO: f32 = 0.4; // new: DECISIONS 2026-11-24
const CURL_EPSILON: f32 = 0.001;

/// Weapon class key matching `data/grips.json` and `data/stances.json`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GripClass {
    #[default]
    None,
    Pistol,
    Rifle,
    Shotgun,
    Melee1H,
    Melee2H,
    Spear,
    Bow,
    Rod,
}

impl GripClass {
    /// Maps a `StanceClass` to the corresponding `GripClass`.
    pub fn from_stance(stance: StanceClass) -> Self {
        match stance {
            StanceClass::Pistol => Self::Pistol,
            StanceClass::Rifle => Self::Rifle,
            StanceClass::Shotgun => Self::Shotgun,
            StanceClass::Melee1H => Self::Melee1H,
            StanceClass::Melee2H => Self::Melee2H,
            StanceClass::Spear => Self::Spear,
            StanceClass::Bow => Self::Bow,
            StanceClass::Rod => Self::Rod,
        }
    }

    fn key(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Pistol => "pistol",
            Self::Rifle => "rifle",
            Self::Shotgun => "shotgun",
            Self::Melee1H => "melee_1h",
            Self::Melee2H => "melee_2h",
            Self::Spear => "spear",
            Self::Bow => "bow",
            Self::Rod => "rod",
        }
    }
}

/// Target curl fractions (`0.0..=1.0`) for each hand's thumb and fingers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GripPose {
    pub right_thumb: f32,
    pub right_fingers: f32,
    pub left_thumb: f32,
    pub left_fingers: f32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawGripsFile {
    #[serde(default)]
    version: u32,
    #[serde(default = "default_fade_rate")]
    fade_rate: f32,
    #[serde(default = "default_max_fingers_deg")]
    max_fingers_curl_deg: f32,
    #[serde(default = "default_max_thumb_deg")]
    max_thumb_curl_deg: f32,
    classes: HashMap<String, GripPose>,
}

fn default_fade_rate() -> f32 {
    DEFAULT_FADE_RATE
}
fn default_max_fingers_deg() -> f32 {
    DEFAULT_MAX_FINGERS_DEG
}
fn default_max_thumb_deg() -> f32 {
    DEFAULT_MAX_THUMB_DEG
}

/// Parsed grip table loaded from `data/grips.json`.
#[derive(Clone, Debug)]
pub struct GripTable {
    fade_rate: f32,
    max_fingers_rad: f32,
    max_thumb_rad: f32,
    classes: HashMap<String, GripPose>,
}

impl Default for GripTable {
    fn default() -> Self {
        Self {
            fade_rate: DEFAULT_FADE_RATE,
            max_fingers_rad: DEFAULT_MAX_FINGERS_DEG.to_radians(),
            max_thumb_rad: DEFAULT_MAX_THUMB_DEG.to_radians(),
            classes: HashMap::new(),
        }
    }
}

impl GripTable {
    /// Parses a `grips.json` document into a `GripTable`.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        let raw: RawGripsFile = serde_json::from_str(text)?;
        let _ = raw.version;
        let classes = raw
            .classes
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    GripPose {
                        right_thumb: v.right_thumb.clamp(0.0, 1.0),
                        right_fingers: v.right_fingers.clamp(0.0, 1.0),
                        left_thumb: v.left_thumb.clamp(0.0, 1.0),
                        left_fingers: v.left_fingers.clamp(0.0, 1.0),
                    },
                )
            })
            .collect();
        Ok(Self {
            fade_rate: raw.fade_rate.max(0.0),
            max_fingers_rad: raw.max_fingers_curl_deg.to_radians(),
            max_thumb_rad: raw.max_thumb_curl_deg.to_radians(),
            classes,
        })
    }

    /// Cached with `OnceLock` from `paths::repo_path("data/grips.json")`.
    pub fn load_shared() -> &'static GripTable {
        static SHARED: OnceLock<GripTable> = OnceLock::new();
        SHARED.get_or_init(|| {
            let path = paths::repo_path("data/grips.json");
            match std::fs::read_to_string(&path) {
                Ok(text) => Self::from_json(&text).unwrap_or_else(|err| {
                    log::warn!("failed to parse {}: {err}; using defaults", path.display());
                    Self::default()
                }),
                Err(err) => {
                    log::warn!("failed to read {}: {err}; using defaults", path.display());
                    Self::default()
                }
            }
        })
    }

    /// Returns the configured grip pose for `class`, or all zeros if missing.
    pub fn for_class(&self, class: GripClass) -> GripPose {
        self.classes.get(class.key()).copied().unwrap_or_default()
    }
}

/// Cached joint indices for the 8 finger/thumb bones (resolved once at init).
#[derive(Clone, Copy, Debug, Default)]
pub struct GripJoints {
    pub thumb_1_l: Option<usize>,
    pub thumb_2_l: Option<usize>,
    pub fingers_1_l: Option<usize>,
    pub fingers_2_l: Option<usize>,
    pub thumb_1_r: Option<usize>,
    pub thumb_2_r: Option<usize>,
    pub fingers_1_r: Option<usize>,
    pub fingers_2_r: Option<usize>,
}

impl GripJoints {
    /// Resolves finger joint indices from skeleton joint names.
    pub fn from_names(names: &[&str]) -> Self {
        let find = |target: &str| names.iter().position(|&n| n == target);
        Self {
            thumb_1_l: find("thumb_1_L"),
            thumb_2_l: find("thumb_2_L"),
            fingers_1_l: find("fingers_1_L"),
            fingers_2_l: find("fingers_2_L"),
            thumb_1_r: find("thumb_1_R"),
            thumb_2_r: find("thumb_2_R"),
            fingers_1_r: find("fingers_1_R"),
            fingers_2_r: find("fingers_2_R"),
        }
    }
}

/// Per-character procedural grip state that fades with weapon and off-hand grip changes.
#[derive(Clone, Debug)]
pub struct GripController {
    class: GripClass,
    target: GripPose,
    current: GripPose,
    fade_rate: f32,
    max_fingers_rad: f32,
    max_thumb_rad: f32,
}

impl Default for GripController {
    fn default() -> Self {
        Self::new()
    }
}

impl GripController {
    /// Creates a controller using the shared `data/grips.json` table.
    pub fn new() -> Self {
        Self::with_table(GripTable::load_shared())
    }

    /// Creates a controller using an explicit `GripTable` (preferred for tests).
    pub fn with_table(table: &GripTable) -> Self {
        Self {
            class: GripClass::None,
            target: GripPose::default(),
            current: GripPose::default(),
            fade_rate: table.fade_rate,
            max_fingers_rad: table.max_fingers_rad,
            max_thumb_rad: table.max_thumb_rad,
        }
    }

    /// Updates the target grip class from the shared table.
    pub fn set_class(&mut self, class: GripClass) {
        self.set_class_from_table(class, GripTable::load_shared());
    }

    /// Updates the target grip class from an explicit `GripTable`.
    pub fn set_class_from_table(&mut self, class: GripClass, table: &GripTable) {
        if self.class == class {
            return;
        }
        self.class = class;
        self.target = table.for_class(class);
        self.fade_rate = table.fade_rate;
        self.max_fingers_rad = table.max_fingers_rad;
        self.max_thumb_rad = table.max_thumb_rad;
    }

    /// Advances curl weights smoothly; left hand curls only while `left_holding` is true.
    pub fn tick(&mut self, dt: f32, left_holding: bool) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let k = 1.0 - (-self.fade_rate * dt).exp();
        let left_t = if left_holding { self.target.left_thumb } else { 0.0 };
        let left_f = if left_holding { self.target.left_fingers } else { 0.0 };

        self.current.right_thumb += (self.target.right_thumb - self.current.right_thumb) * k;
        self.current.right_fingers += (self.target.right_fingers - self.current.right_fingers) * k;
        self.current.left_thumb += (left_t - self.current.left_thumb) * k;
        self.current.left_fingers += (left_f - self.current.left_fingers) * k;
    }

    /// Returns `(right_thumb, right_fingers, left_thumb, left_fingers)` current curl weights.
    pub fn curls(&self) -> (f32, f32, f32, f32) {
        (
            self.current.right_thumb,
            self.current.right_fingers,
            self.current.left_thumb,
            self.current.left_fingers,
        )
    }

    /// Applies procedural finger/thumb curls in place to `local` pose transforms.
    pub fn apply(&self, joints: &GripJoints, local: &mut [(Vec3, Quat, Vec3)]) {
        // Right hand (+Z rotates fingers from +Y toward -X palm; -X opposes thumb)
        apply_fingers(local, joints.fingers_1_r, joints.fingers_2_r, self.current.right_fingers * self.max_fingers_rad);
        apply_thumb(local, joints.thumb_1_r, joints.thumb_2_r, self.current.right_thumb * self.max_thumb_rad, 1.0);

        // Left hand (-Z rotates fingers from +Y toward +X palm; +X opposes thumb)
        apply_fingers(local, joints.fingers_1_l, joints.fingers_2_l, -self.current.left_fingers * self.max_fingers_rad);
        apply_thumb(local, joints.thumb_1_l, joints.thumb_2_l, -self.current.left_thumb * self.max_thumb_rad, -1.0);
    }
}

fn apply_fingers(local: &mut [(Vec3, Quat, Vec3)], j1: Option<usize>, j2: Option<usize>, angle_z: f32) {
    if angle_z.abs() <= CURL_EPSILON {
        return;
    }
    let rot = Quat::from_rotation_z(angle_z);
    for idx in [j1, j2].into_iter().flatten() {
        if let Some(t) = local.get_mut(idx) {
            t.1 = (t.1 * rot).normalize();
        }
    }
}

fn apply_thumb(local: &mut [(Vec3, Quat, Vec3)], j1: Option<usize>, j2: Option<usize>, angle_z: f32, side_sign: f32) {
    if angle_z.abs() <= CURL_EPSILON {
        return;
    }
    let oppose_x = -angle_z.abs() * THUMB_OPPOSE_RATIO * side_sign;
    let rot = Quat::from_rotation_z(angle_z) * Quat::from_rotation_x(oppose_x);
    for idx in [j1, j2].into_iter().flatten() {
        if let Some(t) = local.get_mut(idx) {
            t.1 = (t.1 * rot).normalize();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_grips_json_parses_all_classes() {
        let table = GripTable::load_shared();
        for class in [
            GripClass::None,
            GripClass::Pistol,
            GripClass::Rifle,
            GripClass::Shotgun,
            GripClass::Melee1H,
            GripClass::Melee2H,
            GripClass::Spear,
            GripClass::Bow,
            GripClass::Rod,
        ] {
            let p = table.for_class(class);
            assert!((0.0..=1.0).contains(&p.right_thumb));
            assert!((0.0..=1.0).contains(&p.right_fingers));
            assert!((0.0..=1.0).contains(&p.left_thumb));
            assert!((0.0..=1.0).contains(&p.left_fingers));
        }
        assert_eq!(table.for_class(GripClass::None), GripPose::default());
        assert!(table.for_class(GripClass::Melee2H).left_fingers > 0.5);
    }

    #[test]
    fn left_hand_curls_only_while_holding_and_fades_on_change() {
        let table = GripTable::load_shared();
        let mut ctrl = GripController::with_table(table);
        ctrl.set_class_from_table(GripClass::Rifle, table);

        // Right hand curls, left hand stays open when not holding
        for _ in 0..30 {
            ctrl.tick(0.05, false);
        }
        let (rt, rf, lt, lf) = ctrl.curls();
        assert!(rt > 0.8 && rf > 0.8);
        assert!(lt < 0.01 && lf < 0.01);

        // Left hand curls when holding
        for _ in 0..30 {
            ctrl.tick(0.05, true);
        }
        let (_, _, lt2, lf2) = ctrl.curls();
        assert!(lt2 > 0.7 && lf2 > 0.8);

        // Switching to None fades all curls back to 0
        ctrl.set_class_from_table(GripClass::None, table);
        ctrl.tick(0.02, true);
        let (_, _, lt_mid, _) = ctrl.curls();
        assert!(lt_mid > 0.1 && lt_mid < lt2);

        for _ in 0..40 {
            ctrl.tick(0.05, true);
        }
        let (rt_end, rf_end, lt_end, lf_end) = ctrl.curls();
        assert!(rt_end < 0.01 && rf_end < 0.01 && lt_end < 0.01 && lf_end < 0.01);
    }

    #[test]
    fn apply_rotates_only_finger_joints_in_place() {
        let names = [
            "root",
            "hand_R",
            "fingers_1_R",
            "fingers_2_R",
            "thumb_1_R",
            "thumb_2_R",
            "fingers_1_L",
            "fingers_2_L",
            "thumb_1_L",
            "thumb_2_L",
        ];
        let joints = GripJoints::from_names(&names);
        let table = GripTable::load_shared();
        let mut ctrl = GripController::with_table(table);
        ctrl.set_class_from_table(GripClass::Melee2H, table);
        for _ in 0..40 {
            ctrl.tick(0.05, true);
        }

        let mut local = vec![(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE); names.len()];
        ctrl.apply(&joints, &mut local);

        assert_eq!(local[0].1, Quat::IDENTITY);
        assert_eq!(local[1].1, Quat::IDENTITY);
        for i in 2..names.len() {
            assert_ne!(local[i].1, Quat::IDENTITY, "joint {} should be curled", names[i]);
        }
    }
}
