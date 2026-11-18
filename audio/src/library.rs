//! data/sounds.json: every sound id with its files and how it plays (port of SoundLibrary.cs / SoundDef.cs).

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundDef {
    pub files: Vec<String>,
    #[serde(default)]
    pub volume_db: f32,
    #[serde(default)]
    pub pitch_jitter: f32,
    #[serde(default = "default_max_distance")]
    pub max_distance: f32,
    #[serde(default)]
    pub spatial: bool,
    #[serde(default = "default_bus")]
    pub bus: String,
}

fn default_max_distance() -> f32 {
    30.0
}

fn default_bus() -> String {
    "SFX".into()
}

#[derive(Clone, Debug, Deserialize)]
pub struct EventDef {
    pub sound: String,
    #[serde(default, rename = "maxSeconds")]
    pub max_seconds: f32,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SoundData {
    pub sounds: HashMap<String, SoundDef>,
    /// Weapon id -> {"swing": sound} or {"shot": sound}.
    #[serde(default)]
    pub weapons: HashMap<String, HashMap<String, String>>,
    #[serde(default)]
    pub events: HashMap<String, EventDef>,
}

impl SoundData {
    pub fn from_repo() -> Result<Self, String> {
        let path = paths::repo_path("data/sounds.json");
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("data/sounds.json: {e}"))
    }

    /// "res://assets/audio/sfx/x.wav" -> the file in this checkout.
    pub fn file_path(res: &str) -> std::path::PathBuf {
        paths::repo_path(res.trim_start_matches("res://"))
    }

    /// The sound a weapon makes for an action ("swing" or "shot").
    ///
    /// If the weapon or action is missing from `data/sounds.json`, falls back by
    /// weapon class (gun "shot" -> "pistol_shot", melee "swing" -> "swing_light")
    /// with a log warning, so a missing entry is never silent.
    pub fn weapon_sound(&self, weapon: &str, action: &str) -> Option<&str> {
        if let Some(sound) = self.weapons.get(weapon).and_then(|slots| slots.get(action)) {
            return Some(sound.as_str());
        }
        log::warn!("missing sound for weapon '{weapon}', action '{action}'; using fallback");
        match action {
            "shot" => Some("pistol_shot"),
            "swing" => Some("swing_light"),
            _ => None,
        }
    }
}
