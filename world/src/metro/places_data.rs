//! R42: the serde shape of `data/places/<kind>.json` (hospital, police station, fire station).
//!
//! All positions in the file are building-local GAME metres: `x` = Blender x, `y` = Blender z
//! (up), `z` = -Blender y. The building front (Blender +Y) is game local -Z.
//!
//! Each file is read once and then kept for the life of the process, keyed by its repo path.

use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::metro::layout::Reserved;
use crate::metro::places::PLACE_KINDS;

fn default_scale() -> f32 {
    1.0 // new: DECISIONS 2026-11-27 (a place entry without `scale` is drawn at 1x)
}

/// One prop or crate in a place data file (`props`, `crates`, `yards.*`, `yardCrates.*`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaceEntry {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    /// Model id in the asset pack.
    pub model: String,
    /// Uniform model scale (default 1).
    #[serde(default = "default_scale")]
    pub scale: f32,
    /// Loot table override for a crate entry (the place's table when absent).
    #[serde(default)]
    pub table: Option<String>,
}

/// The three yard lots of a place, each in its own lot-local frame (`a`, `b`, `c`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct YardEntries {
    pub a: Vec<PlaceEntry>,
    pub b: Vec<PlaceEntry>,
    pub c: Vec<PlaceEntry>,
}

/// Shape of one `data/places/<kind>.json` file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceData {
    /// Main building model id (`metro_hospital` / `metro_police` / `metro_fire_station`).
    pub model: String,
    /// Building footprint `[width, depth]` in metres.
    pub footprint: [f32; 2],
    pub floors: u32,
    /// Crates inside or right around the main building (main-lot local).
    #[serde(default)]
    pub crates: Vec<PlaceEntry>,
    /// Props inside or right around the main building (main-lot local).
    #[serde(default)]
    pub props: Vec<PlaceEntry>,
    /// Yard lots `a`, `b`, `c`, each in its own lot-local frame.
    #[serde(default)]
    pub yards: YardEntries,
    /// Yard crates `a`, `b`, `c`, each in its own lot-local frame.
    #[serde(default)]
    pub yard_crates: YardEntries,
}

impl PlaceData {
    /// Parses and checks one place file.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let data: Self = serde_json::from_str(json).map_err(|e| format!("places json: {e}"))?;
        data.validate()?;
        Ok(data)
    }

    /// Reads one place file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&json)
    }

    fn validate(&self) -> Result<(), String> {
        let ok_footprint = self.footprint[0] > 0.0 && self.footprint[1] > 0.0;
        if self.model.is_empty() || self.floors == 0 || !ok_footprint {
            return Err("places json: model, floors or footprint is missing or invalid".into());
        }
        let props = self.props.iter().chain(self.all_yard_props());
        for entry in props {
            if entry.model.is_empty() {
                return Err("places json: a prop entry has no model".into());
            }
        }
        // A crate entry may have an empty model: the crate stands on the prop it sits in.
        let all = self.all_entries();
        for entry in all {
            let finite = entry.x.is_finite()
                && entry.y.is_finite()
                && entry.z.is_finite()
                && entry.yaw.is_finite()
                && entry.scale > 0.0;
            if !finite {
                return Err(format!("places json: bad numbers on entry '{}'", entry.model));
            }
        }
        Ok(())
    }

    /// Every entry of the file: crates, props and all yards, in file order.
    pub fn all_entries(&self) -> impl Iterator<Item = &PlaceEntry> {
        self.crates
            .iter()
            .chain(&self.props)
            .chain(self.all_yard_props())
            .chain(self.all_yard_crates())
    }

    /// Yard props of `a`, `b` and `c` in that order.
    pub fn all_yard_props(&self) -> impl Iterator<Item = &PlaceEntry> {
        self.yards.a.iter().chain(&self.yards.b).chain(&self.yards.c)
    }

    /// Yard crates of `a`, `b` and `c` in that order.
    pub fn all_yard_crates(&self) -> impl Iterator<Item = &PlaceEntry> {
        self.yard_crates
            .a
            .iter()
            .chain(&self.yard_crates.b)
            .chain(&self.yard_crates.c)
    }
}

/// Every loaded place file, keyed by its `data/places/` file stem.
fn cache() -> &'static HashMap<&'static str, PlaceData> {
    static CACHE: OnceLock<HashMap<&'static str, PlaceData>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut map = HashMap::new();
        for (_, key, _) in PLACE_KINDS {
            let path = paths::repo_path(&format!("data/places/{key}.json"));
            // The R42 model tasks write these files; a missing one just means no place yet.
            if !path.exists() {
                continue;
            }
            match PlaceData::load(&path) {
                Ok(data) => {
                    map.insert(key, data);
                }
                Err(err) => eprintln!("[metro] {key} place data skipped: {err}"),
            }
        }
        map
    })
}

/// The place data of one reserved kind, or None when its file is missing or invalid.
pub fn place_data(kind: Reserved) -> Option<&'static PlaceData> {
    for (place_kind, key, _table) in PLACE_KINDS {
        if place_kind == kind {
            return cache().get(key);
        }
    }
    None
}
