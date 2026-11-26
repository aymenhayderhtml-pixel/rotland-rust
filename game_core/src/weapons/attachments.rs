use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::weapons::{WeaponDef, WeaponScope};

/// Mount slot on a weapon. At most one attachment per slot can be active at a time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttachSlot {
    #[default]
    Optic,
    Muzzle,
    Magazine,
    Grip,
    Stock,
}

impl AttachSlot {
    /// All mount slots in stable order.
    pub const ALL: [Self; 5] = [
        Self::Optic,
        Self::Muzzle,
        Self::Magazine,
        Self::Grip,
        Self::Stock,
    ];

    /// Lowercase slot key used in JSON files (`data/attachments.json`, `data/weapon_mounts.json`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Optic => "optic",
            Self::Muzzle => "muzzle",
            Self::Magazine => "magazine",
            Self::Grip => "grip",
            Self::Stock => "stock",
        }
    }
}

/// Stat modifiers applied by one attachment.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AttachEffects {
    pub spread_mult: f32,
    pub kick_mult: f32,
    pub noise_mult: f32,
    pub mag_bonus: f32,
    pub reload_mult: f32,
    pub scope_fov: Option<f32>,
}

impl Default for AttachEffects {
    fn default() -> Self {
        Self {
            spread_mult: 1.0,
            kick_mult: 1.0,
            noise_mult: 1.0,
            mag_bonus: 0.0,
            reload_mult: 1.0,
            scope_fov: None,
        }
    }
}

/// One weapon attachment loaded from `data/attachments.json`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AttachmentDef {
    /// The key in `data/attachments.json`. Filled in on load.
    #[serde(skip)]
    pub id: String,
    pub name: String,
    pub slot: AttachSlot,
    /// Weapon ids (keys in `data/weapons.json`) this attachment fits on.
    pub fits: Vec<String>,
    pub model: String,
    pub price_scrap: u32,
    pub effects: AttachEffects,
}

impl AttachmentDef {
    /// Returns true if this attachment can be mounted on `weapon_id`.
    pub fn fits_weapon(&self, weapon_id: &str) -> bool {
        self.fits.iter().any(|w| w == weapon_id)
    }
}

#[derive(Debug)]
pub enum AttachmentLoadError {
    Io(std::io::Error),
    Parse(serde_json::Error),
    Validation(String),
}

impl std::fmt::Display for AttachmentLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "could not read attachments file: {e}"),
            Self::Parse(e) => write!(f, "could not parse attachments file: {e}"),
            Self::Validation(msg) => write!(f, "invalid attachment definition: {msg}"),
        }
    }
}

impl std::error::Error for AttachmentLoadError {}

/// Every attachment from `data/attachments.json`, keyed by id.
#[derive(Clone, Debug, Default)]
pub struct AttachmentRegistry {
    defs: BTreeMap<String, AttachmentDef>,
}

impl AttachmentRegistry {
    /// Loads `data/attachments.json` from the repo root via `paths::repo_path`.
    pub fn from_repo() -> Result<Self, AttachmentLoadError> {
        let path = paths::repo_path("data/attachments.json");
        Self::load(&path)
    }

    /// Reads an attachments JSON file from `path`.
    pub fn load(path: &Path) -> Result<Self, AttachmentLoadError> {
        let text = std::fs::read_to_string(path).map_err(AttachmentLoadError::Io)?;
        Self::from_json(&text)
    }

    /// Parses attachments JSON text, assigns each entry's `id`, and validates numeric fields.
    pub fn from_json(text: &str) -> Result<Self, AttachmentLoadError> {
        let mut defs: BTreeMap<String, AttachmentDef> =
            serde_json::from_str(text).map_err(AttachmentLoadError::Parse)?;
        if defs.is_empty() {
            return Err(AttachmentLoadError::Validation(
                "attachments data is empty".to_string(),
            ));
        }

        for (id, def) in defs.iter_mut() {
            def.id = id.clone();
            validate_attachment(id, def)?;
        }

        Ok(Self { defs })
    }

    pub fn get(&self, id: &str) -> Option<&AttachmentDef> {
        self.defs.get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.defs.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    /// Every attachment definition in id order.
    pub fn iter(&self) -> impl Iterator<Item = &AttachmentDef> {
        self.defs.values()
    }

    /// Every attachment that fits on `weapon_id`, in id order.
    pub fn for_weapon<'a>(&'a self, weapon_id: &'a str) -> impl Iterator<Item = &'a AttachmentDef> {
        self.defs
            .values()
            .filter(move |def| def.fits_weapon(weapon_id))
    }
}

fn validate_attachment(id: &str, def: &AttachmentDef) -> Result<(), AttachmentLoadError> {
    if def.name.is_empty() {
        return Err(AttachmentLoadError::Validation(format!(
            "attachment '{id}' has empty name"
        )));
    }
    let fx = &def.effects;
    for (field, val) in [
        ("spreadMult", fx.spread_mult),
        ("kickMult", fx.kick_mult),
        ("noiseMult", fx.noise_mult),
        ("reloadMult", fx.reload_mult),
    ] {
        if !val.is_finite() || val <= 0.0 {
            return Err(AttachmentLoadError::Validation(format!(
                "attachment '{id}' has invalid {field} {val}"
            )));
        }
    }
    if !fx.mag_bonus.is_finite() || fx.mag_bonus < 0.0 {
        return Err(AttachmentLoadError::Validation(format!(
            "attachment '{id}' has invalid magBonus {}",
            fx.mag_bonus
        )));
    }
    if let Some(fov) = fx.scope_fov
        && (!fov.is_finite() || fov <= 0.0)
    {
        return Err(AttachmentLoadError::Validation(format!(
            "attachment '{id}' has invalid scopeFov {fov}"
        )));
    }
    Ok(())
}

/// Returns true if `attachment_id` exists in `reg` and fits `weapon_id`.
pub fn can_fit(attachment_id: &str, weapon_id: &str, reg: &AttachmentRegistry) -> bool {
    reg.get(attachment_id)
        .is_some_and(|def| def.fits_weapon(weapon_id))
}

fn collect_fitted_mods<'a>(
    weapon_id: &str,
    mods: &[String],
    reg: &'a AttachmentRegistry,
) -> Vec<&'a AttachmentDef> {
    let mut used_slots = BTreeSet::new();
    let mut result = Vec::new();
    for mod_id in mods {
        if let Some(def) = reg.get(mod_id)
            && def.fits_weapon(weapon_id)
            && used_slots.insert(def.slot)
        {
            result.push(def);
        }
    }
    result
}

/// Combined recoil/kick multiplier for `mods` fitted to `weapon_id`.
/// Unknown ids, wrong-weapon mods, and extra mods sharing a slot are ignored.
pub fn kick_mult(mods: &[String], weapon_id: &str, reg: &AttachmentRegistry) -> f32 {
    collect_fitted_mods(weapon_id, mods, reg)
        .into_iter()
        .fold(1.0, |acc, def| acc * def.effects.kick_mult)
}

/// Computes a weapon's effective stats with `mods` applied.
/// Pure: unknown ids, wrong-weapon mods, and duplicate slot entries are ignored.
pub fn effective_stats(base: &WeaponDef, mods: &[String], reg: &AttachmentRegistry) -> WeaponDef {
    let mut out = base.clone();
    let mut mag_bonus_sum = 0.0_f32;

    for def in collect_fitted_mods(&base.id, mods, reg) {
        let fx = &def.effects;
        out.noise_radius *= fx.noise_mult;
        out.reload_time *= fx.reload_mult;
        out.recoil_climb *= fx.kick_mult;
        mag_bonus_sum += fx.mag_bonus;

        if let Some(fov) = fx.scope_fov {
            // Magnified scopes tighten aimed spread and override zoom FOV without changing hip spread.
            let sensitivity = out.scope.as_ref().map_or(1.0, |s| s.sensitivity);
            out.scope = Some(WeaponScope { fov, sensitivity });
            out.spread_aim_deg *= fx.spread_mult;
        } else {
            out.spread_hip_deg *= fx.spread_mult;
            out.spread_aim_deg *= fx.spread_mult;
        }
    }

    if base.magazine_size > 0 && mag_bonus_sum > 0.0 {
        out.magazine_size = ((base.magazine_size as f32) * (1.0 + mag_bonus_sum))
            .round()
            .max(1.0) as u32;
    }

    out
}
