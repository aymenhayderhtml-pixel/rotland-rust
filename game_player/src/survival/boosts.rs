use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::downed::Downed;

/// Antibiotics heal-over-time config from `data/boosts.json`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AntibioticsConfig {
    pub heal_total: f32,
    pub seconds: f32,
}

impl Default for AntibioticsConfig {
    fn default() -> Self {
        Self {
            // data: boosts.json antibiotics.healTotal
            heal_total: 60.0,
            // data: boosts.json antibiotics.seconds
            seconds: 60.0,
        }
    }
}

/// Adrenaline shot config from `data/boosts.json`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AdrenalineConfig {
    pub free_sprint_seconds: f32,
}

impl Default for AdrenalineConfig {
    fn default() -> Self {
        Self {
            // data: boosts.json adrenaline_shot.freeSprintSeconds
            free_sprint_seconds: 20.0,
        }
    }
}

/// Surgery kit config from `data/boosts.json`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SurgeryKitConfig {
    pub use_seconds: f32,
}

impl Default for SurgeryKitConfig {
    fn default() -> Self {
        Self {
            // data: boosts.json surgery_kit.useSeconds
            use_seconds: 6.0,
        }
    }
}

/// Night vision goggles config from `data/boosts.json`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NightVisionConfig {
    pub seconds_per_battery: f32,
    pub max_charge: f32,
    pub gain: f32,
    pub tint: [f32; 3],
    pub noise: f32,
}

impl Default for NightVisionConfig {
    fn default() -> Self {
        Self {
            // data: boosts.json nightVision.secondsPerBattery
            seconds_per_battery: 300.0,
            // data: boosts.json nightVision.maxCharge
            max_charge: 300.0,
            // data: boosts.json nightVision.gain
            gain: 6.0,
            // data: boosts.json nightVision.tint
            tint: [0.35, 1.0, 0.45],
            // data: boosts.json nightVision.noise
            noise: 0.06,
        }
    }
}

/// Combined boosts & night-vision tuning loaded from `data/boosts.json`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BoostsConfig {
    pub antibiotics: AntibioticsConfig,
    #[serde(rename = "adrenaline_shot", alias = "adrenalineShot")]
    pub adrenaline_shot: AdrenalineConfig,
    #[serde(rename = "surgery_kit", alias = "surgeryKit")]
    pub surgery_kit: SurgeryKitConfig,
    pub night_vision: NightVisionConfig,
}

impl BoostsConfig {
    /// Loads `data/boosts.json` from the repository root.
    pub fn from_repo() -> Result<Self, String> {
        Self::load(&paths::repo_path("data/boosts.json"))
    }

    /// Reads a boosts JSON file at `path`.
    pub fn load(path: &Path) -> Result<Self, String> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&raw)
    }

    /// Parses boosts configuration from JSON text.
    pub fn from_json(raw: &str) -> Result<Self, String> {
        serde_json::from_str(raw).map_err(|e| format!("invalid boosts.json: {e}"))
    }
}

/// Active player medical/stimulant timers (antibiotics heal-over-time + adrenaline free sprint).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Boosts {
    heal_remaining_secs: f32,
    heal_rate_per_sec: f32,
    free_sprint_remaining_secs: f32,
}

impl Boosts {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts or extends heal-over-time (stacking in time, not rate).
    pub fn start_antibiotics(&mut self, cfg: &AntibioticsConfig) {
        if cfg.seconds <= 0.0 || cfg.heal_total <= 0.0 {
            return;
        }
        self.heal_rate_per_sec = cfg.heal_total / cfg.seconds;
        self.heal_remaining_secs += cfg.seconds;
    }

    /// Starts or refreshes the free-sprint timer.
    pub fn start_adrenaline(&mut self, cfg: &AdrenalineConfig) {
        if cfg.free_sprint_seconds <= 0.0 {
            return;
        }
        self.free_sprint_remaining_secs =
            self.free_sprint_remaining_secs.max(cfg.free_sprint_seconds);
    }

    /// Advances active boost timers by `dt` seconds and returns health to add this frame.
    pub fn update(&mut self, dt: f32) -> f32 {
        let dt = if dt > 0.0 && dt.is_finite() { dt } else { 0.0 };
        let heal_dt = dt.min(self.heal_remaining_secs);
        self.heal_remaining_secs = (self.heal_remaining_secs - heal_dt).max(0.0);
        let heal = heal_dt * self.heal_rate_per_sec;
        if self.heal_remaining_secs <= 0.0 {
            self.heal_rate_per_sec = 0.0;
        }
        self.free_sprint_remaining_secs = (self.free_sprint_remaining_secs - dt).max(0.0);
        heal
    }

    /// True while adrenaline free-sprint is active.
    pub fn free_sprint(&self) -> bool {
        self.free_sprint_remaining_secs > 0.0
    }

    /// Remaining seconds of antibiotics heal-over-time.
    pub fn heal_seconds_remaining(&self) -> f32 {
        self.heal_remaining_secs
    }

    /// Remaining seconds of adrenaline free-sprint.
    pub fn free_sprint_seconds_remaining(&self) -> f32 {
        self.free_sprint_remaining_secs
    }
}

/// Revives a downed player immediately using the existing downed revive path.
/// Returns `true` if the player was downed and stood back up.
pub fn adrenaline_revive(downed: &mut Downed) -> bool {
    if !downed.is_down() {
        return false;
    }
    let revive_secs = downed.config().revive_seconds;
    downed.tick(revive_secs, true, false, false).is_revived()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn antibiotics_stack_in_time_and_heal_total() {
        let cfg = AntibioticsConfig::default();
        let mut boosts = Boosts::new();
        boosts.start_antibiotics(&cfg);
        boosts.start_antibiotics(&cfg);
        assert!((boosts.heal_seconds_remaining() - 120.0).abs() < 1e-4);

        let mut healed = 0.0;
        for _ in 0..120 {
            healed += boosts.update(1.0);
        }
        assert!((healed - 120.0).abs() < 0.5);
        assert_eq!(boosts.heal_seconds_remaining(), 0.0);
    }

    #[test]
    fn adrenaline_free_sprint_and_revive() {
        let mut boosts = Boosts::new();
        assert!(!boosts.free_sprint());
        boosts.start_adrenaline(&AdrenalineConfig::default());
        assert!(boosts.free_sprint());
        boosts.update(20.0);
        assert!(!boosts.free_sprint());

        let mut downed = Downed::default();
        assert!(!adrenaline_revive(&mut downed));
        downed.on_lethal_hit(10.0);
        assert!(downed.is_down());
        assert!(adrenaline_revive(&mut downed));
        assert!(downed.is_up());
    }
}
