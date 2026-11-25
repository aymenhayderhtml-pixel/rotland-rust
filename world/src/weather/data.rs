use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::biome::BiomeId;
use crate::weather::WeatherKind;

fn default_tint() -> [f32; 3] {
    // data: weather.json kinds.clear.tint
    [1.0, 1.0, 1.0]
}

/// Weighted weather entry for a single biome in `data/weather.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherWeight {
    pub kind: WeatherKind,
    pub weight: f32,
}

/// Gameplay and visual parameters for one weather kind loaded from `data/weather.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherKindDef {
    #[serde(rename = "_why", default)]
    pub why: String,
    pub fog_mult: f32,
    pub visibility: f32,
    pub noise_mask: f32,
    pub wind: f32,
    #[serde(default)]
    pub particles_per_m3: f32,
    /// Normalized RGB tint (`0.0..=1.0`), matching `WeatherParams.tint`.
    #[serde(default = "default_tint")]
    pub tint: [f32; 3],
}

#[derive(Debug, Deserialize)]
struct WeatherFile {
    fade_seconds: f32,
    min_spell_seconds: f32,
    max_spell_seconds: f32,
    biomes: HashMap<String, Vec<WeatherWeight>>,
    kinds: HashMap<String, WeatherKindDef>,
}

/// Validated weather rules and per-biome probability tables loaded from `data/weather.json`.
#[derive(Debug, Clone, PartialEq)]
pub struct WeatherData {
    pub fade_seconds: f32,
    pub min_spell_seconds: f32,
    pub max_spell_seconds: f32,
    biomes: [Vec<WeatherWeight>; BiomeId::COUNT],
    kinds: [WeatherKindDef; WeatherKind::COUNT],
}

impl WeatherData {
    /// Loads `data/weather.json` via `paths::repo_path`.
    pub fn load() -> Result<Self, String> {
        let path = paths::repo_path("data/weather.json");
        Self::load_from_file(path)
    }

    /// Loads weather definitions from an explicit file path.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&json)
    }

    /// Parses and validates weather definitions from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let parsed: WeatherFile = serde_json::from_str(json)
            .map_err(|e| format!("failed to parse weather JSON: {e}"))?;

        if !parsed.fade_seconds.is_finite() || parsed.fade_seconds < 0.0 {
            return Err(format!(
                "invalid fade_seconds in weather.json: {}",
                parsed.fade_seconds
            ));
        }
        if !parsed.min_spell_seconds.is_finite() || parsed.min_spell_seconds <= 0.0 {
            return Err(format!(
                "invalid min_spell_seconds in weather.json: {}",
                parsed.min_spell_seconds
            ));
        }
        if !parsed.max_spell_seconds.is_finite()
            || parsed.max_spell_seconds < parsed.min_spell_seconds
        {
            return Err(format!(
                "invalid max_spell_seconds in weather.json: {} (min={})",
                parsed.max_spell_seconds, parsed.min_spell_seconds
            ));
        }

        let biomes = parse_biomes(parsed.biomes)?;
        let kinds = parse_kinds(parsed.kinds)?;

        Ok(Self {
            fade_seconds: parsed.fade_seconds,
            min_spell_seconds: parsed.min_spell_seconds,
            max_spell_seconds: parsed.max_spell_seconds,
            biomes,
            kinds,
        })
    }

    /// Returns the weighted weather list for `biome`.
    pub fn biome_weights(&self, biome: BiomeId) -> &[WeatherWeight] {
        &self.biomes[biome.index()]
    }

    /// Returns true if `kind` appears with positive weight in `biome`'s weather list.
    pub fn biome_allows(&self, biome: BiomeId, kind: WeatherKind) -> bool {
        self.biomes[biome.index()]
            .iter()
            .any(|entry| entry.kind == kind && entry.weight > 0.0)
    }

    /// Returns the definition for `kind`. Never panics because `from_json` validates all 5 kinds.
    pub fn kind(&self, kind: WeatherKind) -> &WeatherKindDef {
        &self.kinds[kind.index()]
    }
}

fn parse_biomes(
    raw: HashMap<String, Vec<WeatherWeight>>,
) -> Result<[Vec<WeatherWeight>; BiomeId::COUNT], String> {
    let mut slots: [Option<Vec<WeatherWeight>>; BiomeId::COUNT] = std::array::from_fn(|_| None);

    for (key, list) in raw {
        let Some(id) = BiomeId::from_key(&key) else {
            return Err(format!("unknown biome key in weather.json: '{key}'"));
        };
        if list.is_empty() {
            return Err(format!(
                "empty weather list for biome '{key}' in weather.json"
            ));
        }
        let mut total_weight = 0.0_f32;
        for entry in &list {
            if !entry.weight.is_finite() || entry.weight <= 0.0 {
                return Err(format!(
                    "non-positive weight for biome '{key}' kind '{}' in weather.json",
                    entry.kind.key()
                ));
            }
            total_weight += entry.weight;
        }
        if !total_weight.is_finite() || total_weight <= 0.0 {
            return Err(format!(
                "invalid total weather weight for biome '{key}' in weather.json"
            ));
        }
        slots[id.index()] = Some(list);
    }

    let missing: Vec<&'static str> = BiomeId::ALL
        .iter()
        .filter(|id| slots[id.index()].is_none())
        .map(|id| id.key())
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "missing required biome(s) in weather.json: {}",
            missing.join(", ")
        ));
    }

    let [Some(plains), Some(forest), Some(desert), Some(snow), Some(swamp), Some(mountains)] =
        slots
    else {
        return Err("internal error assembling biome weather table".to_string());
    };

    Ok([plains, forest, desert, snow, swamp, mountains])
}

fn parse_kinds(
    raw: HashMap<String, WeatherKindDef>,
) -> Result<[WeatherKindDef; WeatherKind::COUNT], String> {
    let mut slots: [Option<WeatherKindDef>; WeatherKind::COUNT] = std::array::from_fn(|_| None);

    for (key, def) in raw {
        let Some(kind) = WeatherKind::from_key(&key) else {
            return Err(format!("unknown weather kind in weather.json: '{key}'"));
        };
        validate_kind_def(&key, &def)?;
        slots[kind.index()] = Some(def);
    }

    let missing: Vec<&'static str> = WeatherKind::ALL
        .iter()
        .filter(|kind| slots[kind.index()].is_none())
        .map(|kind| kind.key())
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "missing required weather kind(s) in weather.json: {}",
            missing.join(", ")
        ));
    }

    let [Some(clear), Some(rain), Some(snow), Some(sandstorm), Some(fog)] = slots else {
        return Err("internal error assembling weather kind table".to_string());
    };

    Ok([clear, rain, snow, sandstorm, fog])
}

fn validate_kind_def(key: &str, def: &WeatherKindDef) -> Result<(), String> {
    let unit_fields = [
        ("fog_mult", def.fog_mult),
        ("visibility", def.visibility),
        ("noise_mask", def.noise_mask),
    ];
    for (name, val) in unit_fields {
        if !val.is_finite() || !(0.0..=1.0).contains(&val) {
            return Err(format!(
                "weather kind '{key}' field '{name}' out of 0..=1: {val}"
            ));
        }
    }
    if !def.wind.is_finite() || def.wind < 0.0 {
        return Err(format!(
            "weather kind '{key}' invalid wind speed: {}",
            def.wind
        ));
    }
    if !def.particles_per_m3.is_finite() || def.particles_per_m3 < 0.0 {
        return Err(format!(
            "weather kind '{key}' invalid particles_per_m3: {}",
            def.particles_per_m3
        ));
    }
    for (idx, ch) in def.tint.iter().enumerate() {
        if !ch.is_finite() || !(0.0..=1.0).contains(ch) {
            return Err(format!(
                "weather kind '{key}' tint[{idx}] out of 0..=1: {ch}"
            ));
        }
    }
    Ok(())
}
