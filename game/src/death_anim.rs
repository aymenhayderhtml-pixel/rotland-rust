//! Directional and cause-based player death animation selection.

use std::collections::BTreeMap;

use glam::Vec3;
use serde::Deserialize;

use crate::hit_react::{HitSide, hit_side};

/// Cause of player death used to pick a death animation clip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeathCause {
    Zombie,
    Drowned,
    Other,
}

impl DeathCause {
    /// Key used to look up cause-specific clips in `data/death_anim.json`.
    pub fn as_key(self) -> &'static str {
        match self {
            Self::Zombie => "zombie",
            Self::Drowned => "drowned",
            Self::Other => "other",
        }
    }
}

/// Clip name and playback speed for one death animation entry.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct DeathClipSpec {
    pub clip: String,
    pub speed: f32,
}

impl DeathClipSpec {
    fn new(clip: &str, speed: f32) -> Self {
        Self { clip: clip.to_string(), speed }
    }
}

/// Directional death clips when killed by a zombie.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ZombieDeathClips {
    pub front: DeathClipSpec,
    pub back: DeathClipSpec,
    pub left: DeathClipSpec,
    pub right: DeathClipSpec,
}

/// Configuration loaded from `data/death_anim.json`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeathAnimConfig {
    pub version: u32,
    pub default: DeathClipSpec,
    pub causes: BTreeMap<String, DeathClipSpec>,
    pub zombie_by_side: ZombieDeathClips,
}

impl Default for DeathAnimConfig {
    fn default() -> Self {
        let mut causes = BTreeMap::new();
        causes.insert(
            "drowned".to_string(),
            DeathClipSpec::new("death_drown", 1.0), // data: death_anim.json causes.drowned
        );
        Self {
            version: 1, // data: death_anim.json version
            default: DeathClipSpec::new("die", 1.0), // data: death_anim.json default
            causes,
            zombie_by_side: ZombieDeathClips {
                front: DeathClipSpec::new("death_back", 1.0), // data: death_anim.json zombieBySide.front
                back: DeathClipSpec::new("death_forward", 1.0), // data: death_anim.json zombieBySide.back
                left: DeathClipSpec::new("death_right", 1.0), // data: death_anim.json zombieBySide.left
                right: DeathClipSpec::new("death_left", 1.0), // data: death_anim.json zombieBySide.right
            },
        }
    }
}

/// Selected death animation clip and playback speed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeathChoice<'a> {
    pub clip: &'a str,
    pub speed: f32,
}

impl DeathAnimConfig {
    /// Parses a death animation configuration from a JSON string.
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("invalid death_anim.json: {e}"))
    }

    /// Loads `data/death_anim.json` from the repository root.
    pub fn from_repo() -> Result<Self, String> {
        let path = paths::repo_path("data/death_anim.json");
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_json(&raw)
    }

    /// Selects the death clip and speed for `cause` and optional zombie attacker position.
    pub fn pick(
        &self,
        cause: DeathCause,
        player_pos: Vec3,
        player_yaw: f32,
        attacker: Option<Vec3>,
    ) -> DeathChoice<'_> {
        match cause {
            DeathCause::Zombie => {
                if let Some(pos) = attacker {
                    let spec = match hit_side(player_pos, player_yaw, pos) {
                        HitSide::Front => &self.zombie_by_side.front,
                        HitSide::Back => &self.zombie_by_side.back,
                        HitSide::Left => &self.zombie_by_side.left,
                        HitSide::Right => &self.zombie_by_side.right,
                    };
                    DeathChoice { clip: &spec.clip, speed: spec.speed }
                } else {
                    DeathChoice { clip: &self.default.clip, speed: self.default.speed }
                }
            }
            DeathCause::Drowned | DeathCause::Other => self.pick_cause_key(cause.as_key()),
        }
    }

    /// Selects a clip by raw cause key (falls back to `default` when unknown).
    pub fn pick_cause_key(&self, cause_key: &str) -> DeathChoice<'_> {
        let spec = self.causes.get(cause_key).unwrap_or(&self.default);
        DeathChoice { clip: &spec.clip, speed: spec.speed }
    }
}
