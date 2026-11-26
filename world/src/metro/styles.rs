//! Tower architectural styles, module sizes and skyline wall colours (`data/metro.json` `styles`).

use std::sync::OnceLock;

use serde::{Deserialize, Deserializer};

use crate::metro::layout::MetroDistrict;

/// Allowed square module widths in metres.
pub const VALID_SIZES: [u32; 3] = [16, 24, 32]; // new: DECISIONS 2026-11-26 (fixed module widths)

/// Architectural style of a New Halden tower.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TowerStyle {
    Brick,
    Stone,
    Glass,
    Slab,
    Deco,
}

impl TowerStyle {
    /// Every tower style in canonical order.
    pub const ALL: [TowerStyle; 5] = [ // new: DECISIONS 2026-11-26
        TowerStyle::Brick,
        TowerStyle::Stone,
        TowerStyle::Glass,
        TowerStyle::Slab,
        TowerStyle::Deco,
    ];

    /// Lowercase key used in model names and `--tower-style=`.
    pub fn key(self) -> &'static str {
        match self {
            TowerStyle::Brick => "brick",
            TowerStyle::Stone => "stone",
            TowerStyle::Glass => "glass",
            TowerStyle::Slab => "slab",
            TowerStyle::Deco => "deco",
        }
    }

    /// Parses a style key (`"brick"`, `"stone"`, `"glass"`, `"slab"`, `"deco"`).
    pub fn parse(s: &str) -> Option<TowerStyle> {
        match s.trim().to_ascii_lowercase().as_str() {
            "brick" => Some(TowerStyle::Brick),
            "stone" => Some(TowerStyle::Stone),
            "glass" => Some(TowerStyle::Glass),
            "slab" => Some(TowerStyle::Slab),
            "deco" => Some(TowerStyle::Deco),
            _ => None,
        }
    }
}

/// Style definition loaded from `data/metro.json` `styles`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleDef {
    pub key: TowerStyle,
    pub sizes: Vec<u32>,
    #[serde(deserialize_with = "deserialize_districts")]
    pub districts: Vec<MetroDistrict>,
    pub floors: [u32; 2],
    #[serde(default)]
    pub floor_height: f32,
    pub base_height: f32,
    pub crown_height: f32,
    /// Height of the walkable roof deck above the crown base (from the Blender crown scripts).
    #[serde(default)]
    pub roof_deck_height: f32,
    /// Wall sRGB colour `[r, g, b]` used by the far skyline.
    pub color: [u8; 3],
}

fn deserialize_districts<'de, D>(deserializer: D) -> Result<Vec<MetroDistrict>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Vec::<String>::deserialize(deserializer)?;
    raw.into_iter()
        .map(|s| match s.as_str() {
            "edge" => Ok(MetroDistrict::Edge),
            "midtown" => Ok(MetroDistrict::Midtown),
            "core" => Ok(MetroDistrict::Core),
            other => Err(serde::de::Error::custom(format!("unknown district '{other}'"))),
        })
        .collect()
}

#[derive(Deserialize)]
struct StylesWrapper {
    styles: Vec<StyleDef>,
}

/// Parses and validates the `styles` array from a `metro.json` string.
pub fn load_styles(json: &str) -> Result<Vec<StyleDef>, String> {
    let wrapper: StylesWrapper =
        serde_json::from_str(json).map_err(|e| format!("metro.json styles: {e}"))?;
    validate_styles(&wrapper.styles)?;
    Ok(wrapper.styles)
}

/// Validates style sizes, floor ranges, district coverage and the landmark `deco` style.
pub fn validate_styles(styles: &[StyleDef]) -> Result<(), String> {
    if styles.is_empty() {
        return Err("metro.json: styles must not be empty".into());
    }
    for s in styles {
        if s.sizes.is_empty() || s.sizes.iter().any(|sz| !VALID_SIZES.contains(sz)) {
            return Err(format!(
                "metro.json: style '{}' has invalid sizes {:?} (allowed: 16, 24, 32)",
                s.key.key(),
                s.sizes
            ));
        }
        if s.floors[0] == 0 || s.floors[0] > s.floors[1] {
            return Err(format!(
                "metro.json: style '{}' has invalid floors {:?}",
                s.key.key(),
                s.floors
            ));
        }
        if s.base_height <= 0.0 || s.crown_height <= 0.0 || s.floor_height < 0.0 {
            return Err(format!(
                "metro.json: style '{}' heights must be positive",
                s.key.key()
            ));
        }
    }
    for d in [MetroDistrict::Edge, MetroDistrict::Midtown, MetroDistrict::Core] {
        if !styles.iter().any(|s| s.districts.contains(&d)) {
            return Err(format!("metro.json: no tower style covers district {d:?}"));
        }
    }
    if !styles.iter().any(|s| s.key == TowerStyle::Deco) {
        return Err("metro.json: a 'deco' style is required for landmarks".into());
    }
    Ok(())
}

/// Cached styles loaded from `data/metro.json`.
pub fn default_styles() -> &'static [StyleDef] {
    static CACHE: OnceLock<Vec<StyleDef>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let path = paths::repo_path("data/metro.json");
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        load_styles(&json).expect("data/metro.json styles must be valid")
    })
}

/// Finds the `StyleDef` matching `key`.
pub fn style_for(styles: &[StyleDef], key: TowerStyle) -> Option<&StyleDef> {
    styles.iter().find(|s| s.key == key)
}
