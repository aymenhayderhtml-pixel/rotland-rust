//! Silent life settings: `LifeConfig`, crow perch chances, and the cached
//! `data/metro.json` `life` loader.

use std::sync::OnceLock;

use serde::Deserialize;

use crate::metro::layout::MetroDistrict;

/// Per-district crow perch probability per 64 m block (`data/metro.json` `life.crowPerchChance`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CrowPerchChance {
    pub edge: f32,
    pub midtown: f32,
    pub core: f32,
}

impl Default for CrowPerchChance {
    fn default() -> Self {
        Self {
            edge: 0.55,    // data: metro.json crowPerchChance
            midtown: 0.45, // data: metro.json crowPerchChance
            core: 0.35,    // data: metro.json crowPerchChance
        }
    }
}

impl CrowPerchChance {
    pub fn for_district(&self, d: MetroDistrict) -> f32 {
        match d {
            MetroDistrict::Edge => self.edge,
            MetroDistrict::Midtown => self.midtown,
            MetroDistrict::Core => self.core,
        }
    }
}

/// Silent life settings loaded from `data/metro.json` `life`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LifeConfig {
    pub pole_spacing_m: f32,
    pub pole_avenue_share: f32,
    pub pole_fallen_chance: f32,
    pub pole_leaning_chance: f32,
    pub billboard_roof_chance: f32,
    pub billboard_street_chance: f32,
    pub flag_chance: f32,
    pub plastic_chance: f32,
    pub crow_perch_chance: CrowPerchChance,
    pub crows_per_perch: [u8; 2],
}

impl Default for LifeConfig {
    fn default() -> Self {
        Self {
            pole_spacing_m: 20.0,         // data: metro.json life
            pole_avenue_share: 0.35,      // data: metro.json life
            pole_fallen_chance: 0.15,     // data: metro.json life
            pole_leaning_chance: 0.2,     // data: metro.json life
            billboard_roof_chance: 0.18,  // data: metro.json life
            billboard_street_chance: 0.12,// data: metro.json life
            flag_chance: 0.35,            // data: metro.json life
            plastic_chance: 0.5,          // data: metro.json life
            crow_perch_chance: CrowPerchChance::default(),
            crows_per_perch: [3, 9],      // data: metro.json life
        }
    }
}

impl LifeConfig {
    /// Validates probabilities and ranges.
    pub fn validate(&self) -> Result<(), String> {
        if self.pole_spacing_m <= 0.0 {
            return Err("metro.json life.poleSpacingM must be positive".into());
        }
        let chances = [
            ("poleAvenueShare", self.pole_avenue_share),
            ("poleFallenChance", self.pole_fallen_chance),
            ("poleLeaningChance", self.pole_leaning_chance),
            ("billboardRoofChance", self.billboard_roof_chance),
            ("billboardStreetChance", self.billboard_street_chance),
            ("flagChance", self.flag_chance),
            ("plasticChance", self.plastic_chance),
            ("crowPerchChance.edge", self.crow_perch_chance.edge),
            ("crowPerchChance.midtown", self.crow_perch_chance.midtown),
            ("crowPerchChance.core", self.crow_perch_chance.core),
        ];
        for (name, v) in chances {
            if !(0.0..=1.0).contains(&v) {
                return Err(format!("metro.json life.{name} ({v}) must be within 0..=1"));
            }
        }
        if self.pole_fallen_chance + self.pole_leaning_chance > 1.0 {
            return Err("metro.json life: poleFallenChance + poleLeaningChance must be <= 1".into());
        }
        if self.crows_per_perch[0] > self.crows_per_perch[1] {
            return Err("metro.json life.crowsPerPerch min must be <= max".into());
        }
        Ok(())
    }
}

#[derive(Deserialize)]
struct LifeWrapper {
    #[serde(default)]
    life: LifeConfig,
}

/// Parses and validates the top-level `life` object from a `metro.json` string.
pub fn load_life(json: &str) -> Result<LifeConfig, String> {
    let wrapper: LifeWrapper =
        serde_json::from_str(json).map_err(|e| format!("metro.json life: {e}"))?;
    wrapper.life.validate()?;
    Ok(wrapper.life)
}

/// Cached `LifeConfig` loaded from `data/metro.json` (falls back to `LifeConfig::default()`).
pub fn default_life() -> &'static LifeConfig {
    static CACHE: OnceLock<LifeConfig> = OnceLock::new();
    CACHE.get_or_init(|| {
        let path = paths::repo_path("data/metro.json");
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        let root: serde_json::Value = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()));
        if root.get("life").is_none() {
            return LifeConfig::default();
        }
        load_life(&json).expect("data/metro.json life must be valid")
    })
}
