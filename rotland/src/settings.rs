//! Game settings persistence (mouse sensitivity, invert Y, FOV, render scale, vsync, audio, FPS, preset).
//! Loaded at startup and saved when the settings screen closes. Missing or broken files fall back to defaults.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

pub const MIN_SENSITIVITY: f32 = 0.2;
pub const MAX_SENSITIVITY: f32 = 3.0;
pub const MIN_FOV: f32 = 60.0;
pub const MAX_FOV: f32 = 100.0;
pub const MIN_RENDER_SCALE: f32 = 0.5;
pub const MAX_RENDER_SCALE: f32 = 1.0;

/// Player settings saved to `<data_dir>/rotland-rust/settings.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    #[serde(alias = "mouseSensitivity")]
    pub mouse_sensitivity: f32,

    #[serde(alias = "invertY")]
    pub invert_y: bool,

    #[serde(alias = "fieldOfView", alias = "fov")]
    pub field_of_view: f32,

    #[serde(alias = "renderScale")]
    pub render_scale: f32,

    pub vsync: bool,

    #[serde(alias = "masterVolume")]
    pub master_volume: f32,

    #[serde(alias = "musicVolume")]
    pub music_volume: f32,

    #[serde(alias = "sfxVolume")]
    pub sfx_volume: f32,

    #[serde(alias = "showFps")]
    pub show_fps: bool,

    #[serde(alias = "graphicsPreset")]
    pub graphics_preset: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 1.0,
            invert_y: false,
            field_of_view: 75.0,
            render_scale: 1.0,
            vsync: false,
            master_volume: 1.0,
            music_volume: 1.0,
            sfx_volume: 1.0,
            show_fps: true,
            graphics_preset: "medium".to_string(),
        }
    }
}

impl Settings {
    /// Keeps every value within its valid range so manual edits cannot break gameplay.
    pub fn clamp(&mut self) {
        self.mouse_sensitivity = self.mouse_sensitivity.clamp(MIN_SENSITIVITY, MAX_SENSITIVITY);
        self.field_of_view = self.field_of_view.clamp(MIN_FOV, MAX_FOV);
        self.render_scale = self.render_scale.clamp(MIN_RENDER_SCALE, MAX_RENDER_SCALE);
        self.master_volume = self.master_volume.clamp(0.0, 1.0);
        self.music_volume = self.music_volume.clamp(0.0, 1.0);
        self.sfx_volume = self.sfx_volume.clamp(0.0, 1.0);
        let p = self.graphics_preset.to_ascii_lowercase();
        if p != "low" && p != "medium" && p != "high" {
            self.graphics_preset = "medium".to_string();
        }
    }

    /// Loads settings from `path`. Missing or invalid JSON falls back to default settings without panicking.
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str::<Self>(&content) {
                Ok(mut settings) => {
                    settings.clamp();
                    settings
                }
                Err(err) => {
                    log::warn!("Could not parse settings from {}: {err}", path.display());
                    Self::default()
                }
            },
            Err(err) => {
                log::warn!("Could not read settings from {}: {err}", path.display());
                Self::default()
            }
        }
    }

    /// Saves settings as JSON to `path`. Returns true on success or false on failure.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
                && let Err(err) = std::fs::create_dir_all(parent) {
                    log::warn!("Could not create settings directory {}: {err}", parent.display());
                    return false;
                }
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(err) = std::fs::write(path, json) {
                    log::warn!("Could not write settings to {}: {err}", path.display());
                    false
                } else {
                    true
                }
            }
            Err(err) => {
                log::warn!("Could not serialize settings: {err}");
                false
            }
        }
    }
}

/// Returns `<data_dir>/rotland-rust/settings.json` where `data_dir` is `$XDG_DATA_HOME` or `~/.local/share`.
pub fn settings_path() -> PathBuf {
    let data_dir = if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            PathBuf::from(xdg)
        } else {
            fallback_data_dir()
        }
    } else {
        fallback_data_dir()
    };
    data_dir.join("rotland-rust").join("settings.json")
}

fn fallback_data_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".local").join("share")
    } else {
        std::env::temp_dir()
    }
}
